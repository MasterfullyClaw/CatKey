use super::*;
use crate::config::{self, default_rules, Action, Config, Rule, Scope};
use crate::keys::key_name;
use std::collections::HashSet;

const LALT: u8 = 0xA4;
const RALT: u8 = 0xA5;
const LCTRL: u8 = 0xA2;
const LSHIFT: u8 = 0xA0;
const LWIN: u8 = 0x5B;
const TAB: u8 = 0x09;
const CAPS: u8 = 0x14;
const INSERT: u8 = 0x2D;

#[derive(Default)]
struct Mock {
    exe: String,
    strict_async: bool,
    async_down: HashSet<u8>,
}

impl Env for Mock {
    fn foreground_exe(&mut self) -> &str {
        &self.exe
    }
    fn key_down_async(&mut self, vk: u8) -> bool {
        !self.strict_async || self.async_down.contains(&vk)
    }
}

fn r(trigger: &str, keys: &str) -> Rule {
    Rule {
        id: config::new_id(),
        enabled: true,
        name: String::new(),
        trigger: Shortcut::parse(trigger).unwrap(),
        action: Action::Keys { keys: Shortcut::parse(keys).unwrap() },
        scope: Scope::all(),
    }
}

fn with_action(trigger: &str, action: Action) -> Rule {
    Rule { action, ..r(trigger, "A") }
}

fn engine(rules: Vec<Rule>) -> Engine {
    engine_cfg(Config { rules, ..Config::default() })
}

fn engine_cfg(cfg: Config) -> Engine {
    Engine::new(Arc::new(config::compile(&cfg)), true)
}

fn fmt(out: &[Out]) -> String {
    let arrow = |d: &bool| if *d { "↓" } else { "↑" };
    out.iter()
        .map(|o| match o {
            Out::Key { vk, down } => format!("{}{}", key_name(*vk), arrow(down)),
            Out::Raw { vk, down, .. } => format!("raw({}){}", key_name(*vk), arrow(down)),
            Out::Unicode { unit, down } => format!("u'{}'{}", char::from_u32(*unit as u32).unwrap(), arrow(down)),
            Out::Dummy => "·".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Feed one event; returns (swallowed, what we sent).
fn d(e: &mut Engine, env: &mut Mock, vk: u8) -> (bool, String) {
    let s = e.handle(KeyEvent::down(vk), env);
    (s, fmt(&e.out))
}
fn u(e: &mut Engine, env: &mut Mock, vk: u8) -> (bool, String) {
    let s = e.handle(KeyEvent::up(vk), env);
    (s, fmt(&e.out))
}
fn pass() -> (bool, String) {
    (false, String::new())
}
fn swallow(s: &str) -> (bool, String) {
    (true, s.to_string())
}
fn through(s: &str) -> (bool, String) {
    (false, s.to_string())
}

#[test]
fn alt_c_becomes_ctrl_c() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · LAlt↑ C↓"));
    assert!(e.effects.iter().any(|x| matches!(x, Effect::Fired { rule: 1 })));
    assert_eq!(u(&mut e, m, b'C'), swallow("C↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
}

#[test]
fn chained_copy_paste_keeps_ctrl_down() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · LAlt↑ C↓"));
    assert_eq!(u(&mut e, m, b'C'), swallow("C↑"));
    assert_eq!(d(&mut e, m, b'V'), swallow("V↓"));
    assert_eq!(u(&mut e, m, b'V'), swallow("V↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
}

#[test]
fn alt_tab_still_works_after_a_remap() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, b'C');
    u(&mut e, m, b'C');
    assert_eq!(d(&mut e, m, TAB), swallow("LAlt↓ LCtrl↑ raw(Tab)↓"));
    assert_eq!(u(&mut e, m, TAB), pass());
    assert_eq!(d(&mut e, m, TAB), pass());
    assert_eq!(u(&mut e, m, TAB), pass());
    assert_eq!(u(&mut e, m, LALT), pass());
}

#[test]
fn alt_q_is_alt_f4() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'Q'), swallow("F4↓"));
    assert_eq!(u(&mut e, m, b'Q'), swallow("F4↑"));
    assert_eq!(u(&mut e, m, LALT), pass());
}

#[test]
fn alt_shift_n_is_ctrl_shift_n() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, LSHIFT);
    assert_eq!(d(&mut e, m, b'N'), swallow("LCtrl↓ · LAlt↑ N↓"));
    assert_eq!(u(&mut e, m, b'N'), swallow("N↑"));
    assert_eq!(u(&mut e, m, LSHIFT), through("LCtrl↑"));
    assert_eq!(u(&mut e, m, LALT), pass());
}

#[test]
fn alt_n_targets_generic_ctrl_as_left_ctrl() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'N'), swallow("LCtrl↓ · LAlt↑ N↓"));
}

#[test]
fn plain_alt_tap_and_unmapped_keys_pass_untouched() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(u(&mut e, m, LALT), pass());
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'X'), pass());
    assert_eq!(u(&mut e, m, b'X'), pass());
    assert_eq!(u(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'C'), pass());
    assert_eq!(u(&mut e, m, b'C'), pass());
}

#[test]
fn exact_match_only() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, LSHIFT);
    assert_eq!(d(&mut e, m, b'C'), pass());
}

#[test]
fn holding_repeats_the_target() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    // Alt alone auto-repeats while it is the last key pressed.
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'Z'), swallow("LCtrl↓ · LAlt↑ Z↓"));
    assert_eq!(d(&mut e, m, b'Z'), swallow("Z↓"));
    assert_eq!(d(&mut e, m, b'Z'), swallow("Z↓"));
    assert_eq!(u(&mut e, m, b'Z'), swallow("Z↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
}

fn at(e: &mut Engine, env: &mut Mock, vk: u8, down: bool, time: u32) -> (bool, String) {
    let s = e.handle(KeyEvent { vk, scan: 0, down, extended: false, time, injected: false }, env);
    (s, fmt(&e.out))
}

/// The press-twice bug: when a key's release never reaches Cat Key (Windows skipped
/// the hook for a moment, or another program ate it), the next press of that key
/// used to look like a "repeat" and was swallowed. It must work first time.
#[test]
fn a_missed_release_does_not_eat_the_next_press() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'W'), swallow("LCtrl↓ · LAlt↑ W↓"));
    // W↑ never arrives.
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'W'), swallow("W↑ LCtrl↓ · LAlt↑ W↓"));
    assert_eq!(u(&mut e, m, b'W'), swallow("W↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
}

#[test]
fn a_held_key_only_repeats_while_it_is_the_last_key() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    assert_eq!(at(&mut e, m, LALT, true, 0), pass());
    assert_eq!(at(&mut e, m, b'Q', true, 40), swallow("F4↓"));
    assert_eq!(at(&mut e, m, b'Q', true, 540), swallow("F4↓")); // real repeat
    assert_eq!(at(&mut e, m, b'Q', true, 573), swallow("F4↓"));
    // Q↑ was missed; three seconds later Q goes down again: a new press, not a repeat.
    assert_eq!(at(&mut e, m, b'Q', true, 3_600), swallow("F4↑ F4↓"));
    assert_eq!(at(&mut e, m, b'Q', false, 3_700), swallow("F4↑"));
}

#[test]
fn a_new_alt_press_after_a_missed_alt_release_starts_clean() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'Z'), swallow("LCtrl↓ · LAlt↑ Z↓"));
    assert_eq!(u(&mut e, m, b'Z'), swallow("Z↑"));
    // Alt↑ never arrives; Alt goes down again (Z was the last key, so not a repeat).
    assert_eq!(d(&mut e, m, LALT), swallow("LCtrl↑ raw(LAlt)↓"));
    assert_eq!(d(&mut e, m, b'Z'), swallow("LCtrl↓ · LAlt↑ Z↓"));
    assert_eq!(u(&mut e, m, b'Z'), swallow("Z↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
}

#[test]
fn a_missed_alt_release_that_windows_never_saw_is_released() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'X'), pass());
    assert_eq!(u(&mut e, m, b'X'), pass());
    // Alt↑ eaten by another program; Alt goes down again.
    assert_eq!(d(&mut e, m, LALT), swallow("· LAlt↑ raw(LAlt)↓"));
    assert_eq!(u(&mut e, m, LALT), pass());
}

#[test]
fn the_watchdog_can_report_a_missed_release() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'Q'), swallow("F4↓"));
    assert!(e.missed_release(b'Q'));
    assert_eq!(fmt(&e.out), "F4↑");
    assert!(!e.missed_release(b'Q'));
    assert_eq!(u(&mut e, m, b'Q'), pass());
    assert!(e.missed_release(LALT));
    assert_eq!(fmt(&e.out), "· LAlt↑");
    assert_eq!(d(&mut e, m, b'Q'), pass());
}

#[test]
fn releasing_alt_first_ends_the_shortcut() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, b'C');
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
    assert_eq!(d(&mut e, m, b'C'), swallow("")); // repeat after Alt is gone: no stray "c"
    assert_eq!(u(&mut e, m, b'C'), swallow("C↑"));
}

#[test]
fn either_side_rule_matches_right_alt() {
    let mut e = engine(vec![r("Alt+C", "Ctrl+C")]);
    let m = &mut Mock::default();
    d(&mut e, m, RALT);
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · RAlt↑ C↓"));
}

#[test]
fn capslock_to_esc_works_with_any_modifiers() {
    let mut e = engine(vec![r("CapsLock", "Esc")]);
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, CAPS), swallow("Esc↓"));
    assert_eq!(u(&mut e, m, CAPS), swallow("Esc↑"));
    d(&mut e, m, LSHIFT);
    assert_eq!(d(&mut e, m, CAPS), swallow("Esc↓"));
    assert_eq!(u(&mut e, m, CAPS), swallow("Esc↑"));
}

#[test]
fn capslock_as_ctrl_is_held_like_ctrl() {
    let mut e = engine(vec![r("CapsLock", "LCtrl"), r("LCtrl+K", "Delete")]);
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, CAPS), swallow("LCtrl↓"));
    assert_eq!(d(&mut e, m, CAPS), swallow("")); // auto-repeat
    assert_eq!(d(&mut e, m, b'C'), pass()); // Windows already sees Ctrl
    assert_eq!(u(&mut e, m, b'C'), pass());
    assert_eq!(d(&mut e, m, b'K'), swallow("LCtrl↑ Delete↓")); // rules see CapsLock as Ctrl
    assert_eq!(u(&mut e, m, b'K'), swallow("Delete↑"));
    assert_eq!(u(&mut e, m, CAPS), swallow(""));
}

#[test]
fn right_alt_to_menu_key() {
    let mut e = engine(vec![r("RAlt", "Menu")]);
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, RALT), swallow("Menu↓"));
    assert_eq!(d(&mut e, m, RALT), swallow("Menu↓"));
    assert_eq!(u(&mut e, m, RALT), swallow("Menu↑"));
}

#[test]
fn generic_alt_remap_covers_both_sides() {
    let mut e = engine(vec![r("Alt", "Menu")]);
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, LALT), swallow("Menu↓"));
    assert_eq!(u(&mut e, m, LALT), swallow("Menu↑"));
    assert_eq!(d(&mut e, m, RALT), swallow("Menu↓"));
}

#[test]
fn blocked_single_keys() {
    let mut e = engine(vec![with_action("Insert", Action::Block), with_action("LWin", Action::Block)]);
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, INSERT), swallow(""));
    assert_eq!(u(&mut e, m, INSERT), swallow(""));
    assert_eq!(d(&mut e, m, LWIN), swallow(""));
    assert_eq!(d(&mut e, m, b'E'), pass()); // Win is gone, so this is a plain "e"
    assert_eq!(u(&mut e, m, b'E'), pass());
    assert_eq!(u(&mut e, m, LWIN), swallow(""));
}

#[test]
fn per_app_rules_and_priority() {
    let mut chrome_only = r("LAlt+F", "LCtrl+L");
    chrome_only.scope = Scope::only(&["chrome.exe"]);
    let mut l_rule = r("LAlt+L", "LCtrl+L");
    l_rule.scope = Scope::only(&["chrome.exe"]);
    let mut e = engine(vec![r("LAlt+F", "LCtrl+F"), chrome_only, l_rule]);
    let m = &mut Mock { exe: "code.exe".into(), ..Default::default() };
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'L'), pass());
    u(&mut e, m, b'L');
    assert_eq!(d(&mut e, m, b'F'), swallow("LCtrl↓ · LAlt↑ F↓"));
    u(&mut e, m, b'F');
    u(&mut e, m, LALT);
    m.exe = "chrome.exe".into();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'F'), swallow("LCtrl↓ · LAlt↑ L↓"));
    u(&mut e, m, b'F');
    assert_eq!(d(&mut e, m, b'L'), swallow("L↓"));
}

#[test]
fn except_scope() {
    let mut rule = r("LAlt+W", "LCtrl+W");
    rule.scope = Scope { mode: config::ScopeMode::Except, apps: vec!["code.exe".into()] };
    let mut e = engine(vec![rule]);
    let m = &mut Mock { exe: "code.exe".into(), ..Default::default() };
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'W'), pass());
    u(&mut e, m, b'W');
    m.exe = "notepad.exe".into();
    assert_eq!(d(&mut e, m, b'W'), swallow("LCtrl↓ · LAlt↑ W↓"));
}

#[test]
fn nap_zones_disable_everything() {
    let mut cfg = Config::default();
    cfg.settings.excluded_apps = vec!["game.exe".into()];
    let mut e = engine_cfg(cfg);
    let m = &mut Mock { exe: "game.exe".into(), ..Default::default() };
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'C'), pass());
}

#[test]
fn launch_action_taps_dummy_so_alt_menu_stays_closed() {
    let mut e = engine(vec![with_action("LAlt+T", Action::Launch { target: "wt.exe".into(), args: String::new() })]);
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'T'), swallow("·"));
    assert!(e.effects.iter().any(|x| matches!(x, Effect::Launch { target, .. } if &**target == "wt.exe")));
    assert_eq!(d(&mut e, m, b'T'), swallow("")); // holding does not launch twice
    assert_eq!(u(&mut e, m, b'T'), swallow(""));
    assert_eq!(u(&mut e, m, LALT), pass());
}

#[test]
fn text_action_releases_modifiers_first() {
    let mut e = engine(vec![with_action("LAlt+M", Action::Text { text: "hi\n".into() })]);
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'M'), swallow("· LAlt↑ u'h'↓ u'h'↑ u'i'↓ u'i'↑ Enter↓ Enter↑"));
    assert_eq!(u(&mut e, m, b'M'), swallow(""));
    assert_eq!(u(&mut e, m, LALT), pass());
}

#[test]
fn win_d_blocked_does_not_open_start() {
    let mut e = engine(vec![with_action("LWin+D", Action::Block)]);
    let m = &mut Mock::default();
    d(&mut e, m, LWIN);
    assert_eq!(d(&mut e, m, b'D'), swallow("·"));
    assert_eq!(u(&mut e, m, b'D'), swallow(""));
    assert_eq!(u(&mut e, m, LWIN), pass());
}

#[test]
fn pause_hotkey_toggles_everything() {
    let mut cfg = Config::default();
    cfg.settings.pause_hotkey = Some(Shortcut::parse("LCtrl+LAlt+P").unwrap());
    let mut e = engine_cfg(cfg);
    let m = &mut Mock::default();
    d(&mut e, m, LCTRL);
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'P'), swallow("·"));
    assert!(e.effects.contains(&Effect::Toggled { enabled: false }));
    assert_eq!(u(&mut e, m, b'P'), swallow(""));
    u(&mut e, m, LCTRL);
    assert!(!e.enabled());
    assert_eq!(d(&mut e, m, b'C'), pass()); // napping: Alt+C is just Alt+C
    u(&mut e, m, b'C');
    d(&mut e, m, LCTRL);
    assert_eq!(d(&mut e, m, b'P'), swallow("·"));
    assert!(e.effects.contains(&Effect::Toggled { enabled: true }));
    u(&mut e, m, b'P');
    u(&mut e, m, LCTRL);
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · LAlt↑ C↓"));
}

#[test]
fn pausing_mid_shortcut_puts_windows_back() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, b'C');
    e.set_enabled(false);
    assert_eq!(fmt(&e.out), "C↑ LAlt↓ LCtrl↑");
    assert_eq!(u(&mut e, m, b'C'), pass());
    assert_eq!(u(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'C'), pass());
}

#[test]
fn missed_win_release_is_detected() {
    let mut e = engine(default_rules());
    let m = &mut Mock { strict_async: true, ..Default::default() };
    m.async_down.insert(LWIN);
    d(&mut e, m, LWIN);
    d(&mut e, m, b'L'); // Win+L locks the PC; we never see the Win release
    m.async_down.clear();
    m.async_down.insert(LALT);
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · LAlt↑ C↓"));
}

#[test]
fn hard_reset_releases_what_we_hold() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, b'C');
    e.hard_reset();
    assert_eq!(fmt(&e.out), "C↑ LAlt↓ LCtrl↑ · LAlt↑");
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · LAlt↑ C↓"));
}

#[test]
fn recording_captures_and_swallows() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    e.start_recording(0);
    assert_eq!(d(&mut e, m, LALT), swallow(""));
    assert!(e.effects.contains(&Effect::RecordUpdate { shortcut: "LAlt".into() }));
    assert_eq!(d(&mut e, m, LSHIFT), swallow(""));
    assert_eq!(d(&mut e, m, b'K'), swallow(""));
    assert!(e.effects.contains(&Effect::RecordUpdate { shortcut: "LAlt+LShift+K".into() }));
    assert_eq!(u(&mut e, m, b'K'), swallow(""));
    assert_eq!(u(&mut e, m, LSHIFT), swallow(""));
    assert_eq!(u(&mut e, m, LALT), swallow(""));
    assert!(e.effects.contains(&Effect::RecordDone { shortcut: Some("LAlt+LShift+K".into()) }));
    assert!(!e.is_recording());
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'C'), swallow("LCtrl↓ · LAlt↑ C↓"));
}

#[test]
fn recording_a_lone_modifier_and_held_keys() {
    let mut e = engine(vec![]);
    let m = &mut Mock::default();
    e.start_recording(0);
    d(&mut e, m, RALT);
    u(&mut e, m, RALT);
    assert!(e.effects.contains(&Effect::RecordDone { shortcut: Some("RAlt".into()) }));

    // A key already held when recording starts is not part of it.
    d(&mut e, m, LSHIFT);
    e.start_recording(0);
    assert_eq!(u(&mut e, m, LSHIFT), pass());
    assert_eq!(d(&mut e, m, b'X'), swallow(""));
    assert_eq!(u(&mut e, m, b'X'), swallow(""));
    assert!(e.effects.contains(&Effect::RecordDone { shortcut: Some("X".into()) }));

    // Two modifiers and no key: not usable.
    e.start_recording(0);
    d(&mut e, m, LCTRL);
    d(&mut e, m, LSHIFT);
    u(&mut e, m, LCTRL);
    u(&mut e, m, LSHIFT);
    assert!(e.effects.contains(&Effect::RecordDone { shortcut: None }));
}

#[test]
fn recording_times_out_if_nothing_pressed() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    e.start_recording(1_000);
    let s = e.handle(KeyEvent { vk: b'A', scan: 0, down: true, extended: false, time: 1_000 + RECORD_TIMEOUT_MS + 1, injected: false }, m);
    assert!(!s);
    assert!(e.effects.contains(&Effect::RecordCancelled));
    assert!(!e.is_recording());
}

#[test]
fn packet_and_dummy_keys_are_ignored() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, VK_PACKET), pass());
    assert_eq!(d(&mut e, m, DUMMY_VK), pass());
}

#[test]
fn config_swap_keeps_held_keys_consistent() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT);
    d(&mut e, m, b'C');
    e.set_compiled(Arc::new(config::compile(&Config { rules: vec![], ..Config::default() })));
    assert_eq!(u(&mut e, m, b'C'), swallow("C↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'C'), pass());
}

#[test]
fn swapped_pair_works_both_ways() {
    // Alt+W -> Ctrl+W plus its reverse, Ctrl+W -> Alt+W (what the swap button makes).
    let mut e = engine(vec![r("LAlt+W", "LCtrl+W"), r("LCtrl+W", "LAlt+W")]);
    let m = &mut Mock::default();
    d(&mut e, m, LCTRL);
    assert_eq!(d(&mut e, m, b'W'), swallow("LAlt↓ LCtrl↑ W↓"));
    assert_eq!(u(&mut e, m, b'W'), swallow("W↑"));
    // The tap before Alt goes up keeps the Alt menu closed; nothing is left held.
    assert_eq!(u(&mut e, m, LCTRL).1, "· LAlt↑");
    assert_eq!(e.os_mods, 0);
    // The original direction still works right after.
    d(&mut e, m, LALT);
    assert_eq!(d(&mut e, m, b'W'), swallow("LCtrl↓ · LAlt↑ W↓"));
    u(&mut e, m, b'W');
    u(&mut e, m, LALT);
    assert_eq!(e.os_mods, 0);
}

#[test]
fn every_remap_presses_before_it_releases_alt() {
    // A key must go down while Alt is still held, or apps see a lone Alt tap
    // (menu bar, access-key letters, a "ding" on the next key).
    for rule in default_rules() {
        let mut e = engine(default_rules());
        let m = &mut Mock::default();
        let t = rule.trigger;
        let mut held = vec![];
        for vk in [LALT, LSHIFT] {
            let bit = crate::keys::mod_bit(vk);
            if t.mods.to_state() & bit != 0 {
                d(&mut e, m, vk);
                held.push(vk);
            }
        }
        let (swallowed, out) = d(&mut e, m, t.key);
        assert!(swallowed, "{}", rule.name);
        let first_release = out.find('↑');
        let first_press = out.find('↓');
        if let Some(r) = first_release {
            assert!(first_press.is_some_and(|p| p < r), "{}: {out}", rule.name);
            assert!(!out.starts_with('·') && !out.starts_with("LAlt↑"), "{}: {out}", rule.name);
        }
    }
}

#[test]
fn keys_typed_by_other_programs_are_not_remapped() {
    // Even with a reversed rule, a password manager's / PowerToys' Ctrl+W stays Ctrl+W.
    let mut e = engine(vec![r("LCtrl+W", "LAlt+W"), r("CapsLock", "Esc")]);
    let m = &mut Mock::default();
    let inj = |vk: u8, down: bool| if down { KeyEvent::down(vk).injected() } else { KeyEvent::up(vk).injected() };
    assert!(!e.handle(inj(LCTRL, true), m));
    assert!(!e.handle(inj(b'W', true), m));
    assert!(e.out.is_empty());
    assert!(!e.handle(inj(b'W', false), m));
    assert!(!e.handle(inj(LCTRL, false), m));
    assert!(!e.handle(inj(CAPS, true), m));
    assert!(e.out.is_empty());
    assert!(!e.handle(inj(CAPS, false), m));
    // A real press still works afterwards.
    d(&mut e, m, LCTRL);
    assert_eq!(d(&mut e, m, b'W'), swallow("LAlt↓ LCtrl↑ W↓"));
}

#[test]
fn powertoys_doing_the_same_remap_adds_nothing() {
    // PowerToys first in line turns Alt+W into Ctrl+W by itself; Cat Key must only watch.
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    d(&mut e, m, LALT); // physical
    for (vk, down) in [(0xFF, true), (0xFF, false), (LALT, false), (LCTRL, true), (b'W', true), (b'W', false)] {
        let ev = if down { KeyEvent::down(vk) } else { KeyEvent::up(vk) }.injected();
        assert!(!e.handle(ev, m), "vk {vk:#x}");
        assert!(e.out.is_empty(), "vk {vk:#x}: {}", fmt(&e.out));
    }
    // Physical Alt up, then PowerToys lets go of its Ctrl.
    let (_, out) = u(&mut e, m, LALT);
    assert!(!out.contains('↓'), "{out}");
    e.handle(KeyEvent::up(LCTRL).injected(), m);
    assert_eq!(e.os_mods, 0);
    assert_eq!(e.log_mods, 0);
}

/// AltGr sends a made-up left Ctrl (scan 0x21D) before every Right Alt, repeats
/// included. Holding AltGr must not look like missed releases.
#[test]
fn altgr_repeats_are_still_repeats() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    let ev = |vk: u8, scan: u16, time: u32| KeyEvent { vk, scan, down: true, extended: vk == RALT, time, injected: false };
    assert!(!e.handle(ev(LCTRL, 0x21D, 0), m));
    assert!(!e.handle(ev(RALT, 0x38, 0), m));
    for t in [500, 533, 566] {
        assert!(!e.handle(ev(LCTRL, 0x21D, t), m), "fake Ctrl repeat passes");
        assert!(e.out.is_empty());
        assert!(!e.handle(ev(RALT, 0x38, t), m), "AltGr repeat passes");
        assert!(e.out.is_empty());
    }
}

/// "Pressed together", W landing a hair before Alt: Windows sees W, then a lone Alt
/// tap, and opens the app's menu (the next shortcut then dings). Mask that release.
#[test]
fn alt_pressed_just_after_a_key_does_not_open_the_menu() {
    let mut e = engine(default_rules());
    let m = &mut Mock::default();
    assert_eq!(d(&mut e, m, b'W'), pass());
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(u(&mut e, m, b'W'), pass());
    assert_eq!(u(&mut e, m, LALT), swallow("· raw(LAlt)↑"));
    // A real lone Alt tap still opens menus.
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(u(&mut e, m, LALT), pass());
    // Holding W, then Alt, then pressing W again: the shortcut works.
    assert_eq!(d(&mut e, m, b'W'), pass());
    assert_eq!(d(&mut e, m, LALT), pass());
    assert_eq!(u(&mut e, m, b'W'), pass());
    assert_eq!(d(&mut e, m, b'W'), swallow("LCtrl↓ · LAlt↑ W↓"));
    assert_eq!(u(&mut e, m, b'W'), swallow("W↑"));
    assert_eq!(u(&mut e, m, LALT), through("LCtrl↑"));
}
