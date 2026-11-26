//! "Your apps": which Start-menu shortcuts are apps worth listing, and the exe each one runs.

/// Whole words in a shortcut's name that mean it isn't the app itself.
const SKIP_WORDS: [&str; 20] = [
    "uninstall", "uninst", "uninstaller", "readme", "help", "documentation", "docs", "license", "licence", "website",
    "manual", "changelog", "repair", "troubleshoot", "troubleshooting", "debug", "updater", "setup", "installer", "faq",
];
/// Phrases that mean the same.
const SKIP_PHRASES: [&str; 10] = [
    "read me", "release notes", "what's new", "what’s new", "web site", "safe mode", "report a problem", "send feedback",
    "check for updates", "user guide",
];
/// Programs that only install, remove or host other things.
const SKIP_EXES: [&str; 11] = [
    "unins000.exe", "unins001.exe", "uninstall.exe", "uninst.exe", "uninstaller.exe", "msiexec.exe", "rundll32.exe",
    "hh.exe", "helppane.exe", "mmc.exe", "control.exe",
];
/// Start-menu folders full of system tools nobody needs shortcuts for.
const SKIP_FOLDERS: [&str; 7] =
    ["administrative tools", "windows administrative tools", "windows tools", "system tools", "maintenance", "startup", "windows accessibility"];
/// Launchers whose app runs under another name.
const LAUNCHERS: [(&str, &str); 1] = [("blender-launcher.exe", "blender.exe")];

/// Whether to look inside a Start-menu folder.
pub fn skip_folder(name: &str) -> bool {
    SKIP_FOLDERS.contains(&name.trim().to_lowercase().as_str())
}

/// A shortcut's name, target and arguments → the app's name and exe (lower case), or `None`
/// when it isn't an app (an uninstaller, a help file, a web link…).
pub fn from_shortcut(name: &str, target: &str, args: &str) -> Option<(String, String)> {
    let name = name.trim();
    let lname = name.to_lowercase();
    if name.is_empty() || SKIP_PHRASES.iter().any(|p| lname.contains(p)) {
        return None;
    }
    if lname.split(|c: char| !c.is_alphanumeric()).any(|w| SKIP_WORDS.contains(&w)) {
        return None;
    }
    let file = target.trim().trim_matches('"').rsplit(['\\', '/']).next()?.to_lowercase();
    if !file.ends_with(".exe") {
        return None;
    }
    // Apps installed with Squirrel (Discord, Slack, Figma…) start through Update.exe.
    let exe = if file == "update.exe" { squirrel_exe(args)? } else { file };
    let exe = LAUNCHERS.iter().find(|(l, _)| *l == exe).map(|(_, real)| real.to_string()).unwrap_or(exe);
    if SKIP_EXES.contains(&exe.as_str()) {
        return None;
    }
    Some((name.to_string(), exe))
}

/// `--processStart "Discord.exe"` → `discord.exe`.
fn squirrel_exe(args: &str) -> Option<String> {
    let low = args.to_lowercase();
    let at = low.find("--processstart")?;
    let rest = low[at + "--processstart".len()..].trim_start();
    let rest = rest.strip_prefix('=').unwrap_or(rest).trim_start();
    let exe = match rest.strip_prefix('"') {
        Some(r) => r.split('"').next()?,
        None => rest.split_whitespace().next()?,
    };
    let exe = exe.to_lowercase();
    exe.ends_with(".exe").then_some(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apps_are_kept_with_their_exe() {
        assert_eq!(
            from_shortcut("Adobe Photoshop 2025", r"C:\Program Files\Adobe\Adobe Photoshop 2025\Photoshop.exe", ""),
            Some(("Adobe Photoshop 2025".into(), "photoshop.exe".into()))
        );
        assert_eq!(
            from_shortcut("Discord", r"C:\Users\me\AppData\Local\Discord\Update.exe", "--processStart Discord.exe"),
            Some(("Discord".into(), "discord.exe".into()))
        );
        assert_eq!(
            from_shortcut("Slack", r"C:\Users\me\AppData\Local\slack\Update.exe", r#"--processStart "slack.exe" --process-start-args "--x""#),
            Some(("Slack".into(), "slack.exe".into()))
        );
        assert_eq!(
            from_shortcut("Blender 4.2", r"C:\Program Files\Blender Foundation\Blender 4.2\blender-launcher.exe", ""),
            Some(("Blender 4.2".into(), "blender.exe".into()))
        );
        // A word that only contains a skip word is fine.
        assert!(from_shortcut("Helpdesk Pro", r"C:\x\helpdesk.exe", "").is_some());
    }

    #[test]
    fn helpers_and_links_are_left_out() {
        assert_eq!(from_shortcut("Uninstall Foo", r"C:\x\unins000.exe", ""), None);
        assert_eq!(from_shortcut("Foo Help", r"C:\x\foo.chm", ""), None);
        assert_eq!(from_shortcut("Foo Website", r"C:\x\foo.url", ""), None);
        assert_eq!(from_shortcut("Release Notes", r"C:\x\notes.exe", ""), None);
        assert_eq!(from_shortcut("Foo", r"C:\Windows\System32\rundll32.exe", "shell32.dll,x"), None);
        assert_eq!(from_shortcut("Updater", r"C:\x\Update.exe", ""), None);
        assert_eq!(from_shortcut("Thing", r"C:\x\Update.exe", "--checkForUpdates"), None);
        assert!(skip_folder("Windows Administrative Tools"));
        assert!(!skip_folder("Adobe"));
    }
}
