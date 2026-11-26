//! The app switcher's geometry: when the pointer is resting in the hot spot, where the
//! panel of open windows goes, and when the pointer has left it. Pure maths, so it's
//! tested here; the windows themselves are listed by the app.

/// The hot spots, in the order the settings show them.
pub const SPOTS: [&str; 7] = ["top-left", "top", "top-right", "left", "right", "bottom-left", "bottom-right"];

/// A rectangle on the screen (any unit, as long as everything uses the same one).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
    /// The same rectangle, `by` bigger on every side.
    pub fn grow(&self, by: f64) -> Rect {
        Rect { x: self.x - by, y: self.y - by, w: self.w + 2.0 * by, h: self.h + 2.0 * by }
    }
}

pub fn is_spot(spot: &str) -> bool {
    SPOTS.contains(&spot)
}

/// Is the pointer at (`px`, `py`) in `spot` of `screen`? `tol` is how near the very edge
/// counts. Edge spots only count in the middle half of that edge, so the corners, the
/// scroll bars' ends and the browser tabs stay free.
pub fn in_spot(spot: &str, screen: Rect, px: f64, py: f64, tol: f64) -> bool {
    if !screen.contains(px, py) {
        return false;
    }
    let left = px <= screen.x + tol;
    let right = px >= screen.right() - 1.0 - tol;
    let top = py <= screen.y + tol;
    let bottom = py >= screen.bottom() - 1.0 - tol;
    let mid_y = py >= screen.y + screen.h * 0.25 && py <= screen.y + screen.h * 0.75;
    let mid_x = px >= screen.x + screen.w * 0.25 && px <= screen.x + screen.w * 0.75;
    match spot {
        "top-left" => top && left,
        "top-right" => top && right,
        "bottom-left" => bottom && left,
        "bottom-right" => bottom && right,
        "left" => left && mid_y,
        "right" => right && mid_y,
        "top" => top && mid_x,
        _ => false,
    }
}

/// Where the panel goes for `spot`: inside the work area `work`, `gap` away from its edges
/// (so the very corner stays free for whatever the user meant to click there), shrunk to
/// fit a small screen. Any other spot ("center") puts it in the middle (opened with a
/// shortcut).
pub fn panel_rect(spot: &str, work: Rect, px: f64, py: f64, w: f64, h: f64, gap: f64) -> Rect {
    let w = w.min(work.w - 2.0 * gap);
    let h = h.min(work.h - 2.0 * gap);
    let min_x = work.x + gap;
    let max_x = (work.right() - w - gap).max(min_x);
    let min_y = work.y + gap;
    let max_y = (work.bottom() - h - gap).max(min_y);
    let x = match spot {
        "top-left" | "left" | "bottom-left" => min_x,
        "top-right" | "right" | "bottom-right" => max_x,
        "top" => (px - w / 2.0).clamp(min_x, max_x),
        _ => (work.x + (work.w - w) / 2.0).clamp(min_x, max_x),
    };
    let y = match spot {
        "top-left" | "top-right" | "top" => min_y,
        "bottom-left" | "bottom-right" => max_y,
        "left" | "right" => (py - h / 2.0).clamp(min_y, max_y),
        _ => (work.y + (work.h - h) / 3.0).clamp(min_y, max_y),
    };
    Rect { x, y, w, h }
}

/// The panel's height for `rows` windows: header, rows and footer, never taller than `max`.
pub fn panel_height(rows: usize, head: f64, row: f64, foot: f64, max: f64) -> f64 {
    (head + row * rows.max(1) as f64 + foot).min(max)
}

/// While the panel is open: is the pointer still with it? It stays while the pointer is
/// over the panel (with some slack around it) or still in the hot spot.
pub fn still_there(panel: Rect, slack: f64, in_hot_spot: bool, px: f64, py: f64) -> bool {
    in_hot_spot || panel.grow(slack).contains(px, py)
}

/// Decides when the panel opens: the pointer has to rest in the spot for `delay_ms`
/// without a mouse button held (dragging a window into a corner isn't a request).
#[derive(Debug, Default)]
pub struct Dwell {
    since: Option<u64>,
    /// After it opened, the pointer has to leave the spot before it can open again.
    spent: bool,
}

impl Dwell {
    /// Feed one look at the pointer (`now` in ms). True once: the moment to open.
    pub fn step(&mut self, in_spot: bool, button_down: bool, now: u64, delay_ms: u64) -> bool {
        if !in_spot {
            self.since = None;
            self.spent = false;
            return false;
        }
        if self.spent {
            return false;
        }
        if button_down {
            self.since = None;
            return false;
        }
        let since = *self.since.get_or_insert(now);
        if now.saturating_sub(since) >= delay_ms {
            self.spent = true;
            self.since = None;
            return true;
        }
        false
    }

    /// The panel was opened another way, or closed while the pointer is still in the spot.
    pub fn spend(&mut self) {
        self.spent = true;
        self.since = None;
    }
}

/// Window titles often end with the app's name ("Inbox - Outlook"); the panel shows
/// the app on its own line, so drop it from the title when it's there.
pub fn short_title(title: &str, app: &str) -> String {
    let t = title.trim();
    let a = app.trim();
    if a.is_empty() {
        return t.to_string();
    }
    for sep in [" - ", " — ", " – ", " | "] {
        if let Some(rest) = t.strip_suffix(a).and_then(|r| r.strip_suffix(sep)) {
            if !rest.trim().is_empty() {
                return rest.trim().to_string();
            }
        }
    }
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 };

    #[test]
    fn corners_count_only_at_the_very_corner() {
        assert!(in_spot("top-left", SCREEN, 0.0, 0.0, 2.0));
        assert!(in_spot("top-left", SCREEN, 2.0, 1.0, 2.0));
        assert!(!in_spot("top-left", SCREEN, 3.0, 0.0, 2.0));
        assert!(!in_spot("top-left", SCREEN, 0.0, 40.0, 2.0));
        assert!(in_spot("top-right", SCREEN, 1919.0, 0.0, 2.0));
        assert!(in_spot("bottom-right", SCREEN, 1919.0, 1079.0, 2.0));
        assert!(in_spot("bottom-left", SCREEN, 0.0, 1079.0, 2.0));
        assert!(!in_spot("bottom-left", SCREEN, 0.0, 1000.0, 2.0));
    }

    #[test]
    fn edges_count_in_their_middle_half() {
        assert!(in_spot("left", SCREEN, 0.0, 540.0, 1.0));
        assert!(!in_spot("left", SCREEN, 0.0, 100.0, 1.0));
        assert!(!in_spot("left", SCREEN, 5.0, 540.0, 1.0));
        assert!(in_spot("right", SCREEN, 1919.0, 600.0, 1.0));
        assert!(in_spot("top", SCREEN, 960.0, 0.0, 1.0));
        assert!(!in_spot("top", SCREEN, 100.0, 0.0, 1.0));
    }

    #[test]
    fn a_second_screen_has_its_own_corners() {
        let second = Rect::new(1920.0, -200.0, 2560.0, 1440.0);
        assert!(in_spot("top-left", second, 1920.0, -200.0, 2.0));
        assert!(!in_spot("top-left", second, 10.0, 10.0, 2.0));
        assert!(!in_spot("nowhere", second, 1920.0, -200.0, 2.0));
    }

    #[test]
    fn the_panel_sits_by_its_spot_and_leaves_the_corner_free() {
        let work = Rect::new(0.0, 0.0, 1920.0, 1040.0);
        let at = |spot: &str, px: f64, py: f64| {
            let r = panel_rect(spot, work, px, py, 320.0, 400.0, 8.0);
            assert_eq!((r.w, r.h), (320.0, 400.0));
            (r.x, r.y)
        };
        assert_eq!(at("top-left", 0.0, 0.0), (8.0, 8.0));
        assert_eq!(at("top-right", 1919.0, 0.0), (1592.0, 8.0));
        assert_eq!(at("bottom-left", 0.0, 1079.0), (8.0, 632.0));
        assert_eq!(at("left", 0.0, 500.0), (8.0, 300.0));
        assert_eq!(at("top", 960.0, 0.0), (800.0, 8.0));
        // Near the end of an edge it stays on screen.
        assert_eq!(at("right", 1919.0, 1030.0), (1592.0, 632.0));
        // Opened with a shortcut: in the middle, a bit above centre.
        let (x, y) = at("center", 5.0, 5.0);
        assert_eq!(x, 800.0);
        assert!(y > 8.0 && y < 320.0);
    }

    #[test]
    fn a_tiny_screen_still_gets_a_panel_inside_it() {
        let work = Rect::new(0.0, 0.0, 300.0, 200.0);
        let r = panel_rect("bottom-right", work, 299.0, 199.0, 320.0, 400.0, 8.0);
        assert_eq!(r, Rect::new(8.0, 8.0, 284.0, 184.0));
    }

    #[test]
    fn height_grows_with_rows_up_to_a_limit() {
        assert_eq!(panel_height(0, 50.0, 44.0, 30.0, 600.0), 124.0);
        assert_eq!(panel_height(3, 50.0, 44.0, 30.0, 600.0), 212.0);
        assert_eq!(panel_height(40, 50.0, 44.0, 30.0, 600.0), 600.0);
    }

    #[test]
    fn the_pointer_can_wander_a_little() {
        let panel = Rect::new(8.0, 8.0, 320.0, 300.0);
        assert!(still_there(panel, 24.0, false, 340.0, 100.0));
        assert!(!still_there(panel, 24.0, false, 400.0, 100.0));
        assert!(still_there(panel, 24.0, true, 900.0, 900.0));
    }

    #[test]
    fn it_opens_after_resting_once_per_visit() {
        let mut d = Dwell::default();
        assert!(!d.step(true, false, 1000, 150));
        assert!(!d.step(true, false, 1100, 150));
        assert!(d.step(true, false, 1160, 150));
        // Still in the corner: not again.
        assert!(!d.step(true, false, 2000, 150));
        // Leave and come back: again.
        assert!(!d.step(false, false, 2100, 150));
        assert!(!d.step(true, false, 2200, 150));
        assert!(d.step(true, false, 2400, 150));
    }

    #[test]
    fn dragging_into_the_corner_does_not_open_it() {
        let mut d = Dwell::default();
        assert!(!d.step(true, true, 0, 100));
        assert!(!d.step(true, true, 500, 100));
        // Let go there: the rest starts now.
        assert!(!d.step(true, false, 600, 100));
        assert!(d.step(true, false, 700, 100));
    }

    #[test]
    fn instant_when_there_is_no_delay() {
        let mut d = Dwell::default();
        assert!(d.step(true, false, 5, 0));
    }

    #[test]
    fn spent_waits_for_the_pointer_to_leave() {
        let mut d = Dwell::default();
        d.spend();
        assert!(!d.step(true, false, 0, 0));
        assert!(!d.step(false, false, 10, 0));
        assert!(d.step(true, false, 20, 0));
    }

    #[test]
    fn titles_lose_the_app_name() {
        assert_eq!(short_title("Inbox - Outlook", "Outlook"), "Inbox");
        assert_eq!(short_title("Cat Key – Notepad", "Notepad"), "Cat Key");
        assert_eq!(short_title("Notepad", "Notepad"), "Notepad");
        assert_eq!(short_title("Untitled | Figma", "Figma"), "Untitled");
        assert_eq!(short_title("Report.docx", "Word"), "Report.docx");
        assert_eq!(short_title("  spaced  ", ""), "spaced");
    }
}
