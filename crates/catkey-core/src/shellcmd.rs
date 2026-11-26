//! Reading the command Windows registers for a kind of link, e.g.
//! `"C:\Program Files\Google\Chrome\Application\chrome.exe" --single-argument %1`,
//! so a long link can be handed straight to that program (Windows' own link handling cuts long links short).

/// Split a registered command into the program and the rest.
pub fn split(cmd: &str) -> Option<(String, String)> {
    let cmd = cmd.trim();
    let (exe, rest) = if let Some(r) = cmd.strip_prefix('"') {
        let end = r.find('"')?;
        (&r[..end], r[end + 1..].trim_start())
    } else {
        // Unquoted: the program ends at ".exe" (its path may have spaces).
        let end = cmd.to_ascii_lowercase().find(".exe").map(|i| i + 4).unwrap_or_else(|| cmd.find(' ').unwrap_or(cmd.len()));
        (&cmd[..end], cmd[end..].trim_start())
    };
    (!exe.trim().is_empty()).then(|| (exe.trim().to_string(), rest.to_string()))
}

/// Put the link where the command expects it (%1, %L or %l), or at the end, quoted.
pub fn fill(rest: &str, url: &str) -> String {
    ["%1", "%L", "%l"]
        .iter()
        .find(|p| rest.contains(*p))
        .map(|p| rest.replace(p, url))
        .unwrap_or_else(|| format!("{rest} \"{url}\"").trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://chatgpt.com/?q=Hi%20there%25&hints=search";

    #[test]
    fn chrome_and_edge() {
        let (exe, rest) = split(r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --single-argument %1"#).unwrap();
        assert_eq!(exe, r"C:\Program Files\Google\Chrome\Application\chrome.exe");
        assert_eq!(fill(&rest, URL), format!("--single-argument {URL}"));
    }

    #[test]
    fn firefox_quotes_the_link() {
        let (exe, rest) = split(r#""C:\Program Files\Mozilla Firefox\firefox.exe" -osint -url "%1""#).unwrap();
        assert!(exe.ends_with("firefox.exe"));
        assert_eq!(fill(&rest, URL), format!("-osint -url \"{URL}\""));
    }

    #[test]
    fn unquoted_path_with_spaces() {
        let (exe, rest) = split(r"C:\Program Files\Some Browser\browser.exe %1").unwrap();
        assert_eq!(exe, r"C:\Program Files\Some Browser\browser.exe");
        assert_eq!(rest, "%1");
    }

    #[test]
    fn no_placeholder_puts_the_link_last() {
        let (_, rest) = split(r#""C:\Users\me\AppData\Local\AnthropicClaude\claude.exe""#).unwrap();
        assert_eq!(fill(&rest, "claude://claude.ai/new?q=x"), "\"claude://claude.ai/new?q=x\"");
    }

    #[test]
    fn a_percent_in_the_link_is_left_alone() {
        // %1 is replaced once; the link's own %20 and %25 stay as they are.
        assert_eq!(fill("%1", "https://x.test/?q=%2520%1"), "https://x.test/?q=%2520%1");
    }

    #[test]
    fn broken_quotes_are_refused() {
        assert!(split(r#""C:\no end quote.exe %1"#).is_none());
        assert!(split("   ").is_none());
    }
}
