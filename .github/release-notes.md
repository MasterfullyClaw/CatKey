## Cat Key 1.10.1: a see-through wheel on every PC, the Dock from the start, Mac fixes

**⬇ [Download for Windows](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-Setup-x64.exe)** or the [portable exe](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-portable-x64.exe) · **[Download for Mac](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-mac.dmg)** (Apple Silicon and Intel, macOS 11+).

### Mac
- **Recording keys works before Accessibility is allowed.** The window listens to its own keys until Cat Key may use the keyboard, so you can set up shortcuts (and your CatGPT keys) right away. They start working once it's allowed.
- **One Allow button fixes "it's already switched on".** After an update macOS keeps the old version's switch, which no longer counts, and won't ask again while it's there. The checkup's **Allow** clears Cat Key's own old entry and asks again, for the keyboard (Accessibility) and for CatGPT's screenshots (Screen & System Audio Recording). You still switch it on yourself.
- **Open at login: Fix it now sticks.** The checkup read the login item wrongly, so it always looked missing.
- The checkup shows whether CatGPT can see the screen.
- The wheel has no dark disc behind its petals any more.

### Windows
- **The wheel floats on its own, see-through.** On some PCs the graphics driver drew a box around the CatGPT wheel. Now every see-through part of Cat Key (the wheel, the Dock's cat, tips, the switcher, the cat walk, paw prints, the recording pill) is cut to its own shape, so no box can show, and clicks between the wheel's petals reach the app underneath.
- **The Cat Dock is on from the start**: the cat that sits on the edge of your screen. Switch it off in Settings or with the yarn-ball button at the top.
- **Admin mode follows the copy you run.** Moving from the portable exe to the installed app (or the other way) used to leave the admin start-up task pointing at the old copy. Cat Key now fixes it the next time it runs as administrator.

**Mac, first open:** Cat Key isn't notarized by Apple yet. Open it once, then System Settings → Privacy & Security → **Open Anyway**. If that goes round in circles, paste this into Terminal: `xattr -dr com.apple.quarantine "/Applications/Cat Key.app" && open "/Applications/Cat Key.app"`.
