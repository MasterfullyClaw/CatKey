"""Build Cat Key's library: validate tools/library/packs.py and write
src/library/packs/<id>.json (the shareable catkey-pack files) and src/library/data.js."""
import json, re, sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from apps import APPS
from packs import PACKS

ROOT = pathlib.Path(__file__).resolve().parents[2]
KEYS = set("Backspace Tab Clear Enter Pause CapsLock Kana Kanji Esc Convert NonConvert Space PageUp PageDown End Home Left Up Right Down Select Print Execute PrintScreen Insert Delete Help LWin RWin Menu Sleep NumMultiply NumAdd NumSeparator NumSubtract NumDecimal NumDivide NumLock ScrollLock LShift RShift LCtrl RCtrl LAlt RAlt BrowserBack BrowserForward BrowserRefresh BrowserStop BrowserSearch BrowserFavorites BrowserHome VolumeMute VolumeDown VolumeUp MediaNext MediaPrev MediaStop MediaPlayPause LaunchMail LaunchMedia LaunchApp1 LaunchApp2 Semicolon Equals Comma Minus Period Slash Backtick LBracket Backslash RBracket Quote Oem8 IntlBackslash".split())
KEYS |= {chr(c) for c in range(65, 91)} | {str(d) for d in range(10)} | {f"F{n}" for n in range(1, 25)} | {f"Num{d}" for d in range(10)}
ALIAS = {"[": "LBracket", "]": "RBracket", "\\": "Backslash", "/": "Slash", ",": "Comma", ".": "Period", "`": "Backtick",
         "=": "Equals", "-": "Minus", ";": "Semicolon", "'": "Quote", "Del": "Delete", "Escape": "Esc", "Return": "Enter"}
MODS = {"Mod", "Opt", "Shift", "Ctrl", "Alt", "Win", "Cmd", "Option", "Control", "LCtrl", "RCtrl", "LAlt", "RAlt", "LShift", "RShift", "LWin", "RWin"}
WIN_ONLY = {"Win", "LWin", "RWin", "Alt", "LAlt", "RAlt"}
MAC_ONLY = {"Cmd", "Option", "Control"}

def norm(combo, where, platform):
    parts = combo.split("+")
    if combo.endswith("++"):
        parts = combo[:-2].split("+") + ["Equals"]
    *mods, key = [p.strip() for p in parts]
    key = ALIAS.get(key, key)
    if key not in KEYS:
        sys.exit(f"{where}: unknown key {key!r} in {combo!r}")
    for m in mods:
        if m not in MODS:
            sys.exit(f"{where}: unknown modifier {m!r} in {combo!r}")
        if platform == "mac" and m in WIN_ONLY:
            sys.exit(f"{where}: {m} in a Mac-only pack (use Cmd / Option / Control / Mod / Opt)")
        if platform == "windows" and m in MAC_ONLY:
            sys.exit(f"{where}: {m} in a Windows-only pack")
        if platform == "both" and (m in WIN_ONLY or m in MAC_ONLY):
            sys.exit(f"{where}: {m} in a two-platform pack (use Mod / Opt / Ctrl / Shift)")
    if len(set(mods)) != len(mods):
        sys.exit(f"{where}: repeated modifier in {combo!r}")
    return "+".join(mods + [key])

def scope(v, where):
    if v is None:
        return None
    vals = [v] if isinstance(v, str) else list(v)
    for x in vals:
        if x.startswith("app:") and x[4:] not in APPS:
            sys.exit(f"{where}: unknown app {x}")
    return vals

out_dir = ROOT / "src" / "library" / "packs"
out_dir.mkdir(parents=True, exist_ok=True)
ids, built = set(), []
for p in PACKS:
    pid = p["id"]
    if pid in ids or not re.fullmatch(r"[a-z0-9-]+", pid):
        sys.exit(f"bad or duplicate pack id {pid}")
    ids.add(pid)
    if p.get("app") and p["app"] not in APPS:
        sys.exit(f"{pid}: unknown app {p['app']}")
    plat = p["platform"]
    pack = {"format": "catkey-pack", "version": 1, "id": pid, "name": p["name"], "kind": p["kind"],
            "category": p["category"], "platform": plat, "description": p["description"], "author": "Cat Key"}
    for k in ("app", "tags", "sources"):
        if p.get(k):
            pack[k] = p[k]
    secs, n = [], 0
    for s in p["sections"]:
        items = []
        for it in s["items"]:
            where = f"{pid} / {s['name']} / {it.get('name')}"
            ip = it.get("platform", plat)
            x = {"name": it["name"]}
            if p["kind"] == "cheatsheet":
                x["keys"] = norm(it["keys"], where, ip)
                if it.get("mac"):
                    x["mac"] = norm(it["mac"], where, "mac")
            else:
                x["press"] = norm(it["press"], where, ip)
                if "does" in it:
                    x["does"] = norm(it["does"], where, ip)
                elif "launch" in it:
                    x["launch"] = it["launch"]
                elif it.get("block"):
                    x["block"] = True
                else:
                    sys.exit(f"{where}: remap needs does / launch / block")
            for k in ("only", "except"):
                if it.get(k):
                    x[k] = scope(it[k], where)
            if it.get("fav"):
                x["fav"] = True
            if it.get("off"):
                x["off"] = True
            if it.get("note"):
                x["note"] = it["note"]
            if it.get("platform") and it["platform"] != plat:
                x["platform"] = it["platform"]
            items.append(x)
        n += len(items)
        sec = {"name": s["name"], "items": items}
        if not s.get("on", True):
            sec["off"] = True
        if s.get("note"):
            sec["note"] = s["note"]
        secs.append(sec)
    pack["sections"] = secs
    (out_dir / f"{pid}.json").write_text(json.dumps(pack, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    built.append(pack)
    print(f"  {pid:28} {plat:8} {p['kind']:10} {n:3} shortcuts")

apps = {k: {"name": v[0], "category": v[1], "windows": v[2], "mac": v[3]} for k, v in APPS.items()}
js = ("// Generated by tools/library/build.py from tools/library/packs.py. Do not edit by hand.\n"
      f"export const APPS = {json.dumps(apps, ensure_ascii=False)};\n"
      f"export const PACKS = {json.dumps(built, ensure_ascii=False)};\n")
(ROOT / "src" / "library" / "data.js").write_text(js, encoding="utf-8")
print(f"{len(built)} packs, {sum(len(i['items']) for p in built for i in p['sections'])} shortcuts -> src/library/")
