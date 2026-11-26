# Cat Key built-in library. Run build.py to turn this into src/library/*.
# Tokens: Mod = Ctrl on Windows / Cmd on Mac; Opt = Alt / Option; Ctrl = Ctrl / Control.
def cs(keys, name, fav=0, **kw):  # cheat-sheet entry (the app's own default)
    return dict(keys=keys, name=name, fav=bool(fav), **kw)
def rm(press, does, name, fav=0, **kw):  # remap: press -> does
    return dict(press=press, does=does, name=name, fav=bool(fav), **kw)
def S(name, items, on=True, note=None):
    d = dict(name=name, items=items, on=on)
    if note: d["note"] = note
    return d
def P(id, name, kind, category, platform, description, sections, **kw):
    return dict(id=id, name=name, kind=kind, category=category, platform=platform,
                description=description, sections=sections, **kw)

ADOBE_WIN = "https://helpx.adobe.com/"
SM = {"title": "ShortcutMapper (MIT licence)", "url": "https://github.com/waldobronchart/ShortcutMapper"}
TERM = {"only": "app:terminal"}

PACKS = [
P("mac-on-windows", "Mac keys on Windows", "remap", "switch", "windows",
  "Your Mac muscle memory, on Windows: the Alt key (where ⌘ sits) does what ⌘ does on a Mac, in every app.",
  [
   S("Everyday", [
     rm("Alt+C", "Ctrl+C", "Copy", 1), rm("Alt+V", "Ctrl+V", "Paste", 1), rm("Alt+X", "Ctrl+X", "Cut", 1),
     rm("Alt+Z", "Ctrl+Z", "Undo", 1), rm("Alt+Shift+Z", "Ctrl+Shift+Z", "Redo", 1),
     rm("Alt+A", "Ctrl+A", "Select all", 1), rm("Alt+S", "Ctrl+S", "Save", 1), rm("Alt+Shift+S", "Ctrl+Shift+S", "Save as"),
     rm("Alt+F", "Ctrl+F", "Find", 1), rm("Alt+G", "F3", "Find next"), rm("Alt+Shift+G", "Shift+F3", "Find previous"),
     rm("Alt+N", "Ctrl+N", "New", 1), rm("Alt+Shift+N", "Ctrl+Shift+N", "New folder / private window"),
     rm("Alt+O", "Ctrl+O", "Open"), rm("Alt+P", "Ctrl+P", "Print"),
     rm("Alt+W", "Ctrl+W", "Close tab or window", 1), rm("Alt+Q", "Alt+F4", "Quit the app", 1),
     rm("Alt+Comma", "Ctrl+Comma", "Settings (in apps that have it)"),
   ]),
   S("Text formatting", [
     rm("Alt+B", "Ctrl+B", "Bold"), rm("Alt+I", "Ctrl+I", "Italic"), rm("Alt+U", "Ctrl+U", "Underline"),
     rm("Alt+K", "Ctrl+K", "Insert link"),
   ]),
   S("Browser and tabs", [
     rm("Alt+T", "Ctrl+T", "New tab", 1), rm("Alt+Shift+T", "Ctrl+Shift+T", "Reopen closed tab", 1),
     rm("Alt+R", "Ctrl+R", "Reload", 1), rm("Alt+L", "Ctrl+L", "Address bar", 1), rm("Alt+D", "Ctrl+D", "Bookmark this page"),
     rm("Alt+Y", "Ctrl+H", "History"), rm("Alt+Shift+J", "Ctrl+J", "Downloads"),
     rm("Alt+[", "Alt+Left", "Back", 1), rm("Alt+]", "Alt+Right", "Forward", 1),
     rm("Alt+Shift+[", "Ctrl+Shift+Tab", "Previous tab"), rm("Alt+Shift+]", "Ctrl+Tab", "Next tab"),
     rm("Alt+Equals", "Ctrl+Equals", "Zoom in"), rm("Alt+Minus", "Ctrl+Minus", "Zoom out"), rm("Alt+0", "Ctrl+0", "Actual size"),
   ] + [rm(f"Alt+{n}", f"Ctrl+{n}", f"Go to tab {n}" if n < 9 else "Go to last tab") for n in range(1, 10)]),
   S("Screenshots like a Mac", [
     rm("Alt+Shift+3", "Win+PrintScreen", "Screenshot of the whole screen (saved to Pictures)", 1),
     rm("Alt+Shift+4", "Win+Shift+S", "Screenshot of an area", 1),
     rm("Alt+Shift+5", "Win+Shift+R", "Record the screen (Snipping Tool, Windows 11)"),
   ]),
   S("Text editing like a Mac", [
     rm("Alt+Left", "Home", "Start of line"), rm("Alt+Right", "End", "End of line"),
     rm("Alt+Up", "Ctrl+Home", "Top of document"), rm("Alt+Down", "Ctrl+End", "End of document"),
     rm("Alt+Shift+Left", "Shift+Home", "Select to start of line"), rm("Alt+Shift+Right", "Shift+End", "Select to end of line"),
     rm("Alt+Shift+Up", "Ctrl+Shift+Home", "Select to top"), rm("Alt+Shift+Down", "Ctrl+Shift+End", "Select to end"),
   ], on=False, note="Alt+← stops meaning Back in browsers. Use Alt+[ instead, like ⌘[ on a Mac."),
   S("Option keys on the Windows key", [
     rm("Win+Left", "Ctrl+Left", "Previous word"), rm("Win+Right", "Ctrl+Right", "Next word"),
     rm("Win+Shift+Left", "Ctrl+Shift+Left", "Select previous word"), rm("Win+Shift+Right", "Ctrl+Shift+Right", "Select next word"),
     rm("Win+Backspace", "Ctrl+Backspace", "Delete previous word"),
   ], on=False, note="The Windows key sits where ⌥ Option is. This replaces Windows' Win+← / Win+→ window snapping."),
   S("Apps and windows", [
     rm("Alt+M", "Win+Down", "Minimize the window"),
     rm("Alt+Space", "Win+S", "Search (like Spotlight)", off=True, note="Replaces Alt+Space, the window menu."),
   ]),
   S("Where Windows works differently", [
     rm("Alt+C", "Ctrl+Shift+C", "Copy in terminals", 1, only="app:terminal", note="In a terminal Ctrl+C stops the running command."),
     rm("Alt+V", "Ctrl+Shift+V", "Paste in terminals", 1, only="app:terminal"),
     rm("Alt+Shift+Z", "Ctrl+Y", "Redo in Office and Notepad", 1, only=["winword.exe", "excel.exe", "powerpnt.exe", "outlook.exe", "notepad.exe"]),
     rm("Alt+Backspace", "Delete", "Move to Recycle Bin (File Explorer)", only="app:explorer"),
     rm("Alt+I", "Alt+Enter", "Properties, like Get Info (File Explorer)", only="app:explorer"),
   ], note="App-specific versions win over the everyday ones."),
  ], tags=["mac", "switcher", "starter"]),

P("windows-on-mac", "Windows keys on a Mac", "remap", "switch", "mac",
  "Coming from Windows? Control does what it did there: copy, paste, save, tabs, word jumps. Mac-only keys stay as they are.",
  [
   S("Everyday", [rm(f"Control+{k}", f"Cmd+{k}", n, f) for k, n, f in [
     ("C", "Copy", 1), ("V", "Paste", 1), ("X", "Cut", 1), ("Z", "Undo", 1), ("A", "Select all", 1), ("S", "Save", 1),
     ("F", "Find", 1), ("N", "New", 1), ("O", "Open", 0), ("P", "Print", 0), ("W", "Close tab or window", 1),
     ("T", "New tab", 1), ("R", "Reload", 0), ("L", "Address bar", 0), ("B", "Bold", 0), ("I", "Italic", 0),
     ("U", "Underline", 0), ("K", "Insert link", 0), ("D", "Bookmark", 0)]] + [
     rm("Control+Y", "Cmd+Shift+Z", "Redo", 1), rm("Control+Shift+Z", "Cmd+Shift+Z", "Redo"),
     rm("Control+Shift+T", "Cmd+Shift+T", "Reopen closed tab"), rm("Control+Shift+N", "Cmd+Shift+N", "New folder / private window"),
     rm("Control+H", "Cmd+Y", "History (browsers)"),
   ], note="Replaces Control's text shortcuts (Control+A start of line, Control+K delete to end)."),
   S("Moving around text", [
     rm("Home", "Cmd+Left", "Start of line", 1), rm("End", "Cmd+Right", "End of line", 1),
     rm("Shift+Home", "Cmd+Shift+Left", "Select to start of line"), rm("Shift+End", "Cmd+Shift+Right", "Select to end of line"),
     rm("Control+Home", "Cmd+Up", "Top of document"), rm("Control+End", "Cmd+Down", "End of document"),
     rm("Control+Left", "Option+Left", "Previous word", 1), rm("Control+Right", "Option+Right", "Next word", 1),
     rm("Control+Shift+Left", "Option+Shift+Left", "Select previous word"), rm("Control+Shift+Right", "Option+Shift+Right", "Select next word"),
     rm("Control+Backspace", "Option+Backspace", "Delete previous word", 1),
   ], note="Control+← / → stop switching Spaces."),
   S("Apps and windows", [
     rm("Option+F4", "Cmd+Q", "Quit the app", 1), rm("Option+Tab", "Cmd+Tab", "Switch apps", 1),
     rm("Control+Shift+Esc", "Cmd+Option+Esc", "Force quit"),
   ]),
  ], tags=["windows", "switcher"]),

P("windows-power-keys", "Windows power keys", "remap", "system", "windows",
  "Small upgrades for any Windows keyboard: media keys on F-keys, fewer accidents, apps one shortcut away.",
  [
   S("Media keys on F-keys", [
     rm("Win+F7", "MediaPrev", "Previous track"), rm("Win+F8", "MediaPlayPause", "Play / pause", 1),
     rm("Win+F9", "MediaNext", "Next track"), rm("Win+F10", "VolumeMute", "Mute", 1),
     rm("Win+F11", "VolumeDown", "Volume down", 1), rm("Win+F12", "VolumeUp", "Volume up", 1),
   ], note="For keyboards without media keys."),
   S("Fewer accidents", [
     dict(press="Insert", block=True, name="Insert does nothing (no surprise overwrite mode)"),
     rm("CapsLock", "Esc", "Caps Lock is Escape", off=True),
   ], on=False),
   S("Open things fast", [
     dict(press="Ctrl+Alt+T", launch="wt.exe", name="Open Terminal", fav=True),
     dict(press="Ctrl+Alt+C", launch="calc.exe", name="Open Calculator"),
     dict(press="Ctrl+Alt+N", launch="notepad.exe", name="Open Notepad"),
   ], on=False),
  ], tags=["system"]),

P("windows-builtins", "Windows shortcuts worth knowing", "cheatsheet", "system", "windows",
  "Built into Windows already. Learn them, or give one your own keys.",
  [
   S("Everyday", [
     cs("Win+V", "Clipboard history", 1), cs("Win+Shift+S", "Screenshot of an area", 1), cs("Win+Period", "Emoji and symbols", 1),
     cs("Win+E", "File Explorer", 1), cs("Win+D", "Show the desktop", 1), cs("Win+L", "Lock the PC", 1),
     cs("Ctrl+Shift+Esc", "Task Manager", 1), cs("Alt+Tab", "Switch windows", 1), cs("Win+Tab", "Task view"),
     cs("Win+I", "Settings"), cs("Win+R", "Run"), cs("Win+S", "Search"), cs("Win+A", "Quick settings"),
     cs("Win+N", "Notifications"), cs("Win+H", "Voice typing"), cs("Win+X", "Power-user menu"),
   ]),
   S("Windows and desktops", [
     cs("Win+Left", "Snap left", 1), cs("Win+Right", "Snap right", 1), cs("Win+Up", "Maximize"), cs("Win+Down", "Minimize / restore"),
     cs("Win+Z", "Snap layouts"), cs("Win+Shift+Left", "Move window to the other screen", 1), cs("Win+M", "Minimize everything"),
     cs("Win+Ctrl+D", "New virtual desktop"), cs("Win+Ctrl+Left", "Previous desktop", 1), cs("Win+Ctrl+Right", "Next desktop", 1),
     cs("Win+Ctrl+F4", "Close this desktop"), cs("Win+1", "Open the first taskbar app (Win+2, Win+3…)", 1),
   ]),
   S("Text", [
     cs("Ctrl+Backspace", "Delete previous word", 1), cs("Ctrl+Left", "Previous word"), cs("Ctrl+Shift+Right", "Select next word"),
     cs("Shift+Home", "Select to start of line"), cs("Shift+End", "Select to end of line"),
   ]),
   S("Screen", [
     cs("Win+PrintScreen", "Screenshot to Pictures", 1), cs("Win+Shift+R", "Record the screen (Snipping Tool)"),
     cs("Win+G", "Game Bar"), cs("Win+Alt+R", "Record a game clip"), cs("Win+P", "Second screen mode"),
     cs("Win+K", "Cast to a screen"), cs("Win+Equals", "Magnifier"),
   ]),
  ], sources=[{"title": "Microsoft: Keyboard shortcuts in Windows", "url": "https://support.microsoft.com/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec"}]),

P("macos-builtins", "Mac shortcuts worth knowing", "cheatsheet", "system", "mac",
  "Built into macOS already. Learn them, or give one your own keys.",
  [
   S("Everyday", [
     cs("Cmd+Space", "Spotlight", 1), cs("Cmd+Tab", "Switch apps", 1), cs("Cmd+Backtick", "Switch windows of this app", 1),
     cs("Cmd+Shift+3", "Screenshot of the screen", 1), cs("Cmd+Shift+4", "Screenshot of an area", 1), cs("Cmd+Shift+5", "Screenshot and recording tools", 1),
     cs("Cmd+Option+Esc", "Force quit", 1), cs("Control+Cmd+Q", "Lock the Mac", 1), cs("Cmd+Q", "Quit"),
     cs("Cmd+H", "Hide the app"), cs("Cmd+Option+H", "Hide other apps"), cs("Cmd+M", "Minimize"),
     cs("Control+Cmd+Space", "Emoji and symbols", 1), cs("Cmd+Comma", "Settings of this app"), cs("Control+Cmd+F", "Full screen"),
     cs("Control+Up", "Mission Control"), cs("Control+Left", "Previous Space"), cs("Control+Right", "Next Space"),
   ]),
   S("Text", [
     cs("Option+Left", "Previous word"), cs("Option+Right", "Next word"), cs("Cmd+Left", "Start of line"), cs("Cmd+Right", "End of line"),
     cs("Option+Backspace", "Delete previous word", 1), cs("Cmd+Backspace", "Delete to start of line"),
     cs("Cmd+Option+Shift+V", "Paste without formatting", 1),
   ]),
   S("Finder", [
     cs("Space", "Quick Look", 1), cs("Cmd+Shift+G", "Go to folder", 1), cs("Cmd+Shift+Period", "Show hidden files", 1),
     cs("Cmd+Backspace", "Move to Trash"), cs("Cmd+I", "Get Info"), cs("Enter", "Rename"), cs("Cmd+Up", "Enclosing folder"),
     cs("Cmd+Down", "Open"), cs("Cmd+Shift+N", "New folder"), cs("Cmd+D", "Duplicate"), cs("Cmd+Option+V", "Move here (after Copy)", 1),
   ]),
  ], sources=[{"title": "Apple: Mac keyboard shortcuts", "url": "https://support.apple.com/en-us/102650"}]),

P("browsers", "Web browsers", "cheatsheet", "browser", "both",
  "Chrome, Edge, Brave, Dia, Arc, Vivaldi and Opera share these.", [
   S("Tabs", [
     cs("Mod+T", "New tab", 1), cs("Mod+W", "Close tab", 1), cs("Mod+Shift+T", "Reopen closed tab", 1),
     cs("Ctrl+Tab", "Next tab", 1), cs("Ctrl+Shift+Tab", "Previous tab"), cs("Mod+1", "First tab (Mod+2… Mod+8)"),
     cs("Mod+9", "Last tab"), cs("Mod+N", "New window"), cs("Mod+Shift+N", "Private window", 1),
   ]),
   S("Page", [
     cs("Mod+L", "Address bar", 1), cs("Mod+R", "Reload", 1), cs("Mod+Shift+R", "Reload without cache", 1),
     cs("Mod+F", "Find on page", 1), cs("Mod+D", "Bookmark"), cs("Mod+P", "Print"), cs("Mod+S", "Save page"),
     cs("Mod+Equals", "Zoom in"), cs("Mod+Minus", "Zoom out"), cs("Mod+0", "Reset zoom"),
     cs("Opt+Left", "Back", 1, mac="Cmd+["), cs("Opt+Right", "Forward", 0, mac="Cmd+]"),
     cs("F11", "Full screen", 0, mac="Control+Cmd+F"),
   ]),
   S("Tools", [
     cs("Ctrl+H", "History", 0, mac="Cmd+Y"), cs("Ctrl+J", "Downloads", 0, mac="Cmd+Option+L"),
     cs("Ctrl+Shift+B", "Show bookmarks bar", 0, mac="Cmd+Shift+B"), cs("F12", "Developer tools", 1, mac="Cmd+Option+I"),
     cs("Ctrl+Shift+Delete", "Clear browsing data", 0, mac="Cmd+Shift+Backspace"), cs("Ctrl+U", "View source", 0, mac="Cmd+Option+U"),
   ]),
  ], app="browsers"),

P("photoshop", "Photoshop", "cheatsheet", "design", "both", "The defaults pros reach for most.", [
   S("Tools", [
     cs("V", "Move", 1), cs("M", "Marquee"), cs("L", "Lasso"), cs("W", "Object / quick selection", 1), cs("C", "Crop", 1),
     cs("I", "Eyedropper"), cs("J", "Spot healing brush", 1), cs("B", "Brush", 1), cs("S", "Clone stamp"), cs("E", "Eraser"),
     cs("G", "Gradient / paint bucket"), cs("O", "Dodge / burn"), cs("P", "Pen"), cs("T", "Type", 1), cs("U", "Shapes"),
     cs("H", "Hand"), cs("Z", "Zoom"), cs("R", "Rotate view"),
   ]),
   S("Painting", [
     cs("[", "Smaller brush", 1), cs("]", "Bigger brush", 1), cs("Shift+[", "Softer brush"), cs("Shift+]", "Harder brush"),
     cs("D", "Default colors", 1), cs("X", "Swap colors", 1), cs("Q", "Quick mask"), cs("5", "50% opacity (1–0 set opacity)", 1),
   ]),
   S("Layers", [
     cs("Mod+J", "Duplicate layer / layer via copy", 1), cs("Mod+Shift+N", "New layer", 1), cs("Mod+G", "Group layers", 1),
     cs("Mod+Shift+G", "Ungroup"), cs("Mod+E", "Merge down", 1), cs("Mod+Shift+E", "Merge visible"),
     cs("Mod+Opt+Shift+E", "Stamp visible to a new layer", 1), cs("Mod+Opt+G", "Clipping mask", 1),
     cs("Mod+Shift+]", "Bring to front"), cs("Mod+]", "Bring forward"), cs("Mod+[", "Send backward"),
   ]),
   S("Editing", [
     cs("Mod+T", "Free transform", 1), cs("Mod+Shift+T", "Transform again"), cs("Mod+Z", "Undo", 1), cs("Mod+Shift+Z", "Redo"),
     cs("Shift+F5", "Fill", 1), cs("Mod+A", "Select all"), cs("Mod+D", "Deselect", 1), cs("Mod+Shift+D", "Reselect"),
     cs("Mod+Shift+I", "Invert selection", 1), cs("Mod+Opt+R", "Select and mask", 1),
   ]),
   S("Adjustments", [
     cs("Mod+L", "Levels", 1), cs("Mod+M", "Curves", 1), cs("Mod+U", "Hue / saturation", 1), cs("Mod+B", "Color balance"),
     cs("Mod+Shift+U", "Desaturate"), cs("Mod+I", "Invert colors"), cs("Mod+Shift+L", "Auto tone"), cs("Mod+Shift+A", "Camera Raw filter", 1),
   ]),
   S("View and file", [
     cs("Mod+0", "Fit on screen", 1), cs("Mod+1", "100%", 1), cs("Mod+R", "Rulers"), cs("Tab", "Hide panels", 1), cs("F", "Screen modes"),
     cs("Mod+Shift+S", "Save as", 1), cs("Mod+Opt+Shift+W", "Export as", 1), cs("Mod+Opt+I", "Image size", 1), cs("Mod+Opt+C", "Canvas size"),
   ]),
  ], app="photoshop", sources=[{"title": "Adobe: Photoshop keyboard shortcuts", "url": "https://helpx.adobe.com/photoshop/using/default-keyboard-shortcuts.html"}, SM]),

P("illustrator", "Illustrator", "cheatsheet", "design", "both", "The defaults pros reach for most.", [
   S("Tools", [
     cs("V", "Selection", 1), cs("A", "Direct selection", 1), cs("P", "Pen", 1), cs("T", "Type", 1), cs("M", "Rectangle", 1),
     cs("L", "Ellipse", 1), cs("\\", "Line segment"), cs("B", "Paintbrush"), cs("Shift+B", "Blob brush"), cs("N", "Pencil"),
     cs("R", "Rotate"), cs("O", "Reflect"), cs("S", "Scale"), cs("E", "Free transform", 1), cs("Shift+M", "Shape builder", 1),
     cs("I", "Eyedropper", 1), cs("G", "Gradient"), cs("Shift+O", "Artboard", 1), cs("H", "Hand"), cs("Z", "Zoom"),
   ]),
   S("Color", [cs("X", "Fill / stroke focus", 1), cs("Shift+X", "Swap fill and stroke"), cs("D", "Default fill and stroke"), cs("/", "None")]),
   S("Objects", [
     cs("Mod+G", "Group", 1), cs("Mod+Shift+G", "Ungroup"), cs("Mod+2", "Lock"), cs("Mod+Opt+2", "Unlock all"),
     cs("Mod+3", "Hide"), cs("Mod+Opt+3", "Show all"), cs("Mod+7", "Clipping mask", 1), cs("Mod+Opt+7", "Release clipping mask"),
     cs("Mod+8", "Compound path"), cs("Mod+J", "Join", 1), cs("Mod+D", "Transform again", 1), cs("Mod+Shift+O", "Create outlines", 1),
     cs("Mod+Shift+]", "Bring to front", 1), cs("Mod+]", "Bring forward"), cs("Mod+[", "Send backward"), cs("Mod+Shift+[", "Send to back"),
     cs("Mod+F", "Paste in front", 1), cs("Mod+B", "Paste in back"), cs("Mod+Shift+V", "Paste in place", 1),
   ]),
   S("View and file", [
     cs("Mod+Y", "Outline view", 1), cs("Mod+R", "Rulers"), cs("Mod+0", "Fit artboard", 1), cs("Mod+Opt+0", "Fit all"),
     cs("Mod+1", "100%"), cs("Mod+Opt+E", "Export for screens", 1), cs("Mod+Shift+S", "Save as"),
   ]),
  ], app="illustrator", sources=[{"title": "Adobe: Illustrator keyboard shortcuts", "url": "https://helpx.adobe.com/illustrator/using/default-keyboard-shortcuts.html"}, SM]),

P("after-effects", "After Effects", "cheatsheet", "video", "both", "The defaults motion designers reach for most.", [
   S("Tools", [
     cs("V", "Selection", 1), cs("H", "Hand"), cs("Z", "Zoom"), cs("W", "Rotation"), cs("C", "Camera"), cs("Y", "Pan behind (anchor point)", 1),
     cs("Q", "Shape tools", 1), cs("G", "Pen", 1), cs("Mod+T", "Type", 1), cs("Mod+B", "Brush / clone / eraser"), cs("Opt+W", "Roto brush", 1), cs("Mod+P", "Puppet"),
   ]),
   S("Layers", [
     cs("Mod+Y", "New solid", 1), cs("Mod+Opt+Y", "New adjustment layer", 1), cs("Mod+Opt+Shift+Y", "New null", 1),
     cs("Mod+Shift+C", "Pre-compose", 1), cs("Mod+D", "Duplicate", 1), cs("Mod+Shift+D", "Split layer", 1),
     cs("Mod+Opt+Home", "Center anchor point", 1), cs("Mod+Opt+F", "Fit to comp", 1), cs("Mod+Opt+R", "Time-reverse"),
     cs("Mod+Opt+T", "Time remapping"), cs("Mod+L", "Lock"), cs("Opt+[", "Trim in point to playhead", 1), cs("Opt+]", "Trim out point to playhead", 1),
     cs("[", "Move in point to playhead"), cs("]", "Move out point to playhead"),
   ]),
   S("Properties", [
     cs("P", "Position", 1), cs("S", "Scale", 1), cs("R", "Rotation", 1), cs("T", "Opacity", 1), cs("A", "Anchor point", 1),
     cs("U", "Animated properties (UU: everything changed)", 1), cs("E", "Effects"), cs("M", "Masks"), cs("F", "Mask feather"), cs("L", "Audio levels"),
   ]),
   S("Keyframes", [
     cs("F9", "Easy ease", 1), cs("Shift+F9", "Ease in"), cs("Mod+Shift+F9", "Ease out"), cs("J", "Previous keyframe", 1), cs("K", "Next keyframe", 1),
     cs("Opt+Shift+P", "Position keyframe", 1), cs("Opt+Shift+S", "Scale keyframe"), cs("Opt+Shift+R", "Rotation keyframe"), cs("Opt+Shift+T", "Opacity keyframe"),
   ]),
   S("Comp and time", [
     cs("Mod+N", "New composition", 1), cs("Mod+K", "Composition settings", 1), cs("Mod+M", "Add to render queue", 1),
     cs("Mod+Opt+M", "Add to Media Encoder queue", 1), cs("Mod+I", "Import", 1), cs("B", "Work area start", 1), cs("N", "Work area end", 1),
     cs("Space", "Play", 1), cs("PageUp", "Previous frame", 1), cs("PageDown", "Next frame", 1), cs("Home", "Start"), cs("End", "End"),
     cs("Backtick", "Maximize panel", 1), cs("F2", "Deselect all layers", 1),
   ]),
  ], app="aftereffects", sources=[{"title": "Adobe: After Effects keyboard shortcuts", "url": "https://helpx.adobe.com/after-effects/using/keyboard-shortcuts-reference.html"}, SM]),

P("premiere-pro", "Premiere Pro", "cheatsheet", "video", "both", "The defaults editors reach for most.", [
   S("Tools", [
     cs("V", "Selection", 1), cs("A", "Track select forward", 1), cs("B", "Ripple edit", 1), cs("N", "Rolling edit"),
     cs("R", "Rate stretch"), cs("C", "Razor", 1), cs("Y", "Slip"), cs("U", "Slide"), cs("P", "Pen"), cs("H", "Hand"), cs("Z", "Zoom"), cs("T", "Type", 1),
   ]),
   S("Editing", [
     cs("Mod+K", "Add edit (cut at playhead)", 1), cs("Mod+Shift+K", "Add edit to all tracks", 1),
     cs("Q", "Ripple trim previous edit to playhead", 1), cs("W", "Ripple trim next edit to playhead", 1),
     cs("Shift+Delete", "Ripple delete", 1, mac="Shift+Backspace"), cs("E", "Extend edit to playhead"),
     cs("Mod+L", "Link / unlink"), cs("Mod+D", "Default video transition", 1), cs("Mod+Shift+D", "Default audio transition", 1),
     cs("Mod+R", "Speed / duration", 1), cs("Mod+Opt+V", "Paste attributes", 1), cs("Comma", "Insert", 1), cs("Period", "Overwrite", 1),
   ]),
   S("Timeline", [
     cs("I", "Mark in", 1), cs("O", "Mark out", 1), cs("X", "Mark clip"), cs("J", "Play backward", 1), cs("K", "Stop", 1), cs("L", "Play forward", 1),
     cs("Space", "Play / stop", 1), cs("Up", "Previous edit point", 1), cs("Down", "Next edit point", 1), cs("S", "Snap", 1),
     cs("M", "Add marker", 1), cs("Shift+M", "Next marker"), cs("Mod+Shift+M", "Previous marker"),
     cs("Equals", "Zoom in", 1), cs("Minus", "Zoom out", 1), cs("\\", "Zoom to sequence", 1), cs("Backtick", "Maximize panel", 1),
   ]),
   S("File", [
     cs("Mod+M", "Export media", 1), cs("Mod+Shift+E", "Export frame", 1), cs("Mod+I", "Import", 1), cs("Mod+N", "New sequence"),
   ]),
  ], app="premiere", sources=[{"title": "Adobe: Premiere Pro keyboard shortcuts", "url": "https://helpx.adobe.com/premiere-pro/using/keyboard-shortcuts.html"}]),

P("davinci-resolve", "DaVinci Resolve", "cheatsheet", "video", "both", "Edit and Color page defaults editors use most.", [
   S("Edit modes", [
     cs("A", "Selection (normal edit)", 1), cs("B", "Blade edit mode", 1), cs("T", "Trim edit mode", 1), cs("W", "Dynamic trim"),
     cs("R", "Range selection"), cs("N", "Snapping", 1), cs("D", "Enable / disable clip", 1),
   ]),
   S("Cutting and trimming", [
     cs("Mod+B", "Split clip at playhead", 1), cs("Mod+\\", "Add edit"), cs("Opt+\\", "Join clip"),
     cs("Shift+[", "Trim start to playhead", 1), cs("Shift+]", "Trim end to playhead", 1),
     cs("Mod+Shift+[", "Ripple start to playhead", 1), cs("Mod+Shift+]", "Ripple end to playhead", 1),
     cs("Shift+Backspace", "Ripple delete", 1), cs("Backspace", "Delete (leave the gap)"),
   ]),
   S("Editing", [
     cs("F9", "Insert", 1), cs("F10", "Overwrite", 1), cs("F11", "Replace"), cs("F12", "Place on top"), cs("Shift+F12", "Append to end", 1),
     cs("Mod+T", "Add transition", 1), cs("Opt+T", "Video-only transition"), cs("Shift+T", "Audio crossfade"),
     cs("Mod+R", "Retime controls", 1), cs("Mod+Opt+L", "Link / unlink", 1), cs("Opt+V", "Paste attributes", 1), cs("Opt+Y", "Select clips after playhead"),
   ]),
   S("Timeline", [
     cs("I", "Mark in", 1), cs("O", "Mark out", 1), cs("J", "Play backward"), cs("K", "Stop"), cs("L", "Play forward"), cs("Space", "Play / stop", 1),
     cs("Up", "Previous edit", 1), cs("Down", "Next edit", 1), cs("M", "Add marker", 1), cs("Shift+M", "Modify marker"), cs("Opt+M", "Delete marker"),
     cs("Mod+Equals", "Zoom in"), cs("Mod+Minus", "Zoom out"), cs("Shift+Z", "Zoom to fit", 1),
   ]),
   S("Pages and color", [
     cs("Shift+2", "Media page"), cs("Shift+3", "Cut page"), cs("Shift+4", "Edit page", 1), cs("Shift+5", "Fusion page"),
     cs("Shift+6", "Color page", 1), cs("Shift+7", "Fairlight page"), cs("Shift+8", "Deliver page", 1),
     cs("Opt+S", "Add serial node", 1), cs("Opt+P", "Add parallel node"), cs("Opt+L", "Add layer node"),
     cs("Mod+D", "Disable / enable node", 1), cs("Shift+D", "Bypass all grades", 1), cs("Mod+Opt+G", "Grab still", 1),
   ]),
  ], app="resolve", sources=[{"title": "Blackmagic Design: DaVinci Resolve manual (keyboard shortcuts)", "url": "https://www.blackmagicdesign.com/products/davinciresolve/training"}]),

P("capcut", "CapCut (desktop)", "cheatsheet", "video", "both", "The timeline keys that make rough cuts fast.", [
   S("Cutting", [
     cs("Mod+B", "Split at playhead", 1), cs("Mod+Shift+B", "Split all tracks"), cs("B", "Split mode"), cs("A", "Select mode"),
     cs("Q", "Delete left of playhead", 1), cs("W", "Delete right of playhead", 1), cs("Mod+G", "Group"), cs("Mod+Shift+G", "Ungroup"),
   ]),
   S("Playback and timeline", [
     cs("Space", "Play / pause", 1), cs("Left", "Previous frame"), cs("Right", "Next frame"), cs("Up", "Previous cut", 1), cs("Down", "Next cut", 1),
     cs("J", "Play backward"), cs("K", "Stop"), cs("L", "Play forward"), cs("M", "Add marker"),
     cs("Shift+Z", "Zoom to fit", 1), cs("Mod+Equals", "Zoom in"), cs("Mod+Minus", "Zoom out"),
   ]),
   S("File", [cs("Mod+I", "Import media", 1), cs("Mod+E", "Export", 1)]),
  ], app="capcut", sources=[{"title": "CapCut: desktop shortcuts", "url": "https://www.capcut.com/resource/pc-professional-video-editor"}]),

P("lightroom-classic", "Lightroom Classic", "cheatsheet", "design", "both", "Library and Develop defaults photographers use most.", [
   S("Views", [
     cs("G", "Grid", 1), cs("E", "Loupe", 1), cs("C", "Compare"), cs("N", "Survey"), cs("D", "Develop", 1), cs("F", "Full-screen preview"),
     cs("L", "Lights out"), cs("Tab", "Hide side panels", 1), cs("Shift+Tab", "Hide all panels"), cs("I", "Info overlay"),
   ]),
   S("Culling", [
     cs("P", "Pick", 1), cs("X", "Reject", 1), cs("U", "Unflag"), cs("5", "Five stars (1–5 set stars)", 1), cs("0", "No stars"), cs("6", "Red label (6–9 set labels)"),
   ]),
   S("Develop", [
     cs("R", "Crop", 1), cs("Q", "Spot removal / healing", 1), cs("V", "Black and white", 1), cs("\\", "Before / after", 1), cs("Y", "Before / after side by side"),
     cs("J", "Show clipping", 1), cs("Mod+U", "Auto settings", 1), cs("Mod+Shift+C", "Copy settings", 1), cs("Mod+Shift+V", "Paste settings", 1),
     cs("Mod+Shift+R", "Reset"), cs("Mod+Quote", "Virtual copy", 1),
   ]),
   S("File", [cs("Mod+Shift+I", "Import", 1), cs("Mod+Shift+E", "Export", 1)]),
  ], app="lightroom", sources=[{"title": "Adobe: Lightroom Classic keyboard shortcuts", "url": "https://helpx.adobe.com/lightroom-classic/help/keyboard-shortcuts.html"}, SM]),

P("figma", "Figma", "cheatsheet", "design", "both", "Works in the Figma desktop app.", [
   S("Tools", [
     cs("V", "Move", 1), cs("K", "Scale"), cs("F", "Frame", 1), cs("R", "Rectangle", 1), cs("O", "Ellipse", 1), cs("L", "Line"),
     cs("Shift+L", "Arrow"), cs("P", "Pen", 1), cs("Shift+P", "Pencil"), cs("T", "Text", 1), cs("C", "Comment"), cs("I", "Eyedropper", 1),
   ]),
   S("Layers", [
     cs("Mod+G", "Group", 1), cs("Mod+Shift+G", "Ungroup"), cs("Mod+Opt+G", "Frame selection", 1), cs("Shift+A", "Auto layout", 1),
     cs("Mod+Opt+K", "Create component", 1), cs("Mod+D", "Duplicate", 1), cs("Mod+R", "Rename", 1), cs("Mod+E", "Flatten"),
     cs("Mod+Shift+H", "Show / hide", 1), cs("Mod+Shift+L", "Lock / unlock"),
     cs("Mod+Opt+C", "Copy properties", 1), cs("Mod+Opt+V", "Paste properties", 1),
   ]),
   S("View and file", [
     cs("Mod+/", "Quick actions", 1), cs("Shift+1", "Zoom to fit", 1), cs("Shift+2", "Zoom to selection", 1), cs("Shift+0", "Zoom to 100%"),
     cs("Mod+\\", "Show / hide the UI", 1), cs("Mod+Y", "Outlines"), cs("Mod+Shift+R", "Rulers"),
     cs("Mod+Shift+E", "Export", 1), cs("Mod+Shift+K", "Place image"),
   ]),
  ], app="figma", sources=[{"title": "Figma: Use keyboard shortcuts", "url": "https://help.figma.com/hc/en-us/articles/360040328653"}]),

P("blender", "Blender", "cheatsheet", "3d", "both", "Blender 4 defaults, modeling to render.", [
   S("Transform and modes", [
     cs("G", "Move (grab)", 1), cs("R", "Rotate", 1), cs("S", "Scale", 1), cs("Tab", "Edit mode", 1), cs("Shift+A", "Add", 1),
     cs("Shift+D", "Duplicate", 1), cs("Opt+D", "Linked duplicate"), cs("X", "Delete", 1), cs("Ctrl+A", "Apply (transforms)", 1),
     cs("Shift+R", "Repeat last", 1), cs("F2", "Rename", 1), cs("F3", "Search (menu search)", 1),
   ]),
   S("Modeling", [
     cs("E", "Extrude", 1), cs("I", "Inset", 1), cs("Ctrl+R", "Loop cut", 1), cs("K", "Knife"), cs("Ctrl+B", "Bevel", 1),
     cs("F", "Fill / new face"), cs("M", "Merge (edit) / move to collection", 1), cs("Ctrl+J", "Join", 1), cs("P", "Separate"),
     cs("Ctrl+P", "Parent", 1), cs("Opt+P", "Clear parent"), cs("1", "Vertex select (edit mode)", 1), cs("2", "Edge select"), cs("3", "Face select"),
   ]),
   S("Selection and visibility", [
     cs("A", "Select all", 1), cs("Opt+A", "Deselect all"), cs("Ctrl+I", "Invert selection"), cs("L", "Select linked"), cs("Ctrl+L", "Select linked (all)"),
     cs("B", "Box select"), cs("C", "Circle select"), cs("H", "Hide", 1), cs("Opt+H", "Unhide", 1), cs("Shift+H", "Hide unselected"),
   ]),
   S("View", [
     cs("Num1", "Front view", 1), cs("Num3", "Right view"), cs("Num7", "Top view", 1), cs("Num5", "Perspective / orthographic"),
     cs("Num0", "Camera view", 1), cs("NumDecimal", "Frame selected", 1), cs("Home", "Frame all", 1), cs("Backtick", "View pie menu", 1),
     cs("Z", "Shading pie menu", 1), cs("Opt+Z", "X-ray", 1), cs("Ctrl+Space", "Maximize area", 1), cs("N", "Sidebar", 1), cs("T", "Toolbar"),
   ]),
   S("Animation, render and file", [
     cs("I", "Insert keyframe", 1), cs("Opt+I", "Delete keyframe"), cs("Space", "Play", 1), cs("Shift+Left", "Jump to start"),
     cs("F12", "Render image", 1), cs("Ctrl+F12", "Render animation", 1), cs("F11", "Show render"),
     cs("Mod+S", "Save", 1), cs("Mod+Shift+S", "Save as"), cs("Mod+Z", "Undo"), cs("Mod+Shift+Z", "Redo", 1),
   ]),
  ], app="blender", sources=[{"title": "Blender Manual", "url": "https://docs.blender.org/manual/en/latest/"}]),

P("cinema-4d", "Cinema 4D", "cheatsheet", "3d", "both", "Core Cinema 4D defaults.", [
   S("Essentials", [
     cs("E", "Move", 1), cs("R", "Rotate", 1), cs("T", "Scale", 1), cs("C", "Make editable", 1), cs("Opt+G", "Group objects", 1),
     cs("Shift+C", "Commander (search everything)", 1), cs("Mod+R", "Render view", 1), cs("Shift+R", "Render to Picture Viewer", 1),
     cs("Mod+D", "Project settings"), cs("Mod+E", "Preferences"), cs("Mod+S", "Save", 1),
   ]),
   S("Viewports", [cs("F1", "Perspective"), cs("F2", "Top"), cs("F3", "Right"), cs("F4", "Front"), cs("F5", "All four views")]),
  ], app="cinema4d", sources=[{"title": "Maxon: Cinema 4D help", "url": "https://help.maxon.net/c4d/en-us/"}]),

P("unreal-engine", "Unreal Engine", "cheatsheet", "3d", "both",
  "Unreal Engine 5 editor defaults: building levels, the viewport, play-testing and Blueprints.", [
   S("Move, rotate, scale", [
     cs("W", "Move (translate) tool", 1), cs("E", "Rotate tool", 1), cs("R", "Scale tool", 1), cs("Q", "Selection tool"),
     cs("Space", "Cycle move, rotate and scale", 1), cs("F", "Focus on the selection", 1), cs("End", "Snap to the floor", 1),
     cs("Opt+End", "Snap the pivot to the floor"), cs("Mod+D", "Duplicate", 1), cs("Delete", "Delete"),
     cs("H", "Hide selected", 1), cs("Mod+H", "Unhide all"), cs("Mod+G", "Group"), cs("Shift+G", "Ungroup"),
   ]),
   S("Viewport", [
     cs("G", "Game view (hide editor icons)", 1), cs("Mod+R", "Real-time on / off"), cs("F11", "Immersive full screen"),
     cs("Opt+G", "Perspective view", 1), cs("Opt+J", "Top view"), cs("Opt+H", "Front view"), cs("Opt+K", "Left view"),
     cs("Opt+Shift+J", "Bottom view"), cs("Opt+Shift+H", "Back view"), cs("Opt+Shift+K", "Right view"),
     cs("Opt+4", "Lit", 1), cs("Opt+3", "Unlit"), cs("Opt+2", "Wireframe", 1), cs("Opt+5", "Detail lighting"), cs("Opt+6", "Lighting only"),
     cs("Mod+1", "Set bookmark 1 (works with 0–9)"), cs("1", "Jump to bookmark 1 (works with 0–9)"),
     cs("P", "Show the navigation mesh"), cs("Mod+Shift+H", "Show FPS"),
   ]),
   S("Play and test", [
     cs("Opt+P", "Play in the editor", 1), cs("Opt+S", "Simulate in the editor", 1), cs("Shift+F1", "Get the mouse back while playing", 1),
     cs("Esc", "Stop playing"), cs("Backtick", "Console"),
   ]),
   S("Files and panels", [
     cs("Mod+S", "Save the current level", 1), cs("Mod+Shift+S", "Save all", 1), cs("Mod+Opt+S", "Save the current level as"),
     cs("Mod+N", "New level"), cs("Mod+O", "Open level"), cs("Ctrl+Space", "Content Drawer", 1, note="Control+Space on a Mac too"),
     cs("Mod+B", "Find in the Content Browser", 1), cs("Mod+Z", "Undo"), cs("Mod+Y", "Redo"), cs("F1", "Documentation"),
   ]),
   S("Blueprints", [
     cs("F7", "Compile", 1), cs("C", "Comment around the selection", 1), cs("Home", "Zoom to the selection"),
     cs("Mod+F", "Find in this Blueprint"), cs("Mod+Shift+F", "Find in all Blueprints"), cs("F2", "Rename node"),
     cs("F9", "Breakpoint on / off"), cs("Mod+Shift+F9", "Clear all breakpoints"), cs("PageDown", "Into the child graph"), cs("PageUp", "Back to the parent graph"),
   ]),
  ], app="unreal", sources=[
     {"title": "Epic: Viewport controls in Unreal Engine", "url": "https://dev.epicgames.com/documentation/en-us/unreal-engine/viewport-controls-in-unreal-engine"},
     {"title": "Epic: Blueprint editor cheat sheet", "url": "https://dev.epicgames.com/documentation/en-us/unreal-engine/blueprint-editor-cheat-sheet-in-unreal-engine"},
     {"title": "Epic: Playing and simulating", "url": "https://dev.epicgames.com/documentation/en-us/unreal-engine/playing-and-simulating-in-unreal-engine"},
     {"title": "Epic: Actor snapping", "url": "https://dev.epicgames.com/documentation/en-us/unreal-engine/actor-snapping-in-unreal-engine"},
     {"title": "Epic: Keyboard shortcuts in the Unreal Editor (Fortnite)", "url": "https://dev.epicgames.com/documentation/en-us/fortnite/keyboard-shortcuts-in-unreal-editor-for-fortnite"},
  ]),

P("vscode", "VS Code", "cheatsheet", "code", "both", "Get around code without the mouse.", [
   S("Find anything", [
     cs("Mod+P", "Quick open a file", 1), cs("Mod+Shift+P", "Command palette", 1), cs("Mod+Shift+O", "Go to symbol", 1),
     cs("Mod+G", "Go to line", 1, mac="Control+G"), cs("Mod+Shift+F", "Search all files", 1), cs("F12", "Go to definition", 1),
     cs("Opt+F12", "Peek definition"), cs("Shift+F12", "Find references"),
   ]),
   S("Editing", [
     cs("Mod+/", "Toggle comment", 1), cs("Opt+Up", "Move line up", 1), cs("Opt+Down", "Move line down", 1),
     cs("Shift+Opt+Down", "Copy line down", 1), cs("Mod+D", "Select next match", 1), cs("Mod+Shift+L", "Select all matches"),
     cs("Mod+Shift+K", "Delete line", 1), cs("F2", "Rename symbol", 1), cs("Shift+Opt+F", "Format document", 1),
     cs("Mod+Opt+Up", "Add cursor above", 1), cs("Mod+Opt+Down", "Add cursor below"), cs("Mod+Enter", "New line below", 1), cs("Mod+Shift+Enter", "New line above"),
   ]),
   S("Layout", [
     cs("Mod+B", "Toggle sidebar", 1), cs("Ctrl+Backtick", "Terminal", 1), cs("Mod+J", "Toggle panel", 1), cs("Mod+\\", "Split editor", 1),
     cs("Mod+W", "Close editor"), cs("Ctrl+Tab", "Switch editor"), cs("Mod+Shift+E", "Explorer"), cs("Mod+Shift+G", "Source control", 0, mac="Control+Shift+G"),
     cs("Mod+Shift+X", "Extensions"), cs("Mod+Comma", "Settings"),
   ]),
   S("Debug", [cs("F5", "Start debugging", 1), cs("F9", "Toggle breakpoint", 1), cs("F10", "Step over"), cs("F11", "Step into")]),
  ], app="vscode", sources=[{"title": "VS Code: keyboard shortcuts reference", "url": "https://code.visualstudio.com/docs/getstarted/keybindings"}]),

P("obsidian", "Obsidian", "cheatsheet", "work", "both", "Notes at the speed of thought.", [
   S("Find and open", [
     cs("Mod+O", "Quick switcher", 1), cs("Mod+P", "Command palette", 1), cs("Mod+Shift+F", "Search all notes", 1), cs("Mod+G", "Graph view", 1),
     cs("Mod+N", "New note", 1), cs("Mod+T", "New tab"), cs("Mod+W", "Close tab"), cs("Mod+Opt+Left", "Back", 1), cs("Mod+Opt+Right", "Forward"),
     cs("Mod+Comma", "Settings"),
   ]),
   S("Writing", [
     cs("Mod+E", "Reading / editing view", 1), cs("Mod+K", "Insert link", 1), cs("Mod+Enter", "Toggle checkbox", 1), cs("Mod+B", "Bold"),
     cs("Mod+I", "Italic"), cs("Mod+]", "Indent"), cs("Mod+[", "Unindent"), cs("Mod+F", "Find in note"), cs("Mod+H", "Find and replace"), cs("Mod+/", "Toggle comment"),
   ]),
  ], app="obsidian", sources=[{"title": "Obsidian Help: Hotkeys", "url": "https://help.obsidian.md/Customization/Hotkeys"}]),

P("file-explorer", "File Explorer", "cheatsheet", "system", "windows", "Move through folders without the mouse.", [
   S("Essentials", [
     cs("Ctrl+Shift+N", "New folder", 1), cs("F2", "Rename", 1), cs("Alt+Up", "Up one folder", 1), cs("Alt+Left", "Back"), cs("Alt+Right", "Forward"),
     cs("Alt+D", "Address bar", 1), cs("Ctrl+F", "Search"), cs("Ctrl+T", "New tab", 1), cs("Ctrl+W", "Close tab"), cs("Ctrl+N", "New window"),
     cs("Alt+Enter", "Properties", 1), cs("Alt+P", "Preview pane", 1), cs("Shift+Delete", "Delete permanently", 1),
     cs("Ctrl+Shift+C", "Copy as path", 1), cs("F5", "Refresh"), cs("F11", "Full screen"),
   ]),
  ], app="explorer", sources=[{"title": "Microsoft: Keyboard shortcuts in Windows", "url": "https://support.microsoft.com/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec"}]),

P("premiere-keys-in-resolve", "Premiere keys in DaVinci Resolve", "remap", "video", "both",
  "Moving from Premiere Pro? Keep your hands: Premiere's keys do the matching thing in Resolve.", [
   S("Editing", [
     rm("Mod+K", "Mod+B", "Add edit (cut at playhead)", 1), rm("Mod+D", "Mod+T", "Add the default transition", 1),
     rm("Mod+Shift+D", "Shift+T", "Audio crossfade"), rm("Mod+L", "Mod+Opt+L", "Link / unlink", 1),
     rm("Mod+Opt+V", "Opt+V", "Paste attributes", 1), rm("Shift+Delete", "Shift+Backspace", "Ripple delete", 1, platform="windows"),
     rm("Mod+M", "Shift+8", "Export (Deliver page)"),
   ]),
   S("Single keys", [
     rm("V", "A", "Selection tool", 1), rm("C", "B", "Razor (blade)", 1), rm("B", "T", "Ripple / trim edit tool"),
     rm("Q", "Mod+Shift+[", "Ripple trim to playhead (start)", 1), rm("W", "Mod+Shift+]", "Ripple trim to playhead (end)", 1),
     rm("S", "N", "Snapping"), rm("\\", "Shift+Z", "Zoom to fit the timeline"), rm("Comma", "F9", "Insert"), rm("Period", "F10", "Overwrite"),
   ], note="Single keys pause by themselves while you type in a text box."),
  ], app="resolve", tags=["premiere", "resolve", "switcher"]),

P("premiere-keys-in-capcut", "Premiere keys in CapCut", "remap", "video", "both",
  "Premiere habits that work in CapCut's desktop editor.", [
   S("Editing", [rm("Mod+K", "Mod+B", "Split at playhead", 1), rm("Mod+Shift+K", "Mod+Shift+B", "Split all tracks")]),
   S("Single keys", [rm("C", "B", "Split mode", 1), rm("V", "A", "Select mode", 1), rm("\\", "Shift+Z", "Zoom to fit the timeline")],
     note="Single keys pause by themselves while you type in a text box."),
  ], app="capcut", tags=["premiere", "capcut", "switcher"]),

P("notion", "Notion", "cheatsheet", "work", "both", "Pages, blocks and search without the mouse.", [
   S("Find and open", [
     cs("Mod+P", "Search, or jump to a recent page", 1), cs("Mod+N", "New page", 1), cs("Mod+T", "New tab"), cs("Mod+Shift+T", "Reopen the last closed tab"),
     cs("Mod+Shift+N", "New Notion window"), cs("Mod+[", "Back", 1), cs("Mod+]", "Forward"), cs("Mod+Shift+U", "Up one level", 1),
     cs("Mod+L", "Copy the page link", 1), cs("Mod+F", "Find in page"), cs("Mod+Shift+L", "Dark / light mode"),
   ]),
   S("Text", [
     cs("Mod+B", "Bold"), cs("Mod+I", "Italic"), cs("Mod+U", "Underline"), cs("Mod+Shift+S", "Strikethrough"), cs("Mod+E", "Inline code", 1),
     cs("Mod+K", "Add a link", 1), cs("Mod+Shift+H", "Last text or highlight colour", 1), cs("Mod+Shift+M", "Comment", 1),
   ]),
   S("Make blocks", [
     cs("Mod+Shift+0", "Text", mac="Cmd+Option+0"), cs("Mod+Shift+1", "Heading 1", 1, mac="Cmd+Option+1"), cs("Mod+Shift+2", "Heading 2", 1, mac="Cmd+Option+2"),
     cs("Mod+Shift+3", "Heading 3", mac="Cmd+Option+3"), cs("Mod+Shift+4", "To-do checkbox", 1, mac="Cmd+Option+4"), cs("Mod+Shift+5", "Bulleted list", mac="Cmd+Option+5"),
     cs("Mod+Shift+6", "Numbered list", mac="Cmd+Option+6"), cs("Mod+Shift+7", "Toggle list", mac="Cmd+Option+7"), cs("Mod+Shift+8", "Code block", mac="Cmd+Option+8"),
     cs("Mod+Shift+9", "New page, or turn the line into one", mac="Cmd+Option+9"),
   ]),
   S("Move blocks", [
     cs("Esc", "Select the block you're in", 1), cs("Mod+D", "Duplicate", 1), cs("Mod+/", "Turn into / change", 1), cs("Mod+Shift+Up", "Move up", 1),
     cs("Mod+Shift+Down", "Move down"), cs("Mod+Opt+T", "Open or close all toggles"), cs("Mod+Enter", "Open the page, or tick the to-do"),
   ]),
  ], app="notion", sources=[{"title": "Notion Help: Keyboard shortcuts", "url": "https://www.notion.com/help/keyboard-shortcuts"}]),

P("slack", "Slack", "cheatsheet", "work", "both", "Get around Slack and write faster.", [
   S("Get around", [
     cs("Mod+K", "Jump to a conversation", 1), cs("Mod+J", "Jump to the newest unread", 1), cs("Mod+Shift+A", "All unreads", 1), cs("Mod+Shift+T", "Threads"),
     cs("Mod+Shift+2", "Direct messages", mac="Cmd+Shift+K"), cs("Mod+Shift+M", "Activity"), cs("Mod+Shift+1", "Home", mac="Control+1"),
     cs("Mod+Shift+L", "Browse channels"), cs("Mod+Shift+I", "Conversation details"), cs("Opt+Left", "Back", mac="Cmd+LBracket"),
     cs("Opt+Right", "Forward", mac="Cmd+RBracket"), cs("Opt+Shift+Down", "Next unread conversation", 1), cs("Mod+G", "Search", 1), cs("Mod+F", "Search this conversation"),
   ]),
   S("Messages", [
     cs("Up", "Edit your last message", 1), cs("Esc", "Mark this conversation read", 1), cs("Shift+Esc", "Mark everything read"), cs("Mod+O", "Upload a file"),
     cs("Mod+B", "Bold"), cs("Mod+I", "Italic"), cs("Mod+Shift+X", "Strikethrough"), cs("Mod+Shift+C", "Code"), cs("Mod+Opt+Shift+C", "Code block"),
     cs("Mod+Shift+U", "Link the selected text", 1),
   ]),
   S("Huddles and you", [
     cs("Mod+Shift+H", "Start, join or leave a huddle", 1), cs("Mod+Shift+Space", "Mute in a huddle", 1), cs("Mod+Shift+Y", "Set your status"),
     cs("Mod+Period", "Hide the right sidebar"), cs("Mod+Comma", "Preferences"), cs("F11", "Full screen", mac="Control+Cmd+F"),
   ]),
  ], app="slack", sources=[{"title": "Slack Help: Slack keyboard shortcuts", "url": "https://slack.com/help/articles/201374536-Slack-keyboard-shortcuts"}]),

P("word", "Word", "cheatsheet", "work", "windows", "Styles, layout and review in Word for Windows.", [
   S("Styles and text", [
     cs("Ctrl+Alt+1", "Heading 1", 1), cs("Ctrl+Alt+2", "Heading 2", 1), cs("Ctrl+Alt+3", "Heading 3"), cs("Ctrl+Shift+N", "Normal text", 1),
     cs("Ctrl+Shift+L", "Bulleted list"), cs("Ctrl+]", "Font one point bigger"), cs("Ctrl+[", "Font one point smaller"), cs("Ctrl+Shift+A", "ALL CAPS"),
     cs("Shift+F3", "Cycle upper, lower and title case", 1), cs("Ctrl+Space", "Clear character formatting", 1), cs("Ctrl+Q", "Clear paragraph formatting"),
   ]),
   S("Layout", [
     cs("Ctrl+L", "Align left"), cs("Ctrl+E", "Centre", 1), cs("Ctrl+R", "Align right"), cs("Ctrl+J", "Justify"), cs("Ctrl+1", "Single line spacing"),
     cs("Ctrl+5", "1.5 line spacing"), cs("Ctrl+2", "Double line spacing"), cs("Ctrl+Enter", "Page break", 1),
   ]),
   S("Edit and move", [
     cs("Ctrl+Backspace", "Delete the word before", 1), cs("Ctrl+Shift+V", "Paste as plain text", 1, note="Word for Microsoft 365."),
     cs("Ctrl+Alt+C", "Copy formatting"), cs("Ctrl+Alt+V", "Paste formatting only"), cs("Ctrl+G", "Go to a page", 1),
     cs("Ctrl+Home", "Start of the document"), cs("Ctrl+End", "End of the document"), cs("Ctrl+PageDown", "Next page"), cs("Ctrl+PageUp", "Previous page"),
   ]),
   S("Insert and review", [
     cs("Ctrl+K", "Link", 1), cs("Ctrl+Alt+M", "Comment", 1), cs("Ctrl+Alt+F", "Footnote"), cs("Alt+Shift+D", "Today's date"),
     cs("Ctrl+Shift+E", "Track changes on or off", 1), cs("F7", "Spelling and grammar", 1), cs("Ctrl+Alt+I", "Print preview"),
     cs("Ctrl+Alt+S", "Split the window"), cs("Ctrl+Alt+O", "Outline view"),
   ]),
  ], app="word", sources=[{"title": "Microsoft Support: Keyboard shortcuts in Word", "url": "https://support.microsoft.com/office/keyboard-shortcuts-in-word-95ef89dd-7142-4b50-afb2-f762f663ceb2"}]),

P("excel", "Excel", "cheatsheet", "work", "windows", "Move, select, enter and format in Excel for Windows.", [
   S("Move and select", [
     cs("Ctrl+Right", "Jump to the edge of the data (any arrow)", 1), cs("Ctrl+Home", "Top of the sheet"), cs("Ctrl+End", "Last used cell"),
     cs("Ctrl+PageDown", "Next sheet", 1), cs("Ctrl+PageUp", "Previous sheet", 1), cs("Ctrl+Space", "Select the column", 1),
     cs("Shift+Space", "Select the row", 1), cs("Ctrl+Shift+8", "Select the table around the cell", 1), cs("Ctrl+Shift+End", "Select to the last used cell"),
   ]),
   S("Enter and edit", [
     cs("F2", "Edit the cell", 1), cs("Alt+Enter", "New line in the cell", 1), cs("Ctrl+Enter", "Fill the selection with this entry"), cs("Ctrl+D", "Fill down", 1),
     cs("Ctrl+Shift+Equals", "Insert cells"), cs("Ctrl+Minus", "Delete cells"), cs("Ctrl+Semicolon", "Today's date", 1), cs("Ctrl+Shift+Semicolon", "Time now"),
     cs("Alt+Equals", "AutoSum", 1), cs("Ctrl+E", "Flash Fill", 1), cs("Ctrl+T", "Make a table", 1), cs("Shift+F3", "Insert a function"),
     cs("F4", "Switch $A$1 / A1 references", 1), cs("Ctrl+Backtick", "Show formulas"), cs("F9", "Calculate everything"),
   ]),
   S("Format", [
     cs("Ctrl+1", "Format Cells", 1), cs("Ctrl+Shift+Backtick", "General number format"), cs("Ctrl+Shift+4", "Currency"), cs("Ctrl+Shift+5", "Percentage", 1),
     cs("Ctrl+Shift+3", "Date"), cs("Ctrl+Shift+2", "Time"), cs("Ctrl+Shift+1", "Number with separators"), cs("Ctrl+Shift+7", "Outline border"),
     cs("Ctrl+Shift+Minus", "Remove borders"), cs("Ctrl+9", "Hide rows"), cs("Ctrl+0", "Hide columns"),
   ]),
   S("Insert", [
     cs("Alt+F1", "Chart here", 1), cs("F11", "Chart on its own sheet"), cs("Shift+F2", "Note"), cs("Ctrl+Shift+F2", "Comment"),
     cs("Alt+Shift+F1", "New sheet"), cs("Ctrl+K", "Link"),
   ]),
  ], app="excel", sources=[{"title": "Microsoft Support: Keyboard shortcuts in Excel", "url": "https://support.microsoft.com/office/keyboard-shortcuts-in-excel-1798d9d5-842a-42b8-9c99-9b7213f0040f"}]),

P("powerpoint", "PowerPoint", "cheatsheet", "work", "windows", "Build slides quickly in PowerPoint for Windows.", [
   S("Slides", [
     cs("Ctrl+M", "New slide", 1), cs("Ctrl+D", "Duplicate the selected slide or object", 1), cs("Ctrl+Up", "Move the slide up"), cs("Ctrl+Down", "Move the slide down"),
     cs("F5", "Present from the start", 1), cs("Shift+F5", "Present from this slide", 1), cs("Alt+F5", "Presenter view"),
   ]),
   S("Objects", [
     cs("Ctrl+G", "Group", 1), cs("Ctrl+Shift+G", "Ungroup"), cs("Ctrl+Shift+J", "Regroup"), cs("Ctrl+Shift+C", "Copy formatting", 1),
     cs("Ctrl+Shift+V", "Paste formatting", 1), cs("Alt+Right", "Rotate 15° clockwise"), cs("Alt+Left", "Rotate 15° anticlockwise"), cs("Alt+F10", "Selection pane", 1),
   ]),
   S("Text", [
     cs("Ctrl+Shift+Period", "Bigger font", 1), cs("Ctrl+Shift+Comma", "Smaller font"), cs("Shift+F3", "Cycle sentence, lower and upper case"),
     cs("Ctrl+E", "Centre"), cs("Ctrl+L", "Align left"), cs("Ctrl+R", "Align right"),
   ]),
   S("View and insert", [
     cs("Ctrl+Alt+O", "Fit the slide to the window"), cs("Shift+F9", "Grid"), cs("Alt+F9", "Guides", 1), cs("Ctrl+K", "Link"), cs("Ctrl+Alt+M", "Comment"),
   ]),
  ], app="powerpoint", sources=[{"title": "Microsoft Support: Keyboard shortcuts to create PowerPoint presentations", "url": "https://support.microsoft.com/office/use-keyboard-shortcuts-to-create-powerpoint-presentations-ebb3d20e-dcd4-444f-a38e-bb5c5ed180f4"}]),

P("final-cut-pro", "Final Cut Pro", "cheatsheet", "video", "mac", "Edit at the speed of the keyboard in Final Cut Pro.", [
   S("Tools", [
     cs("A", "Select", 1), cs("T", "Trim", 1), cs("P", "Position"), cs("R", "Range selection"), cs("B", "Blade", 1), cs("Z", "Zoom"), cs("H", "Hand"),
   ]),
   S("Editing", [
     cs("Cmd+B", "Blade at the playhead", 1), cs("Cmd+Shift+B", "Blade all"), cs("E", "Append to the storyline", 1), cs("W", "Insert", 1),
     cs("Q", "Connect", 1), cs("D", "Overwrite"), cs("Shift+R", "Replace"), cs("C", "Select the clip"), cs("N", "Snapping", 1),
     cs("Option+LBracket", "Trim start", 1), cs("Option+RBracket", "Trim end", 1), cs("Option+Backslash", "Trim to selection"),
     cs("Control+Shift+S", "Detach audio"), cs("Cmd+G", "Create a storyline"), cs("Option+G", "New compound clip"), cs("Option+V", "Paste as connected"),
     cs("Cmd+Option+Up", "Lift from the storyline"), cs("Comma", "Nudge left"), cs("Period", "Nudge right"), cs("V", "Enable / disable the clip"),
     cs("Option+F", "Freeze frame"), cs("Option+W", "Insert a gap"), cs("Control+D", "Change duration"),
   ]),
   S("Effects", [
     cs("Cmd+T", "Default transition", 1), cs("Cmd+Option+V", "Paste effects", 1), cs("Cmd+Option+M", "Match colour"), cs("Control+V", "Video animation"),
   ]),
   S("Marking and playing", [
     cs("I", "Range start", 1), cs("O", "Range end", 1), cs("M", "Add a marker", 1), cs("Option+X", "Clear the range"), cs("F", "Favourite"),
     cs("Space", "Play / pause"), cs("J", "Play backwards"), cs("K", "Stop"), cs("L", "Play forwards"), cs("Shift+Slash", "Play around"),
     cs("Cmd+L", "Loop playback"), cs("Quote", "Next edit"), cs("Semicolon", "Previous edit"),
   ]),
   S("General", [
     cs("Cmd+I", "Import media", 1), cs("Control+R", "Render the selection"), cs("Control+Shift+R", "Render all"), cs("Cmd+E", "Share to the default destination", 1),
     cs("Cmd+Equals", "Zoom in"), cs("Cmd+Minus", "Zoom out"), cs("Cmd+Option+K", "Command Editor (your own keys)"),
   ]),
  ], app="finalcut", sources=[{"title": "Apple: Keyboard shortcuts in Final Cut Pro for Mac", "url": "https://support.apple.com/guide/final-cut-pro/ver90ba5929/mac"}]),

P("ableton-live", "Ableton Live", "cheatsheet", "audio", "both", "Views, clips and MIDI in Live without hunting through menus.", [
   S("Views", [
     cs("Tab", "Session / Arrangement", 1), cs("Shift+Tab", "Device / clip view", 1), cs("Mod+Opt+B", "Browser", 1), cs("Mod+Opt+M", "Mixer"),
     cs("Mod+Opt+3", "Clip view"), cs("Mod+Opt+4", "Device view"), cs("Mod+Shift+W", "Second window"), cs("F11", "Full screen", mac="Control+Cmd+F"),
     cs("Mod+Comma", "Settings"),
   ]),
   S("Sets", [
     cs("Mod+N", "New Live Set"), cs("Mod+O", "Open"), cs("Mod+S", "Save", 1), cs("Mod+Shift+S", "Save as"), cs("Mod+Shift+R", "Export audio / video", 1),
     cs("Mod+Shift+E", "Export MIDI"),
   ]),
   S("Editing", [
     cs("Mod+D", "Duplicate", 1), cs("Mod+R", "Rename"), cs("Mod+Y", "Redo", mac="Cmd+Shift+Z"), cs("Mod+E", "Split the clip", 1), cs("Mod+J", "Consolidate", 1),
     cs("Mod+Shift+J", "Crop clips"), cs("Mod+Opt+F", "Fade / crossfade"), cs("Mod+L", "Loop the selection", 1), cs("Mod+I", "Insert silence (or a scene)"),
     cs("Mod+Shift+M", "New MIDI clip", 1), cs("Mod+G", "Group devices"), cs("Mod+Shift+G", "Ungroup devices"),
   ]),
   S("Play and record", [
     cs("Space", "Play / stop", 1), cs("Shift+Space", "Carry on from where it stopped"), cs("F9", "Record", 1), cs("F10", "Back to Arrangement"),
     cs("O", "Metronome", 1), cs("Q", "Hot-swap the device"),
   ]),
   S("MIDI", [
     cs("B", "Draw mode", 1), cs("Mod+U", "Quantise", 1), cs("Mod+Shift+A", "Invert the note selection"), cs("Mod+Opt+J", "Fit notes to the time range"),
     cs("K", "Highlight the scale"), cs("Mod+F", "Search the browser"),
   ]),
  ], app="ableton", sources=[{"title": "Ableton Live 12 Manual: Live Keyboard Shortcuts", "url": "https://www.ableton.com/en/live-manual/12/live-keyboard-shortcuts/"}]),

P("unity", "Unity", "cheatsheet", "3d", "both", "The Unity Editor's default keys (Edit → Shortcuts to see yours).", [
   S("Tools", [
     cs("Q", "Pan (hand)"), cs("W", "Move", 1), cs("E", "Rotate", 1), cs("R", "Scale", 1), cs("T", "Rect", 1), cs("Z", "Pivot / centre"),
     cs("X", "Local / global"), cs("V", "Vertex snap"), cs("F", "Frame the selection", 1), cs("Shift+F", "Follow the selection"),
   ]),
   S("GameObjects", [
     cs("Mod+Shift+N", "New empty GameObject", 1), cs("Opt+Shift+N", "New empty child"), cs("Mod+D", "Duplicate", 1), cs("Mod+Opt+F", "Move to the view"),
     cs("Mod+Shift+F", "Align with the view"),
   ]),
   S("Play", [cs("Mod+P", "Play", 1), cs("Mod+Shift+P", "Pause", 1), cs("Mod+Opt+P", "Step one frame")]),
   S("Windows", [
     cs("Mod+1", "Scene"), cs("Mod+2", "Game"), cs("Mod+3", "Inspector"), cs("Mod+4", "Hierarchy"), cs("Mod+5", "Project"), cs("Mod+6", "Animation"),
     cs("Mod+7", "Profiler"), cs("Mod+Shift+C", "Console", 1), cs("Mod+R", "Refresh assets"), cs("Mod+F", "Search"),
   ]),
  ], app="unity", sources=[{"title": "Unity Manual: Unity shortcuts", "url": "https://docs.unity3d.com/2018.4/Documentation/Manual/UnityHotkeys.html"}]),
]
