//! The remapping state machine.
//!
//! The Windows keyboard hook feeds every physical key event to [`Engine::handle`].
//! The engine answers "swallow it or let it through" and fills [`Engine::out`] with
//! the synthetic key events to send. It owns no OS handles, so all of the tricky
//! timing (Alt menus, key repeat, release order, per-app rules) is unit-tested.
//!
//! Model:
//! * `phys`   – which keys are physically held (as far as we have seen).
//! * `log_mods` – the modifiers that count for matching rules (real held modifiers
//!   plus keys remapped *to* modifiers, e.g. CapsLock -> Ctrl).
//! * `os_mods`  – the modifiers Windows currently believes are down. Swapping
//!   Alt for Ctrl makes these differ from `log_mods`; they are brought back in
//!   line lazily, only when a key needs them to be.

use crate::config::{CAction, CKeyRemap, CRule, Compiled, KAction};
use crate::keys::{self, bit, is_modifier, mod_bit, sided, ModSpec, Shortcut, DUMMY_VK, MOD_VKS, VK_PACKET};
use std::sync::Arc;

/// One physical key event from the hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub vk: u8,
    pub scan: u16,
    pub down: bool,
    pub extended: bool,
    /// Milliseconds since boot (same clock as `GetTickCount`).
    pub time: u32,
}

impl KeyEvent {
    pub fn down(vk: u8) -> Self {
        KeyEvent { vk, scan: 0, down: true, extended: false, time: 0 }
    }
    pub fn up(vk: u8) -> Self {
        KeyEvent { vk, scan: 0, down: false, extended: false, time: 0 }
    }
}

/// A synthetic event for the platform to send (in order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Out {
    /// Press or release a virtual key (scan code looked up by the platform).
    Key { vk: u8, down: bool },
    /// Re-send an original event exactly (after fixing the modifiers in front of it).
    Raw { vk: u8, scan: u16, extended: bool, down: bool },
    /// Type one UTF-16 unit.
    Unicode { unit: u16, down: bool },
    /// Tap the unassigned key so a later Alt/Win release does not open a menu.
    Dummy,
}

/// Things the platform should do besides sending keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// A rule ran (index into the config's rules).
    Fired { rule: u32 },
    /// Open a program / file / URL (done off the hook thread).
    Launch { target: Arc<str>, args: Arc<str> },
    /// The pause hotkey flipped the master switch.
    Toggled { enabled: bool },
    /// Recording: the keys currently held.
    RecordUpdate { shortcut: String },
    /// Recording finished: the shortcut, or `None` if it can't be used.
    RecordDone { shortcut: Option<String> },
    /// Recording gave up (nothing pressed for a while).
    RecordCancelled,
}

/// Lets the engine ask about the outside world.
pub trait Env {
    /// Lowercase exe name of the foreground app ("chrome.exe"), or "" if unknown.
    fn foreground_exe(&mut self) -> &str;
    /// Is this key down according to Windows right now?
    fn key_down_async(&mut self, vk: u8) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Active {
    None,
    /// The physical key triggered a shortcut rule; `target` is held down in its place.
    Shortcut { target: u8, trig: u8, tmods: u8, stale: bool },
    /// Single-key remap to a normal key.
    Sub { target: u8 },
    /// Single-key remap to a modifier (`bit`).
    SubMod { bit: u8 },
    /// Swallowed (launch, text, block): its release is swallowed too.
    Swallow,
}

struct Recorder {
    started: u32,
    any: bool,
    down: [bool; 256],
    /// Keys already held when recording started (their events pass normally).
    pre: [bool; 256],
    mods: u8,
    key: Option<u8>,
    mod_keys: u8,
    last_mod: u8,
}

/// Recording gives up after this long with no key pressed.
pub const RECORD_TIMEOUT_MS: u32 = 20_000;

pub struct Engine {
    compiled: Arc<Compiled>,
    enabled: bool,
    phys: [bool; 256],
    active: [Active; 256],
    log_mods: u8,
    os_mods: u8,
    rec: Option<Box<Recorder>>,
    /// Filled by every call; send these in order.
    pub out: Vec<Out>,
    /// Filled by every call.
    pub effects: Vec<Effect>,
}

impl Engine {
    pub fn new(compiled: Arc<Compiled>, enabled: bool) -> Self {
        Engine {
            compiled,
            enabled,
            phys: [false; 256],
            active: [Active::None; 256],
            log_mods: 0,
            os_mods: 0,
            rec: None,
            out: Vec::with_capacity(16),
            effects: Vec::with_capacity(4),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_recording(&self) -> bool {
        self.rec.is_some()
    }

    /// Swap in new rules. Keys held right now keep doing what they started.
    pub fn set_compiled(&mut self, compiled: Arc<Compiled>) {
        self.compiled = compiled;
    }

    /// Turn remapping on/off. Fills `out` with whatever puts Windows back in line.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.out.clear();
        self.effects.clear();
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        if !enabled {
            self.neutralize();
        }
    }

    /// Start recording a shortcut: the next keys pressed are captured and swallowed.
    pub fn start_recording(&mut self, now: u32) {
        let mut pre = [false; 256];
        for (i, held) in self.phys.iter().enumerate() {
            pre[i] = *held;
        }
        self.rec = Some(Box::new(Recorder {
            started: now,
            any: false,
            down: [false; 256],
            pre,
            mods: 0,
            key: None,
            mod_keys: 0,
            last_mod: 0,
        }));
    }

    pub fn stop_recording(&mut self) {
        self.rec = None;
    }

    /// Forget everything (desktop switch, session lock…). Fills `out` with releases
    /// for anything we were holding down in Windows.
    pub fn hard_reset(&mut self) {
        self.out.clear();
        self.effects.clear();
        self.neutralize();
        let held = self.os_mods;
        if held != 0 {
            if held & bit::ALT_WIN != 0 {
                self.out.push(Out::Dummy);
            }
            for b in bit::ALL {
                if held & b != 0 {
                    self.out.push(Out::Key { vk: keys::bit_vk(b), down: false });
                }
            }
        }
        self.phys = [false; 256];
        self.active = [Active::None; 256];
        self.log_mods = 0;
        self.os_mods = 0;
        self.rec = None;
    }

    /// Process one physical event. Returns `true` to swallow it.
    pub fn handle(&mut self, ev: KeyEvent, env: &mut dyn Env) -> bool {
        self.out.clear();
        self.effects.clear();
        let vk = ev.vk;
        if vk == 0 || vk == VK_PACKET || vk == DUMMY_VK {
            return false;
        }
        if self.rec.is_some() {
            if let Some(swallow) = self.handle_record(ev) {
                return swallow;
            }
        }
        if is_modifier(vk) {
            self.on_modifier(ev, env)
        } else {
            self.on_key(ev, env)
        }
    }

    // ------------------------------------------------------------------ modifiers

    fn on_modifier(&mut self, ev: KeyEvent, env: &mut dyn Env) -> bool {
        let vk = ev.vk as usize;
        let b = mod_bit(ev.vk);
        if ev.down {
            if !self.phys[vk] {
                self.phys[vk] = true;
                if self.enabled {
                    if let Some(km) = self.find_remap(ev.vk, env) {
                        return self.start_remap(ev, km);
                    }
                }
                self.log_mods |= b;
                self.os_mods |= b;
                false
            } else {
                // Auto-repeat of a held modifier.
                match self.active[vk] {
                    Active::Sub { target } => {
                        self.out.push(Out::Key { vk: target, down: true });
                        true
                    }
                    Active::SubMod { .. } | Active::Swallow | Active::Shortcut { .. } => true,
                    // Swapped out of Windows (e.g. Alt became Ctrl): don't press it again.
                    Active::None => self.os_mods & b == 0,
                }
            }
        } else {
            self.phys[vk] = false;
            match std::mem::replace(&mut self.active[vk], Active::None) {
                Active::Sub { target } => {
                    self.out.push(Out::Key { vk: target, down: false });
                    true
                }
                Active::Swallow => true,
                Active::SubMod { .. } | Active::Shortcut { .. } => {
                    self.recompute_log();
                    self.release_unneeded(false);
                    true
                }
                Active::None => {
                    self.recompute_log();
                    for a in self.active.iter_mut() {
                        if let Active::Shortcut { trig, stale, .. } = a {
                            if *trig & b != 0 {
                                *stale = true;
                            }
                        }
                    }
                    // The original release always goes through (harmless if Windows
                    // already saw it), which keeps other keyboard tools in sync.
                    self.os_mods &= !b;
                    self.release_unneeded(true);
                    false
                }
            }
        }
    }

    // ------------------------------------------------------------------ other keys

    fn on_key(&mut self, ev: KeyEvent, env: &mut dyn Env) -> bool {
        let vk = ev.vk as usize;
        if ev.down {
            if self.phys[vk] {
                // Auto-repeat.
                return match self.active[vk] {
                    Active::Shortcut { target, stale: false, .. } | Active::Sub { target } => {
                        self.out.push(Out::Key { vk: target, down: true });
                        true
                    }
                    Active::Shortcut { stale: true, .. } | Active::SubMod { .. } | Active::Swallow => true,
                    Active::None => false,
                };
            }
            self.phys[vk] = true;
            if self.log_mods != 0 {
                self.drop_stale_mods(env);
            }

            // The pause hotkey works even while napping.
            if let Some(p) = self.compiled.pause_hotkey {
                if p.key == ev.vk && p.mods.matches(self.log_mods) {
                    self.dummy_if_alt_win();
                    let now_enabled = !self.enabled;
                    self.enabled = now_enabled;
                    if !now_enabled {
                        self.neutralize(); // appends releases after the dummy tap
                    }
                    self.active[vk] = Active::Swallow;
                    self.effects.push(Effect::Toggled { enabled: now_enabled });
                    return true;
                }
            }

            if !self.enabled {
                return self.sync_and_pass(ev);
            }
            if let Some(rule) = self.find_shortcut(ev.vk, env) {
                return self.fire(ev, rule);
            }
            if let Some(km) = self.find_remap(ev.vk, env) {
                return self.start_remap(ev, km);
            }
            self.sync_and_pass(ev)
        } else {
            self.phys[vk] = false;
            match std::mem::replace(&mut self.active[vk], Active::None) {
                Active::Shortcut { target, .. } | Active::Sub { target } => {
                    self.out.push(Out::Key { vk: target, down: false });
                    true
                }
                Active::SubMod { .. } => {
                    self.recompute_log();
                    self.release_unneeded(false);
                    true
                }
                Active::Swallow => true,
                Active::None => false,
            }
        }
    }

    // ------------------------------------------------------------------ matching

    fn app_excluded(&self, env: &mut dyn Env) -> bool {
        if self.compiled.excluded.is_empty() {
            return false;
        }
        let exe = env.foreground_exe();
        self.compiled.excluded.iter().any(|a| a == exe)
    }

    fn find_shortcut(&self, vk: u8, env: &mut dyn Env) -> Option<CRule> {
        let list = &self.compiled.shortcuts[vk as usize];
        if list.is_empty() || !list.iter().any(|r| r.mods.matches(self.log_mods)) {
            return None;
        }
        if self.app_excluded(env) {
            return None;
        }
        let needs_app = list.iter().any(|r| r.scope.needs_app());
        let exe = if needs_app { env.foreground_exe().to_string() } else { String::new() };
        list.iter().find(|r| r.mods.matches(self.log_mods) && r.scope.matches(&exe)).cloned()
    }

    fn find_remap(&self, vk: u8, env: &mut dyn Env) -> Option<CKeyRemap> {
        let list = &self.compiled.remaps[vk as usize];
        if list.is_empty() || self.app_excluded(env) {
            return None;
        }
        let needs_app = list.iter().any(|r| r.scope.needs_app());
        let exe = if needs_app { env.foreground_exe().to_string() } else { String::new() };
        list.iter().find(|r| r.scope.matches(&exe)).cloned()
    }

    // ------------------------------------------------------------------ actions

    fn fire(&mut self, ev: KeyEvent, rule: CRule) -> bool {
        let vk = ev.vk as usize;
        match &rule.action {
            CAction::Keys { mods, key } => {
                let target = sided(*key);
                let desired = mods.to_state();
                self.set_os_mods(desired);
                self.out.push(Out::Key { vk: target, down: true });
                self.active[vk] = Active::Shortcut { target, trig: self.log_mods, tmods: desired, stale: false };
            }
            CAction::Launch { target, args } => {
                self.dummy_if_alt_win();
                self.effects.push(Effect::Launch { target: target.clone(), args: args.clone() });
                self.active[vk] = Active::Swallow;
            }
            CAction::Text { text } => {
                self.set_os_mods(0);
                for ch in text.chars() {
                    match ch {
                        '\r' => {}
                        '\n' => self.tap(keys::VK_RETURN),
                        '\t' => self.tap(keys::VK_TAB),
                        c => {
                            let mut buf = [0u16; 2];
                            for unit in c.encode_utf16(&mut buf).iter() {
                                self.out.push(Out::Unicode { unit: *unit, down: true });
                                self.out.push(Out::Unicode { unit: *unit, down: false });
                            }
                        }
                    }
                }
                self.active[vk] = Active::Swallow;
            }
            CAction::Block => {
                self.dummy_if_alt_win();
                self.active[vk] = Active::Swallow;
            }
        }
        self.effects.push(Effect::Fired { rule: rule.index });
        true
    }

    fn start_remap(&mut self, ev: KeyEvent, km: CKeyRemap) -> bool {
        let vk = ev.vk as usize;
        match km.action {
            KAction::Key(t) if is_modifier(t) => {
                let t = sided(t);
                let b = mod_bit(t);
                self.active[vk] = Active::SubMod { bit: b };
                self.recompute_log();
                if self.os_mods & b == 0 {
                    self.out.push(Out::Key { vk: t, down: true });
                    self.os_mods |= b;
                }
            }
            KAction::Key(t) => {
                // Make Windows see exactly the modifiers that are held, then the new key.
                if !is_modifier(ev.vk) {
                    self.recompute_log();
                }
                let want = self.log_mods;
                self.set_os_mods(want);
                self.out.push(Out::Key { vk: t, down: true });
                self.active[vk] = Active::Sub { target: t };
            }
            KAction::Block => {
                if !is_modifier(ev.vk) {
                    self.dummy_if_alt_win();
                }
                self.active[vk] = Active::Swallow;
            }
        }
        self.effects.push(Effect::Fired { rule: km.index });
        true
    }

    /// A key with no rule: let it through, first putting Windows' modifiers back to
    /// what is really held if a remap changed them.
    fn sync_and_pass(&mut self, ev: KeyEvent) -> bool {
        if self.os_mods == self.log_mods {
            return false;
        }
        let want = self.log_mods;
        self.set_os_mods(want);
        self.out.push(Out::Raw { vk: ev.vk, scan: ev.scan, extended: ev.extended, down: true });
        true
    }

    // ------------------------------------------------------------------ helpers

    fn tap(&mut self, vk: u8) {
        self.out.push(Out::Key { vk, down: true });
        self.out.push(Out::Key { vk, down: false });
    }

    fn dummy_if_alt_win(&mut self) {
        if self.os_mods & bit::ALT_WIN != 0 {
            self.out.push(Out::Dummy);
        }
    }

    /// Release / press modifiers so Windows sees exactly `desired`.
    fn set_os_mods(&mut self, desired: u8) {
        let release = self.os_mods & !desired;
        let press = desired & !self.os_mods;
        if release & bit::ALT_WIN != 0 {
            self.out.push(Out::Dummy);
        }
        for b in bit::ALL {
            if release & b != 0 {
                self.out.push(Out::Key { vk: keys::bit_vk(b), down: false });
            }
        }
        for b in bit::ALL {
            if press & b != 0 {
                self.out.push(Out::Key { vk: keys::bit_vk(b), down: true });
            }
        }
        self.os_mods = desired;
    }

    /// Modifiers that must stay down in Windows right now.
    fn keep_mods(&self) -> u8 {
        let mut k = self.log_mods;
        for a in self.active.iter() {
            if let Active::Shortcut { tmods, stale: false, .. } = a {
                k |= *tmods;
            }
        }
        k
    }

    /// After a release: let go of modifiers we were holding that nothing needs now.
    fn release_unneeded(&mut self, dummy_for_alt_win: bool) {
        let rel = self.os_mods & !self.keep_mods();
        if rel == 0 {
            return;
        }
        if dummy_for_alt_win && rel & bit::ALT_WIN != 0 {
            self.out.push(Out::Dummy);
        }
        for b in bit::ALL {
            if rel & b != 0 {
                self.out.push(Out::Key { vk: keys::bit_vk(b), down: false });
            }
        }
        self.os_mods &= !rel;
    }

    fn recompute_log(&mut self) {
        let mut m = 0;
        for mv in MOD_VKS {
            if self.phys[mv as usize] && self.active[mv as usize] == Active::None {
                m |= mod_bit(mv);
            }
        }
        for a in self.active.iter() {
            if let Active::SubMod { bit } = a {
                m |= *bit;
            }
        }
        self.log_mods = m;
    }

    /// We think a modifier is held but Windows says it isn't: we missed its release
    /// (lock screen, UAC prompt, admin window…). Forget it so rules match again.
    fn drop_stale_mods(&mut self, env: &mut dyn Env) {
        let mut changed = false;
        for mv in MOD_VKS {
            let i = mv as usize;
            if self.phys[i] && self.active[i] == Active::None {
                let b = mod_bit(mv);
                if self.os_mods & b != 0 && !env.key_down_async(mv) {
                    self.phys[i] = false;
                    self.os_mods &= !b;
                    changed = true;
                }
            }
        }
        if changed {
            self.recompute_log();
        }
    }

    /// Release everything we are holding in Windows and restore the real modifiers.
    fn neutralize(&mut self) {
        for i in 0..256 {
            match self.active[i] {
                Active::Shortcut { target, .. } | Active::Sub { target } => {
                    self.out.push(Out::Key { vk: target, down: false });
                }
                _ => {}
            }
            self.active[i] = Active::None;
        }
        // Keys that were remapped and are still held must not count as modifiers now.
        self.recompute_log();
        let want = self.log_mods;
        self.set_os_mods(want);
    }

    // ------------------------------------------------------------------ recording

    fn handle_record(&mut self, ev: KeyEvent) -> Option<bool> {
        let vk = ev.vk as usize;
        let rec = self.rec.as_mut().expect("recording");
        if rec.pre[vk] {
            if !ev.down {
                rec.pre[vk] = false;
            }
            return None; // held from before: normal handling
        }
        if ev.down {
            if !rec.any && ev.time.wrapping_sub(rec.started) > RECORD_TIMEOUT_MS {
                self.rec = None;
                self.effects.push(Effect::RecordCancelled);
                return None;
            }
            if !rec.down[vk] {
                rec.down[vk] = true;
                rec.any = true;
                if is_modifier(ev.vk) {
                    rec.mods |= mod_bit(ev.vk);
                    rec.mod_keys += 1;
                    rec.last_mod = sided(ev.vk);
                } else {
                    rec.key = Some(ev.vk);
                }
                let preview = describe(rec.mods, rec.key);
                self.effects.push(Effect::RecordUpdate { shortcut: preview });
            }
            Some(true)
        } else {
            if !rec.down[vk] {
                return None;
            }
            rec.down[vk] = false;
            if rec.down.iter().all(|d| !d) {
                let result = match rec.key {
                    Some(k) => Some(Shortcut::new(ModSpec::from_state(rec.mods), k).to_string()),
                    None if rec.mod_keys == 1 => Some(Shortcut::new(ModSpec::NONE, rec.last_mod).to_string()),
                    None => None,
                };
                self.rec = None;
                self.effects.push(Effect::RecordDone { shortcut: result });
            }
            Some(true)
        }
    }
}

/// Text for a partially-recorded shortcut ("LAlt+LShift" while only modifiers are down).
fn describe(mods: u8, key: Option<u8>) -> String {
    match key {
        Some(k) => Shortcut::new(ModSpec::from_state(mods), k).to_string(),
        None => {
            let s = Shortcut::new(ModSpec::from_state(mods), 0).to_string();
            s.trim_end_matches("+VK_00").trim_end_matches("VK_00").to_string()
        }
    }
}

#[cfg(test)]
mod tests;
