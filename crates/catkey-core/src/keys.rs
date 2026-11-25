//! Virtual-key codes, modifier bits and the text format used in config files
//! (for example `"LAlt+LShift+N"`).
//!
//! Modifiers are tracked two ways:
//! * [`ModSpec`] is what a *rule* asks for: per family (Ctrl, Shift, Alt, Win)
//!   none / left / right / either side.
//! * a plain `u8` bit set (see [`bit`]) is what is *held down* right now.

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Bits of a modifier *state* (what is held down).
pub mod bit {
    pub const LCTRL: u8 = 1 << 0;
    pub const RCTRL: u8 = 1 << 1;
    pub const LSHIFT: u8 = 1 << 2;
    pub const RSHIFT: u8 = 1 << 3;
    pub const LALT: u8 = 1 << 4;
    pub const RALT: u8 = 1 << 5;
    pub const LWIN: u8 = 1 << 6;
    pub const RWIN: u8 = 1 << 7;
    /// Alt and Win: releasing these on their own opens the menu bar / Start menu.
    pub const ALT_WIN: u8 = LALT | RALT | LWIN | RWIN;
    pub const ALL: [u8; 8] = [LCTRL, RCTRL, LSHIFT, RSHIFT, LALT, RALT, LWIN, RWIN];
}

pub const VK_SHIFT: u8 = 0x10;
pub const VK_CONTROL: u8 = 0x11;
pub const VK_MENU: u8 = 0x12;
pub const VK_LWIN: u8 = 0x5B;
pub const VK_RWIN: u8 = 0x5C;
pub const VK_LSHIFT: u8 = 0xA0;
pub const VK_RSHIFT: u8 = 0xA1;
pub const VK_LCONTROL: u8 = 0xA2;
pub const VK_RCONTROL: u8 = 0xA3;
pub const VK_LMENU: u8 = 0xA4;
pub const VK_RMENU: u8 = 0xA5;
pub const VK_RETURN: u8 = 0x0D;
pub const VK_TAB: u8 = 0x09;
/// Produced by `SendInput` with `KEYEVENTF_UNICODE`; never remapped.
pub const VK_PACKET: u8 = 0xE7;
/// Unassigned virtual key. Tapping it while Alt/Win is down stops Windows from
/// treating a later Alt/Win release as "pressed alone" (menu bar / Start menu).
pub const DUMMY_VK: u8 = 0xE8;

/// Every virtual key that is a modifier (sided first, then the generic codes).
pub const MOD_VKS: [u8; 11] = [
    VK_LSHIFT, VK_RSHIFT, VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN, VK_SHIFT,
    VK_CONTROL, VK_MENU,
];

pub fn is_modifier(vk: u8) -> bool {
    matches!(vk, VK_SHIFT | VK_CONTROL | VK_MENU | 0xA0..=0xA5 | VK_LWIN | VK_RWIN)
}

/// State bit for a modifier key. Generic codes (VK_SHIFT…) count as the left key.
pub fn mod_bit(vk: u8) -> u8 {
    match vk {
        VK_SHIFT | VK_LSHIFT => bit::LSHIFT,
        VK_RSHIFT => bit::RSHIFT,
        VK_CONTROL | VK_LCONTROL => bit::LCTRL,
        VK_RCONTROL => bit::RCTRL,
        VK_MENU | VK_LMENU => bit::LALT,
        VK_RMENU => bit::RALT,
        VK_LWIN => bit::LWIN,
        VK_RWIN => bit::RWIN,
        _ => 0,
    }
}

/// The key to press for a single state bit.
pub fn bit_vk(b: u8) -> u8 {
    match b {
        bit::LCTRL => VK_LCONTROL,
        bit::RCTRL => VK_RCONTROL,
        bit::LSHIFT => VK_LSHIFT,
        bit::RSHIFT => VK_RSHIFT,
        bit::LALT => VK_LMENU,
        bit::RALT => VK_RMENU,
        bit::LWIN => VK_LWIN,
        bit::RWIN => VK_RWIN,
        _ => 0,
    }
}

/// Generic modifier codes become their left-hand key so we always send a real key.
pub fn sided(vk: u8) -> u8 {
    match vk {
        VK_SHIFT => VK_LSHIFT,
        VK_CONTROL => VK_LCONTROL,
        VK_MENU => VK_LMENU,
        other => other,
    }
}

/// (left bit, right bit) per family, in [`Family`] order.
pub const FAMILY_BITS: [(u8, u8); 4] = [
    (bit::LCTRL, bit::RCTRL),
    (bit::LSHIFT, bit::RSHIFT),
    (bit::LALT, bit::RALT),
    (bit::LWIN, bit::RWIN),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Ctrl = 0,
    Shift = 1,
    Alt = 2,
    Win = 3,
}

/// Which side of a modifier family a rule asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    None = 0,
    Left = 1,
    Right = 2,
    Any = 3,
}

impl Side {
    fn from_bits(v: u8) -> Side {
        match v & 3 {
            1 => Side::Left,
            2 => Side::Right,
            3 => Side::Any,
            _ => Side::None,
        }
    }
}

/// Modifiers a rule asks for: two bits per family (Ctrl, Shift, Alt, Win).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ModSpec(pub u8);

impl ModSpec {
    pub const NONE: ModSpec = ModSpec(0);

    pub fn side(self, f: Family) -> Side {
        Side::from_bits(self.0 >> ((f as u8) * 2))
    }

    pub fn with(self, f: Family, s: Side) -> ModSpec {
        let shift = (f as u8) * 2;
        ModSpec((self.0 & !(3 << shift)) | ((s as u8) << shift))
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Can one key press satisfy both specs? ("either Alt" meets "left Alt", "left" never meets "right")
    pub fn overlaps(self, other: ModSpec) -> bool {
        (0..4u8).all(|f| {
            match (Side::from_bits(self.0 >> (f * 2)), Side::from_bits(other.0 >> (f * 2))) {
                (Side::None, Side::None) => true,
                (Side::None, _) | (_, Side::None) => false,
                (Side::Any, _) | (_, Side::Any) => true,
                (a, b) => a == b,
            }
        })
    }

    /// Exact match: every family must be exactly as the rule says, nothing extra held.
    pub fn matches(self, state: u8) -> bool {
        for f in 0..4u8 {
            let (lb, rb) = FAMILY_BITS[f as usize];
            let l = state & lb != 0;
            let r = state & rb != 0;
            let ok = match Side::from_bits(self.0 >> (f * 2)) {
                Side::None => !l && !r,
                Side::Left => l && !r,
                Side::Right => r && !l,
                Side::Any => l || r,
            };
            if !ok {
                return false;
            }
        }
        true
    }

    /// The modifier keys to hold down to produce this spec ("either" uses the left key).
    pub fn to_state(self) -> u8 {
        let mut s = 0;
        for f in 0..4u8 {
            let (lb, rb) = FAMILY_BITS[f as usize];
            match Side::from_bits(self.0 >> (f * 2)) {
                Side::Left | Side::Any => s |= lb,
                Side::Right => s |= rb,
                Side::None => {}
            }
        }
        s
    }

    /// Spec that matches exactly what is held (both sides held = either side).
    pub fn from_state(state: u8) -> ModSpec {
        let mut m = ModSpec::NONE;
        for (i, f) in [Family::Ctrl, Family::Shift, Family::Alt, Family::Win].into_iter().enumerate() {
            let (lb, rb) = FAMILY_BITS[i];
            let side = match (state & lb != 0, state & rb != 0) {
                (true, false) => Side::Left,
                (false, true) => Side::Right,
                (true, true) => Side::Any,
                (false, false) => Side::None,
            };
            m = m.with(f, side);
        }
        m
    }
}

/// Canonical key names used in config files. The UI has the same table (keys.js).
const KEY_NAMES: &[(u8, &str)] = &[
    (0x08, "Backspace"),
    (0x09, "Tab"),
    (0x0C, "Clear"),
    (0x0D, "Enter"),
    (0x10, "Shift"),
    (0x11, "Ctrl"),
    (0x12, "Alt"),
    (0x13, "Pause"),
    (0x14, "CapsLock"),
    (0x15, "Kana"),
    (0x19, "Kanji"),
    (0x1B, "Esc"),
    (0x1C, "Convert"),
    (0x1D, "NonConvert"),
    (0x20, "Space"),
    (0x21, "PageUp"),
    (0x22, "PageDown"),
    (0x23, "End"),
    (0x24, "Home"),
    (0x25, "Left"),
    (0x26, "Up"),
    (0x27, "Right"),
    (0x28, "Down"),
    (0x29, "Select"),
    (0x2A, "Print"),
    (0x2B, "Execute"),
    (0x2C, "PrintScreen"),
    (0x2D, "Insert"),
    (0x2E, "Delete"),
    (0x2F, "Help"),
    (0x5B, "LWin"),
    (0x5C, "RWin"),
    (0x5D, "Menu"),
    (0x5F, "Sleep"),
    (0x6A, "NumMultiply"),
    (0x6B, "NumAdd"),
    (0x6C, "NumSeparator"),
    (0x6D, "NumSubtract"),
    (0x6E, "NumDecimal"),
    (0x6F, "NumDivide"),
    (0x90, "NumLock"),
    (0x91, "ScrollLock"),
    (0xA0, "LShift"),
    (0xA1, "RShift"),
    (0xA2, "LCtrl"),
    (0xA3, "RCtrl"),
    (0xA4, "LAlt"),
    (0xA5, "RAlt"),
    (0xA6, "BrowserBack"),
    (0xA7, "BrowserForward"),
    (0xA8, "BrowserRefresh"),
    (0xA9, "BrowserStop"),
    (0xAA, "BrowserSearch"),
    (0xAB, "BrowserFavorites"),
    (0xAC, "BrowserHome"),
    (0xAD, "VolumeMute"),
    (0xAE, "VolumeDown"),
    (0xAF, "VolumeUp"),
    (0xB0, "MediaNext"),
    (0xB1, "MediaPrev"),
    (0xB2, "MediaStop"),
    (0xB3, "MediaPlayPause"),
    (0xB4, "LaunchMail"),
    (0xB5, "LaunchMedia"),
    (0xB6, "LaunchApp1"),
    (0xB7, "LaunchApp2"),
    (0xBA, "Semicolon"),
    (0xBB, "Equals"),
    (0xBC, "Comma"),
    (0xBD, "Minus"),
    (0xBE, "Period"),
    (0xBF, "Slash"),
    (0xC0, "Backtick"),
    (0xDB, "LBracket"),
    (0xDC, "Backslash"),
    (0xDD, "RBracket"),
    (0xDE, "Quote"),
    (0xDF, "Oem8"),
    (0xE2, "IntlBackslash"),
];

/// Canonical name of a virtual key (falls back to `VK_xx`).
pub fn key_name(vk: u8) -> String {
    match vk {
        0x30..=0x39 | 0x41..=0x5A => (vk as char).to_string(),
        0x60..=0x69 => format!("Num{}", vk - 0x60),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        _ => KEY_NAMES
            .iter()
            .find(|(v, _)| *v == vk)
            .map(|(_, n)| n.to_string())
            .unwrap_or_else(|| format!("VK_{:02X}", vk)),
    }
}

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, ' ' | '(' | ')' | '_' | '-') || s.len() == 1)
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Parse a key name (case-insensitive, with friendly aliases).
pub fn parse_key(token: &str) -> Option<u8> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    // Single printable character.
    if t.chars().count() == 1 {
        let c = t.chars().next().unwrap().to_ascii_uppercase();
        return match c {
            'A'..='Z' | '0'..='9' => Some(c as u8),
            ';' => Some(0xBA),
            '=' => Some(0xBB),
            ',' => Some(0xBC),
            '-' => Some(0xBD),
            '.' => Some(0xBE),
            '/' => Some(0xBF),
            '`' => Some(0xC0),
            '[' => Some(0xDB),
            '\\' => Some(0xDC),
            ']' => Some(0xDD),
            '\'' => Some(0xDE),
            _ => None,
        };
    }
    let n = norm(t);
    if let Some(hex) = n.strip_prefix("vk").filter(|h| h.len() == 2 && h.chars().all(|c| c.is_ascii_hexdigit())) {
        return u8::from_str_radix(hex, 16).ok().filter(|v| *v != 0);
    }
    if let Some(num) = n.strip_prefix('f').and_then(|d| d.parse::<u8>().ok()) {
        if (1..=24).contains(&num) {
            return Some(0x6F + num);
        }
    }
    for prefix in ["numpad", "num"] {
        if let Some(d) = n.strip_prefix(prefix).and_then(|d| d.parse::<u8>().ok()) {
            if d <= 9 {
                return Some(0x60 + d);
            }
        }
    }
    let alias = match n.as_str() {
        "escape" => Some(0x1B),
        "return" => Some(0x0D),
        "back" | "bksp" => Some(0x08),
        "del" => Some(0x2E),
        "ins" => Some(0x2D),
        "pgup" | "prior" => Some(0x21),
        "pgdn" | "pgdown" | "next" => Some(0x22),
        "caps" | "capital" => Some(0x14),
        "spacebar" => Some(0x20),
        "prtsc" | "prtscn" | "snapshot" => Some(0x2C),
        "apps" | "contextmenu" | "application" => Some(0x5D),
        "control" => Some(VK_CONTROL),
        "win" | "windows" | "meta" | "super" | "leftwin" | "winleft" => Some(VK_LWIN),
        "rightwin" | "winright" => Some(VK_RWIN),
        "leftshift" | "shiftleft" => Some(VK_LSHIFT),
        "rightshift" | "shiftright" => Some(VK_RSHIFT),
        "leftctrl" | "ctrlleft" | "lcontrol" | "leftcontrol" | "controlleft" => Some(VK_LCONTROL),
        "rightctrl" | "ctrlright" | "rcontrol" | "rightcontrol" | "controlright" => Some(VK_RCONTROL),
        "leftalt" | "altleft" | "lmenu" => Some(VK_LMENU),
        "rightalt" | "altright" | "rmenu" | "altgr" => Some(VK_RMENU),
        "grave" | "tilde" | "backquote" => Some(0xC0),
        "plus" | "equal" => Some(0xBB),
        "dash" | "hyphen" => Some(0xBD),
        "dot" => Some(0xBE),
        "apostrophe" => Some(0xDE),
        "openbracket" | "bracketleft" => Some(0xDB),
        "closebracket" | "bracketright" => Some(0xDD),
        "scroll" => Some(0x91),
        "break" => Some(0x13),
        "mute" => Some(0xAD),
        "playpause" => Some(0xB3),
        "nexttrack" => Some(0xB0),
        "prevtrack" | "previoustrack" => Some(0xB1),
        _ => None,
    };
    if alias.is_some() {
        return alias;
    }
    KEY_NAMES
        .iter()
        .find(|(_, name)| norm(name) == n)
        .map(|(v, _)| *v)
}

/// Modifier token -> (family, side). Accepts "LAlt", "Alt", "Alt (Left)", "LeftAlt"…
fn parse_mod(token: &str) -> Option<(Family, Side)> {
    let n = norm(token);
    let (fam, rest): (Family, String) = {
        let mut found = None;
        for (f, names) in [
            (Family::Ctrl, &["control", "ctrl"][..]),
            (Family::Shift, &["shift"][..]),
            (Family::Alt, &["alt"][..]),
            (Family::Win, &["windows", "win", "meta", "super"][..]),
        ] {
            for name in names {
                if let Some(pos) = n.find(name) {
                    let rest = format!("{}{}", &n[..pos], &n[pos + name.len()..]);
                    found = Some((f, rest));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        found?
    };
    let side = match rest.as_str() {
        "" | "any" | "either" => Side::Any,
        "l" | "left" => Side::Left,
        "r" | "right" => Side::Right,
        _ => return None,
    };
    Some((fam, side))
}

/// A key plus the modifiers held with it, e.g. `LAlt+C`, or a single key (`CapsLock`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub mods: ModSpec,
    pub key: u8,
}

impl Shortcut {
    pub fn new(mods: ModSpec, key: u8) -> Self {
        Shortcut { mods, key }
    }

    /// Would one key press fire both shortcuts?
    pub fn overlaps(&self, other: &Shortcut) -> bool {
        self.key == other.key && self.mods.overlaps(other.mods)
    }

    pub fn parse(s: &str) -> Result<Shortcut, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty shortcut".into());
        }
        // Split on '+', but a trailing "++" means the last key is "+" itself (not supported -> Equals).
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.iter().any(|p| p.is_empty()) {
            return Err(format!("can't read shortcut \"{s}\""));
        }
        let (last, mods) = parts.split_last().unwrap();
        let mut spec = ModSpec::NONE;
        for m in mods {
            let (f, side) = parse_mod(m).ok_or_else(|| format!("\"{m}\" is not a modifier (Ctrl, Shift, Alt, Win)"))?;
            spec = spec.with(f, side);
        }
        let key = parse_key(last).ok_or_else(|| format!("unknown key \"{last}\""))?;
        Ok(Shortcut { mods: spec, key })
    }

    /// Is this a single key with no modifiers?
    pub fn is_single(&self) -> bool {
        self.mods.is_empty()
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display order follows Windows: Win, Ctrl, Alt, Shift.
        let order = [
            (Family::Win, "Win"),
            (Family::Ctrl, "Ctrl"),
            (Family::Alt, "Alt"),
            (Family::Shift, "Shift"),
        ];
        let mut first = true;
        for (fam, name) in order {
            let s = match self.mods.side(fam) {
                Side::None => continue,
                Side::Left => format!("L{name}"),
                Side::Right => format!("R{name}"),
                Side::Any => name.to_string(),
            };
            if !first {
                f.write_str("+")?;
            }
            f.write_str(&s)?;
            first = false;
        }
        if !first {
            f.write_str("+")?;
        }
        f.write_str(&key_name(self.key))
    }
}

impl Serialize for Shortcut {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Shortcut {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Shortcut::parse(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_aliases() {
        for s in ["LAlt+C", "LAlt+LShift+N", "Ctrl+Shift+Esc", "LAlt+F4", "CapsLock", "RAlt", "Win+E", "LCtrl+Slash"] {
            let sc = Shortcut::parse(s).unwrap();
            assert_eq!(Shortcut::parse(&sc.to_string()).unwrap(), sc, "{s}");
        }
        assert_eq!(Shortcut::parse("Alt (Left) + Shift (Left) + N").unwrap().to_string(), "LAlt+LShift+N");
        assert_eq!(Shortcut::parse("ctrl+/").unwrap().to_string(), "Ctrl+Slash");
        assert_eq!(Shortcut::parse("lalt+q").unwrap().to_string(), "LAlt+Q");
        assert_eq!(Shortcut::parse("Escape").unwrap().key, 0x1B);
        assert_eq!(Shortcut::parse("Num7").unwrap().key, 0x67);
        assert_eq!(Shortcut::parse("F13").unwrap().key, 0x7C);
        assert_eq!(Shortcut::parse("VK_E9").unwrap().key, 0xE9);
        assert!(Shortcut::parse("Hyper+C").is_err());
        assert!(Shortcut::parse("Alt+").is_err());
    }

    #[test]
    fn exact_matching() {
        let lalt = Shortcut::parse("LAlt+C").unwrap().mods;
        assert!(lalt.matches(bit::LALT));
        assert!(!lalt.matches(bit::RALT));
        assert!(!lalt.matches(bit::LALT | bit::LSHIFT));
        let any_alt = Shortcut::parse("Alt+C").unwrap().mods;
        assert!(any_alt.matches(bit::RALT));
        assert!(any_alt.matches(bit::LALT));
        assert!(!any_alt.matches(0));
        assert_eq!(ModSpec::from_state(bit::LALT | bit::LSHIFT), Shortcut::parse("LAlt+LShift+A").unwrap().mods);
        assert_eq!(Shortcut::parse("Ctrl+N").unwrap().mods.to_state(), bit::LCTRL);
    }
}
