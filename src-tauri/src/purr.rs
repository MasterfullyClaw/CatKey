//! Background purring: a seamless purr loop (made by `tools/make_purr.py`) played by
//! Windows itself, so it keeps going while the window is closed. The volume is
//! applied to the samples, so it doesn't change any other sound on the PC.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_LOOP, SND_MEMORY, SND_NODEFAULT};

static WAV: &[u8] = include_bytes!("../sounds/purr.wav");
/// The buffer Windows is playing from: it must live as long as the sound plays.
static BUF: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static ON: AtomicBool = AtomicBool::new(false);
static VOLUME: AtomicU32 = AtomicU32::new(60);

pub fn is_on() -> bool {
    ON.load(Ordering::SeqCst)
}

pub fn set_volume_only(v: u32) {
    VOLUME.store(v.min(100), Ordering::SeqCst);
}

/// Start or stop purring (and set the volume, 0–100). Returns whether it purrs now.
pub fn set(on: bool, volume: Option<u32>) -> bool {
    if let Some(v) = volume {
        set_volume_only(v);
    }
    let Ok(mut buf) = BUF.lock() else { return false };
    unsafe { PlaySoundW(null(), null_mut(), 0) }; // stop whatever plays (and release the buffer)
    ON.store(false, Ordering::SeqCst);
    if !on {
        return false;
    }
    *buf = scaled(VOLUME.load(Ordering::SeqCst));
    let ok = unsafe { PlaySoundW(buf.as_ptr() as *const u16, null_mut(), SND_MEMORY | SND_ASYNC | SND_LOOP | SND_NODEFAULT) } != 0;
    ON.store(ok, Ordering::SeqCst);
    ok
}

/// The loop with its 16-bit samples turned down to `volume` (felt loudness, so squared).
fn scaled(volume: u32) -> Vec<u8> {
    let mut out = WAV.to_vec();
    let gain = (volume.min(100) as f32 / 100.0).powi(2);
    // Walk the RIFF chunks to the samples.
    let mut i = 12;
    while i + 8 <= out.len() {
        let id = &out[i..i + 4];
        let size = u32::from_le_bytes([out[i + 4], out[i + 5], out[i + 6], out[i + 7]]) as usize;
        if id == b"data" {
            let end = (i + 8 + size).min(out.len());
            let mut j = i + 8;
            while j + 1 < end {
                let s = i16::from_le_bytes([out[j], out[j + 1]]) as f32 * gain;
                let b = (s as i16).to_le_bytes();
                out[j] = b[0];
                out[j + 1] = b[1];
                j += 2;
            }
            break;
        }
        i += 8 + size + (size & 1);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_loop_is_a_16_bit_wave_and_scales() {
        assert_eq!(&super::WAV[0..4], b"RIFF");
        let quiet = super::scaled(0);
        assert_eq!(quiet.len(), super::WAV.len());
        assert!(quiet[44..].iter().all(|b| *b == 0));
    }
}
