// Key names, labels and shortcut parsing. The canonical names match the Rust
// side (crates/catkey-core/src/keys.rs), e.g. "LAlt+LShift+N".

export const FAMILIES = ['Win', 'Ctrl', 'Alt', 'Shift'];

const MOD_TOKEN = /^(L|R)?(Win|Ctrl|Alt|Shift)$/;

const LABELS = {
  Backspace: 'Backspace', Tab: 'Tab', Enter: 'Enter', Esc: 'Esc', Space: 'Space',
  CapsLock: 'Caps Lock', PageUp: 'Page Up', PageDown: 'Page Down', End: 'End', Home: 'Home',
  Left: '←', Up: '↑', Right: '→', Down: '↓', PrintScreen: 'Print Screen', Insert: 'Insert',
  Delete: 'Delete', Pause: 'Pause', Menu: 'Menu', NumLock: 'Num Lock', ScrollLock: 'Scroll Lock',
  NumMultiply: 'Num *', NumAdd: 'Num +', NumSubtract: 'Num −', NumDecimal: 'Num .', NumDivide: 'Num /',
  NumSeparator: 'Num ,', Semicolon: ';', Equals: '=', Comma: ',', Minus: '-', Period: '.', Slash: '/',
  Backtick: '`', LBracket: '[', Backslash: '\\', RBracket: ']', Quote: "'", IntlBackslash: '\\ (ISO)',
  Oem8: 'OEM 8', VolumeMute: 'Mute', VolumeDown: 'Volume −', VolumeUp: 'Volume +', MediaNext: 'Next track',
  MediaPrev: 'Previous track', MediaStop: 'Stop media', MediaPlayPause: 'Play / Pause', BrowserBack: 'Browser back',
  BrowserForward: 'Browser forward', BrowserRefresh: 'Browser refresh', BrowserStop: 'Browser stop',
  BrowserSearch: 'Browser search', BrowserFavorites: 'Favorites', BrowserHome: 'Browser home',
  LaunchMail: 'Mail', LaunchMedia: 'Media player', LaunchApp1: 'App 1', LaunchApp2: 'App 2', Sleep: 'Sleep',
  Clear: 'Clear', Help: 'Help', Select: 'Select', Print: 'Print', Execute: 'Execute', Kana: 'Kana',
  Kanji: 'Kanji', Convert: 'Convert', NonConvert: 'No convert',
};

/** Groups for the "fine-tune by hand" key picker. */
export const KEY_GROUPS = [
  ['Letters', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('')],
  ['Numbers', '0123456789'.split('')],
  ['Function keys', Array.from({ length: 24 }, (_, i) => `F${i + 1}`)],
  ['Arrows and navigation', ['Left', 'Right', 'Up', 'Down', 'Home', 'End', 'PageUp', 'PageDown', 'Insert', 'Delete']],
  ['Editing', ['Enter', 'Tab', 'Space', 'Backspace', 'Esc', 'CapsLock', 'Menu', 'PrintScreen', 'Pause', 'ScrollLock', 'NumLock']],
  ['Symbols', ['Backtick', 'Minus', 'Equals', 'LBracket', 'RBracket', 'Backslash', 'Semicolon', 'Quote', 'Comma', 'Period', 'Slash', 'IntlBackslash']],
  ['Number pad', [...Array.from({ length: 10 }, (_, i) => `Num${i}`), 'NumMultiply', 'NumAdd', 'NumSubtract', 'NumDecimal', 'NumDivide']],
  ['Media and browser', ['VolumeMute', 'VolumeDown', 'VolumeUp', 'MediaPlayPause', 'MediaNext', 'MediaPrev', 'MediaStop', 'BrowserBack', 'BrowserForward', 'BrowserRefresh', 'BrowserHome', 'BrowserSearch', 'LaunchMail', 'LaunchApp1', 'LaunchApp2']],
  ['Modifier keys (as a single key)', ['LCtrl', 'RCtrl', 'LShift', 'RShift', 'LAlt', 'RAlt', 'LWin', 'RWin']],
];

export function keyLabel(name) {
  if (!name) return '';
  if (LABELS[name]) return LABELS[name];
  const m = name.match(MOD_TOKEN);
  if (m) return m[2];
  const num = name.match(/^Num(\d)$/);
  if (num) return `Num ${num[1]}`;
  return name;
}

export function isModifierName(name) {
  return MOD_TOKEN.test(name || '');
}

/** "LAlt+LShift+N" -> { mods: {Win:'',Ctrl:'',Alt:'L',Shift:'L'}, key: 'N' } ('' = not used, L/R/A = side) */
export function parse(str) {
  const out = { mods: { Win: '', Ctrl: '', Alt: '', Shift: '' }, key: '' };
  if (!str) return out;
  const parts = String(str).split('+').map((p) => p.trim()).filter(Boolean);
  if (!parts.length) return out;
  out.key = parts.pop();
  for (const p of parts) {
    const m = p.match(MOD_TOKEN);
    if (m) out.mods[m[2]] = m[1] || 'A';
  }
  return out;
}

export function format(sc) {
  const toks = [];
  for (const f of FAMILIES) {
    const s = sc.mods[f];
    if (s === 'L') toks.push(`L${f}`);
    else if (s === 'R') toks.push(`R${f}`);
    else if (s === 'A') toks.push(f);
  }
  if (sc.key) toks.push(sc.key);
  return toks.join('+');
}

/** Keycaps to draw: [{ t: 'Alt', s: 'L', mod: true }, { t: 'C', s: '' }] */
export function caps(str) {
  if (!str) return [];
  const sc = parse(str);
  const list = [];
  for (const f of FAMILIES) {
    const s = sc.mods[f];
    if (s) list.push({ t: f, s: s === 'A' ? '' : s, mod: true });
  }
  if (sc.key) {
    const m = sc.key.match(MOD_TOKEN);
    list.push(m ? { t: m[2], s: m[1] || '', mod: true } : { t: keyLabel(sc.key), s: '', mod: false });
  }
  return list;
}

/** "Alt C" style plain text, for search and toasts. */
export function plain(str) {
  return caps(str).map((c) => c.t).join(' ');
}

export function hasMods(str) {
  const sc = parse(str);
  return FAMILIES.some((f) => sc.mods[f]);
}

/** Do two triggers ever match the same key press? ("Alt+C" overlaps "LAlt+C") */
export function overlaps(a, b) {
  const x = parse(a);
  const y = parse(b);
  if (!x.key || x.key !== y.key) return false;
  for (const f of FAMILIES) {
    const s = x.mods[f];
    const t = y.mods[f];
    if (!s && !t) continue;
    if (!s || !t) return false;
    if (s !== 'A' && t !== 'A' && s !== t) return false;
  }
  return true;
}

/** Things Windows never lets an app change. */
export function reservedNote(str) {
  const sc = parse(str);
  const only = (fams) => FAMILIES.every((f) => (fams.includes(f) ? !!sc.mods[f] : !sc.mods[f]));
  if (sc.key === 'L' && only(['Win'])) return 'Windows keeps Win + L (lock) for itself, so it can’t be changed.';
  if (sc.key === 'Delete' && only(['Ctrl', 'Alt'])) return 'Ctrl + Alt + Delete belongs to Windows and can’t be changed.';
  return '';
}
