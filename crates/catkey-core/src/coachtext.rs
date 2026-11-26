//! Reading shortcuts out of what apps call their commands (for the shortcut coach).

/// Common Windows commands, for apps that don't say their shortcut themselves.
pub const COMMON: &[(&str, &str)] = &[
    ("copy", "Ctrl+C"),
    ("cut", "Ctrl+X"),
    ("paste", "Ctrl+V"),
    ("undo", "Ctrl+Z"),
    ("redo", "Ctrl+Y"),
    ("save", "Ctrl+S"),
    ("open", "Ctrl+O"),
    ("new", "Ctrl+N"),
    ("print", "Ctrl+P"),
    ("find", "Ctrl+F"),
    ("replace", "Ctrl+H"),
    ("select all", "Ctrl+A"),
    ("bold", "Ctrl+B"),
    ("italic", "Ctrl+I"),
    ("underline", "Ctrl+U"),
    ("zoom in", "Ctrl+Plus"),
    ("zoom out", "Ctrl+Minus"),
    ("new tab", "Ctrl+T"),
    ("new window", "Ctrl+N"),
    ("refresh", "F5"),
    ("reload", "F5"),
    ("back", "Alt+Left"),
    ("forward", "Alt+Right"),
    ("rename", "F2"),
    ("properties", "Alt+Enter"),
    ("new folder", "Ctrl+Shift+N"),
    ("downloads", "Ctrl+J"),
];

/// The shortcut for a command name most Windows apps share ("copy" → "Ctrl+C").
pub fn common_keys(name_lower: &str) -> Option<&'static str> {
    COMMON.iter().find(|(n, _)| *n == name_lower).map(|(_, k)| *k)
}

/// "&Copy\tCtrl+C" → "Copy"; "Back to Documents (Alt + Left Arrow)" → "Back to Documents".
pub fn clean_name(name: &str) -> String {
    let mut s = name.split('\t').next().unwrap_or("").replace('&', "");
    if let Some(i) = s.find(" (") {
        if s.ends_with(')') {
            s.truncate(i);
        }
    }
    s.trim().trim_end_matches("...").trim_end_matches('…').trim().to_string()
}

/// A shortcut written inside a name or tooltip: "(Ctrl+Shift+N)", "\tCtrl+C".
pub fn keys_in(text: &str) -> Option<String> {
    if let Some((_, after)) = text.split_once('\t') {
        if looks_like_keys(after) {
            return Some(tidy_keys(after));
        }
    }
    let mut rest = text;
    while let Some(i) = rest.find('(') {
        let tail = &rest[i + 1..];
        let Some(j) = tail.find(')') else { break };
        let inner = &tail[..j];
        if looks_like_keys(inner) {
            return Some(tidy_keys(inner));
        }
        rest = &tail[j + 1..];
    }
    None
}

pub fn looks_like_keys(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.len() > 40 || s.contains(", ") {
        return false; // "Alt, H, B" is an access-key sequence, not a shortcut
    }
    let lower = s.to_lowercase();
    let has_mod = ["ctrl", "alt", "shift", "win"].iter().any(|m| lower.contains(&format!("{m}+")) || lower.contains(&format!("{m} +")));
    let fkey = lower.len() <= 3 && lower.starts_with('f') && lower[1..].chars().all(|c| c.is_ascii_digit()) && lower.len() > 1;
    has_mod || fkey
}

pub fn tidy_keys(s: &str) -> String {
    s.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()).collect::<Vec<_>>().join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_keys_are_read_from_labels() {
        assert_eq!(clean_name("&Copy\tCtrl+C"), "Copy");
        assert_eq!(clean_name("Back to Documents (Alt + Left Arrow)"), "Back to Documents");
        assert_eq!(clean_name("Save As..."), "Save As");
        assert_eq!(keys_in("&Copy\tCtrl+C").as_deref(), Some("Ctrl+C"));
        assert_eq!(keys_in("Back to Documents (Alt + Left Arrow)").as_deref(), Some("Alt+Left Arrow"));
        assert_eq!(keys_in("New folder (Ctrl+Shift+N)").as_deref(), Some("Ctrl+Shift+N"));
        assert_eq!(keys_in("Refresh (F5)").as_deref(), Some("F5"));
        assert_eq!(keys_in("Photos (2)"), None);
        assert!(!looks_like_keys("Alt, H, B"));
    }

    #[test]
    fn common_names_have_shortcuts() {
        assert_eq!(common_keys("select all"), Some("Ctrl+A"));
        assert_eq!(common_keys("nothing like this"), None);
    }
}
