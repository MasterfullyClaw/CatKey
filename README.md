<div align="center">

<img src="docs/cat.svg" width="132" alt="Cat Key's cat">

# Cat Key

### Every cat walks across a keyboard. This one presses the right keys.

Cat Key sits on your keyboard and turns the keys you press into the shortcuts you mean, in every app or only in the apps you choose. Then it trains you until those shortcuts are second nature.

<br>

[![Download for Windows](https://img.shields.io/badge/⬇%20Download%20for%20Windows-free-ff8cc6?style=for-the-badge)](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-Setup-x64.exe)
&nbsp;
[![Mac](https://img.shields.io/badge/Mac-coming%20soon-b79cff?style=for-the-badge)](https://github.com/MasterfullyClaw/CatKey/releases)

<sub>Windows 10 and 11 · free and open source · no account, no internet needed · <a href="https://masterfullyclaw.github.io/CatKey/">website</a></sub>

</div>

---

## What it's for

**Your hands already know the shortcuts. The apps just disagree.**

- **Switching from a Mac?** Keep your ⌘ habits on Windows. Alt sits right where ⌘ was, so Alt+C copies, Alt+W closes the tab and Alt+Q quits, everywhere. Moving to a Mac, the other way round.
- **Every app wants different keys?** Give the same keys the same job everywhere, or a different job in one app only: Premiere keys inside DaVinci Resolve, Mac keys in Excel, your own keys anywhere.
- **Learning a pro app?** Pick Premiere Pro, Resolve, Blender, Photoshop or any other app, and **Train yourself** with the shortcuts the pros use, until your fingers do them without thinking.

Each app keeps its own shortcuts. Cat Key only adds yours on top, and you can switch it off (or let the cat nap) any time.

## What's inside

| | |
|---|---|
| 🐾 **Your shortcuts** | Press a key, press what it should do. Everywhere, only in some apps, or everywhere except some. Shortcuts can also open apps, type text, or do nothing at all (block a key). |
| 📚 **Library** | 21 ready-made packs, 748 shortcuts: Mac ⇄ Windows switching, Windows power keys, and the pro shortcuts of Premiere Pro, After Effects, DaVinci Resolve, CapCut, Photoshop, Illustrator, Lightroom, Figma, Blender, Cinema 4D, VS Code, Obsidian, browsers and more. Tick what you want, add it. |
| 🎯 **Train yourself** | A game that teaches an app's shortcuts. Cat Key names an action and you press its keys. A right answer goes *boing*, a miss gets a soft meow, and streaks score more. Each shortcut comes back just before you'd forget it (spaced review). Cat Key also counts the times you use them for real in the app. |
| 🪄 **Cat Builder** | Shortcuts for *any* app. Cat Builder writes the perfect request for ChatGPT, Claude, Perplexity or Gemini, opens it in one click, and picks up the answer when you copy it. No AI is built in and nothing is sent by Cat Key itself. |
| 🔁 **Swap & templates** | Flip any shortcut (what you press ⇄ what it does) with one click. Save sets as templates; the starter set can't be deleted. |
| 🔗 **Share** | Share a pack as one line of text (a `CK1:` share code) or a file. Move your whole setup to another PC or Mac; Cat Key translates ⌘ and Ctrl for you. |
| 🩺 **Checkup** | Finds what gets in the way (PowerToys Keyboard Manager, PowerToys Grab And Move on Alt, missing start-up entry, admin apps) and fixes it, or tells you exactly where to click. |

## Install

1. [Download the installer](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-Setup-x64.exe) and run it. It doesn't need admin rights.
2. Cat Key opens with your first shortcuts already working, and it lives in the tray by the clock.
3. Want shortcuts in apps that run as administrator (Task Manager, admin terminals)? Settings → **Works in admin apps** → Allow.

Prefer no installer? Grab [the portable exe](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-portable-x64.exe).

## How it works

Cat Key listens to the keyboard with a standard Windows keyboard hook. When you press one of your shortcuts, it holds that key back and sends the one you meant instead, in under a millisecond. Everything else goes through untouched.

- **It never misses a key.** The hook only decides; a separate thread sends keys, so a slow program elsewhere can't make Windows skip Cat Key. A watchdog reads the keyboard directly and repairs a key release that went missing.
- **It doesn't fight other tools.** Keys typed by other programs (password managers, macro tools) pass untouched. Alt shortcuts work even with PowerToys Grab And Move holding Alt back.
- **It stays out of your typing.** Single-letter remaps pause while a text cursor is active, so "V does A" in an editor never turns "Very" into "Aery".
- **Your keys stay yours.** Cat Key keeps no keystroke log. Train yourself counts presses only for the shortcuts you're learning, and only on your PC. The optional key log (for troubleshooting) runs only while you have it open.

## Make and share packs

Packs are small JSON files. The [library guide](library/README.md) explains the format. Share yours with the [Share a pack form](https://github.com/MasterfullyClaw/CatKey/issues/new?template=share-a-pack.yml) and it can appear in everyone's *Library → Community*.

## Troubleshooting

- **A shortcut needs two presses.** Update to the latest version. If it still happens: Settings → *Try a shortcut* → **Start key log**, use the shortcut, then **Save**, and attach the log to an issue.
- **Alt shortcuts feel slow or odd.** PowerToys **Grab And Move** also uses Alt. Cat Key copes with it, but setting Grab And Move to the Windows key makes Alt yours alone.
- **An error sound with a shortcut.** Two remappers are changing the same keys. Turn off PowerToys Keyboard Manager (Cat Key can import its shortcuts first).

## Build from source

```bash
npm install
cargo test -p catkey-core          # the key engine's tests
npx tauri build                    # on Windows
python3 tools/library/build.py     # after editing the built-in library
```

The keyboard engine is a pure Rust crate (`crates/catkey-core`) with no operating-system code, so every timing case (Alt menus, key repeat, release order, missed releases) is covered by unit tests.

## Credits

Some cheat-sheet entries were checked against [ShortcutMapper](https://github.com/waldobronchart/ShortcutMapper) (MIT) and each app's official documentation (linked in every pack).

MIT licensed. App names belong to their owners; Cat Key isn't affiliated with them.
