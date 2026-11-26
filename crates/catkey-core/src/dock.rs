//! The Cat Dock's data: the things you drop on it, and what you copied.

use serde::{Deserialize, Serialize};

/// Something kept in the food bowl (or, when `cut`, in the litter box).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShelfItem {
    pub id: String,
    /// "files", "text" or "link".
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// In the litter box: dragging it out moves the files instead of copying them.
    #[serde(default)]
    pub cut: bool,
    /// When it arrived, in seconds since 1970.
    #[serde(default)]
    pub added: u64,
}

/// Something you copied.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    /// "text" or "files".
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub at: u64,
}

pub const MAX_CLIPS: usize = 80;
pub const MAX_SHELF: usize = 200;
/// Longer copies are kept cut short (the clipboard still has the whole thing).
pub const MAX_TEXT: usize = 20_000;

/// "text" or "link": a single web address is a link.
pub fn text_kind(text: &str) -> &'static str {
    let t = text.trim();
    let one_word = !t.chars().any(char::is_whitespace);
    if one_word && (t.starts_with("https://") || t.starts_with("http://")) {
        "link"
    } else {
        "text"
    }
}

/// Cut text to `max` characters (on a character boundary).
pub fn clip_text(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((i, _)) => text[..i].to_string(),
        None => text.to_string(),
    }
}

/// Put a copy at the top. The same thing copied again moves up instead of showing twice;
/// when the list is full, the oldest one that isn't pinned goes. Returns whether anything changed.
pub fn push_clip(list: &mut Vec<Clip>, mut clip: Clip) -> bool {
    if clip.kind == "text" && clip.text.trim().is_empty() || clip.kind == "files" && clip.paths.is_empty() {
        return false;
    }
    clip.text = clip_text(&clip.text, MAX_TEXT);
    if let Some(i) = list.iter().position(|c| c.kind == clip.kind && c.text == clip.text && c.paths == clip.paths) {
        if i == 0 {
            return false;
        }
        let mut old = list.remove(i);
        old.at = clip.at;
        list.insert(0, old);
        return true;
    }
    list.insert(0, clip);
    while list.len() > MAX_CLIPS {
        match list.iter().rposition(|c| !c.pinned) {
            Some(i) => {
                list.remove(i);
            }
            None => break,
        }
    }
    true
}

/// Put a dropped thing at the top of the bowl; the same thing dropped again moves up.
pub fn push_shelf(list: &mut Vec<ShelfItem>, item: ShelfItem) -> bool {
    if item.kind == "files" && item.paths.is_empty() || item.kind != "files" && item.text.trim().is_empty() {
        return false;
    }
    if let Some(i) = list.iter().position(|x| x.kind == item.kind && x.text == item.text && x.paths == item.paths) {
        let mut old = list.remove(i);
        old.added = item.added;
        old.cut = item.cut;
        list.insert(0, old);
        return true;
    }
    list.insert(0, item);
    list.truncate(MAX_SHELF);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(id: &str, t: &str) -> Clip {
        Clip { id: id.into(), kind: "text".into(), text: t.into(), paths: vec![], pinned: false, at: 1 }
    }

    #[test]
    fn copies_stay_in_order_without_repeats() {
        let mut l = Vec::new();
        assert!(push_clip(&mut l, text("a", "hello")));
        assert!(push_clip(&mut l, text("b", "world")));
        assert!(!push_clip(&mut l, text("c", "world")), "the same copy twice in a row changes nothing");
        assert!(push_clip(&mut l, text("d", "hello")), "copied again: it moves to the top");
        assert_eq!(l.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        assert!(!push_clip(&mut l, text("e", "   ")), "blank copies are skipped");
    }

    #[test]
    fn a_full_history_drops_the_oldest_unpinned() {
        let mut l = Vec::new();
        let mut first = text("first", "pinned one");
        first.pinned = true;
        push_clip(&mut l, first);
        for i in 0..MAX_CLIPS + 5 {
            push_clip(&mut l, text(&format!("c{i}"), &format!("copy {i}")));
        }
        assert_eq!(l.len(), MAX_CLIPS);
        assert!(l.iter().any(|c| c.id == "first"), "pinned copies stay");
        assert_eq!(l[0].text, format!("copy {}", MAX_CLIPS + 4));
    }

    #[test]
    fn links_text_and_long_copies() {
        assert_eq!(text_kind("https://example.com/a?b=c"), "link");
        assert_eq!(text_kind("see https://example.com"), "text");
        assert_eq!(text_kind("just words"), "text");
        assert_eq!(clip_text("héllo", 2), "hé");
        assert_eq!(clip_text("hi", 5), "hi");
    }

    #[test]
    fn the_bowl_keeps_one_of_each() {
        let mut l = Vec::new();
        let f = |id: &str, p: &str| ShelfItem { id: id.into(), kind: "files".into(), paths: vec![p.into()], text: String::new(), cut: false, added: 1 };
        assert!(push_shelf(&mut l, f("1", "C:/a.png")));
        assert!(push_shelf(&mut l, f("2", "C:/b.png")));
        assert!(push_shelf(&mut l, f("3", "C:/a.png")));
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].id, "1");
        assert!(!push_shelf(&mut l, ShelfItem { id: "x".into(), kind: "text".into(), paths: vec![], text: " ".into(), cut: false, added: 1 }));
    }
}
