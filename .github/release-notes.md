## Cat Key 1.9.0: Claude hands packs straight to Cat Key

**⬇ [Download for Windows](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-Setup-x64.exe)** or the [portable exe](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-portable-x64.exe) · **[Download for Mac](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-mac.dmg)** (Apple Silicon and Intel, macOS 11+).

### Claude Desktop, connected
- **Cat Builder → Connect Claude Desktop.** Cat Key adds itself to Claude Desktop's settings (and keeps a backup of them). Quit and reopen Claude Desktop, then ask it: *"Make me Cat Key shortcuts for Blender"*.
- Claude reads the pack format, looks up the app's real shortcuts and **sends the pack**: Cat Key opens it on your screen so you tick what you want. No copying and pasting, and nothing changes until you add it.
- Claude can also **read your shortcuts** (it can't change them), so the keys it suggests don't clash with yours.
- Other AI apps that speak MCP work too: **Copy the setup** (JSON) or **Copy for Claude Code** (`claude mcp add …`).

### Also new since 1.7
- **Shortcut coach on a Mac** (1.8.0): menu items, close and minimise buttons and buttons the Library knows show their keys when you click them.
- **App switcher** (1.7.0): rest the pointer in a corner and your open windows pop up right there.
- **Eight new app packs** (1.7.1): Notion, Slack, Word, Excel, PowerPoint, Final Cut Pro, Ableton Live and Unity.

**Mac, first open:** Cat Key isn't notarized by Apple yet. Open it once, then System Settings → Privacy & Security → **Open Anyway**. If that goes round in circles, paste this into Terminal: `xattr -dr com.apple.quarantine "/Applications/Cat Key.app" && open "/Applications/Cat Key.app"`.
