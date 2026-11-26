# The Cat Key library

Packs are small JSON files (`catkey-pack`) that Cat Key imports: ready-made shortcuts for an app, a set for switching between Mac and Windows, or a cheat sheet to train on.

* **Built-in packs** live in [`src/library/packs`](../src/library/packs). They're generated from [`tools/library/packs.py`](../tools/library/packs.py) by `python3 tools/library/build.py`, which checks every key name before anything ships.
* **Community packs** live in [`community/`](community). Cat Key lists them in *Library → Community*.

## Share a pack

1. In Cat Key, open the pack (or make one with **Cat Builder**) and press **Share code**, or **Export** to save the file.
2. [Open a "Share a pack" issue](https://github.com/MasterfullyClaw/CatKey/issues/new?template=share-a-pack.yml) and paste the code or attach the file.
3. Once it's checked (real default shortcuts, sources listed, nothing harmful), it's added to `community/` and appears in everyone's Library.

## The format

```json
{
  "format": "catkey-pack",
  "version": 1,
  "name": "Premiere keys in DaVinci Resolve",
  "kind": "remap",
  "platform": "both",
  "app": "DaVinci Resolve",
  "description": "One sentence about what the pack does.",
  "apps": { "windows": ["resolve.exe"], "mac": ["com.blackmagic-design.DaVinciResolve"] },
  "sources": [{ "title": "Official shortcuts", "url": "https://…" }],
  "sections": [
    { "name": "Editing", "items": [
      { "name": "Add edit (cut at playhead)", "press": "Mod+K", "does": "Mod+B", "fav": true }
    ] }
  ]
}
```

* `kind`: `remap` (press → does) or `cheatsheet` (the app's own `keys`, for learning and **Train yourself**).
* Modifiers: `Mod` = Ctrl on Windows / ⌘ on Mac, `Opt` = Alt / ⌥ Option, `Shift`, `Ctrl` (Control on both). Windows-only packs may use `Alt` and `Win`; Mac-only packs `Cmd`, `Option`, `Control`.
* Keys: `A`–`Z`, `0`–`9`, `F1`–`F24`, `Esc`, `Tab`, `Space`, `Enter`, `Backspace`, `Delete`, `Home`, `End`, `PageUp`, `PageDown`, `Left`, `Right`, `Up`, `Down`, `Backtick`, `Minus`, `Equals`, `LBracket`, `RBracket`, `Backslash`, `Semicolon`, `Quote`, `Comma`, `Period`, `Slash`, `Num0`–`Num9`, media keys.
* Per item, optional: `mac` (when the Mac key differs), `note`, `fav` (a pro pick), `off` (not ticked by default), `only` / `except` (app lists).
