## Cat Key 1.10.2: no more grey box around the wheel and the Dock on Windows

**⬇ [Download for Windows](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-Setup-x64.exe)** or the [portable exe](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-portable-x64.exe) · **[Download for Mac](https://github.com/MasterfullyClaw/CatKey/releases/latest/download/CatKey-mac.dmg)** (Apple Silicon and Intel, macOS 11+).

### Windows
- **The CatGPT wheel, the Dock's cat, tips, the cat walk and the recording pill float with nothing around them.** On some PCs (dark mode with Windows' transparency effects off, or some graphics drivers) they sat in a dark grey box. Cat Key used to make see-through windows as a sheet of glass, which Windows fills with grey in those cases. Now the page is drawn straight onto the screen with its own see-through parts, whatever those settings are.
- The release check now also runs in dark mode with transparency effects off.

### From 1.10.1
- Mac: record shortcuts before Accessibility is allowed; **Allow** in the checkup clears the old version's switch and asks again; Open at login: Fix it sticks.
- The Cat Dock is on from the start. Admin mode follows the copy you run.

**Mac, first open:** Cat Key isn't notarized by Apple yet. Open it once, then System Settings → Privacy & Security → **Open Anyway**. If that goes round in circles, paste this into Terminal: `xattr -dr com.apple.quarantine "/Applications/Cat Key.app" && open "/Applications/Cat Key.app"`.
