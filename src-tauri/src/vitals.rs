//! How hard the computer works: processor, memory, graphics and heat, sampled every couple
//! of seconds with the system's own counters (cheap enough to never show up in them).

use serde::Serialize;

#[derive(Serialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vitals {
    /// Processor use, 0–100.
    pub cpu: Option<f32>,
    /// Memory in use, 0–100.
    pub mem: Option<f32>,
    /// Graphics (3D engine) use, 0–100, where the system tells.
    pub gpu: Option<f32>,
    /// Heat, where the system tells: 0 cool, 1 warm, 2 hot, 3 very hot.
    pub heat: Option<u8>,
}

#[cfg(windows)]
pub use win_impl::Sampler;
#[cfg(target_os = "macos")]
pub use mac_impl::Sampler;

#[cfg(windows)]
mod win_impl {
    use super::Vitals;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Performance::{
        PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW, PDH_FMT_COUNTERVALUE_ITEM_W,
        PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
    };
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows_sys::Win32::System::Threading::GetSystemTimes;

    pub struct Sampler {
        prev: Option<(u64, u64)>,
        gpu: Option<(PDH_HQUERY, PDH_HCOUNTER)>,
        gpu_failed: bool,
    }

    // The PDH handles are only used from the Dock's own thread.
    unsafe impl Send for Sampler {}

    fn ft(f: FILETIME) -> u64 {
        ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
    }

    impl Sampler {
        pub fn new() -> Self {
            Sampler { prev: None, gpu: None, gpu_failed: false }
        }

        fn cpu(&mut self) -> Option<f32> {
            let (mut idle, mut kernel, mut user) = unsafe { std::mem::zeroed::<(FILETIME, FILETIME, FILETIME)>() };
            if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
                return None;
            }
            // Kernel time includes idle time.
            let (idle, busy_total) = (ft(idle), ft(kernel) + ft(user));
            let out = self.prev.and_then(|(pi, pt)| {
                let total = busy_total.saturating_sub(pt);
                (total > 0).then(|| (1.0 - idle.saturating_sub(pi) as f32 / total as f32).clamp(0.0, 1.0) * 100.0)
            });
            self.prev = Some((idle, busy_total));
            out
        }

        fn mem() -> Option<f32> {
            let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
            m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            (unsafe { GlobalMemoryStatusEx(&mut m) } != 0).then_some(m.dwMemoryLoad as f32)
        }

        /// The 3D engines' use across every app, as Task Manager counts it.
        fn gpu(&mut self) -> Option<f32> {
            if self.gpu_failed {
                return None;
            }
            unsafe {
                if self.gpu.is_none() {
                    let mut q: PDH_HQUERY = null_mut();
                    let mut c: PDH_HCOUNTER = null_mut();
                    let path = crate::win::wide("\\GPU Engine(*engtype_3D)\\Utilization Percentage");
                    if PdhOpenQueryW(null(), 0, &mut q) != 0 || PdhAddEnglishCounterW(q, path.as_ptr(), 0, &mut c) != 0 {
                        if !q.is_null() {
                            PdhCloseQuery(q);
                        }
                        self.gpu_failed = true;
                        return None;
                    }
                    PdhCollectQueryData(q);
                    self.gpu = Some((q, c));
                    return None; // a rate needs two samples
                }
                let (q, c) = self.gpu?;
                if PdhCollectQueryData(q) != 0 {
                    return None;
                }
                let (mut size, mut count) = (0u32, 0u32);
                if PdhGetFormattedCounterArrayW(c, PDH_FMT_DOUBLE, &mut size, &mut count, null_mut()) != PDH_MORE_DATA || size == 0 {
                    return Some(0.0);
                }
                let mut buf = vec![0u8; size as usize];
                let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
                if PdhGetFormattedCounterArrayW(c, PDH_FMT_DOUBLE, &mut size, &mut count, items) != 0 {
                    return None;
                }
                let mut sum = 0.0f64;
                for i in 0..count as usize {
                    let v = &(*items.add(i)).FmtValue;
                    if v.CStatus == 0 {
                        sum += v.Anonymous.doubleValue;
                    }
                }
                Some(sum.clamp(0.0, 100.0) as f32)
            }
        }

        /// `detailed`: also the graphics (only while the panel shows it).
        pub fn sample(&mut self, detailed: bool) -> Vitals {
            Vitals { cpu: self.cpu(), mem: Self::mem(), gpu: if detailed { self.gpu() } else { None }, heat: None }
        }
    }

    impl Drop for Sampler {
        fn drop(&mut self) {
            if let Some((q, _)) = self.gpu.take() {
                unsafe { PdhCloseQuery(q) };
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod mac_impl {
    use super::Vitals;

    extern "C" {
        fn mach_host_self() -> u32;
        fn host_statistics(host: u32, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
    }
    const HOST_CPU_LOAD_INFO: i32 = 3;

    pub struct Sampler {
        prev: Option<[u32; 4]>,
    }

    impl Sampler {
        pub fn new() -> Self {
            Sampler { prev: None }
        }

        fn cpu(&mut self) -> Option<f32> {
            let mut ticks = [0u32; 4]; // user, system, idle, nice
            let mut count = 4u32;
            if unsafe { host_statistics(mach_host_self(), HOST_CPU_LOAD_INFO, ticks.as_mut_ptr() as *mut i32, &mut count) } != 0 {
                return None;
            }
            let out = self.prev.and_then(|p| {
                let d: Vec<u64> = (0..4).map(|i| ticks[i].wrapping_sub(p[i]) as u64).collect();
                let total: u64 = d.iter().sum();
                (total > 0).then(|| ((d[0] + d[1] + d[3]) as f32 / total as f32 * 100.0).clamp(0.0, 100.0))
            });
            self.prev = Some(ticks);
            out
        }

        #[allow(deprecated)]
        fn mem() -> Option<f32> {
            unsafe {
                let mut total: u64 = 0;
                let mut len = std::mem::size_of::<u64>();
                let name = b"hw.memsize\0";
                if libc::sysctlbyname(name.as_ptr() as *const libc::c_char, &mut total as *mut u64 as *mut libc::c_void, &mut len, std::ptr::null_mut(), 0) != 0 || total == 0 {
                    return None;
                }
                let mut vm: libc::vm_statistics64 = std::mem::zeroed();
                let mut count = libc::HOST_VM_INFO64_COUNT;
                if libc::host_statistics64(libc::mach_host_self(), libc::HOST_VM_INFO64, &mut vm as *mut _ as *mut i32, &mut count) != 0 {
                    return None;
                }
                let page = libc::sysconf(libc::_SC_PAGESIZE).max(4096) as u64;
                // In use the way Activity Monitor counts it: app memory, wired and compressed.
                let used = (vm.active_count as u64 + vm.wire_count as u64 + vm.compressor_page_count as u64) * page;
                Some((used as f64 / total as f64 * 100.0).clamp(0.0, 100.0) as f32)
            }
        }

        fn heat() -> Option<u8> {
            let s = objc2_foundation::NSProcessInfo::processInfo().thermalState();
            Some(s.0.clamp(0, 3) as u8)
        }

        pub fn sample(&mut self, _detailed: bool) -> Vitals {
            Vitals { cpu: self.cpu(), mem: Self::mem(), gpu: None, heat: Self::heat() }
        }
    }
}
