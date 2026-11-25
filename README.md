# Cat Key 🐾

**Purr-fect keyboard shortcuts for Windows.** Cat Key remaps shortcuts system-wide (Alt + C → Ctrl + C and friends) with a tiny Rust engine, a calm tray icon and an animated anime cat who taps along every time a shortcut fires.

- **Your Mac-style shortcuts, out of the box:** Alt + W / C / V / N / F / A / S / Z / R → Ctrl + the same key, Alt + Shift + N → Ctrl + Shift + N, Alt + Q → Alt + F4.
- **Add, edit, delete, duplicate, switch off** any shortcut. Record keys by just pressing them, or fine-tune Left/Right/Either side by hand.
- **Four actions:** press other keys, open an app / file / folder / link, type a piece of text, or block a key.
- **Per app:** *Everywhere*, *Only in…* (e.g. Chrome) or *Everywhere except…*. App-only shortcuts win over everywhere ones.
- **Key remaps too:** CapsLock → Esc, CapsLock → Ctrl (held like a real Ctrl), Right Alt → Menu…
- **Overrides conflicts:** Cat Key's hook runs before app and Windows shortcuts, keeps itself first in line, and taps a hidden key so Alt/Win never pop open menus by mistake. It warns you about clashes inside your own list and about PowerToys Keyboard Manager running at the same time.
- **Nap zones:** apps where Cat Key does nothing (games, Remote Desktop…). Plus a pause hotkey and an Awake/Napping switch.
- **Works in admin apps** when you turn on *Work in admin apps* (runs as administrator, starts with Windows through a logon task).
- **Cute, but light:** close the window and the UI is gone from memory; only the small keyboard engine stays in the tray. Four cat skins (Mochi, Kuro, Tora, Sora), light / dark / match Windows.
- **Import from PowerToys** in one click, plus import / export of your own backups.

## Install

Download the latest **`Cat Key_x.y.z_x64-setup.exe`** (installer) or **`CatKey.exe`** (portable) from the repo's Releases or Actions artifacts, then run it.

> Windows SmartScreen may say *"Windows protected your PC"* because the app isn't code-signed. Click **More info → Run anyway**.

Cat Key starts with Windows by default and lives in the tray: **left-click** the cat to open it, **right-click** for Pause / Quit.

If you were using **PowerToys Keyboard Manager**, turn it off in PowerToys (Keyboard Manager → toggle off). Two remappers on the same keys fight each other.

### Checkup

When something needs you, a **Quick checkup** card appears on the main screen (it's also always at the top of **Settings**). Each line has a button that fixes it or opens the right Windows page:

| Line | What the button does |
| --- | --- |
| The keyboard engine isn't running | **Restart Cat Key** |
| Windows switched Cat Key off in Startup apps | **Open Startup apps** (Settings → Apps → Startup), switch Cat Key on |
| Start with Windows isn't set up yet | **Fix it** writes the startup entry again |
| Admin apps aren't covered yet | **Allow** asks Windows for admin permission once (UAC), so shortcuts also work in Task Manager and admin terminals |
| PowerToys Keyboard Manager is on | **Open PowerToys** to switch it off, or **Import its shortcuts** |

If the window doesn't appear, Cat Key may already be running in the tray (the little cat by the clock; it can hide behind the **^** arrow). Click it to open the window. Problems are written to `%APPDATA%\CatKey\catkey.log`.

## Use it

| Want to… | Do this |
| --- | --- |
| Add a shortcut | **New shortcut** (or Ctrl + N), press the keys, pick what happens, Save |
| Change one | Click its card |
| Turn one off | Its switch |
| Delete / duplicate | The ⋯ menu on the card (Undo is in the toast) |
| Pause everything | The **Awake** switch, the tray menu, or your pause hotkey (Settings) |
| Make it per app | In the editor, *Where* → *Only in…* → *Add app* |

Settings live in `%APPDATA%\CatKey\config.json` (plain JSON, safe to back up). Keys are written like `LAlt+LShift+N`, `Ctrl+C`, `CapsLock`.

## Build from source

Requirements: [Rust](https://rustup.rs) (stable), Node 18+, and on Windows the MSVC build tools + WebView2 (already on Windows 10/11).

```bash
npm install
npm run build          # app + installer -> target/release/ and target/release/bundle/nsis/
npm run tauri dev      # run in development
npm test               # engine unit tests (any OS)
npm run preview        # the UI in a normal browser with a fake backend
```

Cross-compiling from Linux works too: `npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc`.

The **Build Cat Key** GitHub Action builds the exe and installer on demand (Actions → Run workflow) and attaches them to a release when you push a tag like `v1.0.1`.

## How it works

```
crates/catkey-core/   Pure Rust: key names, config, the remap engine, PowerToys import (unit-tested)
src-tauri/src/hook.rs Low-level keyboard hook on its own time-critical thread
src-tauri/src/win.rs  Windows helpers: foreground app, icons, startup, admin mode, launching
src-tauri/src/app.rs  Tray app, config files, effects, window lifecycle
src/                  The UI (plain HTML/CSS/JS, no build step) and the mascot
```

The engine sees every physical key event and either lets it through or swallows it and sends replacements with `SendInput`. It tracks which modifiers are really held versus which ones Windows believes are held, so:

- Alt + C, Alt + V in a row keeps Ctrl down between them (fast copy/paste),
- Alt + Tab still works right after a remap,
- releasing Alt before C never leaves a key stuck or types a stray letter,
- key repeat works (hold Alt + Z to undo repeatedly),
- a missed key release (lock screen, UAC prompt) is detected and cleaned up.

The hook callback never waits on the UI and never touches the disk, so remaps add no noticeable delay. The process also opts out of Windows 11 efficiency mode so the keyboard thread is never throttled.

## Limits

- Windows reserves **Win + L** and **Ctrl + Alt + Del**; no app can change them.
- Without *Work in admin apps*, shortcuts don't reach programs running as administrator (Windows blocks that for security).

## License

MIT © 2026 MASTERFULLY-CLAW. Fonts: Nunito and Fredoka, SIL Open Font License (see `src/assets/fonts`).
