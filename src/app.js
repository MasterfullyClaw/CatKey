// Cat Key UI. Talks to the Rust side through `invoke` + events; everything the
// keyboard does happens in Rust, so this window can be closed at any time.

import * as K from './keys.js';
import { Mascot, SKINS, CATS, GEAR, RANKS, rankFor, gearById } from './mascot.js';
import { sounds, setSoundsEnabled, trainSounds, setTrainSounds, clickSound } from './sound.js';

const backend = window.__TAURI__
  ? { invoke: window.__TAURI__.core.invoke, listen: window.__TAURI__.event.listen }
  : (await import('./mock.js')).createMock();
const invoke = (cmd, args) => backend.invoke(cmd, args);

// ------------------------------------------------------------------ icons

const I = {
  gear: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
  minus: '<path d="M5 12h14"/>',
  chev: '<path d="m6 9 6 6 6-6"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  more: '<circle cx="12" cy="5" r="1.7"/><circle cx="12" cy="12" r="1.7"/><circle cx="12" cy="19" r="1.7"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>',
  app: '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M3 9h18"/>',
  launch: '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M3 9h18M8 14l2.5 2.5L16 12"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/>',
  warn: '<path d="M12 3 2 20h20L12 3z"/><path d="M12 10v4M12 17h.01"/>',
  sparkle: '<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/>',
  keyboard: '<rect x="2.5" y="6" width="19" height="12" rx="3"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"/>',
  paw: '<ellipse cx="12" cy="16" rx="5" ry="4"/><circle cx="6.5" cy="10.5" r="2.2"/><circle cx="10" cy="7" r="2.2"/><circle cx="14" cy="7" r="2.2"/><circle cx="17.5" cy="10.5" r="2.2"/>',
  text: '<path d="M5 6h14M12 6v13M8 19h8"/>',
  block: '<circle cx="12" cy="12" r="9"/><path d="m5.6 5.6 12.8 12.8"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  refresh: '<path d="M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5"/>',
  folder: '<path d="M4 7a2 2 0 0 1 2-2h4l2 2h6a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z"/>',
  down: '<path d="M12 3v12M7 10l5 5 5-5M4 19h16"/>',
  up: '<path d="M12 15V3M7 8l5-5 5 5M4 19h16"/>',
  trash: '<path d="M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3"/>',
  copy: '<rect x="8" y="8" width="12" height="12" rx="2.5"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>',
  edit: '<path d="M4 20h4L19 9l-4-4L4 16v4z"/><path d="m13.5 6.5 4 4"/>',
  link: '<path d="M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1"/><path d="M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1"/>',
  shield: '<path d="M12 3 4 6v6c0 5 3.5 8 8 9 4.5-1 8-4 8-9V6l-8-3z"/>',
  moon: '<path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z"/>',
  swap: '<path d="M4 8h15M15 4l4 4-4 4M20 16H5M9 12l-4 4 4 4"/>',
  layers: '<path d="M12 3 3 8l9 5 9-5-9-5z"/><path d="m3 13 9 5 9-5"/>',
  lock: '<rect x="5" y="11" width="14" height="10" rx="2.5"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>',
  wand: '<path d="m4 20 11-11M14 4v3M19 9h-3M17.5 5.5l-2 2M18 14v2M20 15h-4M8 3v2M9 4H7"/>',
  star: '<path d="m12 3.5 2.6 5.3 5.9.9-4.3 4.1 1 5.8L12 16.9l-5.2 2.7 1-5.8-4.3-4.1 5.9-.9z"/>',
  book: '<path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z"/><path d="M4 20.5A2.5 2.5 0 0 0 6.5 23H20v-5"/>',
  target: '<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.5"/>',
  select: '<rect x="3.5" y="3.5" width="17" height="17" rx="4"/><path d="m8 12 3 3 5-6"/>',
  sound: '<path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4z"/><path d="M15.5 9a4.2 4.2 0 0 1 0 6M18.2 6.5a8 8 0 0 1 0 11"/>',
  mute: '<path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4z"/><path d="m16 9.5 5 5M21 9.5l-5 5"/>',
  purr: '<path d="M4.5 10.5 4 4l4.5 3.2h7L20 4l-.5 6.5"/><path d="M4.5 10.5C4.5 16 7.8 19 12 19s7.5-3 7.5-8.5"/><path d="M9.5 12.5h.01M14.5 12.5h.01M11 15.2q1 .8 2 0"/>',
};
const FILLED = new Set(['paw', 'more', 'sparkle', 'star']);
const icon = (name, extra = '') =>
  `<svg class="i ${FILLED.has(name) ? 'fill' : ''} ${extra}" viewBox="0 0 24 24" aria-hidden="true">${I[name]}</svg>`;

// ------------------------------------------------------------------ DOM helpers

const $ = (sel, root = document) => root.querySelector(sel);
// Overlays (menus, dialogs, confetti) live inside #app, where the theme colors are defined.
const layer = () => document.getElementById('app') || document.body;
function h(tag, props, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'text') el.textContent = v;
    else if (k === 'html') el.innerHTML = v; // trusted static markup only (icons)
    else if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2).toLowerCase(), v);
    else if (k === 'style' && typeof v === 'object') Object.assign(el.style, v);
    else el.setAttribute(k, v === true ? '' : String(v));
  }
  for (const kid of kids.flat(Infinity)) {
    if (kid == null || kid === false) continue;
    el.append(kid instanceof Node ? kid : document.createTextNode(String(kid)));
  }
  return el;
}
const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
const newId = () => 'r' + (crypto.randomUUID ? crypto.randomUUID().replace(/-/g, '').slice(0, 16) : Date.now().toString(36) + Math.random().toString(36).slice(2, 8));
const clone = (x) => JSON.parse(JSON.stringify(x));

// ------------------------------------------------------------------ state

const S = {
  config: null,
  status: { version: '', admin: false, powertoysRunning: false, problems: {} },
  stats: { today: 0, total: 0, perRule: {} },
  filter: 'all',
  query: '',
  fresh: new Set(),
  firedTimers: new Map(),
  bannerDismissed: false,
  view: 'shortcuts', // 'shortcuts' | 'library' | 'templates' | 'ai' | 'train'
  templates: [], // built-in templates (from Rust); the user's own live in config.templates
  tplReversed: new Set(), // template ids shown (and added) with keys swapped
  tplOpen: new Set(), // template ids showing every shortcut
};
let hero = null; // hero Mascot
let REC = null; // active recording target
let E = null; // editor state

// A cat's name comes with the cat: the house cats and warriors keep theirs, and only a cat you make gets a name from you.
const cat = () => catTitle(myCat());
const rules = () => S.config.rules;

const SUGGESTIONS = [
  ['LAlt+X', 'LCtrl+X', 'Cut'],
  ['LAlt+T', 'LCtrl+T', 'New tab'],
  ['LAlt+LShift+T', 'LCtrl+LShift+T', 'Reopen tab'],
  ['LAlt+LShift+Z', 'LCtrl+Y', 'Redo'],
  ['LAlt+P', 'LCtrl+P', 'Print'],
  ['LAlt+O', 'LCtrl+O', 'Open'],
  ['LAlt+L', 'LCtrl+L', 'Address bar'],
  ['LAlt+K', 'LCtrl+K', 'Quick search'],
  ['LAlt+Backspace', 'LCtrl+Backspace', 'Delete word'],
  ['LAlt+Equals', 'LCtrl+Equals', 'Zoom in'],
  ['LAlt+Minus', 'LCtrl+Minus', 'Zoom out'],
];

const KNOWN_APPS = {
  'chrome.exe': 'Chrome', 'msedge.exe': 'Edge', 'firefox.exe': 'Firefox', 'brave.exe': 'Brave', 'opera.exe': 'Opera',
  'code.exe': 'VS Code', 'explorer.exe': 'File Explorer', 'wt.exe': 'Windows Terminal', 'windowsterminal.exe': 'Windows Terminal',
  'notepad.exe': 'Notepad', 'spotify.exe': 'Spotify', 'discord.exe': 'Discord', 'figma.exe': 'Figma', 'slack.exe': 'Slack',
  'mstsc.exe': 'Remote Desktop', 'winword.exe': 'Word', 'excel.exe': 'Excel', 'powerpnt.exe': 'PowerPoint', 'outlook.exe': 'Outlook',
  'claude.exe': 'Claude', 'cursor.exe': 'Cursor', 'obsidian.exe': 'Obsidian', 'notion.exe': 'Notion', 'telegram.exe': 'Telegram',
  'whatsapp.exe': 'WhatsApp', 'blender.exe': 'Blender', 'afterfx.exe': 'After Effects', 'photoshop.exe': 'Photoshop',
};
// Mac apps are known by their bundle id ("com.apple.safari").
const MAC_APP_NAMES = {
  'com.apple.safari': 'Safari', 'com.apple.finder': 'Finder', 'com.apple.terminal': 'Terminal', 'com.apple.textedit': 'TextEdit',
  'com.apple.notes': 'Notes', 'com.apple.mail': 'Mail', 'com.apple.finalcut': 'Final Cut Pro', 'com.apple.dt.xcode': 'Xcode',
  'com.apple.logic10': 'Logic Pro', 'com.apple.keynote': 'Keynote', 'com.apple.iwork.keynote': 'Keynote', 'com.apple.iwork.pages': 'Pages',
  'com.google.chrome': 'Chrome', 'com.microsoft.edgemac': 'Edge', 'org.mozilla.firefox': 'Firefox', 'com.brave.browser': 'Brave',
  'company.thebrowser.browser': 'Arc', 'company.thebrowser.dia': 'Dia', 'com.microsoft.vscode': 'VS Code', 'com.todesktop.230313mzl4w4u92': 'Cursor',
  'com.tinyspeck.slackmacgap': 'Slack', 'notion.id': 'Notion', 'com.figma.desktop': 'Figma', 'md.obsidian': 'Obsidian',
  'com.openai.chat': 'ChatGPT', 'com.anthropic.claudefordesktop': 'Claude', 'com.spotify.client': 'Spotify', 'com.hnc.discord': 'Discord',
  'com.googlecode.iterm2': 'iTerm', 'dev.warp.warp-stable': 'Warp', 'com.blackmagic-design.davinciresolve': 'DaVinci Resolve',
  'com.adobe.photoshop': 'Photoshop', 'com.adobe.illustrator': 'Illustrator', 'com.adobe.aftereffects': 'After Effects',
  'com.adobe.premierepro*': 'Premiere Pro', 'org.blenderfoundation.blender': 'Blender', 'com.microsoft.word': 'Word',
  'com.microsoft.excel': 'Excel', 'com.microsoft.powerpoint': 'PowerPoint', 'com.lemon.lvoverseas': 'CapCut',
};
const appNames = new Map([...Object.entries(KNOWN_APPS), ...Object.entries(MAC_APP_NAMES)]);
function appName(exeOrPath) {
  if (!exeOrPath) return '';
  if (/^[a-z]+:\/\//i.test(exeOrPath)) return exeOrPath.replace(/^https?:\/\//i, '').replace(/\/$/, '');
  const file = exeOrPath.split(/[\\/]/).pop().toLowerCase();
  if (appNames.has(file)) return appNames.get(file);
  if (/^[a-z0-9-]+(\.[a-z0-9-]+){2,}\*?$/.test(file)) {
    const known = [...appNames.entries()].find(([k]) => k.endsWith('*') && file.startsWith(k.slice(0, -1)));
    if (known) return known[1];
    const last = file.replace(/\*$/, '').split('.').pop();
    return last.charAt(0).toUpperCase() + last.slice(1);
  }
  const stem = file.replace(/\.(exe|lnk|bat|cmd|url)$/i, '');
  return stem.charAt(0).toUpperCase() + stem.slice(1);
}
const PALETTE = ['#4F6FE0', '#0F8A7A', '#C0500C', '#7C3AED', '#C0266D', '#0A6CA8', '#4D7C0F', '#A85B07'];
function colorFor(s) {
  let n = 0;
  for (const c of s) n = (n * 31 + c.charCodeAt(0)) >>> 0;
  return PALETTE[n % PALETTE.length];
}

const iconCache = new Map();
function appIcon(key, cls = 'appico') {
  if (!key) return h('span', { class: `${cls} letter`, style: { background: '#777' }, text: '?' });
  const k = key.toLowerCase();
  const url = iconCache.get(k);
  if (url) return h('img', { class: cls, src: url, alt: '' });
  const isUrl = /^[a-z]+:\/\//i.test(key);
  const el = h('span', {
    class: `${cls} letter`, 'data-icon': k, style: { background: isUrl ? 'var(--s3)' : colorFor(k) },
    html: isUrl ? icon('link') : null, text: isUrl ? null : (appName(key)[0] || '?').toUpperCase(),
  });
  if (!iconCache.has(k) && !isUrl) {
    iconCache.set(k, null);
    invoke('app_icon', { path: key }).then((data) => {
      if (!data) return;
      iconCache.set(k, data);
      document.querySelectorAll(`[data-icon="${CSS.escape(k)}"]`).forEach((e) => {
        e.replaceWith(h('img', { class: e.className.replace(/\bletter\b/, '').trim(), src: data, alt: '' }));
      });
    }).catch(() => {});
  }
  return el;
}
function primeApps(list) {
  for (const a of list || []) {
    if (a.name) appNames.set(a.exe, a.name);
    if (a.icon) {
      iconCache.set(a.exe.toLowerCase(), a.icon);
      if (a.path) iconCache.set(a.path.toLowerCase(), a.icon);
    }
  }
}

function capsEl(str, { to = false, big = false, pop = false } = {}) {
  const wrap = h('span', { class: 'caps' });
  for (const c of K.caps(str)) {
    wrap.append(h('span', { class: `kc${to ? ' to' : ''}${big ? ' big' : ''}${pop ? ' kc-in' : ''}` }, c.t, c.s ? h('small', { text: c.s }) : null));
  }
  return wrap;
}

function autoName(r) {
  const a = r.action;
  if (a.type === 'keys') return K.plain(a.keys);
  if (a.type === 'launch') return `Open ${appName(a.target)}`;
  if (a.type === 'text') return 'Types text';
  return 'Does nothing';
}
const ruleName = (r) => r.name || autoName(r);

function scopeText(scope) {
  const names = (scope.apps || []).map(appName);
  const list = names.length > 2 ? `${names.slice(0, 2).join(', ')} +${names.length - 2}` : names.join(', ');
  if (scope.mode === 'only' && names.length) return `Only in ${list}`;
  if (scope.mode === 'except' && names.length) return `Everywhere except ${list}`;
  return 'Everywhere';
}

// ------------------------------------------------------------------ swap (what you press ⇄ what it does)

const REVERSED = ' (reversed)';
const canSwap = (r) => r?.action?.type === 'keys' && !!r.action.keys && !!r.trigger;
function swappedName(name) {
  if (!name) return '';
  return name.endsWith(REVERSED) ? name.slice(0, -REVERSED.length) : (name + REVERSED).slice(0, 60);
}
/** The same rule with the keys you press and the keys it sends trading places. */
function swapped(r) {
  if (!canSwap(r)) return null;
  return { ...clone(r), name: swappedName(r.name), trigger: r.action.keys, action: { type: 'keys', keys: r.trigger } };
}
/** Why a swapped rule couldn't work ('' = fine). */
function swapProblem(sw) {
  const { err } = analyze(sw);
  return err.trigger || err.keys || K.reservedNote(sw.trigger) || '';
}
/** Another switched-on rule that fires on the same keys in the same apps. */
function clashFor(r, list = rules()) {
  return list.find((x) => x.id !== r.id && x.enabled && K.overlaps(x.trigger, r.trigger) && scopesOverlap(x.scope, r.scope));
}

// ------------------------------------------------------------------ boot

async function boot() {
  const st = await invoke('get_state');
  S.config = st.config;
  S.status = st.status;
  K.setMac(PLAT() === 'mac');
  $('#app').classList.toggle('mac', PLAT() === 'mac');
  if (PLAT() === 'mac' && $('#find-kbd')) $('#find-kbd').textContent = '⌘ F';
  S.stats = st.stats || S.stats;
  S.templates = st.templates || [];
  setSoundsEnabled(S.config.settings.sounds);
  await restoreProgress();
  const newTab = st.upgraded && rules().find((r) => r.trigger === 'LAlt+T' && r.action.keys === 'LCtrl+T');
  if (newTab) markFresh(newTab.id);

  $('#btn-settings').innerHTML = icon('gear');
  $('#btn-about').innerHTML = `${icon('info')}<span>About</span>`;
  paintPurr();
  $('#cat-switch').innerHTML = `${icon('swap')}<span>Switch cat</span>`;
  $('#btn-min').innerHTML = icon('minus');
  $('#btn-close').innerHTML = icon('x');
  $('#ico-search').outerHTML = icon('search');
  $('#btn-new').innerHTML = `${icon('plus')}<span>New shortcut</span>`;
  $('#btn-select').innerHTML = `${icon('select')}<span>Select</span>`;
  $('.master .knob').innerHTML = icon('paw');

  hero = new Mascot($('#cushion'), { size: 148, track: true, name: cat(), keyboard: true, look: myLook() });
  applyLook();
  renderAll();
  wire();
  await invoke('ui_ready').catch(() => {});

  if (S.status.firstRun) {
    setTimeout(() => toast(rules().length
      ? `${cat()} brought your ${rules().length} shortcuts over. They already work everywhere!`
      : `Welcome! Pick shortcuts from the Library or make your own, and ${cat()} runs them in every app.`, { ms: 6000 }), 500);
  }
  if (newTab) {
    setTimeout(() => toast('New: Alt + T opens a new tab. Want it the other way round? Click the arrow on its card to swap.', { ms: 10000 }), 900);
  }
  if (S.status.configReset) {
    toast('Your settings file was damaged, so Cat Key started fresh (a copy was kept).', { kind: 'warn', ms: 8000 });
  }
  setTimeout(() => libData().then(() => { renderTabs(); if (S.view === 'library') renderLibrary(); syncCoachDict(); }).catch(() => {}), 400);
  PURR.on = await invoke('purr_state').catch(() => false);
  paintPurr();
  coachStats().then(() => { checkRankUp(); if (S.view === 'warrior') renderWarrior(); });
  setInterval(() => { if (!document.hidden) coachStats(true).then(() => { checkRankUp(); if (S.view === 'warrior' && !document.activeElement?.closest?.('#warrior input')) renderWarrior(); }); }, 30000);
  setInterval(refreshStatus, 6000);
  setInterval(async () => {
    if (document.hidden) return;
    S.stats = await invoke('get_stats').catch(() => S.stats);
    renderHero();
    for (const r of rules()) {
      const el = document.querySelector(`.card[data-id="${CSS.escape(r.id)}"] .uses b`);
      if (el) el.textContent = fmtCount(S.stats.perRule?.[r.id] || 0);
    }
  }, 20000);
}

async function refreshStatus() {
  if (document.hidden) return;
  const st = await invoke('get_status').catch(() => null);
  if (!st) return;
  const key = (x) => JSON.stringify([x.powertoysRunning, x.grabAndMove, x.admin, x.engineRunning, x.startup, x.problems]);
  const changed = key(st) !== key(S.status);
  S.status = st;
  if (changed) {
    renderBanners();
    renderTitle();
    renderList();
    if (SETTINGS_OPEN) renderSettings();
  }
}

function applyLook() {
  const s = S.config.settings;
  const dark = s.theme === 'dark' || (s.theme !== 'light' && matchMedia('(prefers-color-scheme: dark)').matches);
  const app = $('#app');
  app.classList.toggle('dark', dark);
  app.classList.toggle('light', !dark);
  const theme = myCat().theme || 'mochi';
  for (const k of SKINS) app.classList.toggle(k.id, k.id === theme);
  hero?.setLook(myLook());
  hero?.setName(cat());
  E?.mini?.setLook(myLook());
}

function renderAll() {
  renderTitle();
  renderHero();
  renderBanners();
  renderTabs();
  renderFilters();
  renderList();
  renderSuggestions();
  renderTemplates();
  applyView();
}

// ------------------------------------------------------------------ tabs: your shortcuts | templates

function renderTabs() {
  const box = $('#tabs');
  box.textContent = '';
  const libN = L.data ? libPacks().filter((p) => p._origin !== 'community').length : null;
  for (const [id, label, n, ic] of [['shortcuts', 'Your shortcuts', rules().length, 'keyboard'], ['library', 'Library', libN, 'book'],
    ['templates', 'Templates', allTemplates().length, 'layers'], ['train', 'Train yourself', null, 'target'], ['warrior', 'Cat Warrior', null, 'star'], ['ai', 'Cat Builder', null, 'wand']]) {
    box.append(h('button', { class: 'tab', type: 'button', role: 'tab', 'aria-selected': String(S.view === id), onclick: () => setView(id) },
      h('span', { class: 'ic', html: icon(ic) }), h('span', { text: label }), n == null ? null : h('span', { class: 'count', text: String(n) })));
  }
}

const VIEW_BOX = { shortcuts: '#list', library: '#library', templates: '#templates', ai: '#aihelp', train: '#train', warrior: '#warrior', about: '#about-page' };
function applyView() {
  for (const v of Object.keys(VIEW_BOX)) $('#app').classList.toggle(`view-${v}`, S.view === v);
  if (S.view !== 'shortcuts' && SEL.on) setSelecting(false);
  if (S.view === 'library') renderLibrary();
  if (S.view === 'ai') { if (!L.data) libData().then(renderAI); renderAI(); }
  if (S.view === 'train') renderTrain();
  if (S.view !== 'train') { trainStop(); typingStop(); }
  if (S.view === 'warrior') renderWarrior();
  if (S.view === 'about') renderAbout();
  $('#btn-about')?.setAttribute('aria-pressed', String(S.view === 'about'));
}

function setView(v) {
  if (S.view === v) return;
  S.view = v;
  sounds.pop();
  renderTabs();
  applyView();
  const scroll = $('#scroll');
  const gap = $('#tabs').getBoundingClientRect().top - scroll.getBoundingClientRect().top;
  if (gap < 0) scroll.scrollTo({ top: scroll.scrollTop + gap - 12, behavior: reduced() ? 'auto' : 'smooth' });
  if (!reduced()) $(VIEW_BOX[v] || '#list').animate([{ opacity: 0, transform: 'translateY(10px)' }, { opacity: 1, transform: 'none' }], { duration: 280, easing: 'cubic-bezier(.22,1,.36,1)' });
}

function renderTitle() {
  $('#admin-badge').hidden = !S.status.admin;
}

const fmtCount = (n) => (n >= 10000 ? `${Math.round(n / 1000)}k` : n >= 1000 ? `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k` : String(n));

function renderHero() {
  const on = S.config.enabled;
  const active = rules().filter((r) => r.enabled).length;
  const bubble = $('#bubble');
  const text = on ? (active ? `Mrrp! ${active} shortcut${active === 1 ? ' is' : 's are'} purring.` : `${cat()} is ready for shortcuts.`) : `Zzz… ${cat()} is napping.`;
  if (bubble.dataset.text !== text) {
    bubble.dataset.text = text;
    bubble.textContent = text;
    bubble.classList.remove('swap');
    void bubble.offsetWidth;
    bubble.classList.add('swap');
  }
  $('#sub').textContent = !on
    ? 'Every key works like normal until you wake ' + cat() + ' up.'
    : PLAT() === 'mac'
      ? (active ? `Your shortcuts work in every app. Press one anywhere and ${cat()} taps along.` : `Pick shortcuts from the Library or make your own, and ${cat()} runs them in every app.`)
      : 'Alt now works like ⌘ on a Mac, in every app. Press one anywhere and ' + cat() + ' taps along.';
  const master = $('#master');
  master.setAttribute('aria-checked', String(on));
  master.querySelector('.switch').setAttribute('aria-checked', String(on));
  $('#master-label').textContent = on ? 'Awake' : 'Napping';
  $('#chip-today').innerHTML = `${icon('paw')}<b class="tick">${fmtCount(S.stats.today || 0)}</b> remaps today`;
  $('#chip-on').innerHTML = `${icon('keyboard')}${active} of ${rules().length} on`;
  $('#app').classList.toggle('napping', !on);
  hero.setMood(on ? 'awake' : 'nap');
  const xp = xpBreakdown();
  const r = rankFor(xp.total);
  $('#hero-profile').replaceChildren(
    h('span', { class: 'hp-rank' }, h('span', { html: icon('star'), style: { display: 'inline-flex' } }), h('b', { text: r.name })),
    h('span', { class: 'hp-bar', 'aria-hidden': 'true' }, h('span', { style: { width: `${r.pct}%` } })),
    h('span', { class: 'hp-xp', text: `${xp.total.toLocaleString()} XP` }),
    h('span', { class: 'hp-go', text: 'Your profile ›' }));
}

/** The fun part, one click from the top: pick another cat right next to her. */
function catQuickPicker(anchor) {
  const me = myCat();
  const grid = h('div', { class: 'qp-grid' });
  for (const c of allCats()) {
    const mini = h('span', { class: 'mini' });
    grid.append(h('button', { class: `qp-cat${c.id === me.id ? ' on' : ''}`, type: 'button', title: c.full, 'aria-pressed': String(c.id === me.id),
      onclick: () => { closePop(); chooseCat(c).then(() => { hero?.knead(1400); floatHearts($('#cushion')); }); } },
      mini, h('span', { class: 'nm', text: catTitle(c) })));
    new Mascot(mini, { size: 58, look: myLook(c) });
  }
  showPop(anchor, h('div', { class: 'pop qp' },
    h('div', { class: 'qp-head' }, h('b', { text: 'Pick your cat' }), h('span', { class: 'hint', text: 'Just the look: your rank and stats stay yours.' })),
    grid,
    h('div', { class: 'qp-foot' },
      h('button', { class: 'btn small', type: 'button', html: `${icon('star')}<span>Dress up</span>`, onclick: () => { closePop(); setView('warrior'); setTimeout(() => $('.wardrobe')?.scrollIntoView({ behavior: 'smooth', block: 'center' }), 300); } }),
      h('button', { class: 'btn small', type: 'button', html: `${icon('wand')}<span>Make your own</span>`, onclick: () => { closePop(); W.editor = true; setView('warrior'); setTimeout(() => $('#wr-cats')?.scrollIntoView({ behavior: 'smooth', block: 'start' }), 300); } }))));
}

function checkupRows() {
  if (PLAT() === 'mac') return macCheckupRows();
  const st = S.status;
  const s = S.config.settings;
  const rows = [];
  if (st.engineRunning === false) {
    rows.push({ level: 'bad', title: 'The keyboard engine isn’t running', desc: 'Your shortcuts are off until Cat Key restarts.', actions: [['Restart Cat Key', () => invoke('restart_app')]] });
  } else {
    rows.push({ level: 'ok', title: 'Keyboard engine is running', desc: 'Every shortcut is handled the moment you press it.' });
  }
  if (!s.launchAtStartup) {
    rows.push({ level: 'info', title: 'Doesn’t start with Windows', desc: 'After a restart you’d have to open Cat Key yourself.', actions: [['Turn on', () => commitSettings({ launchAtStartup: true }).then(() => setTimeout(refreshStatusNow, 800))]] });
  } else if (st.startup === 'blocked') {
    rows.push({ level: 'warn', title: 'Windows switched Cat Key off in Startup apps', desc: 'Find Cat Key in that list and switch it on, so it wakes up every time you sign in.', actions: [['Open Startup apps', () => invoke('open_system_page', { page: 'startup' }).catch((e) => toast(String(e), { kind: 'err' }))]] });
  } else if (st.startup === 'missing') {
    rows.push({ level: 'warn', title: 'Start with Windows isn’t set up yet', desc: 'One click and Cat Key wakes up quietly every time you sign in.', actions: [['Fix it', async () => {
      try { await invoke('repair_startup'); toast('Fixed. Cat Key now starts with Windows.'); } catch (e) { toast(`Couldn’t fix it: ${e}`, { kind: 'err' }); }
      refreshStatusNow();
    }]] });
  } else {
    rows.push({ level: 'ok', title: 'Starts with Windows', desc: 'Wakes up quietly in the tray when you sign in.' });
  }
  if (st.admin) {
    rows.push({ level: 'ok', title: 'Works in admin apps too', desc: 'Running as administrator, so Task Manager and admin terminals get your shortcuts.' });
  } else if (s.adminMode) {
    rows.push({ level: 'warn', title: 'Admin mode is on, but this copy runs normally', desc: 'Give permission once more so shortcuts reach apps running as administrator.', actions: [['Allow', () => setAdmin(true)]] });
  } else {
    rows.push({ level: 'info', title: 'Admin apps aren’t covered yet', desc: 'Windows keeps apps running as administrator (Task Manager, admin terminals) apart. Allow once and your shortcuts work there too.', actions: [['Allow', () => setAdmin(true)]] });
  }
  if (st.powertoysRunning) {
    rows.push({ level: 'bad', title: 'PowerToys Keyboard Manager is on: switch it off', desc: 'It changes the same keys as Cat Key, so a press can be handled twice: an error sound, or a shortcut you have to press twice. In PowerToys, open Keyboard Manager and turn it off. Your shortcuts already live here.', actions: [
      ['Open PowerToys', () => invoke('open_system_page', { page: 'powertoys' }).catch((e) => toast(String(e), { kind: 'warn' }))],
      ['Import its shortcuts', importPowerToys],
    ] });
  } else {
    rows.push({ level: 'ok', title: 'No other remapper is fighting for your keys', desc: 'Cat Key has the keyboard to itself.' });
  }
  if (st.grabAndMove) {
    rows.push({ level: 'warn', title: 'PowerToys Grab And Move listens to Alt too', desc: 'It holds Alt back for a moment to see whether you drag a window. Cat Key copes with that now, but for the smoothest Alt shortcuts set Grab And Move to the Windows key: PowerToys → Grab And Move → Activation key.', actions: [
      ['Open PowerToys', () => invoke('open_system_page', { page: 'powertoys' }).catch((e) => toast(String(e), { kind: 'warn' }))],
    ] });
  }
  return rows;
}

/** The Mac's checkup: the permission that lets Cat Key use the keyboard, where it runs from, login. */
function macCheckupRows() {
  const st = S.status;
  const s = S.config.settings;
  const rows = [];
  const page = (p) => () => invoke('open_system_page', { page: p }).catch((e) => toast(String(e), { kind: 'err' }));
  if (st.tempPlace) {
    rows.push({ level: 'warn', title: 'Move Cat Key to Applications first', desc: 'It’s running from the disk image or Downloads, where macOS forgets its permissions. Drag Cat Key into Applications and open it from there.' });
  }
  if (!st.keyboardAllowed) {
    rows.push({ level: 'bad', title: 'Let Cat Key use the keyboard', desc: 'macOS asks once. Open Privacy & Security → Accessibility, switch on Cat Key (add it with + if it isn’t listed), and your shortcuts start right away.', actions: [['Open Accessibility', page('accessibility')], ['Input Monitoring', page('input-monitoring')]] });
  } else if (st.engineRunning === false) {
    rows.push({ level: 'warn', title: 'The keyboard is starting', desc: 'Allowed. If it stays like this, quit Cat Key and open it again.', actions: [['Restart Cat Key', () => invoke('restart_app')]] });
  } else {
    rows.push({ level: 'ok', title: 'Keyboard engine is running', desc: 'Allowed in Privacy & Security. Every shortcut is handled the moment you press it.' });
  }
  if (!s.launchAtStartup) {
    rows.push({ level: 'info', title: 'Doesn’t open at login', desc: 'After a restart you’d have to open Cat Key yourself.', actions: [['Turn on', () => commitSettings({ launchAtStartup: true }).then(() => setTimeout(refreshStatusNow, 800))]] });
  } else if (st.startup === 'missing') {
    rows.push({ level: 'warn', title: 'Open at login isn’t set up yet', desc: 'One click and Cat Key opens quietly every time you log in.', actions: [['Fix it', async () => {
      try { await invoke('repair_startup'); toast('Fixed. Cat Key now opens at login.'); } catch (e) { toast(`Couldn’t fix it: ${e}`, { kind: 'err' }); }
      refreshStatusNow();
    }]] });
  } else {
    rows.push({ level: 'ok', title: 'Opens at login', desc: 'Wakes up quietly in the menu bar when you log in.' });
  }
  return rows;
}

function checkupEl(rows) {
  const list = h('div', { class: 'checkup-list' });
  for (const r of rows) {
    const ic = r.level === 'ok' ? 'check' : r.level === 'info' ? 'info' : 'warn';
    list.append(h('div', { class: `check-row ${r.level}` },
      h('span', { class: 'check-ico', html: icon(ic) }),
      h('div', { class: 'txt' }, h('div', { class: 't1', text: r.title }), r.desc ? h('div', { class: 't2', text: r.desc }) : null),
      r.actions ? h('div', { class: 'acts' }, ...r.actions.map(([label, fn], i) =>
        h('button', { class: `btn small${i === 0 && r.level !== 'ok' ? ' strong' : ''}`, type: 'button', text: label, onclick: fn }))) : null));
  }
  return list;
}

// A box that shows exactly what reaches an app when you press a shortcut here.
function testPadEl() {
  const seen = [];
  const log = h('div', { class: 'pad-log' });
  const keyName = (e) => {
    const c = e.code || '';
    if (/^Key[A-Z]$/.test(c)) return c.slice(3);
    if (/^Digit\d$/.test(c)) return c.slice(5);
    if (/^F\d{1,2}$/.test(c)) return c;
    const map = { Escape: 'Esc', ArrowLeft: '←', ArrowRight: '→', ArrowUp: '↑', ArrowDown: '↓', ' ': 'Space' };
    return map[e.key] || e.key;
  };
  const show = () => {
    log.textContent = '';
    if (!seen.length) { log.append(h('span', { class: 'hint', text: 'Nothing pressed yet.' })); return; }
    for (const x of seen.slice(-3).reverse()) {
      log.append(h('div', { class: `pad-row ${x.kind}` }, h('b', { text: x.combo }), h('span', { text: x.note })));
    }
  };
  const pad = h('div', { class: 'pad', tabindex: '0', role: 'textbox', 'aria-label': 'Test pad. Click, then press a shortcut' },
    h('div', { class: 'pad-hint', html: `${icon('keyboard')}<span>Click here, then press one of your shortcuts${PLAT() === 'mac' ? '' : ', like Alt + C'}</span>` }), log);
  pad.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) { pad.blur(); return; } // Esc still closes Settings
    e.preventDefault();
    e.stopPropagation();
    if (['Control', 'Alt', 'Shift', 'Meta', 'AltGraph'].includes(e.key)) return;
    const mods = [e.metaKey && 'Win', e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift'].filter(Boolean);
    const key = keyName(e);
    const combo = [...mods, key].join(' + ');
    const trig = [...mods, key.length === 1 ? key.toUpperCase() : key].join('+');
    const caught = S.config.enabled && rules().some((r) => r.enabled && K.overlaps(r.trigger, trig));
    const x = caught
      ? { kind: 'bad', combo, note: 'slipped through unchanged' }
      : { kind: mods.length ? 'ok' : 'plain', combo, note: mods.length ? 'reached the app' : '' };
    if (e.repeat) return;
    seen.push(x);
    show();
  });
  pad.addEventListener('keyup', (e) => { e.preventDefault(); e.stopPropagation(); });
  show();
  return h('div', { class: 'padwrap' }, pad,
    h('p', { class: 'hint', style: { margin: '0' }, text: PLAT() === 'mac' ? 'Press a shortcut here and you see what your apps get instead, once.' : 'Press Alt + C here and you should see Ctrl + C, once. Alt + Q closes this window, just like everywhere else.' }));
}

// Key log: what Cat Key sees and sends, to understand a shortcut that misbehaves.
const KEYLOG = { on: false, lines: [], timer: null, synced: false };

function keyLogPaint() {
  const pre = $('#keylog-lines');
  if (pre) {
    pre.textContent = KEYLOG.lines.length ? KEYLOG.lines.join('\n') : 'Nothing logged yet.';
    pre.scrollTop = pre.scrollHeight;
  }
  const btn = $('#keylog-toggle');
  if (btn) {
    btn.innerHTML = KEYLOG.on ? `${icon('block')}<span>Stop key log</span>` : `${icon('keyboard')}<span>Start key log</span>`;
    btn.classList.toggle('strong', !KEYLOG.on);
  }
  $('#keylog-dot')?.classList.toggle('on', KEYLOG.on);
}

async function keyLogPoll() {
  const r = await invoke('key_log_take').catch(() => null);
  if (!r) return;
  if (r.lines?.length) {
    KEYLOG.lines.push(...r.lines);
    if (KEYLOG.lines.length > 3000) KEYLOG.lines.splice(0, KEYLOG.lines.length - 3000);
  }
  if (KEYLOG.on !== r.on) KEYLOG.on = r.on;
  if (!KEYLOG.on && KEYLOG.timer) { clearInterval(KEYLOG.timer); KEYLOG.timer = null; }
  keyLogPaint();
}

async function keyLogToggle() {
  const on = !KEYLOG.on;
  if (on) KEYLOG.lines = [];
  await invoke('key_log', { on }).catch(() => {});
  KEYLOG.on = on;
  if (on && !KEYLOG.timer) KEYLOG.timer = setInterval(keyLogPoll, 300);
  await keyLogPoll();
  if (on) toast('Key log is on. Use the shortcut that misbehaves, then come back and save the log.', { ms: 6000 });
}

function keyLogEl() {
  if (!KEYLOG.synced) {
    KEYLOG.synced = true;
    keyLogPoll().then(() => { if (KEYLOG.on && !KEYLOG.timer) KEYLOG.timer = setInterval(keyLogPoll, 300); });
  }
  const el = h('div', { class: 'keylog' },
    h('div', { class: 'row-between' },
      h('div', { class: 'keylog-title' }, h('span', { id: 'keylog-dot', class: `rec-dot${KEYLOG.on ? ' on' : ''}` }), h('b', { text: 'Key log' })),
      h('div', { class: 'keylog-btns' },
        h('button', { class: 'btn small', type: 'button', text: 'Copy', onclick: async () => {
          try { await navigator.clipboard.writeText(KEYLOG.lines.join('\n')); toast('Copied the key log.'); } catch { toast('Couldn’t copy. Use Save instead.', { kind: 'warn' }); }
        } }),
        h('button', { class: 'btn small', type: 'button', text: 'Save…', onclick: async () => {
          try { if (await invoke('save_text', { name: 'CatKey key log.txt', text: KEYLOG.lines.join('\r\n') })) toast('Saved.'); } catch (e) { toast(String(e), { kind: 'err' }); }
        } }),
        h('button', { class: 'btn small ghost', type: 'button', text: 'Clear', onclick: () => { KEYLOG.lines = []; keyLogPaint(); } }))),
    h('pre', { id: 'keylog-lines', class: 'keylog-lines' }),
    h('button', { id: 'keylog-toggle', class: 'btn small', type: 'button', onclick: keyLogToggle }),
    h('p', { class: 'hint', style: { margin: '0' }, text: 'Shows every key Cat Key sees, what it did and what it sent. Turn it on, use the shortcut that misbehaves in the app where it happens, then come back and save. It records everything you type while it’s on (so don’t type passwords) and stops by itself after 10 minutes.' }));
  requestAnimationFrame(keyLogPaint);
  return el;
}

async function refreshStatusNow() {
  const st = await invoke('get_status').catch(() => null);
  if (!st) return;
  S.status = st;
  renderBanners();
  renderTitle();
  if (SETTINGS_OPEN) renderSettings();
}

function renderBanners() {
  const box = $('#banners');
  box.textContent = '';
  const rows = checkupRows();
  const issues = rows.filter((r) => r.level === 'warn' || r.level === 'bad');
  if (S.checkupDismissed || (!issues.length && !S.status.firstRun)) return;
  const shown = S.status.firstRun ? rows : rows.filter((r) => r.level !== 'ok');
  const head = issues.length
    ? `${cat()} found ${issues.length === 1 ? 'one thing that needs' : `${issues.length} things that need`} you.`
    : `All set. ${cat()} checked everything.`;
  box.append(h('section', { class: 'checkup', role: 'status' },
    h('div', { class: 'checkup-head' },
      h('span', { class: 'check-badge', html: icon(issues.length ? 'shield' : 'check') }),
      h('div', { class: 'txt' }, h('div', { class: 't1 display', text: 'Quick checkup' }), h('div', { class: 't2', text: head })),
      h('button', { class: 'tbtn', type: 'button', 'aria-label': 'Hide the checkup', title: 'Hide (it’s also in Settings)', html: icon('x'), onclick: () => { S.checkupDismissed = true; renderBanners(); } })),
    checkupEl(shown)));
}

// ------------------------------------------------------------------ list

function matches(r, q) {
  if (!q) return true;
  const a = r.action;
  const hay = [ruleName(r), K.plain(r.trigger), r.trigger, a.keys ? K.plain(a.keys) : '', a.target || '', a.text || '', appName(a.target || ''), ...(r.scope.apps || []), ...(r.scope.apps || []).map(appName), scopeText(r.scope)]
    .join(' ').toLowerCase();
  return q.split(/\s+/).every((w) => hay.includes(w));
}

function visibleRules() {
  const q = S.query.trim().toLowerCase();
  return rules().filter((r) => {
    const perApp = r.scope.mode === 'only' && r.scope.apps.length > 0;
    if (S.filter === 'global' && perApp) return false;
    if (S.filter === 'apps' && !perApp) return false;
    return matches(r, q);
  });
}

function renderFilters() {
  const all = rules();
  const perApp = all.filter((r) => r.scope.mode === 'only' && r.scope.apps.length).length;
  const items = [['all', 'All', all.length], ['global', 'Everywhere', all.length - perApp], ['apps', 'Per app', perApp]];
  const box = $('#filters');
  box.textContent = '';
  for (const [id, label, n] of items) {
    box.append(h('button', { type: 'button', 'aria-pressed': String(S.filter === id), onclick: () => { S.filter = id; renderFilters(); renderList(); } },
      label, h('span', { class: 'count', text: String(n) })));
  }
}

function cardFor(r) {
  const el = h('div', { class: 'card', role: 'listitem', 'data-id': r.id });
  fillCard(el, r);
  return el;
}

function fillCard(el, r) {
  el.textContent = '';
  el.classList.toggle('off', !r.enabled);
  el.classList.toggle('fresh', S.fresh.has(r.id));
  const a = r.action;
  let to;
  if (a.type === 'keys') to = h('span', { class: 'to' }, capsEl(a.keys, { to: true }));
  else if (a.type === 'launch') to = h('span', { class: 'to launch-to' }, appIcon(a.target, 'appbig'), h('span', { text: appName(a.target) }));
  else if (a.type === 'text') to = h('span', { class: 'to text-to' }, h('span', { class: 'tag-ico', html: icon('text') }), h('span', { text: `“${a.text.replace(/\s+/g, ' ')}”` }));
  else to = h('span', { class: 'to text-to' }, h('span', { class: 'tag-ico', html: icon('block') }), h('span', { text: 'Blocked' }));

  const perApp = r.scope.mode !== 'all' && r.scope.apps.length;
  const problem = S.status.problems?.[r.id];
  const arrowSvg = '<svg viewBox="0 0 40 16" width="40" height="16" aria-hidden="true"><circle class="dot" cx="4" cy="8" r="1.9" fill="currentColor" opacity=".45"/><circle class="dot" cx="11" cy="8" r="1.9" fill="currentColor" opacity=".75"/><path d="M17 8h17M29 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
  const arrow = canSwap(r)
    ? h('button', { class: 'arrow swap', type: 'button', title: `Swap them: ${K.plain(a.keys)} would do ${K.plain(r.trigger)}`,
        'aria-label': `Swap ${ruleName(r)} so ${K.plain(a.keys)} does ${K.plain(r.trigger)}`,
        onclick: (e) => { e.stopPropagation(); swapRule(r); }, onkeydown: (e) => e.stopPropagation() },
      h('span', { class: 'go', html: arrowSvg }), h('span', { class: 'sw', html: icon('swap') }))
    : h('span', { class: 'arrow', html: arrowSvg });
  const picked = SEL.on && SEL.ids.has(r.id);
  el.classList.toggle('picked', picked);
  const open = (e) => (SEL.on ? toggleSelect(r, e) : openEditor(r));
  const main = h('div', { class: 'card-main', role: 'button', tabindex: '0', 'aria-label': SEL.on ? `Select ${ruleName(r)}` : `Edit ${ruleName(r)}`, 'aria-pressed': SEL.on ? String(picked) : null, onclick: open,
    onkeydown: (e) => { if (e.target === e.currentTarget && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); open(e); } } },
    SEL.on ? h('span', { class: `pick${picked ? ' on' : ''}`, 'aria-hidden': 'true', html: picked ? icon('check') : '' }) : null,
    h('span', { class: 'from' }, capsEl(r.trigger)),
    arrow,
    to,
    h('span', { class: 'meta' },
      h('span', { class: 'name' }, h('span', { text: ruleName(r) }),
        S.fresh.has(r.id) ? h('em', { class: 'pill new', text: 'new' }) : null,
        problem ? h('em', { class: 'pill warn', title: problem, text: 'needs a fix' }) : null),
      h('span', { class: 'scope' },
        perApp ? appIcon(r.scope.apps[0]) : h('span', { html: icon('globe') , style: { display: 'inline-flex', width: '14px' } }),
        h('span', { text: scopeText(r.scope) }))));
  const uses = h('span', { class: 'uses', title: 'Times used' }, h('span', { html: icon('paw'), style: { display: 'inline-flex' } }), h('b', { text: fmtCount(S.stats.perRule?.[r.id] || 0) }));
  const sw = h('button', { class: 'switch', type: 'button', role: 'switch', 'aria-checked': String(r.enabled), 'aria-label': `${r.enabled ? 'Turn off' : 'Turn on'} ${ruleName(r)}`,
    onclick: (e) => { e.stopPropagation(); toggleRule(r.id); } }, h('span', { class: 'knob', html: icon('paw') }));
  const more = h('button', { class: 'more', type: 'button', 'aria-label': `More for ${ruleName(r)}`, html: icon('more'), onclick: (e) => { e.stopPropagation(); ruleMenu(e.currentTarget, r); } });
  el.append(main, uses, sw, more);
}

function renderList() {
  const box = $('#list');
  const list = visibleRules();
  $('#list-head').classList.toggle('none', !list.length);
  const before = new Map();
  for (const el of box.children) if (el.dataset.id) before.set(el.dataset.id, el.getBoundingClientRect().top);
  const existing = new Map([...box.querySelectorAll('.card')].map((e) => [e.dataset.id, e]));
  const keep = new Set();
  const frag = [];
  for (const r of list) {
    let el = existing.get(r.id);
    if (el) fillCard(el, r);
    else el = cardFor(r);
    keep.add(r.id);
    frag.push(el);
  }
  for (const [id, el] of existing) if (!keep.has(id)) el.remove();
  box.querySelector('.empty')?.remove();
  frag.forEach((el) => box.append(el));
  if (!list.length) {
    const empty = h('div', { class: 'empty' });
    const m = h('div', { class: 'mascot-mini' });
    empty.append(m);
    new Mascot(m, { size: 84, look: myLook() }).setMood(rules().length ? 'awake' : 'nap');
    empty.append(h('div', { text: rules().length ? `No shortcut matches “${S.query}”. ${cat()} looked everywhere.` : `No shortcuts yet. Press New shortcut to teach ${cat()} one.` }));
    box.append(empty);
  }
  if (reduced()) return;
  for (const el of box.querySelectorAll('.card')) {
    const id = el.dataset.id;
    const top = el.getBoundingClientRect().top;
    if (before.has(id)) {
      const dy = before.get(id) - top;
      if (Math.abs(dy) > 1) el.animate([{ transform: `translateY(${dy}px)` }, { transform: 'none' }], { duration: 320, easing: 'cubic-bezier(.22,1,.36,1)' });
    } else if (before.size) {
      el.animate([{ opacity: 0, transform: 'translateY(8px) scale(.98)' }, { opacity: 1, transform: 'none' }], { duration: 320, easing: 'cubic-bezier(.34,1.3,.64,1)' });
    }
  }
}

function suggestionList() {
  const t = S.templates.find((x) => x.id === 'builtin-mac-more');
  return t ? t.rules.filter((r) => r.action.type === 'keys').map((r) => [r.trigger, r.action.keys, r.name]) : SUGGESTIONS;
}

function renderSuggestions() {
  const box = $('#suggest');
  box.textContent = '';
  const have = rules().map((r) => r.trigger);
  const list = suggestionList().filter(([from]) => !have.some((t) => K.overlaps(t, from))).slice(0, 6);
  if (!list.length) return;
  box.append(h('div', { class: 'row-between' },
    h('h2', { html: icon('sparkle') }, h('span', { text: `${cat()} suggests` })),
    h('button', { class: 'link', type: 'button', text: 'See all templates', onclick: () => setView('templates') })));
  const row = h('div', { class: 'row' });
  for (const [from, to, name] of list) {
    row.append(h('button', { class: 'sg', type: 'button', onclick: (e) => addSuggestion(from, to, name, e.currentTarget) },
      h('span', { class: 'p', html: icon('plus') }), K.plain(from), h('span', { 'aria-hidden': 'true', text: '→', style: { color: 'var(--mu)' } }),
      h('span', { class: 'to', text: K.plain(to) }), h('span', { class: 'nm', text: name })));
  }
  box.append(row);
}

function markFresh(id) {
  S.fresh.add(id);
  setTimeout(() => {
    S.fresh.delete(id);
    const el = document.querySelector(`.card[data-id="${CSS.escape(id)}"]`);
    const r = rules().find((x) => x.id === id);
    if (el && r) fillCard(el, r);
  }, 6000);
}

async function addSuggestion(from, to, name, btn) {
  const r = { id: newId(), enabled: true, name, trigger: from, action: { type: 'keys', keys: to }, scope: { mode: 'all', apps: [] } };
  const rect = btn.getBoundingClientRect();
  confetti(rect.left + rect.width / 2, rect.top);
  markFresh(r.id);
  await commit({ ...S.config, rules: [...rules(), r] });
  sounds.pop();
  toast(`Added ${K.plain(from)} → ${K.plain(to)}`);
  document.querySelector(`.card[data-id="${CSS.escape(r.id)}"]`)?.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'nearest' });
}

// ------------------------------------------------------------------ saving

let saving = Promise.resolve();
function commit(next) {
  S.config = next;
  renderAll();
  saving = saving.then(async () => {
    try {
      const saved = await invoke('save_config', { config: next });
      if (S.config === next) S.config = saved;
      S.status = await invoke('get_status').catch(() => S.status);
      renderTitle();
    } catch (e) {
      toast(`Couldn’t save: ${e}`, { kind: 'err', ms: 7000 });
    }
  });
  return saving;
}
function commitSettings(patch) {
  const next = { ...S.config, settings: { ...S.config.settings, ...patch } };
  const p = commit(next);
  applyLook();
  setSoundsEnabled(next.settings.sounds);
  if (SETTINGS_OPEN) renderSettings();
  return p;
}

function toggleRule(id) {
  const next = rules().map((r) => (r.id === id ? { ...r, enabled: !r.enabled } : r));
  const now = next.find((r) => r.id === id).enabled;
  now ? sounds.on() : sounds.off();
  commit({ ...S.config, rules: next });
}

async function setMaster(on) {
  on ? sounds.on() : sounds.off();
  S.config = { ...S.config, enabled: on };
  renderHero();
  renderList();
  await invoke('set_enabled', { enabled: on }).catch((e) => toast(String(e), { kind: 'err' }));
}

function deleteRule(id) {
  const idx = rules().findIndex((r) => r.id === id);
  if (idx < 0) return;
  const removed = rules()[idx];
  const el = document.querySelector(`.card[data-id="${CSS.escape(id)}"]`);
  const finish = () => {
    commit({ ...S.config, rules: rules().filter((r) => r.id !== id) });
    toast(`Deleted “${ruleName(removed)}”`, {
      ms: 6500,
      action: { label: 'Undo', fn: () => { const list = [...rules()]; list.splice(Math.min(idx, list.length), 0, removed); markFresh(removed.id); commit({ ...S.config, rules: list }); } },
    });
  };
  if (el && !reduced()) {
    el.animate([{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateX(24px) scale(.97)' }], { duration: 220, easing: 'ease-in' }).onfinish = finish;
  } else finish();
}

function swapRule(r) {
  const sw = swapped(r);
  if (!sw) { toast('Only shortcuts that press keys can be swapped.', { kind: 'warn' }); return; }
  const problem = swapProblem(sw);
  if (problem) { toast(`Can’t swap this one. ${problem}`, { kind: 'warn', ms: 6000 }); return; }
  sounds.pop();
  commit({ ...S.config, rules: rules().map((x) => (x.id === r.id ? sw : x)) });
  const main = document.querySelector(`.card[data-id="${CSS.escape(r.id)}"] .card-main`);
  if (main && !reduced()) main.animate([{ transform: 'perspective(700px) rotateX(75deg)', opacity: 0.15 }, { transform: 'none', opacity: 1 }], { duration: 440, easing: 'cubic-bezier(.34,1.3,.64,1)' });
  const undo = { label: 'Undo', fn: () => commit({ ...S.config, rules: rules().map((x) => (x.id === r.id ? r : x)) }) };
  const clash = sw.enabled ? clashFor(sw) : null;
  if (clash) toast(`Swapped. “${ruleName(clash)}” also uses ${K.plain(sw.trigger)}, and the one higher in the list wins.`, { kind: 'warn', ms: 8000, action: undo });
  else toast(`Swapped: ${K.plain(sw.trigger)} now does ${K.plain(sw.action.keys)}.`, { ms: 6000, action: undo });
}

function duplicateRule(r) {
  const copy = { ...clone(r), id: newId(), enabled: false, name: r.name ? `${r.name} (copy)` : '' };
  const list = [...rules()];
  list.splice(list.findIndex((x) => x.id === r.id) + 1, 0, copy);
  markFresh(copy.id);
  commit({ ...S.config, rules: list });
  toast('Duplicated. The copy is off until you change its keys.');
}

// ------------------------------------------------------------------ events from Rust

function onFired(id) {
  S.stats.today = (S.stats.today || 0) + 1;
  if (id) S.stats.perRule[id] = (S.stats.perRule[id] || 0) + 1;
  const tick = $('#chip-today .tick');
  if (tick) {
    tick.textContent = fmtCount(S.stats.today);
    tick.classList.remove('bump');
    void tick.offsetWidth;
    tick.classList.add('bump');
  }
  hero.react();
  if (!id) return;
  const card = document.querySelector(`.card[data-id="${CSS.escape(id)}"]`);
  if (!card) return;
  const b = card.querySelector('.uses b');
  if (b) b.textContent = fmtCount(S.stats.perRule[id]);
  card.classList.add('fired');
  const name = card.querySelector('.name');
  if (name && !name.querySelector('.hot')) name.append(h('em', { class: 'pill hot', text: 'just now' }));
  clearTimeout(S.firedTimers.get(id));
  S.firedTimers.set(id, setTimeout(() => {
    card.classList.remove('fired');
    card.querySelector('.pill.hot')?.remove();
  }, 1400));
}

function wire() {
  $('#btn-settings').onclick = openSettings;
  $('#btn-about').onclick = openAbout;
  $('#btn-purr').onclick = () => setPurr(!PURR.on);
  $('#cat-switch').onclick = (e) => catQuickPicker(e.currentTarget);
  $('#hero-profile').onclick = () => setView('warrior');
  $('#btn-purr').oncontextmenu = (e) => { e.preventDefault(); purrMenu(e.currentTarget); };
  backend.listen('ck://purr', (e) => { PURR.on = !!e.payload; paintPurr(); });
  backend.listen('ck://coach-tip', () => { coachStats(true).then(() => { if (S.view === 'warrior') renderWarrior(); }); });
  // Every click gets a soft paw tap (and now and then a little mew).
  document.addEventListener('click', (e) => { if (e.target.closest?.('button, [role="button"], summary, label')) clickSound(); }, true);
  $('#btn-min').onclick = () => invoke('window_minimize');
  $('#btn-close').onclick = () => invoke('window_close');
  $('#btn-new').onclick = () => openEditor(null);
  $('#btn-select').onclick = () => setSelecting(!SEL.on);
  $('#master').onclick = () => setMaster(!S.config.enabled);
  $('#cushion').onclick = () => {
    if (!S.config.enabled) {
      setMaster(true);
      return;
    }
    hero.knead(1800);
    sounds.knead();
    floatHearts($('#cushion'));
    const lines = ['Making biscuits… 🐾', 'Purrrrr…', 'Mrrp!', 'Hehe, that tickles!', '*knead knead knead*', 'Nya~', 'More pets, please.', `${cat()} approves.`];
    const bubble = $('#bubble');
    bubble.textContent = lines[Math.floor(Math.random() * lines.length)];
    bubble.classList.remove('swap');
    void bubble.offsetWidth;
    bubble.classList.add('swap');
    clearTimeout(bubble.timer);
    bubble.timer = setTimeout(() => { bubble.dataset.text = ''; renderHero(); }, 2200);
  };
  const search = $('#search');
  search.addEventListener('input', () => { S.query = search.value; renderList(); });
  $('#scrim').onclick = () => { if (E) closeEditor(); else if (PK) closePack(); else if (SETTINGS_OPEN) closeSettings(); };

  backend.listen('ck://fired', (e) => onFired(e.payload));
  backend.listen('ck://enabled', (e) => { S.config = { ...S.config, enabled: !!e.payload }; renderHero(); renderList(); });
  backend.listen('ck://error', (e) => toast(String(e.payload), { kind: 'err', ms: 7000 }));
  backend.listen('ck://record', (e) => {
    const { phase, shortcut } = e.payload || {};
    if (!REC) return;
    const target = REC;
    if (phase === 'update') target.onUpdate?.(shortcut);
    else if (phase === 'done') { REC = null; target.onDone?.(shortcut); }
    else if (phase === 'cancel') { REC = null; target.onCancel?.(); toast('Stopped listening. Nothing was pressed.', { kind: 'warn' }); }
  });

  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyLook);
  document.addEventListener('visibilitychange', () => $('#app').classList.toggle('hidden-doc', document.hidden));
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if (closePop()) return;
      if (DIALOG) { DIALOG(false); return; }
      if (E) { closeEditor(); return; }
      if (PK) { closePack(); return; }
      if (SETTINGS_OPEN) { closeSettings(); return; }
      if (SEL.on) { setSelecting(false); return; }
    }
    if (TR.on) return; // training: the keys belong to the drill
    // The app's own shortcuts use the system's command key: Ctrl on Windows, ⌘ on a Mac.
    const cmdKey = PLAT() === 'mac' ? e.metaKey && !e.ctrlKey : e.ctrlKey && !e.metaKey;
    if (SEL.on && cmdKey && e.key.toLowerCase() === 'a' && !E && !DIALOG && !/^(INPUT|TEXTAREA)$/.test(document.activeElement?.tagName)) {
      e.preventDefault(); visibleRules().forEach((r) => SEL.ids.add(r.id)); renderList(); renderBulkBar(); return;
    }
    const ctrl = cmdKey && !e.altKey;
    // In a text box, Ctrl keys do what they do in any text box (copy, paste, undo, select all…).
    const typingHere = /^(INPUT|TEXTAREA)$/.test(document.activeElement?.tagName) && document.activeElement?.id !== 'search';
    if (typingHere && !(E && ctrl && e.key.toLowerCase() === 's')) return;
    if (ctrl && e.key.toLowerCase() === 'f' && !E && !PK && !SETTINGS_OPEN && !DIALOG) {
      e.preventDefault();
      if (S.view === 'library') { $('#lib-search')?.focus(); return; }
      setView('shortcuts'); search.focus(); search.select();
    }
    if (ctrl && e.key.toLowerCase() === 'n' && !E && !PK && !SETTINGS_OPEN && !DIALOG) { e.preventDefault(); openEditor(null); }
    if (ctrl && e.key.toLowerCase() === 's' && E) { e.preventDefault(); saveEditor(); }
  });
}

// ------------------------------------------------------------------ recording

async function startRecording(target) {
  await stopRecording();
  REC = target;
  try {
    await invoke('start_recording');
  } catch (err) {
    REC = null;
    toast(String(err), { kind: 'err' });
  }
}
async function stopRecording() {
  if (!REC) return;
  const t = REC;
  REC = null;
  t.onCancel?.();
  await invoke('stop_recording').catch(() => {});
}

// ------------------------------------------------------------------ editor

function scopesOverlap(a, b) {
  const A = a.mode === 'all' || !a.apps.length ? null : a;
  const B = b.mode === 'all' || !b.apps.length ? null : b;
  if (!A || !B) return true;
  if (A.mode === 'only' && B.mode === 'only') return A.apps.some((x) => B.apps.includes(x));
  if (A.mode === 'only' && B.mode === 'except') return A.apps.some((x) => !B.apps.includes(x));
  if (A.mode === 'except' && B.mode === 'only') return B.apps.some((x) => !A.apps.includes(x));
  return true;
}

function analyze(d) {
  const err = {};
  const notes = [];
  const t = d.trigger;
  const tp = K.parse(t);
  if (!t) err.trigger = 'Press the keys you want to use.';
  else if (K.hasMods(t) && K.isModifierName(tp.key)) err.trigger = 'Add a regular key too, not just Ctrl, Alt, Shift or Win.';
  const a = d.action;
  if (a.type === 'keys') {
    if (!a.keys) err.keys = 'Record what it should press.';
    else if ((K.hasMods(t) || K.hasMods(a.keys)) && K.isModifierName(K.parse(a.keys).key)) err.keys = 'The result needs a regular key, like Ctrl + C.';
  }
  if (a.type === 'launch' && !(a.target || '').trim()) err.launch = 'Pick an app, file, folder or link to open.';
  if (a.type === 'text' && !(a.text || '')) err.text = 'Type what it should write.';
  if (d.scope.mode !== 'all' && !d.scope.apps.length) err.scope = 'Add at least one app, or pick Everywhere.';
  if (t && !err.trigger) {
    if (!K.hasMods(t) && a.type !== 'launch' && /^([A-Z0-9]|Space|Enter|Backspace|Tab)$/.test(tp.key)) {
      notes.push(['warn', `This changes the ${K.keyLabel(tp.key)} key itself, so it also applies while you type.`]);
    }
    if (!K.hasMods(t) && ['launch', 'text'].includes(a.type) && /^[A-Z0-9]$/.test(tp.key)) {
      notes.push(['warn', 'A single letter or number would fire while you type. Add Alt or Ctrl to it.']);
    }
    const reserved = K.reservedNote(t);
    if (reserved) notes.push(['bad', reserved]);
    const m = tp.mods;
    if (m.Alt && m.Shift && !m.Ctrl && !m.Win) notes.push(['info', `Alt + Shift on its own switches your keyboard language. With ${K.keyLabel(tp.key)} added it's a normal shortcut, and your language stays put.`]);
    const pause = S.config.settings.pauseHotkey;
    if (pause && K.overlaps(pause, t)) notes.push(['warn', 'This is also your pause hotkey, which wins.']);
    for (const r of rules()) {
      if (r.id === d.id || !r.enabled || !K.overlaps(r.trigger, t) || !scopesOverlap(r.scope, d.scope)) continue;
      const mineSpecific = d.scope.mode === 'only';
      const theirsSpecific = r.scope.mode === 'only';
      const nm = `“${ruleName(r)}”`;
      if (mineSpecific && !theirsSpecific) notes.push(['info', `In ${d.scope.apps.map(appName).join(', ')} this replaces ${nm}. App-only shortcuts win over everywhere ones.`]);
      else if (!mineSpecific && theirsSpecific) notes.push(['info', `${nm} still wins inside ${r.scope.apps.map(appName).join(', ')}.`]);
      else notes.push(['warn', `${nm} already uses ${K.plain(t)}. The one higher in the list wins.`]);
    }
  }
  return { err, notes };
}

function openEditor(rule, { isNew = !rule } = {}) {
  if (SETTINGS_OPEN) closeSettings();
  if (PK) closePack();
  const d = rule ? clone(rule) : { id: newId(), enabled: true, name: '', trigger: '', action: { type: 'keys', keys: '' }, scope: { mode: 'all', apps: [] } };
  d.scope = d.scope || { mode: 'all', apps: [] };
  d.scope.apps = d.scope.apps || [];
  E = {
    d, isNew, err: {}, rec: null, live: '', manual: { trigger: false, keys: false }, apps: null,
    mem: { keys: d.action.keys || '', launch: { target: d.action.target || '', args: d.action.args || '' }, text: d.action.text || '' },
  };
  const sheet = $('#editor');
  sheet.hidden = false;
  sheet.textContent = '';
  const mini = h('div', { class: 'mini' });
  sheet.append(
    h('div', { class: 'sheet-head' }, mini,
      h('div', { style: { flex: '1', minWidth: '0' } },
        h('h1', { id: 'ed-title', text: E.isNew ? `Teach ${cat()} a new shortcut` : 'Edit shortcut' }),
        h('p', { text: E.isNew ? `Press keys the way you normally would. ${cat()} remembers them.` : `Change anything. ${cat()} picks it up the moment you save.` })),
      h('button', { class: 'tbtn', 'aria-label': 'Close', html: icon('x'), onclick: closeEditor })),
    h('div', { class: 'sheet-body', id: 'ed-body' },
      h('div', { class: 'step' },
        h('div', { class: 'step-title' }, h('span', { class: 'n', text: '1' }), 'When I press'),
        h('div', { id: 'ed-trigger', class: 'step', style: { gap: '10px' } })),
      h('div', { class: 'swaprow', id: 'ed-swaprow' }),
      h('div', { class: 'step' },
        h('div', { class: 'step-title' }, h('span', { class: 'n', text: '2' }), 'Cat Key will…'),
        h('div', { class: 'seg soft modes', role: 'group', 'aria-label': 'Action', id: 'ed-modes' }),
        h('div', { id: 'ed-panel' })),
      h('div', { class: 'step' },
        h('div', { class: 'step-title' }, h('span', { class: 'n', text: '3' }), 'Where'),
        h('div', { id: 'ed-scope', class: 'step', style: { gap: '10px' } })),
      h('label', { class: 'label' }, h('span', {}, 'Name ', h('span', { class: 'opt', text: '(optional, shown on the card)' })),
        h('input', { class: 'field', id: 'ed-name', type: 'text', maxlength: '60', placeholder: autoName(d), value: d.name || '',
          oninput: (e) => { E.d.name = e.target.value; },
          onkeydown: (e) => { if (e.key === 'Enter') saveEditor(); } })),
      h('div', { id: 'ed-notes', class: 'step', style: { gap: '8px' } })),
    h('div', { class: 'sheet-foot' },
      E.isNew ? null : h('button', { class: 'btn ghost danger', html: `${icon('trash')}<span>Delete</span>`, onclick: () => { const id = E.d.id; closeEditor(); deleteRule(id); } }),
      h('span', { class: 'spacer' }),
      h('button', { class: 'btn ghost', text: 'Cancel', onclick: closeEditor }),
      h('button', { class: 'btn primary', id: 'ed-save', html: `${icon('check')}<span>Save shortcut</span>`, onclick: saveEditor })));
  E.mini = new Mascot(mini, { size: 76, look: myLook() });
  renderRecorderSection('trigger');
  renderModes();
  renderPanel();
  renderScope();
  renderNotes();
  $('#scrim').classList.add('on');
  requestAnimationFrame(() => sheet.classList.add('on'));
  if (E.isNew) setTimeout(() => E && !E.d.trigger && startRec('trigger'), 380);
}

async function closeEditor() {
  if (!E) return;
  await stopRecording();
  const sheet = $('#editor');
  sheet.classList.remove('on');
  $('#scrim').classList.remove('on');
  E.mini?.destroy();
  E = null;
  setTimeout(() => { if (!E) { sheet.hidden = true; sheet.textContent = ''; } }, 300);
}

function valueOf(kind) {
  return kind === 'trigger' ? E.d.trigger : E.d.action.keys || '';
}
function setValue(kind, v) {
  if (kind === 'trigger') E.d.trigger = v;
  else { E.d.action.keys = v; E.mem.keys = v; }
  E.err[kind] = null;
}

function recorderEl(kind) {
  const listening = E.rec === kind;
  const value = valueOf(kind);
  const isTo = kind === 'keys';
  const big = kind === 'trigger';
  const btn = h('button', {
    type: 'button', class: `recorder${isTo ? ' small' : ''}${listening ? ' listening' : ''}${E.err[kind] ? ' error' : ''}`,
    'aria-label': listening ? 'Stop listening' : 'Record keys',
    onclick: () => (listening ? stopRec() : startRec(kind)),
  });
  if (listening && E.live) btn.append(capsEl(E.live, { to: isTo, big }), h('span', { class: 'tag', text: 'Let go to finish' }));
  else if (listening) btn.append(h('span', { class: 'placeholder' }, h('span', { class: 'dot' }), 'Listening… press your shortcut'), h('span', { class: 'tag', text: 'Click to stop' }));
  else if (value) btn.append(capsEl(value, { to: isTo, big, pop: E.justRecorded === kind }), h('span', { class: 'tag', html: `${icon('refresh')}<span>Record again</span>` }));
  else btn.append(h('span', { class: 'placeholder', html: `${icon('keyboard')}<span>Click, then press the keys</span>` }));
  return btn;
}

function renderRecorderSection(kind) {
  if (!E) return;
  const box = kind === 'trigger' ? $('#ed-trigger') : $('#ed-keys');
  if (!box) return;
  box.textContent = '';
  box.append(recorderEl(kind));
  box.append(h('div', { class: 'row-between' },
    h('span', { class: 'hint', text: kind === 'trigger' ? (PLAT() === 'mac' ? 'macOS keeps ⌘ Tab for itself.' : 'Windows keeps Win + L and Ctrl + Alt + Del for itself.') : (PLAT() === 'mac' ? 'What your apps will see instead.' : 'What Windows will see instead.') }),
    h('button', { class: 'link', type: 'button', 'aria-expanded': String(E.manual[kind]), text: E.manual[kind] ? 'Hide fine-tuning' : 'Fine-tune by hand',
      onclick: () => { E.manual[kind] = !E.manual[kind]; renderRecorderSection(kind); } })));
  if (E.manual[kind]) box.append(manualEl(kind));
  if (E.err[kind]) box.append(h('div', { class: 'err', role: 'alert', text: E.err[kind] }));
  E.justRecorded = null;
  renderSwapRow();
}

function renderSwapRow() {
  const box = $('#ed-swaprow');
  if (!box || !E) return;
  box.textContent = '';
  if (E.d.action.type !== 'keys') return;
  const ready = !!E.d.trigger && !!E.d.action.keys;
  box.append(h('span', { class: 'line' }),
    h('button', { class: 'swapbtn', type: 'button', disabled: ready ? null : true, onclick: swapInEditor,
      title: ready ? `Swap them: ${K.plain(E.d.action.keys)} would do ${K.plain(E.d.trigger)}` : 'Record both sides first, then you can swap them' },
      h('span', { class: 'ic', html: icon('swap') }), h('span', { text: 'Swap' })),
    h('span', { class: 'line' }));
}

function swapInEditor() {
  if (!E || E.d.action.type !== 'keys') return;
  if (E.rec) stopRec();
  const t = E.d.trigger;
  E.d.trigger = E.d.action.keys || '';
  E.d.action.keys = t || '';
  E.mem.keys = E.d.action.keys;
  if (E.d.name) { E.d.name = swappedName(E.d.name); $('#ed-name').value = E.d.name; }
  E.err.trigger = null;
  E.err.keys = null;
  sounds.pop();
  E.mini?.react(700);
  renderRecorderSection('trigger');
  renderRecorderSection('keys');
  renderNotes();
  $('#ed-name').placeholder = autoName(E.d);
  if (reduced()) return;
  const ease = 'cubic-bezier(.22,1,.36,1)';
  $('#ed-swaprow .ic')?.animate([{ transform: 'rotate(0deg)' }, { transform: 'rotate(180deg)' }], { duration: 440, easing: 'cubic-bezier(.34,1.3,.64,1)' });
  $('#ed-trigger .recorder')?.animate([{ transform: 'translateY(34px)', opacity: 0.25 }, { transform: 'none', opacity: 1 }], { duration: 400, easing: ease });
  $('#ed-keys .recorder')?.animate([{ transform: 'translateY(-34px)', opacity: 0.25 }, { transform: 'none', opacity: 1 }], { duration: 400, easing: ease });
}

function manualEl(kind) {
  const sc = K.parse(valueOf(kind));
  const row = h('div', { class: 'manual' });
  const update = () => { setValue(kind, K.format(sc)); renderRecorderSection(kind); renderNotes(); };
  for (const f of K.FAMILIES) {
    const on = !!sc.mods[f];
    const chip = h('span', { class: `modchip${on ? ' on' : ''}` },
      h('button', { type: 'button', 'aria-pressed': String(on), text: f, onclick: () => { sc.mods[f] = on ? '' : 'A'; update(); } }));
    if (on) {
      const sel = h('select', { 'aria-label': `${f} side`, onchange: (e) => { sc.mods[f] = e.target.value; update(); } },
        ...[['A', 'Either'], ['L', 'Left'], ['R', 'Right']].map(([v, l]) => h('option', { value: v, selected: sc.mods[f] === v ? '' : null, text: l })));
      chip.append(sel);
    }
    row.append(chip);
  }
  row.append(h('span', { class: 'plus', text: '+' }));
  const keySel = h('select', { class: 'keysel', 'aria-label': 'Key', onchange: (e) => { sc.key = e.target.value; update(); } },
    h('option', { value: '', text: 'Pick a key…' }),
    ...K.KEY_GROUPS.map(([label, keys]) => h('optgroup', { label }, ...keys.map((k) => h('option', { value: k, selected: sc.key === k ? '' : null, text: K.keyLabel(k) + (K.isModifierName(k) ? ` (${k[0] === 'L' ? 'left' : 'right'})` : '') })))));
  row.append(keySel);
  return row;
}

function startRec(kind) {
  if (!E) return;
  E.rec = kind;
  E.live = '';
  E.mini?.setMood('listen');
  renderRecorderSection(kind);
  startRecording({
    onUpdate: (s) => { if (!E) return; E.live = s; renderRecorderSection(kind); },
    onDone: (s) => {
      if (!E) return;
      E.rec = null;
      E.live = '';
      E.mini?.setMood('awake');
      if (s) {
        setValue(kind, s);
        E.justRecorded = kind;
        sounds.pop();
        E.mini?.react(900);
        if (kind === 'trigger' && E.d.action.type === 'keys' && !E.d.action.keys) {
          renderRecorderSection(kind);
          renderNotes();
          setTimeout(() => E && E.rec === null && !E.d.action.keys && startRec('keys'), 450);
          return;
        }
      } else {
        toast(PLAT() === 'mac' ? 'Add a regular key too, like ⌘ C.' : 'Add a regular key too, like Alt + C.', { kind: 'warn' });
      }
      renderRecorderSection(kind);
      renderNotes();
    },
    onCancel: () => { if (!E) return; E.rec = null; E.live = ''; E.mini?.setMood('awake'); renderRecorderSection(kind); },
  });
}
const stopRec = () => stopRecording();

function renderModes() {
  const box = $('#ed-modes');
  box.textContent = '';
  for (const [type, label, ic] of [['keys', 'Press keys', 'keyboard'], ['launch', 'Open app', 'launch'], ['text', 'Type text', 'text'], ['block', 'Block it', 'block']]) {
    box.append(h('button', { type: 'button', 'aria-pressed': String(E.d.action.type === type), html: `${icon(ic)}<span>${label}</span>`, onclick: () => setMode(type) }));
  }
}

function setMode(type) {
  const a = E.d.action;
  if (a.type === type) return;
  if (E.rec === 'keys') stopRec();
  if (a.type === 'keys') E.mem.keys = a.keys || '';
  if (a.type === 'launch') E.mem.launch = { target: a.target || '', args: a.args || '' };
  if (a.type === 'text') E.mem.text = a.text || '';
  if (type === 'keys') E.d.action = { type, keys: E.mem.keys };
  else if (type === 'launch') E.d.action = { type, ...E.mem.launch };
  else if (type === 'text') E.d.action = { type, text: E.mem.text };
  else E.d.action = { type: 'block' };
  sounds.pop();
  renderModes();
  renderPanel();
  renderSwapRow();
  renderNotes();
  $('#ed-name').placeholder = autoName(E.d);
}

function renderPanel() {
  const box = $('#ed-panel');
  box.textContent = '';
  const a = E.d.action;
  if (a.type === 'keys') {
    box.append(h('div', { class: 'panel step', id: 'ed-keys', style: { gap: '10px' } }));
    renderRecorderSection('keys');
  } else if (a.type === 'launch') {
    const panel = h('div', { class: 'panel box' });
    const picked = h('div', { class: 'picked' });
    const fillPicked = () => {
      picked.textContent = '';
      if (a.target) {
        picked.append(h('span', { class: 'appbig' }, appIcon(a.target, 'appbig-img')),
          h('span', { class: 'who' }, h('b', { text: appName(a.target) }), h('span', { text: a.target })));
      } else {
        picked.append(h('span', { class: 'appbig', html: icon('launch') }),
          h('span', { class: 'who' }, h('b', { text: 'Nothing picked yet' }), h('span', { text: 'Pick a running app, browse, or paste a link below.' })));
      }
      picked.append(h('button', { class: 'btn small', type: 'button', text: 'Browse…', onclick: async () => {
        const p = await invoke('pick_program').catch(() => null);
        if (p) { primeApps([{ exe: p.exe, name: p.name, path: p.path, icon: p.icon }]); setTarget(p.path); }
      } }));
    };
    const targetInput = h('input', { class: 'field', type: 'text', placeholder: 'C:\\Path\\app.exe, a folder, or https://…', value: a.target || '',
      oninput: (e) => { a.target = e.target.value; E.mem.launch.target = a.target; E.err.launch = null; fillPicked(); } });
    const setTarget = (t) => { a.target = t; E.mem.launch.target = t; E.err.launch = null; targetInput.value = t; fillPicked(); renderNotes(); $('#ed-name').placeholder = autoName(E.d); };
    fillPicked();
    const grid = h('div', { class: 'apps-grid' }, h('span', { class: 'hint', text: 'Looking around…' }));
    panel.append(picked, h('div', { class: 'hint', style: { fontWeight: '800' }, text: 'Running now' }), grid,
      h('label', { class: 'label' }, 'Program, file, folder or link', targetInput),
      h('label', { class: 'label' }, h('span', {}, 'Open with ', h('span', { class: 'opt', text: '(optional)' })),
        h('input', { class: 'field', type: 'text', placeholder: 'Extra arguments, e.g. a folder path', value: a.args || '', oninput: (e) => { a.args = e.target.value; E.mem.launch.args = a.args; } })));
    box.append(panel);
    const fill = (apps) => {
      grid.textContent = '';
      if (!apps.length) grid.append(h('span', { class: 'hint', text: 'No windows open right now. Use Browse instead.' }));
      for (const app of apps.slice(0, 12)) {
        grid.append(h('button', { class: 'app-tile', type: 'button', title: app.path, onclick: () => setTarget(app.path) },
          appIcon(app.path || app.exe, 'letter-or-img'), h('span', { class: 'nm', text: app.name || appName(app.exe) })));
      }
      grid.querySelectorAll('.letter-or-img').forEach((e) => { e.style.width = '26px'; e.style.height = '26px'; e.style.borderRadius = '7px'; });
    };
    if (E.apps) fill(E.apps);
    else invoke('running_apps').then((apps) => { primeApps(apps); if (E) { E.apps = apps; fill(apps); } }).catch(() => fill([]));
  } else if (a.type === 'text') {
    box.append(h('label', { class: 'label panel' }, 'Text to type',
      h('textarea', { class: 'field', rows: '3', maxlength: '4000', placeholder: 'Thanks! I’ll get back to you today.',
        oninput: (e) => { a.text = e.target.value; E.mem.text = a.text; E.err.text = null; } }, a.text || ''),
      h('span', { class: 'hint', text: 'Typed into whatever app is in front. Enter starts a new line.' })));
  } else {
    box.append(h('div', { class: 'note panel' }, h('span', { html: icon('block') }),
      h('span', { text: 'Nothing happens when you press it. Handy for keys you hit by accident, like Insert or F1.' })));
  }
  const errKey = a.type === 'keys' ? null : a.type === 'launch' ? 'launch' : a.type === 'text' ? 'text' : null;
  if (errKey && E.err[errKey]) box.append(h('div', { class: 'err', role: 'alert', text: E.err[errKey], style: { marginTop: '8px' } }));
}

function renderScope() {
  const box = $('#ed-scope');
  box.textContent = '';
  const s = E.d.scope;
  const seg = h('div', { class: 'seg soft', role: 'group', 'aria-label': 'Where', style: { alignSelf: 'flex-start', flexWrap: 'wrap' } });
  for (const [mode, label] of [['all', 'Everywhere'], ['only', 'Only in…'], ['except', 'Everywhere except…']]) {
    seg.append(h('button', { type: 'button', 'aria-pressed': String(s.mode === mode), text: label, onclick: () => {
      s.mode = mode;
      E.err.scope = null;
      renderScope();
      renderNotes();
      if (mode !== 'all' && !s.apps.length) setTimeout(() => $('#ed-addapp')?.click(), 60);
    } }));
  }
  box.append(seg);
  if (s.mode !== 'all') {
    const list = h('div', { class: 'applist' });
    for (const exe of s.apps) {
      list.append(h('span', { class: 'appchip' }, appIcon(exe), appName(exe),
        h('button', { class: 'x', type: 'button', 'aria-label': `Remove ${exe}`, html: icon('x'), onclick: () => { s.apps = s.apps.filter((x) => x !== exe); renderScope(); renderNotes(); } })));
    }
    list.append(h('button', { class: 'btn dashed', id: 'ed-addapp', type: 'button', html: `${icon('plus')}<span>Add app</span>`,
      onclick: (e) => appPicker(e.currentTarget, { title: 'Pick an app', exclude: s.apps, onPick: (app) => {
        if (!s.apps.includes(app.exe)) s.apps.push(app.exe);
        E.err.scope = null;
        renderScope();
        renderNotes();
      } }) }));
    box.append(list);
    box.append(h('span', { class: 'hint', text: s.mode === 'only' ? 'App-only shortcuts win over everywhere ones.' : 'Everywhere except these apps.' }));
  }
  if (E.err.scope) box.append(h('div', { class: 'err', role: 'alert', text: E.err.scope }));
}

function renderNotes() {
  if (!E) return;
  const box = $('#ed-notes');
  box.textContent = '';
  const { notes } = analyze(E.d);
  const seen = new Set();
  for (const [kind, text] of notes) {
    if (seen.has(text)) continue;
    seen.add(text);
    box.append(h('div', { class: `note${kind === 'warn' ? ' warn' : kind === 'bad' ? ' bad' : ''}`, role: 'note' },
      h('span', { html: icon(kind === 'info' ? 'info' : 'warn') }), h('span', { text })));
  }
}

async function saveEditor() {
  if (!E) return;
  if (E.rec) await stopRec();
  const { err } = analyze(E.d);
  E.err = err;
  if (Object.keys(err).length) {
    renderRecorderSection('trigger');
    if (E.d.action.type === 'keys') renderRecorderSection('keys');
    else renderPanel();
    renderScope();
    sounds.off();
    const first = $('#ed-body .err');
    first?.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'center' });
    return;
  }
  const d = clone(E.d);
  d.name = (d.name || '').trim();
  if (d.scope.mode === 'all') d.scope.apps = [];
  if (d.action.type === 'launch') { d.action.target = d.action.target.trim(); d.action.args = (d.action.args || '').trim(); }
  const isNew = E.isNew;
  const list = isNew ? [...rules(), d] : rules().map((r) => (r.id === d.id ? d : r));
  const btn = $('#ed-save').getBoundingClientRect();
  confetti(btn.left + btn.width / 2, btn.top);
  sounds.saved();
  markFresh(d.id);
  await closeEditor();
  await commit({ ...S.config, rules: list });
  toast(isNew ? `${cat()} learned ${K.plain(d.trigger)}!` : 'Saved. It works right away.');
  hero.react(1300);
  document.querySelector(`.card[data-id="${CSS.escape(d.id)}"]`)?.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'nearest' });
}

// ------------------------------------------------------------------ templates

const STARTER = 'builtin-starter';
const allTemplates = () => [...S.templates, ...(S.config.templates || [])];
/** "LCtrl+C" and "Ctrl+C" send the same keys (a plain Ctrl goes out as the left one). */
const sideless = (x) => String(x || '').split('+').map((t) => t.replace(/^L(Ctrl|Alt|Shift|Win)$/, '$1')).join('+');
const actKey = (a) => [a.type, sideless(a.keys), a.target || '', a.args || '', a.text || ''].join('\u0001');
const scopeKey = (s) => `${s?.mode || 'all'}:${[...(s?.apps || [])].sort().join(',')}`;
/** Same keys, same result, same apps (on/off and name don't matter). */
const sameRule = (a, b) => (a.trigger === b.trigger || K.overlaps(a.trigger, b.trigger)) && actKey(a.action) === actKey(b.action) && scopeKey(a.scope) === scopeKey(b.scope);
const inList = (x) => rules().some((r) => sameRule(r, x));

/** A template's shortcuts as they'd be added (swapped when Reverse is on). */
function tplRules(t) {
  return S.tplReversed.has(t.id) ? t.rules.map((r) => swapped(r) || r) : t.rules;
}

function templateCard(t) {
  const list = tplRules(t);
  const have = list.filter(inList).length;
  const missing = list.length - have;
  const rev = S.tplReversed.has(t.id);
  const open = S.tplOpen.has(t.id) || list.length <= 6;
  const shown = open ? list : list.slice(0, 5);
  const rows = h('div', { class: 'tpl-rules' }, ...shown.map((x) => {
    const has = inList(x);
    return h('div', { class: `tpl-rule${has ? ' has' : ''}` },
      h('span', { class: 'from' }, capsEl(x.trigger)),
      h('span', { class: 'mini-arrow', 'aria-hidden': 'true', text: '→' }),
      h('span', { class: 'to' }, x.action.type === 'keys' ? capsEl(x.action.keys, { to: true }) : h('span', { class: 'nm', text: autoName(x) })),
      h('span', { class: 'nm', title: ruleName(x), text: ruleName(x) }),
      h('span', { class: 'tick', title: has ? 'Already in your list' : '', html: has ? icon('check') : '' }));
  }));
  const more = list.length > 6
    ? h('button', { class: 'link', type: 'button', text: open ? 'Show fewer' : `Show all ${list.length}`, onclick: () => { open ? S.tplOpen.delete(t.id) : S.tplOpen.add(t.id); renderTemplates(); } })
    : null;
  const primary = missing === 0
    ? h('span', { class: 'in-list', html: `${icon('check')}<span>All in your list</span>` })
    : h('button', { class: 'btn strong small', type: 'button', html: `${icon('plus')}<span>${have ? `Add ${missing} missing` : list.length === 1 ? 'Add it' : `Add all ${list.length}`}</span>`, onclick: (e) => addTemplate(t, e.currentTarget) });
  const remove = have ? h('button', { class: 'btn ghost small', type: 'button', text: missing ? `Remove ${have}` : 'Remove from my list', onclick: () => removeTemplateRules(t) }) : null;
  const reverse = t.rules.some(canSwap)
    ? h('button', { class: `chipbtn${rev ? ' on' : ''}`, type: 'button', 'aria-pressed': String(rev),
        title: rev ? 'Showing it reversed: what you press and what it does trade places' : 'Flip the whole template: what you press and what it does trade places',
        onclick: () => { rev ? S.tplReversed.delete(t.id) : S.tplReversed.add(t.id); sounds.pop(); renderTemplates(); } },
      h('span', { html: icon('swap'), style: { display: 'inline-flex' } }), rev ? 'Reversed' : 'Reverse')
    : null;
  const badge = t.builtin ? h('span', { class: 'lock', title: 'Comes with Cat Key, so it can’t be changed or deleted.', html: `${icon('lock')}<span>Built-in</span>` }) : null;
  return h('article', { class: `tpl${t.id === STARTER ? ' starter' : ''}`, 'data-id': t.id },
    h('div', { class: 'tpl-head' },
      h('span', { class: 'tpl-ico', html: icon(t.id === STARTER ? 'paw' : t.builtin ? 'sparkle' : 'layers') }),
      h('div', { class: 'txt' },
        h('div', { class: 't1' }, h('span', { class: 'display', text: t.name }), badge),
        h('div', { class: 't2', text: t.description || `${t.rules.length} shortcut${t.rules.length === 1 ? '' : 's'}` })),
      t.builtin ? null : h('button', { class: 'more', type: 'button', 'aria-label': `More for ${t.name}`, html: icon('more'), onclick: (e) => templateMenu(e.currentTarget, t) })),
    rows, more,
    h('div', { class: 'tpl-foot' }, reverse,
      list.some((x) => x.action.type === 'keys') ? h('button', { class: 'chipbtn', type: 'button', title: 'Practise this set in Train yourself', onclick: () => {
        const p = selfPacks().find((x) => x.id === `tpl-${t.id}`);
        if (p) { setView('train'); startTrain(p); } else toast('This set has too few key shortcuts to train on.', { kind: 'warn' });
      } }, h('span', { html: icon('target'), style: { display: 'inline-flex' } }), 'Train') : null,
      h('span', { class: 'spacer' }), remove, primary));
}

function renderTemplates() {
  const box = $('#templates');
  box.textContent = '';
  box.append(h('div', { class: 'tpl-intro' },
    h('div', { class: 'txt' },
      h('h2', { class: 'display', text: 'Templates' }),
      h('p', { class: 'hint', text: `Ready-made sets of shortcuts. Add a whole set in one click, or flip it with Reverse first. The built-in ones are always here, ${cat()} won’t let them be deleted.` })),
    h('div', { class: 'lib-actions' },
      h('button', { class: 'btn strong', type: 'button', html: `${icon('wand')}<span>Make your own with AI</span>`, onclick: () => makeWithAI('remap') }),
      h('button', { class: 'btn', type: 'button', html: `${icon('layers')}<span>Save my shortcuts as a template</span>`, onclick: saveAsTemplate }))));
  box.append(youDontNeed());
  const grid = h('div', { class: 'tpl-grid' });
  for (const t of allTemplates()) grid.append(templateCard(t));
  box.append(grid);
}

async function addTemplate(t, btn) {
  const fresh = [];
  let clashes = 0;
  for (const x of tplRules(t)) {
    if (inList(x)) continue;
    const r = { ...clone(x), id: newId() };
    // Keys you already use stay yours: the template's copy comes in switched off.
    r.enabled = !clashFor(r, [...rules(), ...fresh]);
    if (!r.enabled) clashes++;
    fresh.push(r);
  }
  if (!fresh.length) return;
  if (btn) { const rc = btn.getBoundingClientRect(); confetti(rc.left + rc.width / 2, rc.top); }
  fresh.forEach((r) => markFresh(r.id));
  sounds.saved();
  await commit({ ...S.config, rules: [...rules(), ...fresh] });
  hero.react(1300);
  const n = fresh.length;
  const msg = `Added ${n} shortcut${n === 1 ? '' : 's'} from “${t.name}”.` +
    (clashes ? ` ${clashes} ${clashes === 1 ? 'is' : 'are'} switched off because you already use ${clashes === 1 ? 'its keys' : 'their keys'}.` : '');
  toast(msg, { kind: clashes ? 'warn' : 'ok', ms: 8000, action: { label: 'Undo', fn: () => commit({ ...S.config, rules: rules().filter((r) => !fresh.some((f) => f.id === r.id)) }) } });
}

function removeTemplateRules(t) {
  const list = tplRules(t);
  const before = rules();
  const keep = before.filter((r) => !list.some((x) => sameRule(r, x)));
  const n = before.length - keep.length;
  if (!n) return;
  sounds.off();
  commit({ ...S.config, rules: keep });
  toast(`Removed ${n} shortcut${n === 1 ? '' : 's'} from your list. “${t.name}” is still here.`, { ms: 8000, action: { label: 'Undo', fn: () => commit({ ...S.config, rules: before }) } });
}

const shortDate = () => new Date().toLocaleDateString(undefined, { month: 'short', day: 'numeric' });

async function saveAsTemplate() {
  const n = rules().length;
  if (!n) { toast('Add a shortcut first, then save your list as a template.', { kind: 'warn' }); return; }
  const name = await promptBox('Save as a template', `Keeps a copy of your ${n} shortcut${n === 1 ? '' : 's'}, so you can bring them back in one click.`, `My shortcuts · ${shortDate()}`, 'Save template');
  if (!name) return;
  const t = { id: `t${newId().slice(1)}`, name, description: `${n} shortcut${n === 1 ? '' : 's'}, saved ${shortDate()}.`, builtin: false, rules: rules().map((r) => ({ ...clone(r), id: newId() })) };
  sounds.saved();
  await commit({ ...S.config, templates: [...(S.config.templates || []), t] });
  toast(`Saved “${name}”. It’s in Templates.`);
}

function templateMenu(anchor, t) {
  showPop(anchor, h('div', { class: 'pop', role: 'menu' },
    menuItem('edit', 'Rename', () => renameTemplate(t)),
    menuItem('refresh', 'Replace with my current shortcuts', () => refillTemplate(t)),
    h('div', { class: 'sep' }),
    menuItem('trash', 'Delete template', () => deleteTemplate(t), 'danger')));
}

const updateTemplate = (id, change) => commit({ ...S.config, templates: (S.config.templates || []).map((x) => (x.id === id ? change(x) : x)) });

async function renameTemplate(t) {
  const name = await promptBox('Rename template', 'Give it a name you’ll recognise.', t.name, 'Rename');
  if (name && name !== t.name) updateTemplate(t.id, (x) => ({ ...x, name }));
}

async function refillTemplate(t) {
  const n = rules().length;
  if (!(await confirmBox(`Replace “${t.name}”?`, `It will hold your current ${n} shortcut${n === 1 ? '' : 's'} instead of the ones it has now.`, 'Replace'))) return;
  updateTemplate(t.id, (x) => ({ ...x, description: `${n} shortcut${n === 1 ? '' : 's'}, saved ${shortDate()}.`, rules: rules().map((r) => ({ ...clone(r), id: newId() })) }));
  toast(`“${t.name}” now holds your current shortcuts.`);
}

function deleteTemplate(t) {
  const before = S.config.templates || [];
  commit({ ...S.config, templates: before.filter((x) => x.id !== t.id) });
  toast(`Deleted the template “${t.name}”. Your shortcuts weren’t touched.`, { ms: 7000, action: { label: 'Undo', fn: () => commit({ ...S.config, templates: before }) } });
}

// ------------------------------------------------------------------ popovers, menus, dialogs, toasts

let POP = null;
function closePop() {
  if (!POP) return false;
  POP.remove();
  POP = null;
  document.removeEventListener('pointerdown', onOutside, true);
  return true;
}
function onOutside(e) {
  if (POP && !POP.contains(e.target)) closePop();
}
function showPop(anchor, pop) {
  closePop();
  POP = pop;
  layer().append(pop);
  const r = anchor.getBoundingClientRect();
  const w = pop.offsetWidth;
  const left = Math.max(8, Math.min(window.innerWidth - w - 8, r.right - w));
  let top = r.bottom + 6;
  if (top + pop.offsetHeight > window.innerHeight - 8) top = Math.max(56, r.top - pop.offsetHeight - 6);
  pop.style.left = `${left}px`;
  pop.style.top = `${top}px`;
  setTimeout(() => document.addEventListener('pointerdown', onOutside, true), 0);
  pop.querySelector('button, input')?.focus();
}

const menuItem = (ic, text, fn, cls = '') => h('button', { class: `item ${cls}`, type: 'button', role: 'menuitem', onclick: () => { closePop(); fn(); } }, h('span', { html: icon(ic) }), h('span', { class: 'nm', text }));

function ruleMenu(anchor, r) {
  const item = menuItem;
  showPop(anchor, h('div', { class: 'pop', role: 'menu' },
    item('edit', 'Edit', () => openEditor(r)),
    canSwap(r) ? item('swap', `Swap: ${K.plain(r.action.keys)} does ${K.plain(r.trigger)}`, () => swapRule(r)) : null,
    item('copy', 'Duplicate', () => duplicateRule(r)),
    item(r.enabled ? 'moon' : 'paw', r.enabled ? 'Turn off' : 'Turn on', () => toggleRule(r.id)),
    h('div', { class: 'sep' }),
    item('trash', 'Delete', () => deleteRule(r.id), 'danger')));
}

async function appPicker(anchor, { title, onPick, exclude = [] }) {
  const pop = h('div', { class: 'pop', role: 'menu', style: { width: '320px' } }, h('div', { class: 'head', text: title }));
  const list = h('div', {}, h('div', { class: 'head', text: 'Looking for open apps…' }));
  const input = h('input', { class: 'field', type: 'text', placeholder: 'or type a name, e.g. chrome.exe', 'aria-label': 'App name' });
  const add = () => { const v = input.value.trim(); if (!v) return; let exe = v.split(/[\\/]/).pop().toLowerCase(); if (!exe.includes('.')) exe += '.exe'; closePop(); onPick({ exe }); };
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') add(); });
  pop.append(list, h('div', { class: 'sep' }),
    h('button', { class: 'item', type: 'button', onclick: async () => {
      closePop();
      const p = await invoke('pick_program').catch(() => null);
      if (p) { primeApps([{ exe: p.exe, name: p.name, path: p.path, icon: p.icon }]); onPick(p); }
    } }, h('span', { html: icon('folder') }), h('span', { class: 'nm', text: 'Browse for an app…' })),
    h('div', { class: 'type' }, input, h('button', { class: 'btn small', type: 'button', text: 'Add', onclick: add })));
  showPop(anchor, pop);
  input.focus();
  const apps = await invoke('running_apps').catch(() => []);
  primeApps(apps);
  if (POP !== pop) return;
  list.textContent = '';
  const usable = apps.filter((a) => !exclude.includes(a.exe));
  if (!usable.length) list.append(h('div', { class: 'head', text: 'No other open apps right now.' }));
  for (const a of usable) {
    list.append(h('button', { class: 'item', type: 'button', role: 'menuitem', title: a.path, onclick: () => { closePop(); onPick(a); } },
      appIcon(a.path || a.exe), h('span', { class: 'nm', text: a.name || appName(a.exe) }), h('span', { class: 'ex', text: a.exe })));
  }
}

let DIALOG = null;
function confirmBox(title, text, okLabel = 'OK', danger = false) {
  return new Promise((resolve) => {
    const scrim = h('div', { class: 'dialog-scrim' });
    const box = h('div', { class: 'dialog', role: 'alertdialog', 'aria-modal': 'true' },
      h('h2', { text: title }), h('p', { text }),
      h('div', { class: 'row' },
        h('button', { class: 'btn ghost', text: 'Cancel', onclick: () => done(false) }),
        h('button', { class: `btn ${danger ? '' : 'primary'}`, style: danger ? { background: 'var(--bad)', color: '#fff', borderColor: 'transparent' } : null, text: okLabel, onclick: () => done(true) })));
    const done = (v) => { scrim.remove(); box.remove(); DIALOG = null; resolve(v); };
    DIALOG = done;
    scrim.onclick = () => done(false);
    layer().append(scrim, box);
    box.querySelector('.btn.ghost').focus();
  });
}

function promptBox(title, text, value = '', okLabel = 'Save') {
  return new Promise((resolve) => {
    const scrim = h('div', { class: 'dialog-scrim' });
    const input = h('input', { class: 'field', type: 'text', maxlength: '40', value, 'aria-label': title, style: { width: '100%', marginBottom: '18px' } });
    const done = (v) => { scrim.remove(); box.remove(); DIALOG = null; resolve(v); };
    const ok = () => done(input.value.trim() || null);
    const box = h('div', { class: 'dialog', role: 'dialog', 'aria-modal': 'true' },
      h('h2', { text: title }), h('p', { text }), input,
      h('div', { class: 'row' },
        h('button', { class: 'btn ghost', type: 'button', text: 'Cancel', onclick: () => done(null) }),
        h('button', { class: 'btn primary', type: 'button', text: okLabel, onclick: ok })));
    input.addEventListener('keydown', (e) => { if (e.key === 'Enter') ok(); });
    DIALOG = (yes) => (yes ? ok() : done(null));
    scrim.onclick = () => done(null);
    layer().append(scrim, box);
    input.focus();
    input.select();
  });
}

function toast(text, { kind = 'ok', action = null, ms = 3800 } = {}) {
  const box = $('#toasts');
  const t = h('div', { class: `toast ${kind}`, role: 'status' },
    h('span', { html: icon(kind === 'ok' ? 'paw' : kind === 'warn' ? 'info' : 'warn'), style: { display: 'inline-flex' } }),
    h('span', { text }));
  const dismiss = () => { if (!t.isConnected) return; t.classList.add('out'); setTimeout(() => t.remove(), 260); };
  if (action) t.append(h('button', { class: 'btn small', type: 'button', text: action.label, onclick: () => { action.fn(); dismiss(); } }));
  box.append(t);
  while (box.children.length > 3) box.firstElementChild.remove();
  setTimeout(dismiss, ms);
}

function confetti(x, y) {
  if (reduced()) return;
  for (let i = 0; i < 7; i++) {
    const p = h('span', { class: 'confetti', html: icon('paw') });
    p.style.left = `${x - 8}px`;
    p.style.top = `${y - 8}px`;
    layer().append(p);
    const ang = (-90 + (i - 3) * 22 + (Math.random() * 14 - 7)) * (Math.PI / 180);
    const dist = 60 + Math.random() * 50;
    p.animate([
      { transform: 'translate(0,0) scale(.6) rotate(0deg)', opacity: 1 },
      { transform: `translate(${Math.cos(ang) * dist}px, ${Math.sin(ang) * dist}px) scale(1) rotate(${Math.random() * 120 - 60}deg)`, opacity: 0 },
    ], { duration: 750 + Math.random() * 250, easing: 'cubic-bezier(.22,1,.36,1)' }).onfinish = () => p.remove();
  }
}

// ------------------------------------------------------------------ PowerToys import

async function importPowerToys() {
  try {
    const res = await invoke('import_powertoys');
    S.config = res.config;
    renderAll();
    if (res.added) toast(`Imported ${res.added} new shortcut${res.added === 1 ? '' : 's'} from PowerToys. Now turn its Keyboard Manager off.`, { ms: 7000 });
    else toast(`All ${res.found} PowerToys shortcuts are already here. Now turn its Keyboard Manager off.`, { ms: 7000 });
  } catch (e) {
    toast(String(e), { kind: 'warn', ms: 6000 });
  }
}

// ------------------------------------------------------------------ settings

let SETTINGS_OPEN = false;
let settingMinis = [];

function openSettings() {
  if (E) closeEditor();
  SETTINGS_OPEN = true;
  const d = $('#settings');
  d.hidden = false;
  renderSettings();
  $('#scrim').classList.add('on');
  requestAnimationFrame(() => d.classList.add('on'));
}

async function closeSettings() {
  if (!SETTINGS_OPEN) return;
  if (REC?.kind === 'pause') await stopRecording();
  SETTINGS_OPEN = false;
  const d = $('#settings');
  d.classList.remove('on');
  $('#scrim').classList.remove('on');
  setTimeout(() => { if (!SETTINGS_OPEN) { settingMinis.forEach((m) => m.destroy()); settingMinis = []; d.hidden = true; d.textContent = ''; } }, 420);
}

function settingRow(title, desc, control) {
  return h('div', { class: 'setting' }, h('div', { class: 'txt' }, h('div', { class: 't1', text: title }), h('div', { class: 't2', text: desc })), control);
}
function switchEl(on, label, fn) {
  return h('button', { class: 'switch', type: 'button', role: 'switch', 'aria-checked': String(!!on), 'aria-label': label, onclick: fn }, h('span', { class: 'knob', html: icon('paw') }));
}

let pauseListening = false;

function renderSettings() {
  const d = $('#settings');
  const scrollTop = $('.drawer-body', d)?.scrollTop || 0;
  settingMinis.forEach((m) => m.destroy());
  settingMinis = [];
  d.textContent = '';
  const s = S.config.settings;

  const me = myCat();
  const holder = h('span', {});
  const skins = h('div', { class: 'cat-now' }, holder,
    h('div', { class: 'txt' }, h('b', { text: me.full || me.name }), h('span', { class: 'hint', text: `${rankFor(xpBreakdown().total).name} · ${xpBreakdown().total.toLocaleString()} XP` })),
    h('button', { class: 'btn small', type: 'button', html: `${icon('star')}<span>Change cat or outfit</span>`, onclick: () => { closeSettings(); setView('warrior'); } }));
  settingMinis.push(new Mascot(holder, { size: 74, look: myLook() }));
  const purrVol = h('input', { type: 'range', min: '0', max: '100', step: '5', value: String(s.purrVolume ?? 45), 'aria-label': 'Purr volume', class: 'range',
    onchange: (e) => commitSettings({ purrVolume: Number(e.target.value) }) });
  const theme = h('div', { class: 'seg soft', role: 'group', 'aria-label': 'Theme' },
    ...[['auto', PLAT() === 'mac' ? 'Match macOS' : 'Match Windows'], ['light', 'Light'], ['dark', 'Dark']].map(([v, l]) => h('button', { type: 'button', 'aria-pressed': String(s.theme === v), text: l, onclick: () => commitSettings({ theme: v }) })));

  const pauseCtl = h('div', { style: { display: 'flex', gap: '6px', alignItems: 'center' } });
  if (pauseListening) pauseCtl.append(h('button', { class: 'btn small dashed', type: 'button', onclick: () => stopRecording() }, h('span', { class: 'dot', style: { width: '9px', height: '9px', borderRadius: '5px', background: 'var(--ac)', animation: 'blink 1s infinite' } }), 'Press keys…'));
  else if (s.pauseHotkey) pauseCtl.append(capsEl(s.pauseHotkey), h('button', { class: 'tbtn', 'aria-label': 'Clear pause hotkey', html: icon('x'), onclick: () => commitSettings({ pauseHotkey: null }) }));
  else pauseCtl.append(h('button', { class: 'btn small dashed', type: 'button', onclick: recordPause }, h('span', { style: { width: '9px', height: '9px', borderRadius: '5px', background: 'var(--ac)', display: 'inline-block' } }), 'Record'));

  const nap = h('div', { class: 'applist' });
  for (const exe of s.excludedApps) {
    nap.append(h('span', { class: 'appchip' }, appIcon(exe), appName(exe),
      h('button', { class: 'x', type: 'button', 'aria-label': `Remove ${exe}`, html: icon('x'), onclick: () => commitSettings({ excludedApps: s.excludedApps.filter((x) => x !== exe) }) })));
  }
  nap.append(h('button', { class: 'btn dashed', type: 'button', html: `${icon('plus')}<span>Add app</span>`,
    onclick: (e) => appPicker(e.currentTarget, { title: `Where should ${cat()} nap?`, exclude: s.excludedApps, onPick: (app) => commitSettings({ excludedApps: [...s.excludedApps, app.exe] }) }) }));

  const mac = PLAT() === 'mac';
  const adminDesc = S.status.admin
    ? 'On. Cat Key is running as administrator, so shortcuts also work in Task Manager and admin terminals.'
    : 'Runs as administrator so shortcuts also work in Task Manager and admin terminals. Windows asks you once.';

  d.append(
    h('div', { class: 'drawer-head' }, h('h1', { id: 'st-title', text: 'Settings' }), h('button', { class: 'tbtn', 'aria-label': 'Close settings', html: icon('x'), onclick: closeSettings })),
    h('div', { class: 'drawer-body' },
      h('section', { style: { gap: '0' } }, h('h2', { text: 'Quick settings', style: { marginBottom: '4px' } }),
        mac
          ? settingRow('Swap ⌘ and Control', '⌘ works as Control and Control as ⌘, in every app, like Ctrl and Alt sit on a Windows keyboard. Off unless you turn it on.',
            switchEl(!!s.swapAltCtrl, 'Swap Command and Control', () => commitSettings({ swapAltCtrl: !s.swapAltCtrl }).then(() => toast(!s.swapAltCtrl ? 'Swapped: ⌘ is Control and Control is ⌘ now.' : '⌘ and Control are back where they were.'))))
          : settingRow('Swap Alt and Ctrl', 'Alt works as Ctrl and Ctrl as Alt, in every app, like the ⌘ and Control keys on a Mac. Off unless you turn it on.',
            switchEl(!!s.swapAltCtrl, 'Swap Alt and Ctrl', () => commitSettings({ swapAltCtrl: !s.swapAltCtrl }).then(() => toast(!s.swapAltCtrl ? 'Swapped: Alt is Ctrl and Ctrl is Alt now.' : 'Alt and Ctrl are back where they were.'))))),
      h('section', {}, h('div', { class: 'row-between' }, h('h2', { text: 'Checkup' }),
        h('button', { class: 'link', type: 'button', text: 'Check again', onclick: () => { refreshStatusNow(); toast(`${cat()} checked everything again.`); } })),
        checkupEl(checkupRows())),
      h('section', {}, h('h2', { text: 'Try a shortcut' }), testPadEl(), keyLogEl()),
      h('section', {}, h('h2', { text: 'Your cat' }), skins,
        h('div', { class: 'row-between', style: { fontWeight: '800' } }, 'Theme', theme)),
      h('section', { style: { gap: '0' } }, h('h2', { text: 'How it behaves', style: { marginBottom: '4px' } }),
        mac
          ? settingRow('Open at login', 'Wakes up quietly in the menu bar when you log in.', switchEl(s.launchAtStartup, 'Open at login', () => commitSettings({ launchAtStartup: !s.launchAtStartup }).then(() => setTimeout(refreshStatusNow, 900))))
          : settingRow('Start with Windows', 'Wakes up quietly in the tray when you sign in.', switchEl(s.launchAtStartup, 'Start with Windows', () => commitSettings({ launchAtStartup: !s.launchAtStartup }).then(() => setTimeout(refreshStatusNow, 900)))),
        mac ? null : settingRow('Work in admin apps', adminDesc, switchEl(s.adminMode, 'Work in admin apps', () => setAdmin(!s.adminMode))),
        settingRow('Cat sounds', 'A soft paw tap on every click, a little mew now and then, and a purr when you pet the cat.', switchEl(s.sounds, 'Cat sounds', () => { commitSettings({ sounds: !s.sounds }); setSoundsEnabled(!s.sounds); sounds.on(); })),
        settingRow('Purr volume', 'For the background purr (the paw button at the top, or the tray menu).', purrVol),
        settingRow('Pause hotkey', 'One shortcut that pauses every remap. Handy for games.', pauseCtl)),
      mac ? null : h('section', { style: { gap: '0' } }, h('h2', { text: 'Shortcut coach', style: { marginBottom: '4px' } }),
        settingRow('Tips when you do it the long way', `When you click something that has a shortcut, ${cat()} shows its keys in a small note that fades away. Nothing is recorded: only the name of what you clicked is read, the way screen readers do.`,
          switchEl(s.coach, 'Shortcut coach', () => commitSettings({ coach: !s.coach }))),
        settingRow('Also in Chrome-based apps', 'Chrome, Edge, VS Code, Slack and Discord do a little extra work each time they’re asked what’s under the pointer, so they’re left out unless you want them.',
          switchEl(s.coachChromium, 'Coach in Chrome-based apps', () => commitSettings({ coachChromium: !s.coachChromium }))),
        h('div', { class: 'row-between', style: { paddingTop: '10px' } },
          h('button', { class: 'btn small', type: 'button', html: `${icon('sparkle')}<span>Show me a tip</span>`, onclick: () => invoke('coach_demo').catch((e) => toast(String(e), { kind: 'warn' })) }),
          (s.coachMuted || []).length ? h('button', { class: 'link', type: 'button', text: `Turn ${s.coachMuted.length} muted tip${s.coachMuted.length === 1 ? '' : 's'} back on`, onclick: () => commitSettings({ coachMuted: [] }) }) : null)),
      h('section', {}, h('h2', { text: 'Nap zones' }),
        h('p', { class: 'hint', style: { margin: '0', fontSize: '13px' }, text: `${cat()} naps while these apps are in front, so every key works as normal there.` }), nap),
      h('section', {}, h('h2', { text: 'Your shortcuts' }),
        h('div', { class: 'grid2' },
          mac ? null : h('button', { class: 'btn', type: 'button', html: `${icon('down')}<span>Import from PowerToys</span>`, onclick: importPowerToys }),
          h('button', { class: 'btn', type: 'button', html: `${icon('folder')}<span>Import…</span>`, title: 'A Cat Key setup, a pack, an AI answer or a PowerToys file', onclick: importFile }),
          h('button', { class: 'btn', type: 'button', html: `${icon('up')}<span>Export my setup…</span>`, title: 'Everything: shortcuts, templates, saved packs and settings. Works on a Mac too.', onclick: exportProfile }),
          h('button', { class: 'btn', type: 'button', html: `${icon('refresh')}<span>Restore defaults</span>`, onclick: async () => {
            const n = S.templates.find((t) => t.id === STARTER)?.rules.length || 12;
            if (!(await confirmBox('Restore the starter shortcuts?', `This replaces your shortcut list with the starter set (${n} Alt shortcuts). Settings and templates stay as they are.`, 'Restore', true))) return;
            S.config = await invoke('restore_defaults');
            renderAll();
            toast('Starter shortcuts are back.');
          } })),
        h('p', { class: 'hint', style: { margin: '2px 0 0' } }, 'Moving to another PC or a Mac? “Export my setup” makes one file with your shortcuts, templates, packs and settings.')),
      dataSection()),
    h('div', { class: 'drawer-foot' },
      h('span', {}, `Cat Key ${S.status.version || ''} · `, h('button', { class: 'link', type: 'button', text: 'About', onclick: openAbout })),
      h('button', { class: 'btn small', type: 'button', text: 'Quit Cat Key', onclick: async () => {
        if (await confirmBox('Quit Cat Key?', 'Your shortcuts stop working until Cat Key starts again.', 'Quit', true)) invoke('quit_app');
      } })));
  const body = $('.drawer-body', d);
  if (body) body.scrollTop = scrollTop;
}

const ME = { name: 'Masterfully', github: 'https://github.com/MasterfullyClaw', x: 'https://x.com/MasterfullyClaw' };

/** Open the About page from anywhere (the top bar, Settings) and bring it into view. */
function openAbout() {
  if (SETTINGS_OPEN) closeSettings();
  setView('about');
  requestAnimationFrame(() => $('#tabs')?.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'start' }));
}

/** About: who makes Cat Key and where to find them. Its own page, opened from the top bar. */
let ABOUT_CAT = null;
function renderAbout() {
  const box = $('#about-page');
  if (!box) return;
  ABOUT_CAT?.destroy();
  box.textContent = '';
  const go = (url) => () => invoke('open_link', { url }).catch((e) => toast(String(e), { kind: 'warn' }));
  const link = (ic, title, sub, url) => h('button', { class: 'about-link', type: 'button', title: url.replace('https://', ''), onclick: go(url) },
    h('span', { class: 'ic', html: icon(ic) }), h('span', { class: 'txt' }, h('b', { text: title }), h('span', { text: sub })));
  const holder = h('span', { class: 'about-cat' });
  box.append(
    h('div', { class: 'about-hero' }, holder,
      h('div', { class: 'txt' },
        h('h2', { class: 'display', text: 'Cat Key' }),
        h('span', { class: 'about-ver', text: `Version ${S.status.version || ''}`.trim() }),
        h('p', { class: 'hint', text: 'The all-in-one shortcut manager. Every shortcut of every app in one place, a coach that catches the long way, and a cat that levels up with you. Free.' }),
        h('span', { class: 'about-by' }, 'Built by ', h('b', { text: ME.name })))),
    h('section', { class: 'about-block' }, h('h3', { text: `Made by ${ME.name}` }),
      h('div', { class: 'about-links' },
        link('link', 'GitHub', '@MasterfullyClaw', ME.github),
        link('link', 'X (Twitter)', '@MasterfullyClaw', ME.x))),
    h('section', { class: 'about-block' }, h('h3', { text: 'Cat Key' }),
      h('div', { class: 'about-links' },
        link('star', 'Cat Key on GitHub', 'Code, downloads and the store', `https://github.com/${REPO}`),
        link('sparkle', 'What’s new', 'Every version and what changed', `https://github.com/${REPO}/releases`),
        link('up', 'Share a pack', 'Post your shortcuts to the store', `https://github.com/${REPO}/issues/new?template=share-a-pack.yml`),
        link('info', 'Report a problem', 'Opens a new issue on GitHub', `https://github.com/${REPO}/issues/new`))),
    h('section', { class: 'about-block' }, h('h3', { text: 'Community' }),
      h('p', { class: 'hint about-soon', text: 'A Discord server, Cat Key’s own website and its social pages are on the way. They’ll show up right here.' })));
  ABOUT_CAT = new Mascot(holder, { size: 104, look: myLook() });
}

function recordPause() {
  pauseListening = true;
  renderSettings();
  startRecording({
    kind: 'pause',
    onUpdate: () => {},
    onDone: (sc) => {
      pauseListening = false;
      if (!sc || !K.hasMods(sc)) {
        toast('Use at least one of Ctrl, Alt, Shift or Win with a key.', { kind: 'warn' });
        renderSettings();
        return;
      }
      commitSettings({ pauseHotkey: sc });
      toast(`${K.plain(sc)} now pauses and wakes ${cat()}.`);
    },
    onCancel: () => { pauseListening = false; if (SETTINGS_OPEN) renderSettings(); },
  });
}

async function setAdmin(on) {
  if (on && !S.status.admin) {
    toast('Windows will ask for permission. Cat Key restarts as administrator.', { ms: 5000 });
  }
  try {
    const res = await invoke('set_admin_mode', { enabled: on });
    if (res === 'saved') {
      S.config = { ...S.config, settings: { ...S.config.settings, adminMode: on } };
      if (SETTINGS_OPEN) renderSettings();
      renderBanners();
      if (!on && S.status.admin) {
        toast('Admin mode off. It takes effect when Cat Key restarts.', { ms: 8000, action: { label: 'Restart now', fn: () => invoke('restart_normal') } });
      }
      setTimeout(refreshStatusNow, 900);
    }
  } catch (e) {
    const msg = String(e);
    toast(msg === 'cancelled' ? 'No problem, Cat Key stays in normal mode.' : `Couldn’t switch: ${msg}`, { kind: msg === 'cancelled' ? 'ok' : 'err' });
    if (SETTINGS_OPEN) renderSettings();
  }
}

boot().catch((e) => {
  document.body.append(h('pre', { style: { color: '#fff', padding: '20px', whiteSpace: 'pre-wrap' }, text: `Cat Key UI failed to start:\n${e?.stack || e}` }));
  invoke('ui_ready').catch(() => {});
});

// ------------------------------------------------------------------ library: ready-made packs (apps, switching OS, yours, community)

const PLAT = () => (S.status.platform === 'mac' ? 'mac' : 'windows');
const LIB_CATS = [
  ['all', 'Everything'], ['switch', 'Switching OS'], ['system', 'System'], ['video', 'Video & motion'], ['design', 'Design & photo'],
  ['3d', '3D'], ['code', 'Code'], ['browser', 'Browsers'], ['work', 'Work & notes'], ['mine', 'Yours'], ['community', 'Community'],
];
const L = { data: null, loading: null, cat: 'all', q: '', community: null, communityState: 'idle' };
const REPO = 'MasterfullyClaw/CatKey';
const COMMUNITY_INDEX = `https://raw.githubusercontent.com/${REPO}/main/library/community/index.json`;
const SHARE_PACK_URL = `https://github.com/${REPO}/issues/new?template=share-a-pack.yml`;
/** Packs that only make sense on one system: they don't travel to the other one. */
const SWITCHERS = new Set(['mac-on-windows', 'windows-on-mac']);

function libData() {
  if (L.data) return Promise.resolve(L.data);
  L.loading ||= import('./library/data.js').then((m) => {
    L.data = { APPS: m.APPS, PACKS: m.PACKS };
    for (const a of Object.values(m.APPS)) for (const exe of a.windows || []) if (!appNames.has(exe)) appNames.set(exe, a.name.replace(/ \(.*\)$/, ''));
    return L.data;
  });
  return L.loading;
}

/** "Mod+Shift+T" -> "Ctrl+Shift+T" here (Win+Shift+T, i.e. ⌘⇧T, on a Mac). */
function combo(str, plat = PLAT()) {
  if (!str) return '';
  const map = { Mod: plat === 'mac' ? 'Win' : 'Ctrl', Opt: 'Alt', Option: 'Alt', Cmd: 'Win', Command: 'Win', Control: 'Ctrl' };
  return String(str).split('+').map((t) => map[t] || t).join('+');
}
const packFits = (p, plat = PLAT()) => !p.platform || p.platform === 'both' || p.platform === plat;
const itemFits = (it, plat = PLAT()) => !it.platform || it.platform === 'both' || it.platform === plat;
const isRemap = (p) => p.kind !== 'cheatsheet';

function appIds(ref) {
  const plat = PLAT();
  const apps = L.data?.APPS || {};
  const out = [];
  for (const x of [].concat(ref || [])) {
    if (typeof x !== 'string' || !x.trim()) continue;
    if (x.startsWith('app:')) out.push(...(apps[x.slice(4)]?.[plat] || []));
    else out.push(x.trim().toLowerCase());
  }
  return [...new Set(out)];
}
function packApps(p) {
  if (p.apps && typeof p.apps === 'object' && !Array.isArray(p.apps)) return appIds(p.apps[PLAT()] || []);
  if (p.app && L.data?.APPS[p.app]) return appIds(`app:${p.app}`);
  return [];
}
function packAppName(p) {
  return (p.app && L.data?.APPS[p.app]?.name) || (typeof p.app === 'string' ? p.app : '') || '';
}
function itemScope(p, it) {
  if (it.only) return { mode: 'only', apps: appIds(it.only) };
  if (it.except) return { mode: 'except', apps: appIds(it.except) };
  const apps = packApps(p);
  return apps.length ? { mode: 'only', apps } : { mode: 'all', apps: [] };
}
function packItems(p) {
  const out = [];
  (p.sections || []).forEach((s, si) => (s.items || []).forEach((it, ii) => { if (itemFits(it)) out.push({ s, si, it, ii, key: `${si}:${ii}` }); }));
  return out;
}
function itemRule(p, it) {
  let action;
  if (it.does) action = { type: 'keys', keys: combo(it.does) };
  else if (it.launch) action = { type: 'launch', target: it.launch, args: it.args || '' };
  else if (it.text != null) action = { type: 'text', text: String(it.text) };
  else action = { type: 'block' };
  return { id: newId(), enabled: true, name: it.name || '', trigger: combo(it.press), action, scope: itemScope(p, it), source: p.id || '' };
}
const itemKeys = (it) => combo(PLAT() === 'mac' && it.mac ? it.mac : it.keys);
/** Rules from this pack already in the list. */
function packHave(p) {
  if (!isRemap(p)) return [];
  const list = packItems(p).map((x) => itemRule(p, x.it));
  return rules().filter((r) => (p.id && r.source === p.id) || list.some((x) => sameRule(r, x)));
}

function libPacks() {
  if (!L.data) return [];
  const mine = (S.config.packs || []).map((p) => ({ ...p, _origin: 'mine' }));
  const built = L.data.PACKS.map((p) => ({ ...p, _origin: 'builtin' }));
  const comm = (L.community || []).map((p) => ({ ...p, _origin: 'community' }));
  return [...mine, ...built, ...comm].filter((p) => packFits(p));
}
function libMatches(p, q) {
  if (!q) return true;
  const hay = [p.name, p.description, packAppName(p), ...(p.tags || []), ...packItems(p).map((x) => x.it.name)].join(' ').toLowerCase();
  return q.split(/\s+/).every((w) => hay.includes(w));
}

function libTile(p, cls = 'lib-ico') {
  const exe = packApps(p)[0];
  if (exe && PLAT() === 'windows' && p.category !== 'switch') return appIcon(exe, cls);
  const ic = p._origin === 'self' ? 'keyboard' : p._origin === 'template' ? 'layers' : p._origin === 'coach' ? 'target'
    : p.category === 'switch' ? 'swap' : p._origin === 'mine' ? 'paw' : p.kind === 'cheatsheet' ? 'keyboard' : 'sparkle';
  return h('span', { class: `${cls} glyph`, html: icon(ic) });
}

function libCard(p) {
  const items = p._summary ? [] : packItems(p);
  const favs = items.filter((x) => x.it.fav).length;
  const have = p._summary ? 0 : packHave(p).length;
  const n = p._summary ? p.count || 0 : items.length;
  const meta = [`${n} shortcut${n === 1 ? '' : 's'}`];
  if (favs) meta.push(`${favs} pro pick${favs === 1 ? '' : 's'}`);
  const badge = (t, cls = '') => h('span', { class: `lib-badge ${cls}`, text: t });
  return h('article', { class: `lib-card${have ? ' has' : ''}`, tabindex: '0', role: 'button', 'aria-label': `Open ${p.name}`,
    onclick: () => openPackFrom(p), onkeydown: (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); openPackFrom(p); } } },
    h('div', { class: 'lib-top' }, libTile(p),
      h('div', { class: 'lib-badges' },
        badge(isRemap(p) ? 'Remap' : 'Cheat sheet', isRemap(p) ? 'remap' : ''),
        p._origin === 'mine' ? badge('Yours', 'mine') : p._origin === 'community' ? badge('Community', 'comm') : badge('Comes with Cat Key', 'builtin'),
        p.platform === 'mac' ? badge('Mac') : p.platform === 'windows' ? badge('Windows') : null)),
    h('div', { class: 'lib-name display', text: p.name }),
    h('div', { class: 'lib-desc', text: p.description || '' }),
    h('div', { class: 'lib-meta' }, h('span', { text: meta.join(' · ') }),
      have ? h('span', { class: 'lib-have', html: `${icon('check')}<span>${have} in your list</span>` }) : null),
    p._origin === 'community' ? h('div', { class: 'lib-by' },
      h('span', { text: p.author ? `by @${p.author}` : 'shared by the community' }),
      h('button', { class: 'heart', type: 'button', title: 'Love it on GitHub (opens the pack’s page)', 'aria-label': `Love ${p.name}`,
        onclick: (e) => { e.stopPropagation(); if (p.issue) invoke('open_link', { url: `https://github.com/${REPO}/issues/${p.issue}` }).catch(() => {}); } },
        h('span', { text: '♥' }), h('b', { text: String(p.likes || 0) }))) : null);
}

function renderLibrary() {
  const box = $('#library');
  if (!box) return;
  if (!L.data) {
    box.textContent = '';
    box.append(h('p', { class: 'hint', text: 'Opening the library…' }));
    libData().then(() => { renderLibrary(); renderTabs(); }).catch((e) => toast(`Couldn’t open the library: ${e}`, { kind: 'err' }));
    return;
  }
  const keepFocus = document.activeElement?.id === 'lib-search';
  box.textContent = '';
  box.append(h('div', { class: 'tpl-intro' },
    h('div', { class: 'txt' },
      h('h2', { class: 'display', text: 'Library' }),
      h('p', { class: 'hint', text: `Ready-made shortcuts for the apps you use and for switching between Mac and Windows. Open one, tick what you want, add it. ${cat()} keeps every app’s own shortcuts as they are, and only adds yours on top.` })),
    h('div', { class: 'lib-actions' },
      h('button', { class: 'btn strong', type: 'button', html: `${icon('wand')}<span>Make your own with AI</span>`, onclick: () => makeWithAI('cheatsheet') }),
      h('button', { class: 'btn', type: 'button', html: `${icon('folder')}<span>Import…</span>`, onclick: importFile }),
      h('button', { class: 'btn', type: 'button', html: `${icon('link')}<span>Paste a share code</span>`, onclick: pasteShareCode }))));
  box.append(youDontNeed());
  const search = h('input', { id: 'lib-search', class: 'field', type: 'search', placeholder: 'Search apps, packs or actions (e.g. “ripple”, “Figma”)', value: L.q, autocomplete: 'off', spellcheck: 'false',
    oninput: (e) => { L.q = e.target.value; renderLibGrid(); } });
  const cats = h('div', { class: 'cats', role: 'group', 'aria-label': 'Categories' });
  for (const [id, label] of LIB_CATS) {
    cats.append(h('button', { type: 'button', class: 'cat', 'aria-pressed': String(L.cat === id), text: label,
      onclick: () => { L.cat = id; if (id === 'community' && !L.community) loadCommunity(); renderLibrary(); } }));
  }
  box.append(h('div', { class: 'lib-tools' }, h('label', { class: 'search' }, h('span', { html: icon('search') }), search), cats));
  box.append(h('div', { id: 'lib-grid', class: 'lib-grid' }));
  renderLibGrid();
  if (keepFocus) { search.focus(); search.setSelectionRange(search.value.length, search.value.length); }
}

function renderLibGrid() {
  const grid = $('#lib-grid');
  if (!grid) return;
  grid.textContent = '';
  const q = L.q.trim().toLowerCase();
  let list = libPacks().filter((p) => libMatches(p, q));
  if (L.cat === 'mine') list = list.filter((p) => p._origin === 'mine');
  else if (L.cat === 'community') list = list.filter((p) => p._origin === 'community');
  else if (L.cat !== 'all') list = list.filter((p) => p.category === L.cat && p._origin !== 'community');
  for (const p of list) grid.append(libCard(p));
  if (L.cat === 'community' || L.cat === 'all') grid.append(communityCard());
  if (L.cat === 'mine' && !list.length) {
    grid.append(h('div', { class: 'lib-empty' }, h('b', { text: 'Nothing here yet.' }),
      h('span', { text: ' Packs you import, get as a share code or make with Cat Builder show up here.' }),
      h('button', { class: 'btn small strong', type: 'button', html: `${icon('wand')}<span>Make one with Cat Builder</span>`, onclick: () => setView('ai') })));
  } else if (!list.length && L.cat !== 'community') {
    grid.append(h('div', { class: 'lib-empty' }, h('b', { text: `No pack matches “${L.q}”.` }),
      h('span', { text: ' Make one in a minute: describe it, and any AI writes it for you.' }),
      h('button', { class: 'btn small strong', type: 'button', html: `${icon('wand')}<span>Make it with Cat Builder</span>`, onclick: () => { AI.want = L.q; setView('ai'); } })));
  }
}

function communityCard() {
  const st = L.communityState;
  const text = st === 'loading' ? 'Fetching packs shared by other people…'
    : st === 'error' ? 'Couldn’t reach the community library. Check your connection and try again.'
    : st === 'done' ? (L.community?.length ? `${L.community.length} pack${L.community.length === 1 ? '' : 's'} shared by people like you.` : 'No community packs yet. Yours could be the first!')
    : 'Packs shared by other Cat Key users, checked before they’re listed. Each shows who made it and how many ♥ it has.';
  return h('div', { class: 'lib-card comm-card' },
    h('div', { class: 'lib-top' }, h('span', { class: 'lib-ico glyph', html: icon('globe') })),
    h('div', { class: 'lib-name display', text: 'Community library' }),
    h('div', { class: 'lib-desc', text }),
    h('div', { class: 'lib-meta row' },
      st !== 'done' ? h('button', { class: 'btn small', type: 'button', text: st === 'loading' ? 'Loading…' : st === 'error' ? 'Try again' : 'Show them', onclick: loadCommunity }) : null,
      h('button', { class: 'btn small ghost', type: 'button', html: `${icon('up')}<span>Share yours</span>`, onclick: () => { L.cat = 'mine'; renderLibrary(); toast('Open one of your packs and press “Post to the store”.'); } })));
}

async function loadCommunity() {
  if (L.communityState === 'loading') return;
  L.communityState = 'loading';
  renderLibGrid();
  try {
    const res = await fetch(COMMUNITY_INDEX, { cache: 'no-store' });
    if (!res.ok) throw new Error(res.status);
    const idx = await res.json();
    L.community = (idx.packs || []).filter((p) => p && p.id && p.name && p.file)
      .map((p) => ({ ...p, likes: Math.max(0, Number(p.likes) || 0), issue: Number(p.issue) || 0, author: /^[A-Za-z0-9-]{1,39}$/.test(p.author || '') ? p.author : '', _summary: true, id: `community-${p.id}` }))
      .sort((a, b) => b.likes - a.likes || String(b.created || '').localeCompare(String(a.created || '')));
    L.communityState = 'done';
  } catch {
    L.communityState = 'error';
  }
  renderLibGrid();
}

async function openPackFrom(p) {
  if (!p._summary) return openPack(p, { origin: p._origin });
  try {
    const url = new URL(p.file, COMMUNITY_INDEX).href;
    const res = await fetch(url, { cache: 'no-store' });
    if (!res.ok) throw new Error(res.status);
    const { pack, problems } = checkPack(await res.json());
    pack.id = p.id;
    openPack(pack, { origin: 'community', problems });
  } catch (e) {
    toast(`Couldn’t open that pack (${e.message || e}).`, { kind: 'warn' });
  }
}

// ------------------------------------------------------------------ one pack: pick what to add

let PK = null;
function openPack(pack, { origin = 'builtin', problems = [] } = {}) {
  if (E) closeEditor();
  if (SETTINGS_OPEN) closeSettings();
  const items = packItems(pack);
  const sel = new Set();
  if (isRemap(pack)) {
    for (const x of items) {
      if (x.s.off || x.it.off) continue;
      const r = itemRule(pack, x.it);
      if (!inList(r)) sel.add(x.key);
    }
  }
  PK = { pack, origin, problems, sel, q: '', favOnly: false, last: null };
  const sheet = $('#packsheet');
  sheet.hidden = false;
  renderPack();
  $('#scrim').classList.add('on');
  requestAnimationFrame(() => sheet.classList.add('on'));
}

function closePack() {
  if (!PK) return;
  PK = null;
  const sheet = $('#packsheet');
  sheet.classList.remove('on');
  if (!E && !SETTINGS_OPEN) $('#scrim').classList.remove('on');
  setTimeout(() => { if (!PK) { sheet.hidden = true; sheet.textContent = ''; } }, 260);
}

function packClash(r) {
  return clashFor(r, rules());
}

function pkRow(x) {
  const { pack, sel } = PK;
  const it = x.it;
  if (!isRemap(pack)) {
    const keys = itemKeys(it);
    return h('div', { class: 'pk-row sheet-row' },
      h('span', { class: 'from' }, capsEl(keys)),
      h('span', { class: 'nm' }, it.fav ? h('span', { class: 'star', title: 'Pro pick', html: icon('star') }) : null, h('span', { text: it.name }),
        it.note ? h('small', { text: ` · ${it.note}` }) : null),
      h('button', { class: 'btn small ghost', type: 'button', title: 'Pick your own keys that do this',
        html: `${icon('edit')}<span>Use my keys</span>`, onclick: () => useMyKeys(pack, it) }));
  }
  const r = itemRule(pack, it);
  const has = inList(r);
  const clash = !has && packClash(r);
  const on = sel.has(x.key);
  let to;
  if (r.action.type === 'keys') to = capsEl(r.action.keys, { to: true });
  else if (r.action.type === 'launch') to = h('span', { class: 'nm', text: `Opens ${appName(r.action.target)}` });
  else if (r.action.type === 'text') to = h('span', { class: 'nm', text: 'Types text' });
  else to = h('span', { class: 'nm', text: 'Does nothing (blocked)' });
  const status = has ? h('span', { class: 'pk-st ok', html: `${icon('check')}<span>In your list</span>` })
    : clash ? h('span', { class: 'pk-st warn', title: `You already use ${K.plain(r.trigger)} for “${ruleName(clash)}”. This one comes in switched off.`, html: `${icon('info')}<span>Keys in use</span>` })
    : null;
  const cb = h('input', { type: 'checkbox', class: 'cb', checked: on && !has, disabled: has, 'aria-label': `Add ${it.name}` });
  const row = h('label', { class: `pk-row${has ? ' has' : ''}${on && !has ? ' on' : ''}` },
    cb,
    h('span', { class: 'from' }, capsEl(r.trigger)),
    h('span', { class: 'mini-arrow', 'aria-hidden': 'true', text: '→' }),
    h('span', { class: 'to' }, to),
    h('span', { class: 'nm' }, it.fav ? h('span', { class: 'star', title: 'Pro pick', html: icon('star') }) : null,
      h('span', { text: it.name }), it.note ? h('small', { text: ` · ${it.note}` }) : null,
      r.scope.mode !== 'all' ? h('small', { class: 'sc', text: ` · ${scopeText(r.scope)}` }) : null),
    status);
  cb.addEventListener('click', (e) => {
    const keys = pkVisible().filter((y) => !inList(itemRule(pack, y.it))).map((y) => y.key);
    if (e.shiftKey && PK.last && keys.includes(PK.last)) {
      const [a, b] = [keys.indexOf(PK.last), keys.indexOf(x.key)].sort((m, n) => m - n);
      for (const k of keys.slice(a, b + 1)) (cb.checked ? sel.add(k) : sel.delete(k));
    } else if (cb.checked) sel.add(x.key);
    else sel.delete(x.key);
    PK.last = x.key;
    renderPack(true);
  });
  return row;
}

function pkVisible() {
  const { pack, q, favOnly } = PK;
  const words = q.trim().toLowerCase().split(/\s+/).filter(Boolean);
  return packItems(pack).filter((x) => (!favOnly || x.it.fav) && (!words.length || words.every((w) =>
    [x.it.name, x.it.note || '', x.s.name, K.plain(x.it.press ? combo(x.it.press) : itemKeys(x.it)), x.it.does ? K.plain(combo(x.it.does)) : ''].join(' ').toLowerCase().includes(w))));
}

function renderPack(keepScroll = false) {
  if (!PK) return;
  const { pack, origin, problems, sel } = PK;
  const sheet = $('#packsheet');
  const prevScroll = $('#pk-body')?.scrollTop || 0;
  const hadFocus = document.activeElement?.id === 'pk-search';
  sheet.textContent = '';
  const remap = isRemap(pack);
  const visible = pkVisible();
  const addable = visible.filter((x) => !inList(itemRule(pack, x.it)));
  const total = packItems(pack).length;
  const chosen = remap ? [...sel].length : 0;
  const have = packHave(pack);
  const appNm = packAppName(pack);
  const sources = (pack.sources || []).filter((s) => /^https:\/\//.test(s.url || ''));
  sheet.append(h('div', { class: 'sheet-head' },
    libTile(pack, 'pk-ico'),
    h('div', { style: { flex: '1', minWidth: '0' } },
      h('h1', { id: 'pk-title', text: pack.name }),
      h('p', { text: [pack.description, appNm && !pack.description?.includes(appNm) ? `Works in ${appNm}.` : ''].filter(Boolean).join(' ') })),
    h('button', { class: 'tbtn', 'aria-label': 'Close', html: icon('x'), onclick: closePack })));
  const search = h('input', { id: 'pk-search', class: 'field', type: 'search', placeholder: 'Find in this pack', value: PK.q, autocomplete: 'off',
    oninput: (e) => { PK.q = e.target.value; renderPack(true); } });
  const tools = h('div', { class: 'pk-tools' },
    h('label', { class: 'search' }, h('span', { html: icon('search') }), search),
    h('button', { class: `chipbtn${PK.favOnly ? ' on' : ''}`, type: 'button', 'aria-pressed': String(PK.favOnly), html: `${icon('star')}<span>Pro picks</span>`,
      onclick: () => { PK.favOnly = !PK.favOnly; renderPack(true); } }),
    remap ? h('button', { class: 'btn small ghost', type: 'button', text: 'Select all', onclick: () => { addable.forEach((x) => sel.add(x.key)); renderPack(true); } }) : null,
    remap ? h('button', { class: 'btn small ghost', type: 'button', text: 'Select none', onclick: () => { visible.forEach((x) => sel.delete(x.key)); renderPack(true); } }) : null);
  const body = h('div', { class: 'sheet-body', id: 'pk-body' }, tools);
  if (problems.length) {
    body.append(h('div', { class: 'pk-problems' }, h('b', { text: `${problems.length} line${problems.length === 1 ? '' : 's'} skipped:` }),
      h('ul', {}, ...problems.slice(0, 8).map((m) => h('li', { text: m }))), problems.length > 8 ? h('div', { text: `…and ${problems.length - 8} more.` }) : null));
  }
  if (!remap) body.append(h('p', { class: 'hint pk-hint', text: `These are ${appNm || 'the app'}’s own shortcuts: they already work. Learn them, or press “Use my keys” to do one with keys you like better.` }));
  const bySection = new Map();
  for (const x of visible) {
    if (!bySection.has(x.si)) bySection.set(x.si, []);
    bySection.get(x.si).push(x);
  }
  for (const [si, list] of bySection) {
    const s = pack.sections[si];
    const secAdd = list.filter((x) => !inList(itemRule(pack, x.it)));
    const allOn = remap && secAdd.length && secAdd.every((x) => sel.has(x.key));
    body.append(h('div', { class: 'pk-sec' },
      h('div', { class: 'pk-sec-head' },
        remap && secAdd.length ? h('input', { type: 'checkbox', class: 'cb', checked: allOn, 'aria-label': `Select all in ${s.name}`,
          onclick: (e) => { secAdd.forEach((x) => (e.target.checked ? sel.add(x.key) : sel.delete(x.key))); renderPack(true); } }) : null,
        h('b', { text: s.name }), h('span', { class: 'n', text: String(list.length) }),
        s.off ? h('span', { class: 'lib-badge', text: 'Optional' }) : null),
      s.note ? h('div', { class: 'pk-note', text: s.note }) : null,
      h('div', { class: 'pk-rows' }, ...list.map(pkRow))));
  }
  if (!visible.length) body.append(h('p', { class: 'hint', text: 'Nothing matches. Try another word.' }));
  if (sources.length) {
    body.append(h('div', { class: 'pk-sources' }, h('span', { text: 'Sources: ' }),
      ...sources.map((s, i) => [i ? ', ' : '', h('a', { href: '#', text: s.title || s.url, onclick: (e) => { e.preventDefault(); invoke('open_link', { url: s.url }).catch(() => {}); } })])));
  }
  const saved = origin === 'mine';
  const foot = h('div', { class: 'sheet-foot' },
    h('button', { class: 'btn ghost', type: 'button', title: 'Copy a code anyone can paste into Cat Key', html: `${icon('link')}<span>Share code</span>`, onclick: () => copyShareCode(pack) }),
    h('button', { class: 'btn ghost', type: 'button', html: `${icon('up')}<span>Export</span>`, onclick: () => exportPack(pack) }),
    origin !== 'builtin' && !saved ? h('button', { class: 'btn ghost', type: 'button', html: `${icon('down')}<span>Keep in my Library</span>`, onclick: () => savePack(pack) }) : null,
    saved ? h('button', { class: 'btn ghost danger', type: 'button', html: `${icon('trash')}<span>Remove from Library</span>`, onclick: () => deletePack(pack) }) : null,
    origin === 'mine' || origin === 'new' ? h('button', { class: 'btn ghost', type: 'button', title: 'Share it with everyone: it shows with your GitHub name', html: `${icon('globe')}<span>Post to the store</span>`, onclick: () => postPack(pack) }) : null,
    trainItems(pack).length ? h('button', { class: 'btn ghost', type: 'button', html: `${icon('target')}<span>Train</span>`, onclick: () => { closePack(); setView('train'); startTrain(pack); } }) : null,
    h('span', { class: 'spacer' }),
    remap && have.length ? h('button', { class: 'btn ghost', type: 'button', text: `Remove ${have.length} from my list`, onclick: () => removePackRules(pack) }) : null,
    remap
      ? h('button', { class: 'btn primary', id: 'pk-add', type: 'button', disabled: !chosen, html: `${icon('plus')}<span>${chosen ? `Add ${chosen} shortcut${chosen === 1 ? '' : 's'}` : 'Pick some to add'}</span>`, onclick: (e) => addFromPack(e.currentTarget) })
      : h('button', { class: 'btn primary', type: 'button', text: 'Done', onclick: closePack }));
  sheet.append(body, foot);
  if (remap) foot.prepend(h('span', { class: 'pk-count', text: `${chosen} of ${total} picked` }));
  if (keepScroll) $('#pk-body').scrollTop = prevScroll;
  if (hadFocus) { const s2 = $('#pk-search'); s2.focus(); s2.setSelectionRange(s2.value.length, s2.value.length); }
}

async function addFromPack(btn) {
  if (!PK) return;
  const { pack, sel } = PK;
  const fresh = [];
  let clashes = 0;
  for (const x of packItems(pack)) {
    if (!sel.has(x.key)) continue;
    const r = itemRule(pack, x.it);
    if (inList(r) || !r.trigger) continue;
    r.enabled = !clashFor(r, [...rules(), ...fresh]);
    if (!r.enabled) clashes++;
    fresh.push(r);
  }
  if (!fresh.length) return;
  if (btn) { const rc = btn.getBoundingClientRect(); confetti(rc.left + rc.width / 2, rc.top); }
  fresh.forEach((r) => markFresh(r.id));
  sounds.saved();
  closePack();
  await commit({ ...S.config, rules: [...rules(), ...fresh] });
  hero.react(1300);
  const n = fresh.length;
  toast(`Added ${n} shortcut${n === 1 ? '' : 's'} from “${pack.name}”.` + (clashes ? ` ${clashes} came in switched off: you already use those keys.` : ''),
    { kind: clashes ? 'warn' : 'ok', ms: 8000, action: { label: 'Undo', fn: () => commit({ ...S.config, rules: rules().filter((r) => !fresh.some((f) => f.id === r.id)) }) } });
}

function removePackRules(pack) {
  const before = rules();
  const gone = new Set(packHave(pack).map((r) => r.id));
  if (!gone.size) return;
  sounds.off();
  commit({ ...S.config, rules: before.filter((r) => !gone.has(r.id)) });
  renderPack(true);
  toast(`Removed ${gone.size} shortcut${gone.size === 1 ? '' : 's'} from “${pack.name}”.`, { ms: 8000, action: { label: 'Undo', fn: () => commit({ ...S.config, rules: before }) } });
}

function useMyKeys(pack, it) {
  const draft = { id: newId(), enabled: true, name: it.name, trigger: '', action: { type: 'keys', keys: itemKeys(it) }, scope: itemScope(pack, it), source: pack.id || '' };
  closePack();
  setTimeout(() => openEditor(draft, { isNew: true }), 120);
}

function stripPack(pack) {
  const out = {};
  for (const [k, v] of Object.entries(pack)) if (!k.startsWith('_')) out[k] = v;
  return { format: 'catkey-pack', version: 1, ...out };
}

async function savePack(pack) {
  const clean = stripPack(pack);
  if (!clean.id || L.data?.PACKS.some((p) => p.id === clean.id) || (S.config.packs || []).some((p) => p.id === clean.id)) clean.id = `my-${slug(clean.name)}-${Math.random().toString(36).slice(2, 6)}`;
  await commit({ ...S.config, packs: [...(S.config.packs || []), clean] });
  if (PK) { PK.pack = clean; PK.origin = 'mine'; renderPack(true); }
  toast(`“${clean.name}” is in your Library now (under Yours).`);
}

async function deletePack(pack) {
  if (!(await confirmBox(`Remove “${pack.name}” from your Library?`, 'Shortcuts you already added from it stay in your list.', 'Remove', true))) return;
  const before = S.config.packs || [];
  closePack();
  await commit({ ...S.config, packs: before.filter((p) => p.id !== pack.id) });
  toast(`Removed “${pack.name}”.`, { action: { label: 'Undo', fn: () => commit({ ...S.config, packs: before }) } });
}

const slug = (s) => String(s || 'pack').toLowerCase().normalize('NFKD').replace(/[^\w\s-]/g, '').trim().replace(/[\s_]+/g, '-').slice(0, 40) || 'pack';

async function exportPack(pack) {
  try {
    if (await invoke('save_text', { name: `${slug(pack.name)}.catkey.json`, text: JSON.stringify(stripPack(pack), null, 2) })) toast('Exported. Anyone can import that file into Cat Key.');
  } catch (e) { toast(String(e), { kind: 'err' }); }
}

// ------------------------------------------------------------------ share codes: a whole pack as one line of text

const b64url = (bytes) => { let s = ''; for (const b of bytes) s += String.fromCharCode(b); return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, ''); };
const unb64url = (s) => Uint8Array.from(atob(s.replace(/-/g, '+').replace(/_/g, '/') + '==='.slice((s.length + 3) % 4)), (c) => c.charCodeAt(0));
async function pipe(bytes, stream) { return new Uint8Array(await new Response(new Blob([bytes]).stream().pipeThrough(stream)).arrayBuffer()); }

async function shareCodeFor(obj) {
  const raw = new TextEncoder().encode(JSON.stringify(obj));
  return `CK1:${b64url(await pipe(raw, new CompressionStream('deflate-raw')))}`;
}
async function readShareCode(code) {
  const m = String(code).trim().match(/CK1:([A-Za-z0-9_-]+)/);
  if (!m) throw new Error('That isn’t a Cat Key share code (they start with CK1:).');
  const bytes = await pipe(unb64url(m[1]), new DecompressionStream('deflate-raw'));
  return new TextDecoder().decode(bytes);
}

async function copyShareCode(pack) {
  try {
    const code = await shareCodeFor(stripPack(pack));
    await navigator.clipboard.writeText(code);
    toast(`Share code copied (${code.length.toLocaleString()} characters). Paste it in a message; the other person pastes it into Cat Key.`, { ms: 7000 });
  } catch (e) { toast(`Couldn’t make a share code: ${e.message || e}`, { kind: 'err' }); }
}

async function pasteShareCode() {
  let text = '';
  try { text = await navigator.clipboard.readText(); } catch { /* ask instead */ }
  if (!/CK1:/.test(text)) text = await promptBox('Paste a share code', 'Share codes start with CK1:. Paste the whole thing.', '', 'Open') || '';
  if (!text) return;
  importText(text, 'share code');
}

// ------------------------------------------------------------------ importing anything: packs, AI answers, whole setups, PowerToys

const KEY_SET = new Set([...K.KEY_GROUPS.flatMap(([, ks]) => ks), 'Clear', 'Kana', 'Kanji', 'Convert', 'NonConvert', 'Select', 'Print', 'Execute', 'Help', 'Sleep', 'NumSeparator', 'LaunchMedia', 'BrowserStop', 'BrowserFavorites', 'Oem8', 'MediaStop']);
const KEY_ALIAS = { '[': 'LBracket', ']': 'RBracket', '\\': 'Backslash', '/': 'Slash', ',': 'Comma', '.': 'Period', '`': 'Backtick', '~': 'Backtick', '=': 'Equals', '-': 'Minus', ';': 'Semicolon', "'": 'Quote',
  plus: 'Equals', del: 'Delete', escape: 'Esc', esc: 'Esc', return: 'Enter', enter: 'Enter', space: 'Space', spacebar: 'Space', backspace: 'Backspace', tab: 'Tab', capslock: 'CapsLock', caps: 'CapsLock',
  pgup: 'PageUp', pageup: 'PageUp', pgdn: 'PageDown', pagedown: 'PageDown', home: 'Home', end: 'End', insert: 'Insert', ins: 'Insert', delete: 'Delete', forwarddelete: 'Delete',
  left: 'Left', right: 'Right', up: 'Up', down: 'Down', arrowleft: 'Left', arrowright: 'Right', arrowup: 'Up', arrowdown: 'Down', '←': 'Left', '→': 'Right', '↑': 'Up', '↓': 'Down',
  printscreen: 'PrintScreen', prtsc: 'PrintScreen', grave: 'Backtick', backquote: 'Backtick', tilde: 'Backtick', bracketleft: 'LBracket', leftbracket: 'LBracket', bracketright: 'RBracket', rightbracket: 'RBracket',
  comma: 'Comma', period: 'Period', dot: 'Period', slash: 'Slash', backslash: 'Backslash', minus: 'Minus', hyphen: 'Minus', equal: 'Equals', equals: 'Equals', semicolon: 'Semicolon', quote: 'Quote', apostrophe: 'Quote' };
const MOD_ALIAS = { mod: 'Mod', cmdorctrl: 'Mod', commandorcontrol: 'Mod', opt: 'Opt', option: 'Opt', '⌥': 'Opt', alt: 'Alt', cmd: 'Cmd', command: 'Cmd', '⌘': 'Cmd', meta: 'Win', super: 'Win', win: 'Win', windows: 'Win',
  ctrl: 'Ctrl', control: 'Ctrl', ctl: 'Ctrl', '⌃': 'Ctrl', shift: 'Shift', '⇧': 'Shift', lctrl: 'LCtrl', rctrl: 'RCtrl', lalt: 'LAlt', ralt: 'RAlt', lshift: 'LShift', rshift: 'RShift', lwin: 'LWin', rwin: 'RWin' };

/** Tidy one combo ("cmd + shift + [") into Cat Key's spelling ("Cmd+Shift+LBracket"). */
function tidyCombo(str) {
  if (typeof str !== 'string' || !str.trim()) return { error: 'empty keys' };
  let s = str.trim().replace(/\s*\+\s*/g, '+');
  let plusKey = false;
  if (/\+\+$/.test(s) || s === '+') { plusKey = true; s = s.replace(/\+?\+$/, ''); }
  if (/\s/.test(s)) return { error: `“${str}” looks like two shortcuts in a row; Cat Key takes one at a time` };
  const parts = s ? s.split('+') : [];
  let key = plusKey ? 'Equals' : parts.pop();
  const mods = [];
  for (const m of parts) {
    const mm = MOD_ALIAS[m.toLowerCase()];
    if (!mm) return { error: `“${m}” isn’t a modifier (use Mod, Opt, Shift, Ctrl…)` };
    if (!mods.includes(mm)) mods.push(mm);
  }
  if (/^[a-z0-9]$/i.test(key)) key = key.toUpperCase();
  else if (/^f([1-9]|1\d|2[0-4])$/i.test(key)) key = key.toUpperCase();
  else if (/^num(pad)?\s*\d$/i.test(key)) key = `Num${key.slice(-1)}`;
  else if (KEY_ALIAS[key] || KEY_ALIAS[key.toLowerCase()]) key = KEY_ALIAS[key] || KEY_ALIAS[key.toLowerCase()];
  else if (MOD_ALIAS[key.toLowerCase()] && !mods.length) key = { Ctrl: 'LCtrl', Alt: 'LAlt', Shift: 'LShift', Win: 'LWin', Cmd: 'LWin', Opt: 'LAlt' }[MOD_ALIAS[key.toLowerCase()]] || key;
  if (!KEY_SET.has(key)) return { error: `unknown key “${key}”` };
  return { value: [...mods, key].join('+') };
}

/** Read a pack from anywhere (file, share code, AI answer): keep what's usable, list what isn't. */
function checkPack(raw) {
  const problems = [];
  const src = raw && typeof raw === 'object' ? raw : {};
  let sections = Array.isArray(src.sections) ? src.sections : [];
  if (!sections.length && Array.isArray(src.shortcuts || src.items)) sections = [{ name: 'Shortcuts', items: src.shortcuts || src.items }];
  const cheat = src.kind === 'cheatsheet' || (!src.kind && sections.some((s) => (s.items || []).some((i) => i && i.keys && !i.press)));
  const out = [];
  for (const [si, s] of sections.entries()) {
    const items = [];
    for (const [ii, it] of (Array.isArray(s?.items) ? s.items : []).entries()) {
      const at = `${s?.name || `Section ${si + 1}`}, line ${ii + 1}${it?.name ? ` (${it.name})` : ''}`;
      if (!it || typeof it !== 'object') { problems.push(`${at}: not a shortcut`); continue; }
      const x = { name: String(it.name || '').slice(0, 80) };
      const field = (k) => { const t = tidyCombo(it[k]); if (t.error) { problems.push(`${at}: ${t.error}`); return null; } return t.value; };
      if (cheat) {
        x.keys = field('keys'); if (!x.keys) continue;
        if (it.mac) { const t = tidyCombo(it.mac); if (!t.error) x.mac = t.value; }
      } else {
        x.press = field('press'); if (!x.press) continue;
        if (it.does != null) { x.does = field('does'); if (!x.does) continue; }
        else if (it.launch) x.launch = String(it.launch);
        else if (it.text != null) x.text = String(it.text);
        else if (it.block) x.block = true;
        else { problems.push(`${at}: says what to press but not what it does`); continue; }
        if (x.does && x.does === x.press) { problems.push(`${at}: presses and does the same keys`); continue; }
      }
      if (!x.name) x.name = cheat ? K.plain(combo(x.keys)) : x.does ? `${K.plain(combo(x.press))} → ${K.plain(combo(x.does))}` : K.plain(combo(x.press));
      for (const k of ['only', 'except']) if (it[k]) x[k] = [].concat(it[k]).map(String);
      if (it.fav) x.fav = true;
      if (it.off) x.off = true;
      if (it.note) x.note = String(it.note).slice(0, 140);
      if (typeof it.demo === 'string' && it.demo.length <= 20000 && /<svg[\s>]/i.test(it.demo)) x.demo = it.demo;
      if (it.platform && ['windows', 'mac', 'both'].includes(it.platform)) x.platform = it.platform;
      items.push(x);
    }
    if (items.length) out.push({ name: String(s?.name || `Section ${si + 1}`).slice(0, 60), items, ...(s?.off ? { off: true } : {}), ...(s?.note ? { note: String(s.note).slice(0, 200) } : {}) });
  }
  const pack = {
    format: 'catkey-pack', version: 1,
    id: typeof src.id === 'string' && /^[a-z0-9-]{2,60}$/.test(src.id) ? src.id : '',
    name: String(src.name || 'Imported pack').slice(0, 60),
    kind: cheat ? 'cheatsheet' : 'remap',
    category: LIB_CATS.some(([c]) => c === src.category) ? src.category : 'work',
    platform: ['windows', 'mac', 'both'].includes(src.platform) ? src.platform : 'both',
    description: String(src.description || '').slice(0, 300),
    author: String(src.author || '').slice(0, 60),
    sections: out,
  };
  if (typeof src.app === 'string') pack.app = src.app.slice(0, 60);
  if (src.apps && typeof src.apps === 'object') pack.apps = { windows: [].concat(src.apps.windows || []).map(String), mac: [].concat(src.apps.mac || []).map(String) };
  if (Array.isArray(src.sources)) pack.sources = src.sources.filter((x) => x && /^https:\/\//.test(x.url || '')).slice(0, 12).map((x) => ({ title: String(x.title || x.url).slice(0, 120), url: x.url }));
  if (Array.isArray(src.tags)) pack.tags = src.tags.map(String).slice(0, 12);
  return { pack, problems };
}

/** Pull the JSON out of whatever was pasted: plain JSON, an AI answer with a ```json block, or a share code. */
async function parseAnything(text) {
  let t = String(text || '').trim();
  if (/CK1:[A-Za-z0-9_-]{8,}/.test(t)) t = await readShareCode(t);
  const fence = t.match(/```(?:json)?\s*([\s\S]*?)```/i);
  if (fence) t = fence[1].trim();
  try { return JSON.parse(t); } catch { /* try the outermost braces */ }
  const a = t.indexOf('{');
  const b = t.lastIndexOf('}');
  if (a >= 0 && b > a) return JSON.parse(t.slice(a, b + 1));
  throw new Error('There’s no Cat Key pack in there. Paste the whole answer, including the part in { }.');
}

async function importText(text, from = 'file') {
  let data;
  try { data = await parseAnything(text); } catch (e) { toast(e.message || String(e), { kind: 'warn', ms: 7000 }); return false; }
  if (data && (data.format === 'catkey-profile' || (Array.isArray(data.rules) && !data.sections))) return importProfile(data);
  if (data && (data.format === 'catkey-cat' || (data.colors && !data.sections && !data.rules))) return importCat(data);
  if (data && (data.remapKeys || data.remapShortcuts || data.properties?.remapKeys || data.properties?.remapShortcuts)) {
    try { return mergeRules(await invoke('powertoys_rules', { text: JSON.stringify(data) }), 'PowerToys'); } catch (e) { toast(String(e), { kind: 'warn' }); return false; }
  }
  const { pack, problems } = checkPack(data);
  if (!pack.sections.length) {
    toast(problems.length ? `Nothing usable in that ${from}: ${problems[0]}.` : `There are no shortcuts in that ${from}.`, { kind: 'warn', ms: 8000 });
    return false;
  }
  setView('library');
  openPack(pack, { origin: 'new', problems });
  return true;
}

async function importFile() {
  try {
    const f = await invoke('open_text_file');
    if (f) await importText(f.text, 'file');
  } catch (e) { toast(String(e), { kind: 'warn' }); }
}

async function mergeRules(incoming, label) {
  const fresh = [];
  let clashes = 0;
  for (const x of incoming || []) {
    const r = { ...clone(x), id: newId() };
    if (inList(r)) continue;
    r.enabled = r.enabled !== false && !clashFor(r, [...rules(), ...fresh]);
    if (!r.enabled) clashes++;
    fresh.push(r);
  }
  if (!fresh.length) { toast(`Nothing new from ${label}: you have all of it already.`); return false; }
  fresh.forEach((r) => markFresh(r.id));
  await commit({ ...S.config, rules: [...rules(), ...fresh] });
  setView('shortcuts');
  toast(`Added ${fresh.length} shortcut${fresh.length === 1 ? '' : 's'} from ${label}.` + (clashes ? ` ${clashes} came in switched off: you already use those keys.` : ''),
    { kind: clashes ? 'warn' : 'ok', ms: 8000, action: { label: 'Undo', fn: () => commit({ ...S.config, rules: rules().filter((r) => !fresh.some((f) => f.id === r.id)) }) } });
  return true;
}

// ------------------------------------------------------------------ whole setup (profile): move to another PC or Mac

function exportProfileData() {
  return {
    format: 'catkey-profile', version: 1, platform: PLAT(), app: `Cat Key ${S.status.version || ''}`.trim(), exported: new Date().toISOString().slice(0, 10),
    enabled: S.config.enabled, rules: rules(), templates: S.config.templates || [], packs: S.config.packs || [], settings: S.config.settings,
  };
}

async function exportProfile() {
  try {
    if (await invoke('save_text', { name: `Cat Key setup ${new Date().toISOString().slice(0, 10)}.json`, text: JSON.stringify(exportProfileData(), null, 2) })) {
      toast('Saved your whole setup. Import it on any PC or Mac with Cat Key.');
    }
  } catch (e) { toast(String(e), { kind: 'err' }); }
}

/** Windows' Ctrl is the Mac's ⌘ (Cat Key writes ⌘ as Win): swap the two families. */
function crossCombo(str) {
  return String(str || '').split('+').map((t) => t.replace(/^(L|R)?(Ctrl|Win)$/, (_, side, fam) => `${side || ''}${fam === 'Ctrl' ? 'Win' : 'Ctrl'}`)).join('+');
}
function crossApps(apps, to) {
  const from = to === 'mac' ? 'windows' : 'mac';
  const out = [];
  for (const a of apps || []) {
    const hit = Object.values(L.data?.APPS || {}).find((x) => (x[from] || []).some((y) => y.toLowerCase() === String(a).toLowerCase()));
    if (!hit || !(hit[to] || []).length) return null;
    out.push(...hit[to]);
  }
  return [...new Set(out)];
}

async function importProfile(data) {
  await libData().catch(() => {});
  const from = data.platform === 'mac' ? 'mac' : 'windows';
  const to = PLAT();
  let incoming = Array.isArray(data.rules) ? data.rules : [];
  let dropped = 0;
  if (from !== to) {
    const next = [];
    for (const r of incoming) {
      if (SWITCHERS.has(r.source)) { dropped++; continue; }
      const x = clone(r);
      x.trigger = crossCombo(x.trigger);
      if (x.action?.type === 'keys') x.action.keys = crossCombo(x.action.keys);
      if (x.action?.type === 'launch') { dropped++; continue; } // programs differ between systems
      if (x.scope?.mode !== 'all' && x.scope?.apps?.length) {
        const apps = crossApps(x.scope.apps, to);
        if (!apps) { dropped++; continue; }
        x.scope.apps = apps;
      }
      next.push(x);
    }
    incoming = next;
  }
  const tpls = (data.templates || []).filter((t) => t && !t.builtin);
  const packs = (data.packs || []).filter((p) => p && p.sections && !(S.config.packs || []).some((q) => q.id === p.id));
  const sys = from !== to ? ` It comes from a ${from === 'mac' ? 'Mac' : 'Windows PC'}, so ${cat()} translates ⌘ and Ctrl for this ${to === 'mac' ? 'Mac' : 'PC'}${dropped ? ` and leaves out ${dropped} that only make sense there` : ''}.` : '';
  const choice = await choiceBox('Import this setup?', `${incoming.length} shortcut${incoming.length === 1 ? '' : 's'}, ${tpls.length} template${tpls.length === 1 ? '' : 's'} and ${packs.length} saved pack${packs.length === 1 ? '' : 's'}.${sys}`,
    [['cancel', 'Cancel', 'ghost'], ['replace', 'Replace mine', ''], ['add', 'Add to mine', 'primary']]);
  if (!choice || choice === 'cancel') return false;
  const before = S.config;
  if (choice === 'replace') {
    const next = { ...S.config, rules: incoming.map((r) => ({ ...r, id: newId() })), templates: tpls, packs: [...(S.config.packs || []), ...packs] };
    if (data.settings && from === to) next.settings = { ...S.config.settings, ...data.settings };
    await commit(next);
    setView('shortcuts');
    toast(`Your setup is now the imported one (${next.rules.length} shortcuts).`, { ms: 8000, action: { label: 'Undo', fn: () => commit(before) } });
    return true;
  }
  const names = new Set((S.config.templates || []).map((t) => t.name.toLowerCase()));
  await commit({ ...S.config, templates: [...(S.config.templates || []), ...tpls.filter((t) => !names.has(String(t.name).toLowerCase())).map((t) => ({ ...t, id: '' }))], packs: [...(S.config.packs || []), ...packs] });
  return mergeRules(incoming, 'that setup');
}

function choiceBox(title, text, choices) {
  return new Promise((resolve) => {
    const scrim = h('div', { class: 'dialog-scrim' });
    const done = (v) => { scrim.remove(); box.remove(); DIALOG = null; resolve(v); };
    const box = h('div', { class: 'dialog', role: 'alertdialog', 'aria-modal': 'true' },
      h('h2', { text: title }), h('p', { text }),
      h('div', { class: 'row wrap' }, ...choices.map(([v, label, cls]) => h('button', { class: `btn ${cls}`, type: 'button', text: label, onclick: () => done(v) }))));
    DIALOG = (yes) => done(yes ? choices[choices.length - 1][0] : null);
    scrim.onclick = () => done(null);
    layer().append(scrim, box);
    box.querySelector('.btn.primary')?.focus();
  });
}

// ------------------------------------------------------------------ make a pack with any AI (Cat Key writes the request; you bring the AI)

const AI = { want: '', wants: {}, kind: 'cheatsheet', app: '', plat: null, catName: '', answer: '', showPrompt: false, context: true, waiting: false, seq: 0, poll: null, until: 0 };
/** Switch what's being made; each kind keeps its own description. */
function setAiKind(v) {
  if (AI.kind === v) return;
  AI.wants[AI.kind] = AI.want;
  AI.kind = v;
  AI.want = AI.wants[v] || '';
}
const AI_KINDS = [
  ['cheatsheet', 'book', 'Learn an app', 'Its real shortcuts, checked by the AI, ready to train'],
  ['remap', 'swap', 'Remap my keys', 'Your own keys for an app’s actions'],
  ['cat', 'paw', 'Design a new cat', 'A cat of your own: its name, colours and gear'],
];
const AI_IDEAS = {
  cheatsheet: [['The Blender shortcuts pros use most', 'Blender'], ['Premiere Pro: the editing shortcuts top editors use', 'Premiere Pro'],
    ['Unreal Engine: moving around and building levels fast', 'Unreal Engine'], ['Figma shortcuts I should know', 'Figma']],
  remap: [['Make DaVinci Resolve feel like Premiere Pro', 'DaVinci Resolve'], ['One-hand editing in CapCut: everything on the left side of the keyboard', 'CapCut'],
    ['Mac-style shortcuts inside Excel', 'Excel']],
  cat: [['A sleepy grey cat with round glasses'], ['A tiny black ninja cat with a red headband'], ['A golden royal cat with a crown and a purple cape'],
    ['A very sad cat with big teary eyes'], ['A grumpy white cat who is unimpressed by everything']],
};

function catPrompt() {
  const want = AI.want.trim() || 'Surprise me: a cute, original cat.';
  const name = AI.catName.trim();
  const gear = GEAR.map((g) => `${g.id} (${g.name})`).join(', ');
  const taken = [...new Set(CATS.flatMap((c) => [c.name, c.full]).filter(Boolean))].join(', ');
  return [
    'You are designing a new cat mascot for Cat Key, a keyboard-shortcut app whose mascot is a cute chibi cat. Cat Key imports the JSON you write and draws the cat from it.',
    '',
    '## The cat I want',
    want,
    name ? `Its name: ${name}` : 'Its name: pick a short, catchy one that fits it.',
    '',
    '## How to do it well',
    `1. The cat is new, so its name must be new too: not one of Cat Key\u2019s own cats (${taken}).`,
    '2. Pick colours that work together and look good on both a dark and a light background. Fur and chest need contrast, and the eyes should pop.',
    '3. "stripes": true adds tabby stripes in a darker shade of the fur. "muzzle": true gives a muzzle in the chest colour. "ears": "fold" gives folded (Scottish Fold) ears.',
    `4. Optional "gear" (at most 3 pieces, one per body part), only from this list: ${gear}.`,
    '5. Optional extra colours in "colors" (hex): line (outline), ear (inside of the ears), blush, nose, stripe, tip (tail tip), whisker.',
    '6. "theme" sets the app\u2019s accent colour: "mochi" (pink), "kuro" (purple), "tora" (orange) or "sora" (blue).',
    '',
    '## Output',
    'Reply with exactly one JSON code block and nothing after it:',
    '```json',
    JSON.stringify({ format: 'catkey-cat', version: 1, name: name || 'Pixel', full: `${name || 'Pixel'} the Brave`, tag: 'Your cat', theme: 'tora',
      colors: { fur: '#F6B26B', belly: '#FFF6EC', eye: '#4CAF50', collar: '#C0392B' }, stripes: true, muzzle: true, ears: 'normal', gear: ['band-red'] }, null, 2),
    '```',
  ].join('\n');
}

function aiPrompt() {
  if (AI.kind === 'cat') return catPrompt();
  const plat = AI.plat || PLAT();
  const platText = plat === 'both' ? 'both Windows and macOS' : plat === 'mac' ? 'macOS' : 'Windows';
  const want = AI.want.trim() || (AI.kind === 'remap' ? 'Useful shortcuts for this app.' : 'The shortcuts professionals use most.');
  const app = AI.app.trim();
  const remap = AI.kind === 'remap';
  return [
    'You are helping me make a keyboard-shortcut pack for Cat Key, a keyboard-remapping app for Windows and macOS. Cat Key imports the JSON you write.',
    '',
    '## What I want',
    want,
    `App: ${app || '(not tied to one app: works everywhere)'}`,
    `Pack type: ${remap ? 'remap (my own keys that trigger the app’s existing shortcuts)' : 'cheatsheet (the app’s own default shortcuts, to learn and to build on)'}`,
    `Computer: ${platText}`,
    '',
    '## How to do it well',
    `1. Research first. Check ${app ? `${app}’s` : 'the'} official keyboard-shortcut documentation for the current version, then trusted sources pros use (official manuals and help pages, well-known cheat sheets). If you can browse the web, do it. Never invent a shortcut: if you aren’t sure a default exists in the current version, leave it out.`,
    remap
      ? '2. Each entry says what I press ("press") and which of the app’s REAL default shortcuts Cat Key should send ("does"). Cat Key can only trigger actions that already have a default shortcut, so every "does" must be a real default. Choose "press" keys that are easy to reach and don’t replace something important in the app. Single letters are fine for tools (Cat Key pauses them while I type in a text box).'
      : '2. List the app’s real default shortcuts ("keys"), grouped into clear sections (tools, editing, view…), about 25–60 entries: the ones professionals really use, not every menu item. Mark the 10–15 most valuable with "fav": true.',
    '3. Names are short and plain, like a menu item: "Split clip at playhead". Add a "note" only when it helps (under 12 words).',
    '4. Do your due diligence: check every shortcut against the official documentation for the current version, drop anything you can\u2019t confirm, and cite your sources in "sources".',
    '5. Demos: for the 8\u201312 entries marked "fav", add "demo": a tiny animated SVG showing what the action does in this app (a clip on a timeline splitting in two, a vertex of a 3D cube being copied, a layer being duplicated\u2026). It\u2019s a symbol of the action, not a picture of the app. Rules: viewBox="0 0 240 120", under 1,800 characters, simple shapes, animated with SMIL (<animate>, <animateTransform>, repeatCount="indefinite"), single quotes inside the SVG so the JSON stays valid; no scripts, no images, no links, no CSS, and at most two short words of text.',
    '',
    '## Output',
    'Reply with exactly one JSON code block and nothing after it. Use this format:',
    '```json',
    JSON.stringify({
      format: 'catkey-pack', version: 1, name: remap ? 'Premiere keys in DaVinci Resolve' : 'Blender essentials', kind: remap ? 'remap' : 'cheatsheet',
      platform: plat === 'both' ? 'both' : plat, app: app || 'App name', description: 'One sentence about what this pack does.',
      apps: { windows: ['resolve.exe'], mac: ['com.blackmagic-design.DaVinciResolve'] },
      sources: [{ title: 'Official keyboard shortcuts', url: 'https://example.com/shortcuts' }],
      sections: [{ name: 'Editing', items: remap
        ? [{ name: 'Split clip at playhead', press: 'Mod+K', does: 'Mod+B', fav: true }, { name: 'Ripple delete', press: 'Shift+Delete', does: 'Shift+Backspace', note: 'Closes the gap' }]
        : [{ name: 'Split clip at playhead', keys: 'Mod+B', fav: true, demo: "<svg viewBox='0 0 240 120' xmlns='http://www.w3.org/2000/svg'><rect x='20' y='45' width='200' height='30' rx='6' fill='#7C6CF0'/><line x1='120' y1='30' x2='120' y2='90' stroke='#FF6FAE' stroke-width='4'><animate attributeName='y2' values='30;90;90' dur='1.6s' repeatCount='indefinite'/></line></svg>" }, { name: 'Ripple delete', keys: 'Shift+Delete', mac: 'Shift+Backspace' }] }],
    }, null, 2),
    '```',
    '',
    '## Key names (exact spelling)',
    '- Modifiers: Mod = Ctrl on Windows and ⌘ Command on Mac. Opt = Alt on Windows and ⌥ Option on Mac. Shift. Ctrl = Control on both. In a Windows-only pack you may also use Alt and Win; in a Mac-only pack Cmd, Option and Control.',
    '- Keys: A–Z, 0–9, F1–F24, Esc, Tab, CapsLock, Space, Enter, Backspace (the Mac “delete” key), Delete (forward delete), Insert, Home, End, PageUp, PageDown, Left, Right, Up, Down, Backtick, Minus, Equals, LBracket, RBracket, Backslash, Semicolon, Quote, Comma, Period, Slash, Num0–Num9, NumAdd, NumSubtract, NumMultiply, NumDivide, NumDecimal, PrintScreen, MediaPlayPause, MediaNext, MediaPrev, VolumeUp, VolumeDown, VolumeMute.',
    '- Write one combo with "+", e.g. "Mod+Shift+LBracket". No sequences like "Ctrl+K Ctrl+S".',
    '- "platform": "both" only when every combo uses Mod / Opt / Ctrl / Shift and is the same on both systems; where the Mac key differs, add "mac": "<combo>" to that entry.',
    '- "apps": the app’s Windows .exe file name(s) and macOS bundle id(s), so the pack only acts inside that app. Leave it out for packs that work everywhere.',
    '- Optional per entry: "note", "fav": true, "off": true (not ticked by default).',
    '- Never use as "press": Win+L, Ctrl+Alt+Delete, Cmd+Option+Esc.',
    ...(remap && AI.context ? aiContext() : []),
  ].join('\n');
}

/** What the AI should know about this computer: keys already taken. */
function aiContext() {
  const used = [...new Set(rules().filter((r) => r.enabled).map((r) => K.plain(r.trigger).replace(/ /g, '+')))].slice(0, 80);
  return used.length ? ['', '## Keys I already use with Cat Key (don’t use these as "press")', used.join(', ')] : [];
}

// Where a request can go. In the browser the whole request rides in the link; the Claude app takes it in its own
// link (claude://); the ChatGPT app and Gemini get it from the clipboard.
const enc = (q) => encodeURIComponent(q);
const AI_TARGETS = [
  { id: 'chatgpt-search', where: 'web', name: 'ChatGPT', label: 'ChatGPT with web search', link: (q) => `https://chatgpt.com/?hints=search&q=${enc(q)}`, home: 'https://chatgpt.com/', max: 12000,
    hint: 'Opens ChatGPT in your browser with web search on and sends the whole request.' },
  { id: 'chatgpt', where: 'web', name: 'ChatGPT', label: 'ChatGPT', link: (q) => `https://chatgpt.com/?q=${enc(q)}`, home: 'https://chatgpt.com/', max: 12000,
    hint: 'Opens ChatGPT in your browser and sends the whole request.' },
  { id: 'claude', where: 'web', name: 'Claude', label: 'Claude', link: (q) => `https://claude.ai/new?q=${enc(q)}`, home: 'https://claude.ai/new', max: 14000,
    hint: 'Opens Claude in your browser with the whole request typed in. Check it and press Enter.' },
  { id: 'perplexity', where: 'web', name: 'Perplexity', label: 'Perplexity', link: (q) => `https://www.perplexity.ai/search?q=${enc(q)}`, home: 'https://www.perplexity.ai/', max: 8000,
    hint: 'Opens Perplexity in your browser and sends the whole request.' },
  { id: 'gemini', where: 'web', name: 'Gemini', label: 'Gemini', home: 'https://gemini.google.com/app',
    hint: 'Opens Gemini in your browser. The request waits on your clipboard: paste it (Ctrl+V) and send it.' },
  { id: 'claude-app', where: 'app', name: 'Claude', label: 'Claude app', app: 'claude', deep: (q) => `claude://claude.ai/new?q=${enc(q)}`, max: 14000, web: 'claude',
    hint: 'Opens the Claude app on this computer with the whole request typed in. Check it and press Enter.' },
  { id: 'chatgpt-app', where: 'app', name: 'ChatGPT', label: 'ChatGPT app', app: 'chatgpt', web: 'chatgpt',
    hint: 'Opens the ChatGPT app on this computer. The request waits on your clipboard: paste it (Ctrl+V) and send it.' },
  { id: 'copy', where: 'copy', label: 'Copy the request', hint: 'Copies the whole request, so you can paste it into any AI.' },
  { id: 'file', where: 'file', label: 'Save as a file', hint: 'Saves the whole request as a text file you can drop into any AI chat.' },
];
const TARGET_KEY = 'catkey.ai.target';
function aiTarget() {
  let id = null;
  try { id = JSON.parse(localStorage.getItem(TARGET_KEY) || '{}')[AI.kind]; } catch { /* first time */ }
  id ||= AI.kind === 'cat' ? 'claude' : 'chatgpt-search';
  return AI_TARGETS.find((t) => t.id === id) || AI_TARGETS[0];
}
function setAiTarget(id) {
  try { const m = JSON.parse(localStorage.getItem(TARGET_KEY) || '{}'); m[AI.kind] = id; localStorage.setItem(TARGET_KEY, JSON.stringify(m)); } catch { /* not kept */ }
}
const targetIcon = (t) => icon(t.where === 'web' ? 'globe' : t.where === 'app' ? 'app' : t.where === 'copy' ? 'copy' : 'down');
const targetVerb = (t) => (t.where === 'web' ? `Open ${t.label}` : t.where === 'app' ? `Open the ${t.label}` : t.label);
const targetBadge = (t) => (AI.kind === 'cat' ? (t.name === 'Claude' ? 'Best for cats' : '') : (t.id === 'chatgpt-search' ? 'Searches the web' : ''));
function targetHint(t, q = aiPrompt()) {
  if (t.where === 'web' && t.link && t.link(q).length > t.max) return `Opens ${t.name} in your browser. This request is long, so it waits on your clipboard: paste it (Ctrl+V) and send it.`;
  return t.hint;
}

async function sendRequest(t = aiTarget()) {
  const q = aiPrompt();
  if (t.where === 'copy') {
    try { await navigator.clipboard.writeText(q); bridgeStart(); toast('Copied the whole request. Paste it into any AI.'); } catch { AI.showPrompt = true; renderAI(); toast('Couldn’t copy. Select the text below and copy it.', { kind: 'warn' }); }
    return;
  }
  if (t.where === 'file') {
    try { if (await invoke('save_text', { name: `Cat Builder request${AI.kind !== 'cat' && AI.app ? ` - ${AI.app}` : ''}.txt`, text: q })) { bridgeStart(); toast('Saved. Drop it into your AI chat.'); } } catch (e) { toast(String(e), { kind: 'err' }); }
    return;
  }
  const ok = await confirmBox(`${targetVerb(t)}?`, `${targetHint(t, q)} When the answer is ready, click Copy on it: ${cat()} brings it back and sets it up.`, targetVerb(t));
  if (!ok) return;
  try { await navigator.clipboard.writeText(q); } catch { /* the link carries it */ }
  bridgeStart();
  try {
    if (t.where === 'app') {
      const deep = t.deep?.(q);
      const r = await invoke('open_ai_app', { app: t.app, link: deep && deep.length <= t.max ? deep : null });
      if (r === 'filled') toast(`${t.name} is open with your request typed in. Check it and press Enter.`, { ms: 8000 });
      else if (r === 'paste') toast(`${t.name} is opening. Paste the request (Ctrl+V) and send it.`, { ms: 10000 });
      else {
        toast(`The ${t.label} isn’t on this computer, so ${t.name} opens in your browser instead.`, { kind: 'warn', ms: 8000 });
        await openAiWeb(AI_TARGETS.find((x) => x.id === t.web), q);
      }
      return;
    }
    await openAiWeb(t, q);
  } catch (e) { toast(String(e), { kind: 'warn' }); }
}
async function openAiWeb(t, q) {
  const url = t.link?.(q);
  if (url && url.length <= t.max) await invoke('open_link', { url });
  else { await invoke('open_link', { url: t.home }); toast(`${t.name} is open. Paste the request (Ctrl+V) and send it.`, { ms: 10000 }); }
}
function targetMenu(anchor) {
  const cur = aiTarget();
  const row = (t) => h('button', { class: `item${t.id === cur.id ? ' on' : ''}`, type: 'button', role: 'menuitemradio', 'aria-checked': String(t.id === cur.id),
    onclick: () => { closePop(); setAiTarget(t.id); renderAI(); sendRequest(t); } },
    h('span', { html: targetIcon(t) }), h('span', { class: 'nm', text: t.label }), targetBadge(t) ? h('span', { class: 'best', text: targetBadge(t) }) : null);
  const group = (where) => AI_TARGETS.filter((t) => t.where === where).map(row);
  showPop(anchor, h('div', { class: 'pop ai-targets', role: 'menu', 'aria-label': 'Where to send the request' },
    h('div', { class: 'head', text: 'In your browser' }), ...group('web'),
    h('div', { class: 'sep' }), h('div', { class: 'head', text: 'Apps on this computer' }), ...group('app'),
    h('div', { class: 'sep' }), ...group('copy'), ...group('file')));
}

const partEl = (x) => h('span', { class: 'part', role: 'listitem' }, h('span', { class: 'ic', html: icon('check') }), h('span', { text: x }));

/** What goes into the request, shown before it's sent. */
function requestParts() {
  if (AI.kind === 'cat') {
    const name = AI.catName.trim();
    return ['Your description', name ? `The name ${name}` : 'Pick a new name', 'Colour tips', `${GEAR.length} pieces of gear`, 'Names already taken', 'The format Cat Key reads'];
  }
  const parts = ['Your words', AI.app.trim() || 'Any app', AI.plat === 'both' ? 'Windows and Mac' : AI.plat === 'mac' ? 'Mac' : 'Windows',
    'Research the official docs', 'Due diligence and sources'];
  if (AI.kind === 'cheatsheet') parts.push('Animated demos');
  parts.push('Exact key names', 'The format Cat Key reads');
  if (AI.kind === 'remap' && AI.context) {
    const n = new Set(rules().filter((r) => r.enabled).map((r) => r.trigger)).size;
    if (n) parts.push(`${n} keys you already use`);
  }
  return parts;
}

/** The way back: while waiting, Cat Key watches the clipboard for a pack (only while waiting, and only for packs). */
async function bridgeStart() {
  bridgeStop();
  AI.waiting = true;
  AI.until = Date.now() + 15 * 60e3;
  AI.seq = await invoke('clipboard_seq').catch(() => 0);
  AI.poll = setInterval(bridgeTick, 1200);
  renderAI();
}
function bridgeStop() {
  clearInterval(AI.poll);
  AI.poll = null;
  if (AI.waiting) { AI.waiting = false; if (S.view === 'ai') renderAI(); }
}
async function bridgeTick() {
  if (!AI.waiting || Date.now() > AI.until) { bridgeStop(); return; }
  const seq = await invoke('clipboard_seq').catch(() => 0);
  if (!seq || seq === AI.seq) return;
  AI.seq = seq;
  const text = await invoke('read_clipboard').catch(() => null);
  if (!text || !/catkey-pack|catkey-cat|CK1:|"sections"\s*:/.test(text)) return;
  if (text.includes('Cat Key imports the JSON you write')) return; // that's the request itself, copied again
  bridgeStop();
  await invoke('focus_window').catch(() => {});
  sounds.saved();
  toast(`${cat()} caught the AI’s answer from your clipboard!`);
  await importText(text, 'answer');
}

function renderAI() {
  const box = $('#aihelp');
  if (!box) return;
  const hadFocus = document.activeElement?.id;
  box.textContent = '';
  if (!AI.plat) AI.plat = PLAT();
  const isCat = AI.kind === 'cat';
  box.append(h('div', { class: 'tpl-intro' }, h('div', { class: 'txt' },
    h('h2', { class: 'display', text: 'Cat Builder' }),
    h('p', { class: 'hint', text: `Make shortcuts for any app, or a whole new cat, with the AI you already use. ${cat()} writes the request, the AI does the work, and ${cat()} picks up the answer when you copy it. ${cat()} sends nothing anywhere by itself.` }))));

  // 1. what you're making
  box.append(h('section', { class: 'ai-step' },
    h('div', { class: 'step-title' }, h('span', { class: 'n', text: '1' }), 'What are you making?'),
    h('div', { class: 'ai-kinds', role: 'group', 'aria-label': 'What are you making?' }, ...AI_KINDS.map(([v, ic, title, sub]) =>
      h('button', { class: 'ai-kind', type: 'button', 'aria-pressed': String(AI.kind === v), onclick: () => { if (AI.kind !== v) { setAiKind(v); clickSound(); renderAI(); } } },
        h('span', { class: 'ic', html: icon(ic) }), h('span', { class: 'txt' }, h('b', { text: title }), h('span', { text: sub })))))));

  // 2. the details: only what this kind needs
  const want = h('textarea', { id: 'ai-want', class: 'field area', rows: '3', oninput: (e) => { AI.want = e.target.value; refreshPrompt(); },
    placeholder: isCat ? 'Describe it in your own words: colours, mood, outfit… e.g. “a very sad grey cat with big teary eyes”'
      : AI.kind === 'remap' ? 'e.g. “Make DaVinci Resolve work with my Premiere keys”' : 'Anything special? e.g. “the ones top editors use” (optional)' });
  want.value = AI.want;
  const ideas = h('div', { class: 'ai-ideas' }, ...AI_IDEAS[AI.kind].map(([t, appName]) => h('button', { class: 'cat', type: 'button', text: t,
    onclick: () => { AI.want = t; if (appName) AI.app = appName; renderAI(); } })));
  const step2 = h('section', { class: 'ai-step' });
  if (isCat) {
    const name = h('input', { id: 'ai-name', class: 'field', type: 'text', maxlength: '24', placeholder: 'Its name (or let the AI pick one)', 'aria-label': 'Name of your new cat', value: AI.catName,
      oninput: (e) => { AI.catName = e.target.value; refreshPrompt(); } });
    step2.append(h('div', { class: 'step-title' }, h('span', { class: 'n', text: '2' }), 'Describe your new cat'),
      h('div', { class: 'ai-row' }, name), want, ideas,
      h('p', { class: 'hint', text: `Cat Key’s own cats keep their names. This makes a new cat that’s all yours, and you can switch back any time.` }));
  } else {
    const app = h('input', { id: 'ai-app', class: 'field', type: 'text', list: 'ai-apps', placeholder: 'Which app? Premiere Pro, Blender, Unreal Engine…', 'aria-label': 'Which app', value: AI.app,
      oninput: (e) => { AI.app = e.target.value; refreshPrompt(); } });
    const apps = h('datalist', { id: 'ai-apps' }, ...Object.values(L.data?.APPS || {}).map((x) => h('option', { value: x.name.replace(/ \(.*\)$/, '') })));
    const plat = h('div', { class: 'seg soft', role: 'group', 'aria-label': 'Computer' }, ...[['windows', 'Windows'], ['mac', 'Mac'], ['both', 'Both']].map(([v, label]) =>
      h('button', { type: 'button', 'aria-pressed': String(AI.plat === v), text: label, onclick: () => { AI.plat = v; renderAI(); } })));
    step2.append(h('div', { class: 'step-title' }, h('span', { class: 'n', text: '2' }), AI.kind === 'remap' ? 'Which app, and what should your keys do?' : 'Which app, and what do you want to learn?'),
      h('div', { class: 'ai-row' }, app, apps, plat), want, ideas);
    if (AI.kind === 'remap') step2.append(h('label', { class: 'ai-check' }, h('input', { type: 'checkbox', class: 'cb', checked: AI.context, onchange: (e) => { AI.context = e.target.checked; refreshPrompt(); } }),
      h('span', { text: 'Tell the AI which keys I already use, so it picks free ones' })));
  }
  box.append(step2);

  // 3. send it: everything above goes in the request
  const t = aiTarget();
  const pre = h('pre', { id: 'ai-prompt', class: 'ai-prompt', hidden: !AI.showPrompt });
  pre.textContent = aiPrompt();
  box.append(h('section', { class: 'ai-step' },
    h('div', { class: 'step-title' }, h('span', { class: 'n', text: '3' }), 'Send it to an AI'),
    h('div', { class: 'ai-parts', id: 'ai-parts', role: 'list', 'aria-label': 'What the request includes' }, ...requestParts().map(partEl)),
    h('div', { class: 'ai-send' },
      h('div', { class: 'split' },
        h('button', { class: 'btn primary main', type: 'button', id: 'ai-go', html: `${targetIcon(t)}<span>${targetVerb(t)}</span>`, onclick: () => sendRequest(t) }),
        h('button', { class: 'btn primary more', type: 'button', 'aria-label': 'Choose where to send it', 'aria-haspopup': 'menu', title: 'Choose where to send it', html: icon('chev'), onclick: (e) => targetMenu(e.currentTarget.parentElement) })),
      h('div', { class: 'ai-send-txt' },
        h('p', { class: 'hint', id: 'ai-send-hint', text: targetHint(t) }),
        targetBadge(t) ? h('span', { class: 'best', text: targetBadge(t) }) : null)),
    h('div', { class: 'ai-row' },
      h('button', { class: 'link', type: 'button', text: 'Choose another AI or app', onclick: (e) => targetMenu(e.currentTarget) }),
      h('span', { class: 'spacer' }),
      h('button', { class: 'link', type: 'button', text: AI.showPrompt ? 'Hide the request' : 'Show the whole request', onclick: () => { AI.showPrompt = !AI.showPrompt; renderAI(); } })),
    pre));

  // 4. bring it back
  const ans = h('textarea', { id: 'ai-answer', class: 'field area', rows: '4', placeholder: 'Or paste the AI’s whole answer here.', oninput: (e) => { AI.answer = e.target.value; } });
  ans.value = AI.answer;
  box.append(h('section', { class: 'ai-step' },
    h('div', { class: 'step-title' }, h('span', { class: 'n', text: '4' }), 'Bring it back'),
    AI.waiting
      ? h('div', { class: 'ai-bridge' }, h('span', { class: 'dot' }), h('span', { text: `Waiting for the answer. Click “Copy” on the AI’s code block and ${cat()} picks it up by itself.` }),
          h('span', { class: 'spacer' }), h('button', { class: 'btn small ghost', type: 'button', text: 'Stop waiting', onclick: bridgeStop }))
      : h('p', { class: 'hint', text: `After you send the request, ${cat()} watches for the answer on your clipboard, so just click Copy in the AI.` }),
    ans,
    h('div', { class: 'ai-row' },
      h('button', { class: 'btn', type: 'button', html: `${icon('check')}<span>Check what I pasted</span>`, onclick: async () => { if (await importText(AI.answer, 'answer')) AI.answer = ''; } }),
      h('span', { class: 'hint', text: isCat ? `${cat()} checks the colours and gear and shows you the new cat.` : `${cat()} checks every key, skips what it can’t use, and you pick before anything is added. Cheat sheets go straight into Train yourself.` }))));
  if (hadFocus && $(`#${hadFocus}`)) { const el = $(`#${hadFocus}`); el.focus(); if (el.setSelectionRange && el.value != null) el.setSelectionRange(el.value.length, el.value.length); }
}
function refreshPrompt() {
  const pre = $('#ai-prompt');
  if (pre) pre.textContent = aiPrompt();
  const parts = $('#ai-parts');
  if (parts) {
    parts.textContent = '';
    parts.append(...requestParts().map(partEl));
  }
  const hint = $('#ai-send-hint');
  if (hint) hint.textContent = targetHint(aiTarget());
}

// ------------------------------------------------------------------ select many shortcuts at once

const SEL = { on: false, ids: new Set(), last: null };
function setSelecting(on) {
  SEL.on = on;
  if (!on) { SEL.ids.clear(); SEL.last = null; }
  $('#app').classList.toggle('selecting', on);
  renderList();
  renderBulkBar();
}
function toggleSelect(r, e) {
  const ids = visibleRules().map((x) => x.id);
  if (e?.shiftKey && SEL.last && ids.includes(SEL.last)) {
    const [a, b] = [ids.indexOf(SEL.last), ids.indexOf(r.id)].sort((m, n) => m - n);
    const on = !SEL.ids.has(r.id);
    for (const id of ids.slice(a, b + 1)) (on ? SEL.ids.add(id) : SEL.ids.delete(id));
  } else if (SEL.ids.has(r.id)) SEL.ids.delete(r.id);
  else SEL.ids.add(r.id);
  SEL.last = r.id;
  renderList();
  renderBulkBar();
}
function renderBulkBar() {
  const bar = $('#bulkbar');
  if (!bar) return;
  bar.hidden = !SEL.on;
  bar.textContent = '';
  if (!SEL.on) return;
  for (const id of [...SEL.ids]) if (!rules().some((r) => r.id === id)) SEL.ids.delete(id);
  const n = SEL.ids.size;
  const picked = () => rules().filter((r) => SEL.ids.has(r.id));
  const act = (ic, label, fn, cls = '') => h('button', { class: `btn small ${cls}`, type: 'button', disabled: !n, html: `${icon(ic)}<span>${label}</span>`, onclick: fn });
  bar.append(
    h('b', { class: 'bulk-n', text: n ? `${n} selected` : 'Pick shortcuts' }),
    h('button', { class: 'btn small ghost', type: 'button', text: 'All', onclick: () => { visibleRules().forEach((r) => SEL.ids.add(r.id)); renderList(); renderBulkBar(); } }),
    h('button', { class: 'btn small ghost', type: 'button', text: 'None', onclick: () => { SEL.ids.clear(); renderList(); renderBulkBar(); } }),
    h('span', { class: 'spacer' }),
    act('paw', 'Turn on', () => bulkSet(true)),
    act('moon', 'Turn off', () => bulkSet(false)),
    act('layers', 'Template', async () => {
      const name = await promptBox('Save as a template', `${n} shortcut${n === 1 ? '' : 's'} as one set you can add back any time.`, `My set ${shortDate()}`);
      if (!name) return;
      await commit({ ...S.config, templates: [...(S.config.templates || []), { id: '', name, description: '', rules: picked().map((r) => ({ ...r })), builtin: false }] });
      toast(`Saved “${name}” in Templates.`);
    }),
    act('up', 'Export', () => exportPack({ name: `My shortcuts ${shortDate()}`, kind: 'remap', platform: PLAT(), category: 'work', description: '', sections: [{ name: 'Shortcuts', items: picked().map(ruleToItem) }] })),
    act('link', 'Share code', () => copyShareCode({ name: `Shortcuts from a friend`, kind: 'remap', platform: PLAT(), category: 'work', description: '', sections: [{ name: 'Shortcuts', items: picked().map(ruleToItem) }] })),
    act('trash', 'Delete', async () => {
      const before = rules();
      if (!(await confirmBox(`Delete ${n} shortcut${n === 1 ? '' : 's'}?`, 'You can undo right after.', 'Delete', true))) return;
      await commit({ ...S.config, rules: before.filter((r) => !SEL.ids.has(r.id)) });
      SEL.ids.clear();
      renderBulkBar();
      toast(`Deleted ${n}.`, { action: { label: 'Undo', fn: () => commit({ ...S.config, rules: before }) } });
    }, 'danger'),
    h('button', { class: 'btn small strong', type: 'button', text: 'Done', onclick: () => setSelecting(false) }));
}
function bulkSet(on) {
  const n = SEL.ids.size;
  commit({ ...S.config, rules: rules().map((r) => (SEL.ids.has(r.id) ? { ...r, enabled: on } : r)) });
  (on ? sounds.saved : sounds.off)();
  toast(`${on ? 'Turned on' : 'Turned off'} ${n}.`);
  renderBulkBar();
}
function ruleToItem(r) {
  const it = { name: ruleName(r), press: r.trigger };
  const a = r.action;
  if (a.type === 'keys') it.does = a.keys;
  else if (a.type === 'launch') { it.launch = a.target; if (a.args) it.args = a.args; }
  else if (a.type === 'text') it.text = a.text;
  else it.block = true;
  if (r.scope?.mode === 'only' && r.scope.apps?.length) it.only = r.scope.apps;
  if (r.scope?.mode === 'except' && r.scope.apps?.length) it.except = r.scope.apps;
  return it;
}

// ------------------------------------------------------------------ train yourself: learn an app's shortcuts like a game
//
// Built on how skills stick:
// * retrieval practice: you recall the keys; you're never just shown them (except once, the first time);
// * spaced review (Leitner boxes): a shortcut comes back right before you'd forget it, then less and less often;
// * interleaving: sections are mixed, never drilled one block at a time;
// * immediate feedback in sound and picture (Grossman et al., CHI 2007: audio feedback speeds up hotkey learning);
// * speed matters: a shortcut is "yours" when it comes back fast, not just right;
// * the real goal is use in the app, so Cat Key counts the times you use them for real.

const TR = { on: false, pack: null, queue: [], cur: null, learn: false, streak: 0, best: 0, ok: 0, miss: 0, n: 0, score: 0, len: 20, mode: 'smart', phase: 'pick',
  live: '', last: null, misses: [], times: [], t0: 0, tPress: 0, started: 0, sound: true, mascot: null, timer: null, clock: null };
const TRAIN_KEY = 'catkey.train.v2';
const BOX_WAIT = [0, 0, 10 * 60e3, 24 * 3600e3, 3 * 24 * 3600e3, 7 * 24 * 3600e3]; // box 1..5
const FLUENT_MS = 2500;
const NEW_PER_ROUND = 5;
const today = () => new Date().toLocaleDateString('en-CA');
function trainDb() {
  try { const d = JSON.parse(localStorage.getItem(TRAIN_KEY) || '{}'); return { items: d.items || {}, days: d.days || {}, sessions: d.sessions || [], track: d.track !== false }; } catch { return { items: {}, days: {}, sessions: [], track: true }; }
}
function trainSave(db) { try { localStorage.setItem(TRAIN_KEY, JSON.stringify(db)); } catch { /* storage full or private: progress just isn't kept */ } scheduleProgress(); }
try { TR.sound = localStorage.getItem('catkey.train.sound') !== '0'; } catch { /* default on */ }
setTrainSounds(TR.sound);

function trainItems(p) {
  return packItems(p).map((x) => {
    const keys = isRemap(p) ? combo(x.it.press) : itemKeys(x.it);
    return keys && trainSafe(keys) ? { id: `${x.it.name}|${keys}`, name: x.it.name, keys, section: x.s.name, fav: !!x.it.fav, does: isRemap(p) && x.it.does ? combo(x.it.does) : '', scope: itemScope(p, x.it), demo: x.it.demo || '' } : null;
  }).filter(Boolean);
}
const trainable = () => [...selfPacks(), ...libPacks()].filter((p) => !p._summary && trainItems(p).length >= (p._origin === 'self' || p._origin === 'template' ? 2 : 4));
function packTrainStats(p, db = trainDb()) {
  const items = trainItems(p);
  const st = db.items[p.id] || {};
  const now = Date.now();
  let learned = 0, due = 0, fresh = 0, real = 0;
  for (const i of items) {
    const s = st[i.id];
    if (!s || !s.box) { fresh++; continue; }
    if (s.box >= 4) learned++;
    if ((s.due || 0) <= now) due++;
    real += s.real || 0;
  }
  return { total: items.length, learned, due, fresh, real, pct: items.length ? Math.round((100 * learned) / items.length) : 0, started: Object.keys(st).length > 0 };
}
function dayStreak(db) {
  let n = 0;
  const d = new Date();
  if (!db.days[today()]?.answers) d.setDate(d.getDate() - 1); // today not done yet: the streak is still alive
  for (;;) {
    const k = d.toLocaleDateString('en-CA');
    if (!db.days[k]?.answers) break;
    n++;
    d.setDate(d.getDate() - 1);
  }
  return n;
}
const fmtTime = (ms) => { const s = Math.round(ms / 1000); return s >= 3600 ? `${Math.floor(s / 3600)} h ${Math.round((s % 3600) / 60)} min` : s >= 60 ? `${Math.floor(s / 60)} min` : `${s} s`; };
const clock = (ms) => { const s = Math.floor(ms / 1000); return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`; };

function renderTrain() {
  const box = $('#train');
  if (!box) return;
  if (!L.data) { box.textContent = ''; box.append(h('p', { class: 'hint', text: 'Opening the library…' })); libData().then(() => S.view === 'train' && renderTrain()); return; }
  if (TR.on) return renderTrainSession();
  if (TY.on) return renderTyping(true);
  const db = trainDb();
  box.textContent = '';
  box.append(h('div', { class: 'tpl-intro' },
    h('div', { class: 'txt' },
      h('h2', { class: 'display', text: 'Train yourself' }),
      h('p', { class: 'hint', text: `Pick an app. ${cat()} names an action, you press its keys. While a round runs, every key stays inside the trainer: nothing reaches your apps or the system, so nothing closes by accident (keys that close windows aren’t drilled at all). Each shortcut comes back right before you'd forget it, until it's automatic.` })),
    h('div', { class: 'lib-actions' },
      h('button', { class: 'btn strong', type: 'button', html: `${icon('wand')}<span>Make your own with AI</span>`, onclick: () => makeWithAI('cheatsheet') }),
      h('button', { class: `chipbtn${TR.sound ? ' on' : ''}`, type: 'button', 'aria-pressed': String(TR.sound), html: `${icon('paw')}<span>Sounds ${TR.sound ? 'on' : 'off'}</span>`,
        onclick: () => { TR.sound = !TR.sound; setTrainSounds(TR.sound); try { localStorage.setItem('catkey.train.sound', TR.sound ? '1' : '0'); } catch { /* fine */ } renderTrain(); } }))));
  // the progress dashboard
  const t = db.days[today()] || {};
  const week = Object.entries(db.days).filter(([k]) => Date.now() - new Date(k).getTime() < 7 * 864e5).reduce((n, [, v]) => n + (v.real || 0), 0);
  const learned = trainable().reduce((n, p) => n + packTrainStats(p, db).learned, 0);
  const dueAll = trainable().map((p) => [p, packTrainStats(p, db)]).filter(([, st]) => st.started && st.due > 0);
  const stat = (big, small, cls = '') => h('div', { class: `tstat ${cls}` }, h('b', { class: 'display', text: big }), h('span', { text: small }));
  box.append(h('div', { class: 'tdash' },
    stat(`${dayStreak(db)}🔥`, 'day streak', 'fire'),
    stat(fmtTime(t.ms || 0), `today · ${t.answers || 0} answers`),
    stat(String(learned), 'shortcuts learned'),
    stat(String(week), 'used for real this week'),
    h('div', { class: 'tchart', title: 'Answers per day, last 14 days' }, ...Array.from({ length: 14 }, (_, i) => {
      const d = new Date(); d.setDate(d.getDate() - 13 + i);
      const v = db.days[d.toLocaleDateString('en-CA')] || {};
      const max = Math.max(10, ...Object.values(db.days).map((x) => x.answers || 0));
      return h('span', { class: 'col', title: `${d.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' })}: ${v.answers || 0} answers, ${v.real || 0} real uses` },
        h('span', { class: 'fill', style: { height: `${Math.round((100 * (v.answers || 0)) / max)}%` } }), h('span', { class: 'real', style: { height: `${Math.min(100, (v.real || 0) * 4)}%` } }));
    }))));
  if (dueAll.length) {
    box.append(h('div', { class: 'tdue' }, h('span', { html: icon('target') }),
      h('span', { text: `Time to review: ${dueAll.reduce((n, [, st]) => n + st.due, 0)} shortcut${dueAll.length === 1 ? '' : 's'} are about to slip. Five minutes keeps them.` }),
      h('button', { class: 'btn small strong', type: 'button', text: `Review ${dueAll[0][0].name}`, onclick: () => startTrain(dueAll[0][0]) })));
  }
  const seg = (items, val, set) => h('div', { class: 'seg soft', role: 'group' }, ...items.map(([v, label]) =>
    h('button', { type: 'button', 'aria-pressed': String(val === v), text: label, onclick: () => { set(v); renderTrain(); } })));
  box.append(h('div', { class: 'train-opts' },
    h('span', { class: 'hint', text: 'Practice' }), seg([['smart', 'Smart (due + new)'], ['fav', 'Pro picks'], ['all', 'Everything'], ['weak', 'My misses']], TR.mode, (v) => { TR.mode = v; }),
    h('span', { class: 'hint', text: 'Round' }), seg([[10, '10'], [20, '20'], [0, 'Endless']], TR.len, (v) => { TR.len = v; })));
  const grid = h('div', { class: 'lib-grid' });
  const ty = tyDb();
  grid.append(h('article', { class: 'lib-card train-card typing', tabindex: '0', role: 'button', 'aria-label': 'Touch typing',
    onclick: startTyping, onkeydown: (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); startTyping(); } } },
    h('div', { class: 'lib-top' }, h('span', { class: 'lib-ico glyph', html: icon('keyboard') }), h('div', { class: 'lib-badges' }, h('span', { class: 'lib-badge', text: 'Typing' }))),
    h('div', { class: 'lib-name display', text: 'Touch typing' }),
    h('div', { class: 'mastery' }, h('span', { class: 'bar' }, h('span', { style: { width: `${Math.round((100 * ty.open) / TY_ORDER.length)}%` } })),
      h('span', { text: `${ty.open} of ${TY_ORDER.length} keys${ty.best ? ` · best ${ty.best} wpm` : ''}` })),
    h('div', { class: 'lib-meta', text: 'Fingers home, eyes on the text. It drills the keys you miss.' })));
  for (const p of trainable()) {
    const st = packTrainStats(p, db);
    grid.append(h('article', { class: `lib-card train-card${st.due && st.started ? ' due' : ''}`, tabindex: '0', role: 'button', 'aria-label': `Train ${p.name}`,
      onclick: () => startTrain(p), onkeydown: (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); startTrain(p); } } },
      h('div', { class: 'lib-top' }, libTile(p), h('div', { class: 'lib-badges' },
        st.started && st.due ? h('span', { class: 'lib-badge comm', text: `${st.due} due` }) : null,
        h('span', { class: 'lib-badge', text: isRemap(p) ? 'Your keys' : 'App keys' }))),
      h('div', { class: 'lib-name display', text: p.name }),
      h('div', { class: 'mastery' }, h('span', { class: 'bar' }, h('span', { style: { width: `${st.pct}%` } })),
        h('span', { text: st.started ? `${st.learned}/${st.total} learned` : `${st.total} to learn` })),
      st.real ? h('div', { class: 'lib-meta', text: `Used ${st.real}× for real` }) : null));
  }
  grid.append(h('article', { class: 'lib-card comm-card' },
    h('div', { class: 'lib-top' }, h('span', { class: 'lib-ico glyph', html: icon('wand') })),
    h('div', { class: 'lib-name display', text: 'Any other app' }),
    h('div', { class: 'lib-desc', text: 'Cat Builder asks ChatGPT or Claude for the pro shortcuts of any app. Import the answer and it’s here to practice.' }),
    h('div', { class: 'lib-meta row' }, h('button', { class: 'btn small strong', type: 'button', html: `${icon('wand')}<span>Open Cat Builder</span>`, onclick: () => { setAiKind('cheatsheet'); setView('ai'); } }))));
  box.append(grid);
  box.append(h('label', { class: 'track-opt' },
    h('input', { type: 'checkbox', class: 'cb', checked: db.track, onchange: (e) => { const d = trainDb(); d.track = e.target.checked; trainSave(d); syncTracked(); } }),
    h('span', {}, h('b', { text: 'Count the shortcuts I use for real. ' }),
      h('span', { class: 'hint', text: `${cat()} notices when you press a shortcut you’re learning in its app, and counts it here. Only those shortcuts are counted; nothing you type is kept.` }))));
  if (db.sessions.length) {
    box.append(h('div', { class: 'tsessions' }, h('b', { text: 'Recent rounds' }),
      ...db.sessions.slice(-5).reverse().map((x) => h('div', { class: 'tsess' }, h('span', { text: x.pack }), h('span', { class: 'hint', text: `${x.score} pts · ${x.n ? Math.round((100 * x.ok) / x.n) : 0}% · ${(x.avg / 1000).toFixed(1)} s avg · ${new Date(x.ts).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}` })))));
  }
}

/** Build the round: due reviews first, a few new ones (shown once, then asked), all mixed. */
function startTrain(p, only = null) {
  const items = trainItems(p);
  const db = trainDb();
  const st = db.items[p.id] || {};
  const now = Date.now();
  let pool = only || items;
  if (!only && TR.mode === 'fav') { const f = items.filter((i) => i.fav); if (f.length >= 4) pool = f; }
  if (!only && TR.mode === 'weak') {
    const w = items.filter((i) => st[i.id]?.miss > 0 && (st[i.id].box || 0) < 4);
    if (w.length) pool = w; else toast('No misses to practice yet: here’s a smart round instead.', { kind: 'warn' });
  }
  const shuffle = (xs) => { for (let i = xs.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [xs[i], xs[j]] = [xs[j], xs[i]]; } return xs; };
  const due = shuffle(pool.filter((i) => st[i.id]?.box && (st[i.id].due || 0) <= now)).sort((a, b) => (st[a.id].box || 0) - (st[b.id].box || 0));
  const fresh = pool.filter((i) => !st[i.id]?.box).sort((a, b) => Number(b.fav) - Number(a.fav));
  const known = shuffle(pool.filter((i) => st[i.id]?.box && (st[i.id].due || 0) > now));
  const queue = [];
  for (const i of due) queue.push({ item: i });
  for (const i of fresh.slice(0, only ? fresh.length : NEW_PER_ROUND)) {
    const at = Math.min(queue.length, Math.floor(Math.random() * (queue.length + 1)));
    queue.splice(at, 0, { item: i, learn: true }, ...(queue.length > at ? [] : []));
    queue.splice(Math.min(queue.length, at + 3), 0, { item: i });
  }
  if (TR.mode !== 'smart' || only) for (const i of shuffle(pool.filter((x) => !queue.some((q) => q.item.id === x.id)))) queue.push({ item: i, learn: !st[i.id]?.box });
  while (queue.length < (TR.len || 30) && known.length) queue.push({ item: known.shift() });
  if (!queue.length) queue.push(...shuffle(pool.slice()).map((i) => ({ item: i })));
  Object.assign(TR, { on: true, pack: p, queue, cur: null, learn: false, streak: 0, best: 0, ok: 0, miss: 0, n: 0, score: 0, phase: 'ask', live: '', last: null, misses: [], times: [], started: performance.now() });
  $('#app').classList.add('training');
  clearInterval(TR.clock);
  TR.clock = setInterval(() => { const c = $('#tr-clock'); if (c && TR.on && TR.phase !== 'end') c.textContent = clock(performance.now() - TR.started); }, 500);
  renderTrainSession(true);
  trainNext();
}

function trainNext() {
  if (!TR.on) return;
  if ((TR.len && TR.n >= TR.len) || (!TR.queue.length && TR.len)) return trainEnd();
  if (!TR.queue.length) { // endless: keep going with whatever is weakest
    const st = trainDb().items[TR.pack.id] || {};
    const items = trainItems(TR.pack).sort((a, b) => (st[a.id]?.box || 0) - (st[b.id]?.box || 0) || Math.random() - 0.5);
    TR.queue.push(...items.slice(0, 8).map((i) => ({ item: i, learn: !st[i.id]?.box })));
  }
  const next = TR.queue.shift();
  TR.cur = next.item;
  TR.learn = !!next.learn;
  TR.phase = 'ask';
  TR.live = '';
  TR.last = null;
  TR.t0 = performance.now();
  TR.tPress = 0;
  renderTrainSession();
  trainListen();
}

function trainAlternatives(target, scope) {
  return rules().filter((r) => r.enabled && r.action.type === 'keys' && K.overlaps(r.action.keys, target) &&
    (r.scope.mode === 'all' || (scope.mode === 'only' && r.scope.mode === 'only' && r.scope.apps.some((x) => scope.apps.includes(x))))).map((r) => r.trigger);
}

/** While a round runs (and Cat Key is in front), every key goes to the trainer only. */
function trainCapture(on) {
  invoke('set_capture', { on: !!on }).catch(() => {});
}
/** Shortcuts never drilled: they close, lock or leave the screen. */
const NOT_TRAINED = ['Alt+F4', 'Ctrl+Alt+Delete', 'Win+L', 'Win+Q', 'Win+Tab', 'Win+Space', 'Ctrl+Win+Q'];
const trainSafe = (keys) => !NOT_TRAINED.some((k) => K.overlaps(keys, k));

function trainListen() {
  if (!TR.on || TR.phase !== 'ask') return;
  trainCapture(true);
  startRecording({
    onUpdate: (sc) => { if (!TR.tPress && sc) TR.tPress = performance.now(); TR.live = sc || ''; paintTrainKeys(); },
    onDone: (sc) => trainJudge(sc),
    onCancel: () => { if (TR.on && TR.phase === 'ask' && S.view === 'train' && !document.hidden) setTimeout(trainListen, 60); },
  });
}

function trainJudge(sc) {
  if (!TR.on || TR.phase !== 'ask') return;
  const cur = TR.cur;
  if (!sc) { trainListen(); return; }
  if (sc === 'Esc' && !K.overlaps(cur.keys, 'Esc')) { trainPause(); return; }
  const hint = sc === '__hint__';
  const good = !hint && (K.overlaps(sc, cur.keys) || trainAlternatives(cur.does || cur.keys, cur.scope).some((x) => K.overlaps(sc, x)));
  const rt = Math.max(0, (TR.tPress || performance.now()) - TR.t0);
  const db = trainDb();
  const st = ((db.items[TR.pack.id] ||= {})[cur.id] ||= { box: 0, ok: 0, miss: 0, seen: 0, due: 0 });
  const day = (db.days[today()] ||= { ms: 0, answers: 0, ok: 0, real: 0 });
  st.seen++;
  st.last = Date.now();
  day.answers++;
  TR.n++;
  TR.last = { sc: hint ? '' : sc, good };
  if (good) {
    st.ok++;
    day.ok++;
    if (!TR.learn) {
      const fast = rt <= FLUENT_MS;
      st.box = Math.min(5, Math.max(1, (st.box || 0) + (fast || st.box < 2 ? 1 : 0)));
      st.best = st.best ? Math.min(st.best, rt) : rt;
      TR.times.push(rt);
    } else st.box = Math.max(st.box || 0, 1);
    st.due = Date.now() + BOX_WAIT[st.box];
    TR.ok++;
    TR.streak++;
    TR.best = Math.max(TR.best, TR.streak);
    const speed = TR.learn ? 0 : Math.max(0, Math.min(50, Math.round(50 - (rt - 700) / 60)));
    TR.score += Math.round((TR.learn ? 25 : 100 + speed) * (1 + Math.min(TR.streak - 1, 10) * 0.1));
    trainSounds.boing();
    if (TR.streak % 5 === 0) setTimeout(() => trainSounds.streak(TR.streak / 5), 220);
  } else {
    st.miss++;
    st.box = 1;
    st.due = Date.now();
    TR.miss++;
    TR.streak = 0;
    if (!TR.misses.some((m) => m.id === cur.id)) TR.misses.push(cur);
    TR.queue.splice(Math.min(TR.queue.length, 2), 0, { item: cur }); // ask again soon
    trainSounds.meow();
  }
  trainSave(db);
  if (good) checkRankUp();
  TR.phase = good ? 'right' : 'wrong';
  renderTrainSession();
  clearTimeout(TR.timer);
  TR.timer = setTimeout(trainNext, good ? 850 : 2200);
}

function trainPause() {
  TR.phase = 'paused';
  trainCapture(false);
  stopRecording();
  renderTrainSession();
}

function trainLog() {
  if (!TR.n) return;
  const db = trainDb();
  const ms = performance.now() - TR.started;
  (db.days[today()] ||= { ms: 0, answers: 0, ok: 0, real: 0 }).ms += ms;
  db.sessions.push({ pack: TR.pack.name, ts: Date.now(), ms: Math.round(ms), n: TR.n, ok: TR.ok, score: TR.score, best: TR.best, avg: TR.times.length ? Math.round(TR.times.reduce((a, b) => a + b, 0) / TR.times.length) : 0 });
  db.sessions = db.sessions.slice(-60);
  trainSave(db);
  syncTracked();
}

function trainStop() {
  if (!TR.on) return;
  clearTimeout(TR.timer);
  clearInterval(TR.clock);
  if (TR.phase !== 'end') trainLog();
  TR.on = false;
  TR.phase = 'pick';
  $('#app').classList.remove('training');
  trainCapture(false);
  stopRecording();
}

function trainEnd() {
  clearTimeout(TR.timer);
  TR.phase = 'end';
  trainCapture(false);
  stopRecording();
  trainLog();
  trainSounds.done();
  renderTrainSession();
}

/** A tiny animation of what the action does (a cut, a ripple, a copy…), never a copy of the app. */
const CUES = [
  ['ripple', /ripple|close (the )?gap/i], ['cut', /\b(cut|split|razor|blade|slice|add edit|knife|loop cut)\b/i],
  ['delete', /delete|remove|clear|trash|reject|lift/i], ['copy', /copy|duplicate|clone|stamp/i],
  ['paste', /paste|insert|place|import|overwrite|append/i], ['undo', /\bundo\b|back\b|previous|backward/i],
  ['redo', /\bredo\b|forward|next/i], ['zoom', /zoom|fit|100%/i], ['play', /play|stop|pause|shuttle|preview/i],
  ['select', /select|marquee|lasso|pick\b|deselect|invert/i], ['save', /save|export|render|deliver/i],
  ['marker', /mark|marker|keyframe|\bin point|\bout point|flag|star/i], ['group', /group|join|merge|link|combine|parent/i],
  ['new', /\bnew\b|add|create|open/i],
];
const cueFor = (name) => (CUES.find(([, re]) => re.test(name)) || ['spark'])[0];
const CUE_ICON = { undo: 'refresh', redo: 'refresh', zoom: 'search', save: 'down', new: 'plus', spark: 'sparkle' };
function cueSvg(kind) {
  const ic = (I[CUE_ICON[kind] || 'sparkle'] || '').replace(/<(path|circle|rect|ellipse)/g, '<$1 fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"');
  return `<svg class="cue-svg" viewBox="0 0 260 74" aria-hidden="true">
    <rect class="trk" x="6" y="21" width="248" height="32" rx="10"/>
    <rect class="b b1" x="14" y="26" width="70" height="22" rx="6"/>
    <rect class="b b2a" x="90" y="26" width="38" height="22" rx="6"/><rect class="b b2b" x="128" y="26" width="38" height="22" rx="6"/>
    <rect class="b b3" x="172" y="26" width="74" height="22" rx="6"/>
    <rect class="b b4" x="90" y="26" width="76" height="22" rx="6"/>
    <line class="blade" x1="128" y1="12" x2="128" y2="62"/>
    <rect class="lasso" x="84" y="20" width="88" height="34" rx="8"/>
    <path class="pin" d="M128 4c-6 0-10 4-10 9 0 7 10 15 10 15s10-8 10-15c0-5-4-9-10-9z"/>
    <circle class="play" cx="128" cy="37" r="15"/><path class="playtri" d="M123 29v16l13-8z"/>
    <g class="glyph"><svg x="112" y="21" width="32" height="32" viewBox="0 0 24 24">${ic}</svg></g>
  </svg>`;
}

function paintTrainKeys() {
  const box = $('#tr-keys');
  if (!box) return;
  box.textContent = '';
  const show = TR.phase === 'ask' ? TR.live : TR.last?.sc || '';
  if (show) box.append(capsEl(show, { big: true, pop: TR.phase !== 'ask' }));
  else box.append(h('span', { class: 'tr-wait', text: TR.phase === 'ask' ? (TR.learn ? 'Now press it…' : 'Press the shortcut…') : '' }));
}

function renderTrainSession(fresh = false) {
  const box = $('#train');
  if (!box || !TR.on) return;
  const p = TR.pack;
  if (fresh || !$('#tr-stage')) {
    box.textContent = '';
    const mini = h('div', { class: 'tr-cat' });
    box.append(h('div', { class: 'tr-session' },
      h('div', { class: 'tr-head' },
        libTile(p, 'pk-ico'),
        h('div', { class: 'txt' }, h('b', { class: 'display', text: p.name }), h('span', { id: 'tr-progress', class: 'hint' })),
        h('div', { class: 'tr-score' }, h('b', { id: 'tr-points', class: 'display' }), h('span', { id: 'tr-clock', class: 'hint', text: '0:00' })),
        h('span', { id: 'tr-streak', class: 'tr-streak' }),
        h('button', { class: 'btn small ghost', type: 'button', html: `${icon('x')}<span>Stop</span>`, onclick: () => { trainStop(); renderTrain(); } })),
      h('div', { class: 'tr-bar' }, h('span', { id: 'tr-bar' })),
      h('div', { id: 'tr-stage', class: 'tr-stage' }, mini,
        h('div', { class: 'tr-card' },
          h('div', { id: 'tr-section', class: 'tr-section' }),
          h('div', { id: 'tr-task', class: 'tr-task display' }),
          h('div', { id: 'tr-cue', class: 'tr-cue' }),
          h('div', { id: 'tr-learn', class: 'tr-learn' }),
          h('div', { id: 'tr-keys', class: 'tr-keys', 'aria-live': 'polite' }),
          h('div', { id: 'tr-answer', class: 'tr-answer' }))),
      h('div', { id: 'tr-foot', class: 'tr-foot' })));
    TR.mascot?.destroy?.();
    TR.mascot = new Mascot(mini, { size: 96, look: myLook() });
  }
  const cur = TR.cur;
  $('#tr-points').textContent = `${TR.score.toLocaleString()} pts`;
  $('#tr-progress').textContent = TR.phase === 'end' ? 'Round done' : `${Math.min(TR.n + (TR.phase === 'ask' ? 1 : 0), TR.len || 1e9)} of ${TR.len || '∞'} · ${TR.ok} right${TR.miss ? ` · ${TR.miss} missed` : ''}`;
  $('#tr-bar').style.width = TR.len ? `${Math.min(100, (100 * TR.n) / TR.len)}%` : '100%';
  const streak = $('#tr-streak');
  streak.textContent = TR.streak >= 2 ? `🔥 ${TR.streak}` : '';
  streak.classList.toggle('hot', TR.streak >= 5);
  const stage = $('#tr-stage');
  stage.classList.remove('right', 'wrong', 'paused', 'end', 'learn');
  if (['right', 'wrong', 'paused', 'end'].includes(TR.phase)) stage.classList.add(TR.phase);
  if (TR.learn && TR.phase === 'ask') stage.classList.add('learn');
  const demo = cur?.demo ? `demo:${cur.id}` : '';
  const kind = demo || (cur ? cueFor(cur.name) : 'spark');
  const cue = $('#tr-cue');
  if (demo) {
    if (cue.dataset.kind !== kind) {
      const svg = cleanDemo(cur.demo);
      cue.textContent = '';
      if (svg) cue.append(svg); else cue.innerHTML = cueSvg(cueFor(cur.name));
      cue.dataset.kind = kind;
    } else if (TR.phase === 'right') {
      const svg = cue.querySelector('svg');
      try { svg?.setCurrentTime?.(0); svg?.unpauseAnimations?.(); } catch { /* not animated */ }
    }
    cue.className = 'tr-cue cue-demo';
  } else {
    if (cue.dataset.kind !== kind || TR.phase === 'ask') { cue.innerHTML = cueSvg(kind); cue.dataset.kind = kind; }
    cue.className = `tr-cue cue-${kind}`;
  }
  const learnBox = $('#tr-learn');
  learnBox.textContent = '';
  const foot = $('#tr-foot');
  foot.textContent = '';
  if (TR.phase === 'end') {
    const acc = TR.n ? Math.round((100 * TR.ok) / TR.n) : 0;
    const avg = TR.times.length ? TR.times.reduce((a, b) => a + b, 0) / TR.times.length : 0;
    $('#tr-section').textContent = 'Round done';
    $('#tr-task').textContent = acc >= 90 ? 'Purr-fect paws!' : acc >= 70 ? 'Nice round!' : 'Good practice: that’s how it sticks.';
    $('#tr-keys').textContent = '';
    $('#tr-answer').textContent = `${TR.score.toLocaleString()} points · ${TR.ok}/${TR.n} right (${acc}%) · ${avg ? `${(avg / 1000).toFixed(1)} s per shortcut` : ''} · best streak ${TR.best} · ${clock(performance.now() - TR.started)}`;
    TR.mascot.react(2000);
    foot.append(...[
      TR.misses.length ? h('button', { class: 'btn strong', type: 'button', html: `${icon('target')}<span>Practice the ${TR.misses.length} I missed</span>`, onclick: () => startTrain(p, TR.misses.slice()) }) : null,
      h('button', { class: 'btn', type: 'button', html: `${icon('refresh')}<span>Another round</span>`, onclick: () => startTrain(p) }),
      h('button', { class: 'btn ghost', type: 'button', text: 'Back to all apps', onclick: () => { trainStop(); renderTrain(); } })].filter(Boolean));
    return;
  }
  $('#tr-section').textContent = [TR.learn && TR.phase === 'ask' ? 'New' : '', cur?.section, cur?.scope?.mode === 'only' ? scopeText(cur.scope).replace(/^Only in /, 'in ') : ''].filter(Boolean).join(' · ');
  $('#tr-task').textContent = cur?.name || '';
  if (TR.learn && TR.phase === 'ask') learnBox.append(h('span', { class: 'hint', text: 'New one. It’s ' }), capsEl(cur.keys), h('span', { class: 'hint', text: '. Press it to learn it.' }));
  const ans = $('#tr-answer');
  ans.textContent = '';
  if (TR.phase === 'wrong') ans.append(h('span', { text: TR.last?.sc ? 'Not quite. It’s ' : 'It’s ' }), capsEl(cur.keys), ...(cur.does ? [h('span', { class: 'hint', text: ` (sends ${K.plain(cur.does)})` })] : []));
  else if (TR.phase === 'right') ans.append(h('span', { class: 'good', text: TR.streak >= 5 ? ['Unstoppable!', 'On fire!', 'Paw-some!', 'Meow-velous!'][TR.streak % 4] : ['Yes!', 'Right!', 'Nice!', 'Got it!'][TR.n % 4] }));
  else if (TR.phase === 'paused') ans.append(h('span', { text: 'Paused. Your keyboard works normally again.' }));
  paintTrainKeys();
  if (TR.phase === 'right') { TR.mascot.react(900); requestAnimationFrame(() => cue.classList.add('go')); }
  if (TR.phase === 'wrong') { TR.mascot.setMood('awake'); stage.animate([{ transform: 'translateX(0)' }, { transform: 'translateX(-6px)' }, { transform: 'translateX(6px)' }, { transform: 'translateX(0)' }], { duration: 260 }); }
  if (TR.phase === 'ask') TR.mascot.setMood('listen');
  foot.append(...[
    TR.phase === 'paused'
      ? h('button', { class: 'btn strong', type: 'button', html: `${icon('paw')}<span>Keep going</span>`, onclick: () => { TR.phase = 'ask'; TR.t0 = performance.now(); TR.tPress = 0; renderTrainSession(); trainListen(); } })
      : h('button', { class: 'btn ghost', type: 'button', html: `${icon('info')}<span>Show me</span>`, disabled: TR.phase !== 'ask', onclick: () => trainJudge('__hint__') }),
    h('button', { class: 'btn ghost', type: 'button', text: 'Skip', disabled: TR.phase !== 'ask', onclick: () => { stopRecording(); trainNext(); } }),
    TR.phase === 'ask' ? h('button', { class: 'btn ghost', type: 'button', text: 'Pause', onclick: trainPause } ) : null,
    h('span', { class: 'spacer' }),
    h('span', { class: 'hint', text: TR.phase === 'ask' ? 'Esc pauses. Keys pressed here don’t reach any app.' : '' })].filter(Boolean));
}

window.addEventListener('blur', () => { if (TR.on && TR.phase === 'ask') trainPause(); });
document.addEventListener('visibilitychange', () => { if (document.hidden && TR.on && TR.phase === 'ask') trainPause(); });

// Real use: the shortcuts you're learning, counted when you press them in their app.
async function syncTracked() {
  if (!L.data) await libData().catch(() => null);
  const db = trainDb();
  const list = [];
  if (db.track) {
    for (const p of trainable()) {
      const st = db.items[p.id];
      if (!st) continue;
      for (const i of trainItems(p)) if (st[i.id]) list.push({ id: `${p.id}\u0001${i.id}`, keys: i.keys, apps: i.scope.mode === 'only' ? i.scope.apps : [] });
    }
  }
  invoke('set_tracked', { items: list.slice(0, 3000) }).catch(() => {});
}
async function pullTracked() {
  const got = await invoke('tracked_counts').catch(() => null);
  if (!got || !Object.keys(got).length) return;
  const db = trainDb();
  const day = (db.days[today()] ||= { ms: 0, answers: 0, ok: 0, real: 0 });
  for (const [key, n] of Object.entries(got)) {
    const [pid, iid] = key.split('\u0001');
    const st = db.items[pid]?.[iid];
    if (st) st.real = (st.real || 0) + n;
    day.real += n;
  }
  trainSave(db);
  if (S.view === 'train' && !TR.on) renderTrain();
}
setTimeout(syncTracked, 1500);
setInterval(pullTracked, 15000);

// ------------------------------------------------------------------ make your own, post to the store

function makeWithAI(kind = 'cheatsheet') {
  setAiKind(kind);
  setView('ai');
  setTimeout(() => $('#ai-want')?.focus(), 350);
}

function youDontNeed() {
  return h('div', { class: 'own-banner' }, h('span', { class: 'ic', html: icon('wand') }),
    h('div', { class: 'txt' }, h('b', { text: 'You don’t have to use these.' }),
      h('span', { class: 'hint', text: ` Make your own for any app in a minute: say what you want, ${cat()} writes the request, ChatGPT or Claude does the research, and the answer installs itself.` })),
    h('button', { class: 'btn small strong', type: 'button', text: 'Make my own', onclick: () => makeWithAI(S.view === 'templates' ? 'remap' : 'cheatsheet') }));
}

async function importCat(data) {
  const c = checkCat(data);
  if (!c) { toast('That cat needs a name and a fur colour.', { kind: 'warn' }); return false; }
  if (houseName(c.name)) { toast(`${c.name} is one of Cat Key’s own cats, so yours needs a name of its own. Change "name" in the answer, or ask the AI for another.`, { kind: 'warn', ms: 8000 }); return false; }
  await saveLook({ custom: [...(LOOK().custom || []).filter((x) => x.name !== c.name), c], cat: c.id });
  setView('warrior');
  sounds.mew();
  toast(`Say hi to ${c.name}!`);
  return true;
}

async function postPack(pack) {
  const ok = await confirmBox('Post this pack to the store?', 'GitHub opens in your browser with the pack filled in. Post it there: it shows with your GitHub name, and people can ♥ it. Cat Key never sees your login.', 'Open GitHub');
  if (!ok) return;
  const code = await shareCodeFor(stripPack(pack));
  try { await navigator.clipboard.writeText(code); } catch { /* the link carries it */ }
  const full = issueUrl('share-a-pack.yml', { title: `Pack: ${pack.name}`, app: packAppName(pack) || '', what: pack.description || '', pack: code });
  const url = full.length < 7900 ? full : issueUrl('share-a-pack.yml', { title: `Pack: ${pack.name}`, app: packAppName(pack) || '', what: pack.description || '' });
  invoke('open_link', { url }).catch((e) => toast(String(e), { kind: 'warn' }));
  if (url !== full) toast('The pack is big, so its code is on your clipboard: paste it into the form.', { ms: 9000 });
}

// ------------------------------------------------------------------ your cat: profiles, outfits, custom cats

const LOOK = () => S.config?.settings?.catLook || {};
const HEX = /^#[0-9a-f]{6}$/i;
const COLOR_KEYS = ['fur', 'belly', 'line', 'ear', 'eye', 'eyed', 'blush', 'nose', 'collar', 'gold', 'stripe', 'tip', 'whisker', 'muzzle', 'accent', 'zz'];

function mix(hex, to, t) {
  const p = (x, i) => parseInt(x.slice(1 + i * 2, 3 + i * 2), 16);
  const c = [0, 1, 2].map((i) => Math.round(p(hex, i) + (p(to, i) - p(hex, i)) * t));
  return `#${c.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

/** A full set of fur colours from the few a person picks (or an AI suggests). */
function catColors({ fur = '#FFF7F0', belly = '#FFFFFF', eye = '#9B6BFF', collar = '#FF6FAE', stripes = false, muzzle = false } = {}, extra = {}) {
  const c = {
    fur, belly, line: mix(fur, '#1a0f1f', 0.62), ear: '#FFB8D2', eye, eyed: mix(eye, '#000000', 0.72), blush: '#FF9EC4', nose: '#FF7FAF',
    collar, gold: '#FFD24D', stripe: mix(fur, '#000000', 0.28), stripeOp: stripes ? 1 : 0, tip: stripes ? mix(fur, '#000000', 0.28) : mix(fur, '#000000', 0.08),
    whisker: mix(fur, '#000000', 0.45), muzzle: belly, muzzleOp: muzzle ? 1 : 0, accent: collar, zz: mix(eye, '#ffffff', 0.35),
  };
  for (const k of COLOR_KEYS) if (HEX.test(extra[k] || '')) c[k] = extra[k];
  return c;
}

/** Check a cat made by hand or by an AI: only colours, a name, ears and known outfit pieces survive. */
function checkCat(raw) {
  if (!raw || typeof raw !== 'object') return null;
  const src = raw.cat && typeof raw.cat === 'object' ? raw.cat : raw;
  const name = String(src.name || '').trim().slice(0, 24);
  const col = src.colors || src.colours || {};
  if (!name || !HEX.test(col.fur || '')) return null;
  return {
    id: /^c[a-z0-9]{4,20}$/.test(src.id || '') ? src.id : `c${Math.random().toString(36).slice(2, 10)}`,
    name,
    full: String(src.full || src.fullName || name).trim().slice(0, 60),
    tag: String(src.tag || 'Your cat').slice(0, 20),
    custom: true,
    theme: SKINS.some((k) => k.id === src.theme) ? src.theme : 'mochi',
    ears: src.ears === 'fold' ? 'fold' : 'normal',
    gear: [].concat(src.gear || []).filter((g) => gearById(g)).slice(0, 6),
    colors: catColors({ fur: col.fur, belly: HEX.test(col.belly || '') ? col.belly : '#FFFFFF', eye: HEX.test(col.eye || '') ? col.eye : '#9B6BFF',
      collar: HEX.test(col.collar || '') ? col.collar : '#FF6FAE', stripes: !!(src.stripes ?? col.stripes), muzzle: !!(src.muzzle ?? col.muzzle) }, col),
  };
}

const customCats = () => (LOOK().custom || []).map(checkCat).filter(Boolean);
const allCats = () => [...CATS, ...customCats()];
function myCat() {
  const s = S.config?.settings || {};
  const id = ({ sumbul: 'sonbol' })[LOOK().cat] || LOOK().cat || s.skin || 'mochi';
  return allCats().find((c) => c.id === id) || CATS.find((c) => c.id === s.skin) || CATS[0];
}
function gearOk(id, c = myCat()) {
  const g = gearById(id);
  return !!g && (g.rank <= rankFor(xpBreakdown().total).index || (c.gear || []).includes(id));
}
function myLook(c = myCat()) {
  const worn = LOOK().wear?.[c.id] ?? c.gear ?? [];
  return { skin: c.skin, colors: c.colors, ears: c.ears, gear: worn.filter((g) => gearOk(g, c)) };
}
function saveLook(patch, extra = {}) {
  return commitSettings({ catLook: { ...LOOK(), ...patch }, ...extra }).then(() => { applyLook(); renderHero(); if (S.view === 'warrior') renderWarrior(); });
}
function catTitle(c) { return c.meme ? c.full : c.name; }
// True when a name belongs to one of Cat Key's own cats (theirs can't be taken or changed).
function houseName(n) {
  const k = String(n || '').trim().toLowerCase();
  return !!k && CATS.some((c) => [c.name, c.full].some((x) => x && x.toLowerCase() === k));
}
function chooseCat(c) {
  const extra = {};
  if (c.skin) extra.skin = c.skin;
  sounds.mew();
  hero?.knead(1200);
  return saveLook({ cat: c.id }, extra);
}
function wear(id) {
  const c = myCat();
  const g = gearById(id);
  const now = myLook(c).gear;
  const on = now.includes(id);
  const next = on ? now.filter((x) => x !== id) : [...now.filter((x) => gearById(x)?.slot !== g.slot), id];
  clickSound();
  return saveLook({ wear: { ...(LOOK().wear || {}), [c.id]: next } });
}

function floatHearts(el) {
  if (!el || reduced()) return;
  const r = el.getBoundingClientRect();
  for (let i = 0; i < 6; i++) {
    const d = h('span', { class: 'heart-pop', text: ['♥', '🐾', '♥', '✦'][i % 4] });
    d.style.left = `${r.left + r.width * (0.2 + Math.random() * 0.6)}px`;
    d.style.top = `${r.top + r.height * 0.45}px`;
    d.style.animationDelay = `${i * 90}ms`;
    layer().append(d);
    setTimeout(() => d.remove(), 1700);
  }
}

// ------------------------------------------------------------------ Cat Warrior: XP, ranks, time saved

const COACH = { stats: null, at: 0, loading: null };
async function coachStats(force = false) {
  if (!force && COACH.stats && Date.now() - COACH.at < 15000) return COACH.stats;
  COACH.loading ||= invoke('coach_stats').then((x) => { COACH.stats = x; COACH.at = Date.now(); return x; }).catch(() => COACH.stats).finally(() => { COACH.loading = null; });
  return COACH.loading;
}

function learnedCount(db = trainDb()) {
  let n = 0;
  for (const p of Object.values(db.items)) for (const s of Object.values(p)) if ((s.box || 0) >= 4) n++;
  return n;
}

/**
 * XP, all from things already counted (so it can't drift): 10 per right answer,
 * 25 per day you practised, 50 per shortcut learned, 3 per real use of one you're
 * learning, 1 per shortcut you use anywhere (up to 400 a day) and 1 per 10 letters
 * typed right in the typing trainer (up to 600 a day).
 */
function xpBreakdown(db = trainDb(), cs = COACH.stats) {
  const x = { train: 0, practice: 0, learned: 50 * learnedCount(db), real: 0, used: 0, typed: 0 };
  for (const d of Object.values(db.days)) {
    x.train += 10 * (d.ok || 0);
    if (d.answers || d.typed) x.practice += 25;
    x.real += 3 * (d.real || 0);
    x.typed += Math.min(600, Math.floor((d.typed || 0) / 10));
  }
  for (const apps of Object.values(cs?.days || {})) {
    let n = 0;
    for (const a of Object.values(apps)) n += a.used || 0;
    x.used += Math.min(400, n);
  }
  x.total = x.train + x.practice + x.learned + x.real + x.used + x.typed;
  return x;
}

function checkRankUp() {
  const r = rankFor(xpBreakdown().total);
  let seen = null;
  try { seen = localStorage.getItem('catkey.rank'); localStorage.setItem('catkey.rank', String(r.index)); } catch { return; }
  if (seen == null || r.index <= Number(seen)) return;
  const gear = GEAR.filter((g) => g.rank === r.index).map((g) => g.name);
  hero?.knead(2200);
  trainSounds.done();
  const c = $('#cushion')?.getBoundingClientRect();
  if (c) confetti(c.left + c.width / 2, c.top + c.height / 3);
  toast(`Rank up! ${cat()} is now a ${r.name}.${gear.length ? ` New gear: ${gear.join(', ')}.` : ''}`, { ms: 9000, action: { label: 'Dress up', fn: () => setView('warrior') } });
}

const dayKeys = (n) => Array.from({ length: n }, (_, i) => { const d = new Date(); d.setDate(d.getDate() - i); return d.toLocaleDateString('en-CA'); });

/** Shortcut use and mouse trips per app, over the last 7 days. */
function timeStats(cs = COACH.stats) {
  const week = new Set(dayKeys(7));
  const per = new Map();
  const t = { used: 0, caught: 0, lostMs: 0, usedToday: 0 };
  for (const [day, apps] of Object.entries(cs?.days || {})) {
    for (const [exe, a] of Object.entries(apps)) {
      if (day === today()) t.usedToday += a.used || 0;
      if (!week.has(day)) continue;
      const p = per.get(exe) || { exe, used: 0, caught: 0, lostMs: 0 };
      p.used += a.used || 0;
      p.caught += a.caught || 0;
      p.lostMs += a.lostMs || 0;
      per.set(exe, p);
      t.used += a.used || 0;
      t.caught += a.caught || 0;
      t.lostMs += a.lostMs || 0;
    }
  }
  t.perShortcut = cs?.savedPerShortcutMs || 1600;
  t.savedMs = t.used * t.perShortcut;
  t.apps = [...per.values()].sort((a, b) => b.used + b.caught - (a.used + a.caught));
  return t;
}

const kcaps = (keys) => h('span', { class: 'keys-inline' }, ...String(keys).split('+').map((k) => h('span', { class: 'kc', text: k.trim() })));

/** "Alt+Left Arrow" (as an app writes it) → "Alt+Left" (as Cat Key writes it). */
function coachCombo(keys) {
  const map = { 'left arrow': 'Left', 'right arrow': 'Right', 'up arrow': 'Up', 'down arrow': 'Down', 'page up': 'PageUp', 'page down': 'PageDown', pgup: 'PageUp', pgdn: 'PageDown',
    plus: 'Equals', minus: 'Minus', del: 'Delete', ins: 'Insert', esc: 'Esc', escape: 'Esc', return: 'Enter', backspace: 'Backspace', spacebar: 'Space' };
  const parts = String(keys).split('+').map((p) => p.trim()).filter(Boolean).map((p) => map[p.toLowerCase()] || p);
  const t = tidyCombo(parts.join('+'));
  return t.error ? null : t.value;
}

function coachTips() {
  return Object.entries(COACH.stats?.tips || {}).map(([id, t]) => ({ id, ...t })).sort((a, b) => b.n - a.n || b.last - a.last);
}

/** The things you did the long way, as a pack to practise. */
function coachPack(tips = coachTips()) {
  const items = [];
  for (const t of tips) {
    const keys = coachCombo(t.keys);
    if (!keys || items.some((i) => i.name === t.name && i.keys === keys)) continue;
    items.push({ name: t.name, keys, ...(t.app && !t.id.startsWith('*|') ? { only: [t.app] } : {}), note: `${t.n}× the long way` });
  }
  return { id: 'coach-misses', name: 'Caught by the coach', kind: 'cheatsheet', platform: 'both', description: 'Things you did with the mouse that have a shortcut.', sections: [{ name: 'Done the long way', items: items.slice(0, 40) }], _origin: 'coach' };
}

async function renderWarrior() {
  const box = $('#warrior');
  if (!box) return;
  if (!COACH.stats && !COACH.loading) coachStats().then(() => { if (S.view === 'warrior') renderWarrior(); });
  for (const m of W.minis) m.destroy();
  W.minis = [];
  const keepScroll = $('#scroll').scrollTop;
  box.textContent = '';
  const xp = xpBreakdown();
  const r = rankFor(xp.total);
  const me = myCat();
  const t = timeStats();
  const db = trainDb();

  // ---- the cat and its rank
  const holder = h('div', { class: 'wr-cat', title: 'Pet me', onclick: () => { W.minis[0]?.knead(1500); sounds.knead(); floatHearts(holder); } });
  const unlocks = r.next ? GEAR.filter((g) => g.rank === r.index + 1).map((g) => g.name) : [];
  box.append(h('div', { class: 'wr-hero' }, holder,
    h('div', { class: 'wr-rank' },
      h('span', { class: 'hint', text: `You · your cat today: ${cat()}${me.tag ? ` (${me.tag})` : ''}` }),
      h('h2', { class: 'display', text: r.name }),
      h('div', { class: 'xpbar', role: 'progressbar', 'aria-label': 'XP to the next rank', 'aria-valuemin': '0', 'aria-valuemax': '100', 'aria-valuenow': String(r.pct) }, h('span', { style: { width: `${r.pct}%` } })),
      h('div', { class: 'hint', text: r.next
        ? `${xp.total.toLocaleString()} XP · ${(r.next.xp - xp.total).toLocaleString()} more to ${r.next.name}${unlocks.length ? `, which unlocks ${unlocks.join(', ')}` : ''}.`
        : `${xp.total.toLocaleString()} XP. The highest rank there is.` }),
      h('div', { class: 'wr-ranks' }, ...RANKS.map((x, i) => h('span', { class: `wr-step${i <= r.index ? ' on' : ''}${i === r.index ? ' now' : ''}`, title: `${x.name}: ${x.xp.toLocaleString()} XP` }))),
      h('div', { class: 'row wr-actions' },
        h('button', { class: 'btn small strong', type: 'button', html: `${icon('copy')}<span>Copy my Cat Card</span>`, onclick: copyCardPicture }),
        h('button', { class: 'btn small', type: 'button', html: `${icon('link')}<span>Copy card code</span>`, onclick: copyCardCode }),
        h('button', { class: 'btn small', type: 'button', html: `${icon('up')}<span>Post to the leaderboard</span>`, onclick: postCard })))));
  W.minis.push(new Mascot(holder, { size: 168, look: myLook(), name: cat(), keyboard: true }));

  // ---- the numbers
  const stat = (big, small, cls = '') => h('div', { class: `tstat ${cls}` }, h('b', { class: 'display', text: big }), h('span', { text: small }));
  box.append(h('div', { class: 'tdash wr-stats' },
    stat(fmtTime(t.savedMs), 'saved by shortcuts this week', 'good'),
    stat(fmtTime(t.lostMs), 'lost to the mouse this week', t.lostMs ? 'bad' : ''),
    stat(fmtCount(t.usedToday), 'shortcuts used today'),
    stat(String(learnedCount(db)), 'shortcuts learned'),
    stat(`${dayStreak(db)}🔥`, 'day streak', 'fire')));

  // ---- where the time goes, app by app
  if (t.apps.length) {
    const rows = t.apps.slice(0, 8).map((a) => h('div', { class: 'wr-app' },
      h('span', { class: 'nm' }, appIcon(a.exe), h('span', { text: appName(a.exe) })),
      h('span', { class: 'n', text: fmtCount(a.used) }),
      h('span', { class: 'n good', text: fmtTime(a.used * t.perShortcut) }),
      h('span', { class: 'n', text: fmtCount(a.caught) }),
      h('span', { class: `n${a.lostMs ? ' bad' : ''}`, text: fmtTime(a.lostMs) })));
    box.append(h('section', { class: 'wr-card' }, h('h3', { text: 'App by app, this week' }),
      h('div', { class: 'wr-app head' }, h('span', { text: 'App' }), h('span', { class: 'n', text: 'Shortcuts' }), h('span', { class: 'n', text: 'Saved' }), h('span', { class: 'n', text: 'Mouse trips' }), h('span', { class: 'n', text: 'Lost' })),
      ...rows));
  }

  // ---- caught by the coach
  const tips = coachTips();
  const coachOn = S.config.settings.coach !== false;
  const caught = h('section', { class: 'wr-card' },
    h('div', { class: 'row-between' }, h('h3', { text: 'Caught doing it the long way' }),
      tips.length ? h('button', { class: 'btn small strong', type: 'button', html: `${icon('target')}<span>Train these</span>`, onclick: () => { const p = coachPack(); if (p.sections[0].items.length) { setView('train'); startTrain(p, trainItems(p)); } else toast('Nothing to practise yet.', { kind: 'warn' }); } }) : null));
  if (!tips.length) {
    caught.append(h('p', { class: 'hint', text: coachOn
      ? `When you click something that has a shortcut, ${cat()} shows the keys in a little note at the bottom right and lists it here. Nothing yet: either you're already a warrior, or it's early days.`
      : 'The shortcut coach is off. Turn it on in Settings to see what you still do with the mouse.' }),
    h('button', { class: 'btn small', type: 'button', html: `${icon('sparkle')}<span>Show me a tip</span>`, onclick: () => invoke('coach_demo').catch(() => {}) }));
  }
  for (const tip of tips.slice(0, 10)) {
    const muted = (S.config.settings.coachMuted || []).includes(tip.id.toLowerCase());
    caught.append(h('div', { class: 'wr-tip' },
      h('span', { class: 'what' }, h('b', { text: tip.name }), h('span', { class: 'hint', text: `${tip.id.startsWith('*|') ? 'Any app' : appName(tip.app)} · ${tip.n}×` })),
      kcaps(tip.keys),
      h('button', { class: 'link', type: 'button', text: muted ? 'Remind me again' : 'Don’t remind me', onclick: () => {
        const list = new Set(S.config.settings.coachMuted || []);
        if (muted) list.delete(tip.id.toLowerCase()); else list.add(tip.id.toLowerCase());
        commitSettings({ coachMuted: [...list] }).then(renderWarrior);
      } }),
      h('button', { class: 'tbtn', type: 'button', 'aria-label': `Forget ${tip.name}`, title: 'Forget', html: icon('x'), onclick: async () => {
        await invoke('coach_forget', { id: tip.id }).catch(() => {});
        await coachStats(true);
        renderWarrior();
      } })));
  }
  box.append(caught);

  // ---- choose your cat
  const picker = h('div', { class: 'cat-grid' });
  for (const c of allCats()) {
    const mini = h('span', { class: 'mini' });
    picker.append(h('button', { class: `cat-pick${c.id === me.id ? ' on' : ''}`, type: 'button', 'aria-pressed': String(c.id === me.id), title: c.full, onclick: () => chooseCat(c) },
      mini, h('b', { text: catTitle(c) }), h('span', { class: 'tag', text: c.tag || '' })));
    W.minis.push(new Mascot(mini, { size: 70, look: myLook(c) }));
  }
  picker.append(h('button', { class: 'cat-pick make', type: 'button', onclick: () => { W.editor = !W.editor; renderWarrior(); } },
    h('span', { class: 'plus', html: icon('plus') }), h('b', { text: 'Make one' }), h('span', { class: 'tag', text: 'Colours or AI' })));
  box.append(h('section', { class: 'wr-card', id: 'wr-cats' }, h('h3', { text: 'Choose your cat' }),
    h('p', { class: 'hint', text: 'Just for fun: switch any time. Your rank, XP and stats are yours, not the cat’s, so they stay the same whichever cat you pick.' }), picker, W.editor ? catEditor() : null));

  // ---- the wardrobe
  const worn = new Set(myLook().gear);
  const slots = [['head', 'Head'], ['face', 'Face'], ['back', 'Back'], ['belt', 'Belt'], ['hand', 'Paw'], ['aura', 'Glow']];
  const wardrobe = h('div', { class: 'wardrobe' });
  for (const [slot, label] of slots) {
    const items = GEAR.filter((g) => g.slot === slot);
    wardrobe.append(h('div', { class: 'ward-row' }, h('span', { class: 'slot', text: label }),
      h('div', { class: 'ward-items' }, ...items.map((g) => {
        const ok = gearOk(g.id, me);
        return h('button', { class: `gear${worn.has(g.id) ? ' on' : ''}${ok ? '' : ' locked'}`, type: 'button', disabled: !ok, 'aria-pressed': String(worn.has(g.id)),
          title: ok ? g.name : `${g.name}: unlocks at ${RANKS[g.rank].name}`, onclick: () => wear(g.id) },
          g.color ? h('span', { class: 'swatch', style: { background: g.color } }) : null,
          h('span', { text: g.name }), ok ? null : h('span', { class: 'lock', html: icon('lock') }));
      }))));
  }
  box.append(h('section', { class: 'wr-card' }, h('h3', { text: 'Wardrobe' }),
    h('p', { class: 'hint', text: 'Every rank unlocks new gear. Each warrior cat also keeps its own.' }), wardrobe));

  // ---- boards
  box.append(boardsEl());

  // ---- how it's counted
  box.append(h('p', { class: 'hint wr-data' }, `Your cat, rank, training and stats are saved in your data folder. `,
    h('button', { class: 'link', type: 'button', text: 'Open it', onclick: openDataFolder }), ' · ',
    h('button', { class: 'link', type: 'button', text: 'Back it up…', onclick: backupNow })));
  box.append(h('details', { class: 'wr-how' }, h('summary', { text: 'How the time and XP are counted' }),
    h('p', { text: `Time comes from the Keystroke-Level Model (Card, Moran and Newell), the classic way to time computer tasks: reaching for the mouse takes about 0.4 s each way, pointing about 1.1 s and a click 0.2 s, while a shortcut is two or three key presses at about 0.2 s each. So every shortcut you use counts as ${(t.perShortcut / 1000).toFixed(1)} s saved (as if it replaced one button: the cautious end), and every trip the coach catches counts what it cost: about 1.6 s for a button, 2.9 s for a menu.` }),
    h('p', { text: 'Only shortcuts with Ctrl, Alt or Win are counted (not typing), and only the count per app is kept: never which keys, never any text.' }),
    h('p', { text: `XP: 10 per right answer in training, 25 for each day you practise, 50 per shortcut learned, 3 each time you use one you're learning, 1 per shortcut you use anywhere (up to 400 a day) and 1 per 10 letters typed right in the typing trainer.` })));
  $('#scroll').scrollTop = keepScroll;
}

function catEditor() {
  const d = W.draft ||= { name: '', fur: '#F6B26B', belly: '#FFFFFF', eye: '#4CAF50', collar: '#E91E63', stripes: true, muzzle: true, fold: false };
  const preview = h('span', { class: 'mini' });
  const draw = () => {
    const c = { id: 'preview', name: d.name || 'New cat', colors: catColors(d), ears: d.fold ? 'fold' : 'normal', gear: [] };
    W.preview?.destroy();
    W.preview = new Mascot(preview, { size: 96, look: myLook(c) });
  };
  const color = (k, label) => h('label', { class: 'ce-color' }, h('input', { type: 'color', value: d[k], oninput: (e) => { d[k] = e.target.value; draw(); } }), h('span', { text: label }));
  const check = (k, label) => h('label', { class: 'ai-check' }, h('input', { type: 'checkbox', class: 'cb', checked: d[k], onchange: (e) => { d[k] = e.target.checked; draw(); } }), h('span', { text: label }));
  const el = h('div', { class: 'cat-editor' }, preview,
    h('div', { class: 'ce-fields' },
      h('input', { class: 'field', id: 'ce-name', type: 'text', maxlength: '24', placeholder: 'Name your new cat', 'aria-label': 'Name your new cat', value: d.name, oninput: (e) => { d.name = e.target.value; } }),
      h('div', { class: 'ce-row' }, color('fur', 'Fur'), color('belly', 'Chest'), color('eye', 'Eyes'), color('collar', 'Collar')),
      h('div', { class: 'ce-row' }, check('stripes', 'Stripes'), check('muzzle', 'White muzzle'), check('fold', 'Folded ears')),
      h('div', { class: 'ce-row' },
        h('button', { class: 'btn small strong', type: 'button', html: `${icon('check')}<span>Save my cat</span>`, onclick: () => {
          if (!d.name.trim()) { toast('Give your new cat a name first.', { kind: 'warn' }); $('#ce-name')?.focus(); return; }
          if (houseName(d.name)) { toast(`${d.name.trim()} is one of Cat Key’s own cats. Pick a name of your own.`, { kind: 'warn' }); $('#ce-name')?.focus(); return; }
          const c = checkCat({ name: d.name.trim(), colors: { fur: d.fur, belly: d.belly, eye: d.eye, collar: d.collar }, stripes: d.stripes, muzzle: d.muzzle, ears: d.fold ? 'fold' : 'normal' });
          if (!c) return;
          W.editor = false;
          W.draft = null;
          saveLook({ custom: [...(LOOK().custom || []), c], cat: c.id }).then(() => toast(`Say hi to ${c.name}!`));
        } }),
        h('button', { class: 'btn small', type: 'button', html: `${icon('wand')}<span>Design one with AI</span>`, onclick: () => { setAiKind('cat'); if (d.name.trim()) AI.catName = d.name.trim(); setView('ai'); } }),
        LOOK().custom?.length ? h('button', { class: 'btn small ghost danger', type: 'button', text: 'Remove my own cats', onclick: async () => {
          if (await confirmBox('Remove your own cats?', 'The cats you made are deleted. The house cats and warriors stay.', 'Remove', true)) saveLook({ custom: [], cat: CATS.some((c) => c.id === LOOK().cat) ? LOOK().cat : 'mochi' });
        } }) : null)));
  requestAnimationFrame(draw);
  return el;
}

// ------------------------------------------------------------------ Cat Card, friends and the leaderboard

const W = { minis: [], editor: false, draft: null, preview: null, board: null, boardState: 'idle' };
const LEADERBOARD = `https://raw.githubusercontent.com/${REPO}/main/leaderboard/index.json`;
const FRIENDS_KEY = 'catkey.friends';

function cardData() {
  const xp = xpBreakdown();
  const t = timeStats();
  const c = myCat();
  return { format: 'catkey-card', v: 1, name: cat(), cat: c.id, catName: c.name, ...(c.custom ? { colors: c.colors, ears: c.ears } : {}), gear: myLook().gear,
    rank: rankFor(xp.total).name, xp: xp.total, learned: learnedCount(), savedMin: Math.round(t.savedMs / 60000), streak: dayStreak(trainDb()), at: today() };
}
async function cardCode() {
  return `CKC1:${(await shareCodeFor(cardData())).replace(/^CK1:/, '')}`;
}
async function readCard(code) {
  const m = String(code || '').match(/CKC1:([A-Za-z0-9_-]{8,})/);
  if (!m) return null;
  const raw = await parseAnything(`CK1:${m[1]}`);
  if (!raw || raw.format !== 'catkey-card' || typeof raw.name !== 'string') return null;
  const num = (x) => (Number.isFinite(+x) ? Math.max(0, Math.round(+x)) : 0);
  return { name: raw.name.slice(0, 24), cat: String(raw.cat || 'mochi').slice(0, 24), rank: String(raw.rank || '').slice(0, 30), xp: num(raw.xp), learned: num(raw.learned), savedMin: num(raw.savedMin), streak: num(raw.streak), at: String(raw.at || '').slice(0, 10) };
}
async function copyCardCode() {
  try { await navigator.clipboard.writeText(await cardCode()); toast('Card code copied. Friends paste it on their Cat Warrior page.'); } catch (e) { toast(String(e), { kind: 'warn' }); }
}
function issueUrl(template, fields) {
  const q = new URLSearchParams({ template, ...fields });
  return `https://github.com/${REPO}/issues/new?${q}`;
}
async function postCard() {
  const code = await cardCode();
  const ok = await confirmBox('Post to the leaderboard?', 'GitHub opens in your browser with your card filled in. Post it there: your GitHub name shows on the board. Cat Key never sees your login.', 'Open GitHub');
  if (!ok) return;
  try { await navigator.clipboard.writeText(code); } catch { /* it's in the link too */ }
  invoke('open_link', { url: issueUrl('cat-card.yml', { title: `Cat Card: ${cat()} (${rankFor(xpBreakdown().total).name})`, card: code }) }).catch((e) => toast(String(e), { kind: 'warn' }));
}

function friends() {
  try { return JSON.parse(localStorage.getItem(FRIENDS_KEY) || '[]').filter((f) => f && f.name); } catch { return []; }
}
async function addFriend() {
  const code = await promptBox('Add a friend', 'Paste the Cat Card code your friend copied (it starts with CKC1:).', '', 'Add');
  if (!code) return;
  const card = await readCard(code).catch(() => null);
  if (!card) { toast('That isn’t a Cat Card code.', { kind: 'warn' }); return; }
  const list = friends().filter((f) => f.name !== card.name);
  list.push(card);
  try { localStorage.setItem(FRIENDS_KEY, JSON.stringify(list.slice(-50))); } catch { /* fine */ }
  scheduleProgress();
  toast(`${card.name} joined your board.`);
  renderWarrior();
}
async function loadBoard() {
  W.boardState = 'loading';
  renderWarrior();
  try {
    const res = await fetch(LEADERBOARD, { cache: 'no-store' });
    if (!res.ok) throw new Error(res.status);
    const idx = await res.json();
    W.board = (idx.players || []).filter((p) => p && p.login).slice(0, 100);
    W.boardState = 'done';
  } catch {
    W.boardState = 'error';
  }
  if (S.view === 'warrior') renderWarrior();
}
function boardsEl() {
  const me = { name: cat(), rank: rankFor(xpBreakdown().total).name, xp: xpBreakdown().total, savedMin: Math.round(timeStats().savedMs / 60000), streak: dayStreak(trainDb()), you: true };
  const list = [me, ...friends()].sort((a, b) => b.xp - a.xp);
  const row = (p, i) => h('div', { class: `wr-row${p.you ? ' you' : ''}` },
    h('span', { class: 'pos', text: String(i + 1) }),
    h('span', { class: 'who' }, h('b', { text: p.login ? `@${p.login}` : p.name }), h('span', { class: 'hint', text: [p.login && p.name ? p.name : '', p.rank].filter(Boolean).join(' · ') })),
    h('span', { class: 'n', text: `${(p.xp || 0).toLocaleString()} XP` }),
    h('span', { class: 'n hint', text: `${fmtTime((p.savedMin || 0) * 60000)} saved` }));
  const friendsBox = h('div', { class: 'wr-board' }, ...list.map(row));
  const globalBox = h('div', { class: 'wr-board' });
  if (W.boardState === 'done') {
    if (W.board.length) globalBox.append(...W.board.slice(0, 20).map(row));
    else globalBox.append(h('p', { class: 'hint', text: 'Nobody has posted yet. Be the first name on the board!' }));
  } else {
    globalBox.append(h('p', { class: 'hint', text: W.boardState === 'loading' ? 'Fetching the board…' : W.boardState === 'error' ? 'Couldn’t reach the leaderboard. Check your connection.' : 'Everyone who posted their Cat Card, ranked by XP.' }),
      W.boardState !== 'loading' ? h('button', { class: 'btn small', type: 'button', text: W.boardState === 'error' ? 'Try again' : 'Show the leaderboard', onclick: loadBoard }) : null);
  }
  return h('section', { class: 'wr-card wr-boards' },
    h('div', {}, h('div', { class: 'row-between' }, h('h3', { text: 'You and your friends' }),
      h('button', { class: 'btn small', type: 'button', html: `${icon('plus')}<span>Add a friend</span>`, onclick: addFriend })), friendsBox),
    h('div', {}, h('div', { class: 'row-between' }, h('h3', { text: 'Leaderboard' }),
      h('button', { class: 'link', type: 'button', text: 'Post my card', onclick: postCard })), globalBox));
}

/** The cat as a picture: copy the drawn SVG with its colours written in, so it looks the same anywhere. */
function catSvgText(svgEl) {
  const clone = svgEl.cloneNode(true);
  const src = [svgEl, ...svgEl.querySelectorAll('*')];
  const dst = [clone, ...clone.querySelectorAll('*')];
  src.forEach((el, i) => {
    const cs = getComputedStyle(el);
    const d = dst[i];
    if (cs.display === 'none') { d.setAttribute('display', 'none'); return; }
    for (const p of ['fill', 'stroke', 'opacity', 'stroke-width', 'fill-opacity']) {
      const v = cs.getPropertyValue(p);
      if (v && v !== 'normal') d.setAttribute(p, v);
    }
    d.removeAttribute('class');
    d.style.cssText = '';
  });
  clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
  clone.removeAttribute('style');
  return new XMLSerializer().serializeToString(clone);
}

async function copyCardPicture() {
  const svgEl = W.minis[0]?.svg || hero?.svg;
  if (!svgEl) return;
  const data = cardData();
  const cv = document.createElement('canvas');
  cv.width = 1200;
  cv.height = 630;
  const g = cv.getContext('2d');
  const bg = g.createLinearGradient(0, 0, 1200, 630);
  bg.addColorStop(0, '#2A2235');
  bg.addColorStop(1, '#16121D');
  g.fillStyle = bg;
  g.fillRect(0, 0, 1200, 630);
  g.fillStyle = 'rgba(255,111,174,.18)';
  for (let i = 0; i < 14; i++) { g.beginPath(); g.arc(80 + (i * 97) % 1100, 60 + ((i * 173) % 520), 10 + (i % 3) * 6, 0, Math.PI * 2); g.fill(); }
  const img = new Image();
  img.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(catSvgText(svgEl))}`;
  await img.decode().catch(() => {});
  g.drawImage(img, 70, 120, 420, 400);
  await document.fonts.ready;
  g.fillStyle = '#F7F0FB';
  g.font = '700 30px Fredoka, Nunito, sans-serif';
  g.fillText('CAT KEY · CAT CARD', 540, 150);
  g.font = '700 76px Fredoka, Nunito, sans-serif';
  g.fillText(data.name, 540, 245);
  g.fillStyle = '#FFB8D2';
  g.font = '800 44px Nunito, sans-serif';
  g.fillText(data.rank, 540, 310);
  g.fillStyle = '#B7AAC8';
  g.font = '800 30px Nunito, sans-serif';
  [`${data.xp.toLocaleString()} XP`, `${fmtTime(data.savedMin * 60000)} saved by shortcuts`, `${data.learned} shortcuts learned · ${data.streak}-day streak`].forEach((line, i) => g.fillText(line, 540, 380 + i * 48));
  g.fillStyle = '#FF6FAE';
  g.font = '800 24px Nunito, sans-serif';
  g.fillText('github.com/MasterfullyClaw/CatKey', 540, 560);
  try {
    const blob = await new Promise((res) => cv.toBlob(res, 'image/png'));
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    toast('Your Cat Card picture is copied. Paste it anywhere (Ctrl+V).');
    sounds.saved();
  } catch {
    toast('Couldn’t copy the picture here. The card code still works.', { kind: 'warn' });
  }
}

// ------------------------------------------------------------------ background purr

const PURR = { on: false };
function paintPurr() {
  const b = $('#btn-purr');
  if (!b) return;
  b.innerHTML = `${icon('purr')}${icon(PURR.on ? 'sound' : 'mute', 'snd')}<span class="waves" aria-hidden="true"></span>`;
  b.classList.toggle('on', PURR.on);
  b.setAttribute('aria-pressed', String(PURR.on));
  b.setAttribute('aria-label', PURR.on ? 'Purring: on. Click to mute' : 'Purring: off. Click to purr');
  b.title = PURR.on ? 'Purring in the background. Click to mute (right-click: volume)' : 'Purr in the background (right-click: volume)';
}
async function setPurr(on) {
  PURR.on = await invoke('purr_set', { on }).catch(() => false);
  paintPurr();
  if (on && PURR.on) { hero?.knead(1400); toast(`${cat()} is purring. It keeps going with the window closed; stop it here or from the tray.`, { ms: 5000 }); }
}
function purrMenu(anchor) {
  const s = S.config.settings;
  const vol = h('input', { type: 'range', min: '0', max: '100', step: '5', value: String(s.purrVolume ?? 45), class: 'range', 'aria-label': 'Purr volume',
    onchange: (e) => commitSettings({ purrVolume: Number(e.target.value) }) });
  showPop(anchor, h('div', { class: 'pop purr-pop' },
    h('b', { text: 'Background purr' }),
    h('p', { class: 'hint', text: 'A warm purr loop while you work. It keeps going when the window is closed.' }),
    h('label', { class: 'row-between' }, h('span', { text: 'Volume' }), vol),
    h('button', { class: 'btn small strong', type: 'button', text: PURR.on ? 'Stop purring' : 'Start purring', onclick: () => { closePop(); setPurr(!PURR.on); } })));
}

// ------------------------------------------------------------------ the coach learns the apps' own words from the packs

let coachDictSent = '';
async function syncCoachDict() {
  if (!L.data) await libData().catch(() => null);
  if (!L.data) return;
  const dict = {};
  for (const p of libPacks()) {
    if (isRemap(p) || p._summary) continue;
    const apps = packApps(p);
    if (!apps.length) continue;
    for (const x of packItems(p)) {
      const keys = itemKeys(x.it);
      if (!keys || !x.it.name) continue;
      const pretty = K.plain(keys).replace(/ /g, '+');
      for (const exe of apps) (dict[exe] ||= []).push([x.it.name, pretty]);
    }
  }
  const sig = JSON.stringify(Object.keys(dict).map((k) => [k, dict[k].length]));
  if (sig === coachDictSent) return;
  coachDictSent = sig;
  invoke('set_coach_dict', { dict }).catch(() => {});
}

// ------------------------------------------------------------------ train any set: your shortcuts and every template

function selfPacks() {
  const out = [];
  const toPack = (id, name, list, origin) => ({ id, name, kind: 'remap', platform: 'both', _origin: origin, sections: [{ name, items: list.filter((r) => r.action.type === 'keys').map(ruleToItem) }] });
  const mine = toPack('self-shortcuts', 'Your shortcuts', rules().filter((r) => r.enabled), 'self');
  if (mine.sections[0].items.length >= 2) out.push(mine);
  for (const t of allTemplates()) {
    const p = toPack(`tpl-${t.id}`, t.name, tplRules(t), 'template');
    if (p.sections[0].items.length >= 2) out.push(p);
  }
  return out;
}

// ------------------------------------------------------------------ demos the AI draws (safe SVG animations)

const SVG_TAGS = new Set(['svg', 'g', 'defs', 'title', 'desc', 'rect', 'circle', 'ellipse', 'line', 'polyline', 'polygon', 'path', 'text', 'tspan', 'linearGradient', 'radialGradient', 'stop',
  'clipPath', 'mask', 'use', 'symbol', 'marker', 'pattern', 'animate', 'animateTransform', 'animateMotion', 'set', 'mpath']);
const SVG_ATTRS = new Set(['id', 'x', 'y', 'x1', 'y1', 'x2', 'y2', 'cx', 'cy', 'r', 'rx', 'ry', 'width', 'height', 'd', 'points', 'fill', 'stroke', 'stroke-width', 'stroke-linecap',
  'stroke-linejoin', 'stroke-dasharray', 'stroke-dashoffset', 'stroke-opacity', 'fill-opacity', 'fill-rule', 'clip-rule', 'opacity', 'transform', 'viewBox', 'preserveAspectRatio',
  'font-size', 'font-family', 'font-weight', 'text-anchor', 'dominant-baseline', 'letter-spacing', 'offset', 'stop-color', 'stop-opacity', 'gradientUnits', 'gradientTransform',
  'spreadMethod', 'fx', 'fy', 'clip-path', 'mask', 'marker-start', 'marker-mid', 'marker-end', 'markerWidth', 'markerHeight', 'refX', 'refY', 'orient', 'patternUnits',
  'attributeName', 'attributeType', 'from', 'to', 'by', 'values', 'keyTimes', 'keySplines', 'calcMode', 'dur', 'begin', 'end', 'repeatCount', 'repeatDur', 'additive',
  'accumulate', 'type', 'rotate', 'path', 'keyPoints', 'visibility', 'display', 'xmlns', 'version']);

/** An AI-made demo, kept only if it's plain drawing and animation: no scripts, links, pictures or styles. */
function cleanDemo(str) {
  if (typeof str !== 'string' || str.length > 20000 || !/<svg[\s>]/i.test(str)) return null;
  const doc = new DOMParser().parseFromString(str, 'image/svg+xml');
  const root = doc.documentElement;
  if (!root || root.nodeName !== 'svg' || doc.getElementsByTagName('parsererror').length) return null;
  const clean = (el) => {
    for (const a of [...el.attributes]) {
      const n = a.name;
      const local = (n === 'href' || n === 'xlink:href') && /^#[\w-]+$/.test(a.value);
      if ((!SVG_ATTRS.has(n) && !local) || /^on/i.test(n) || /url\((?!\s*['"]?#)/i.test(a.value) || /javascript:|data:/i.test(a.value)) el.removeAttribute(n);
    }
    if (/^(animate|set)$/.test(el.nodeName) && /href|^on/i.test(el.getAttribute('attributeName') || '')) { el.remove(); return; }
    for (const kid of [...el.children]) {
      if (!SVG_TAGS.has(kid.nodeName)) kid.remove();
      else clean(kid);
    }
  };
  clean(root);
  const out = document.importNode(root, true);
  out.removeAttribute('width');
  out.removeAttribute('height');
  if (!out.getAttribute('viewBox')) out.setAttribute('viewBox', '0 0 240 120');
  out.setAttribute('class', 'demo-svg');
  out.setAttribute('aria-hidden', 'true');
  return out;
}

// ------------------------------------------------------------------ touch typing: fingers home, eyes up, speed comes by itself
//
// How it trains:
// * a few keys at a time, home row first (f j d k s l a), a new key only when every
//   key you have is accurate (95%) and quick (about 28 words a minute);
// * a mistake isn't skipped: you type the right key before moving on, so the
//   finger learns the right move, not the wrong one;
// * the lines lean on your weakest key (the slowest or most missed), so practice
//   goes where it's needed;
// * "no peeking": the cat lies on the keyboard picture, so your eyes stay on the text;
// * short rounds, sound on every key, a streak: the brain likes small wins.

const TY_KEY = 'catkey.typing.v1';
const TY_ORDER = 'fjdksla eirntouhgcmpwybvxqz,.'.replace(' ', '').split('');
const TY_START = 7;
const TY_TARGET_MS = 430; // per key: about 28 words a minute
const TY_FINGER = { q: 0, a: 0, z: 0, w: 1, s: 1, x: 1, e: 2, d: 2, c: 2, r: 3, f: 3, v: 3, t: 3, g: 3, b: 3, y: 4, h: 4, n: 4, u: 4, j: 4, m: 4, i: 5, k: 5, ',': 5, o: 6, l: 6, '.': 6, p: 7, ';': 7, '/': 7 };
const TY_FINGERS = ['left pinky', 'left ring finger', 'left middle finger', 'left index finger', 'right index finger', 'right middle finger', 'right ring finger', 'right pinky'];
const TY_WORDS = ('a as ask add dad fad fall falls flask salad lad lads all alas sad ads jak deal desk side slid kid kids idea ideas said sail life like likes file files fine find kind '
  + 'line lines dine skin sink link links land lands lake lakes sake jade fade fill skill still till field fields first fresh tide ride rides tried dries fries drift fit '
  + 'kit list fist dish fish wish this these that there their then than the they them think thin thing things into in it its is if of off on one ones out our ours '
  + 'are art arts and end ends ran run runs sun fun gun nut nuts tune turn turns hunt hunter under understand true truth tree trees three street strong string '
  + 'go good goods god dog dogs hog hug huge high hight light lights night nights right rights sight fight eight weight might mighty tonight hello hold holds '
  + 'cold old gold told come comes came camp map maps make makes made mode model more most much must music magic mouse cat cats chat chats catch match '
  + 'keys key keyboard short shortcut shortcuts paw paws purr purrs warrior warriors power powers word words work works world write writes wrote type types '
  + 'speed fast faster quick quickly brown fox jumps over lazy zebra quiz box boxes six mix fix next text vex very ever every never seven even view views '
  + 'we were wear west wet way ways with without why who what when where which while white whale page pages paper people place plan plans play please price').split(' ');

const TY = { on: false, line: '', pos: 0, wrongAt: -1, lineStart: 0, last: 0, right: 0, wrong: 0, streak: 0, best: 0, peek: true, started: 0, sad: 0, lines: 0 };

function tyDb() {
  try { const d = JSON.parse(localStorage.getItem(TY_KEY) || '{}'); return { open: Math.max(TY_START, d.open || TY_START), keys: d.keys || {}, best: d.best || 0, peek: d.peek !== false }; } catch { return { open: TY_START, keys: {}, best: 0, peek: true }; }
}
function tySave(db) { try { localStorage.setItem(TY_KEY, JSON.stringify(db)); } catch { /* fine */ } scheduleProgress(); }
const tyLetters = (db = tyDb()) => TY_ORDER.slice(0, db.open);

function tyWeakest(db = tyDb()) {
  let worst = null;
  let score = -1;
  for (const k of tyLetters(db)) {
    const s = db.keys[k] || { n: 0, miss: 0, ms: 900 };
    const v = (s.n < 8 ? 2 : 0) + (s.miss / Math.max(1, s.n)) * 6 + (s.ms || 900) / TY_TARGET_MS;
    if (v > score) { score = v; worst = k; }
  }
  return worst;
}

function tyNewLine() {
  const db = tyDb();
  const ok = new Set(tyLetters(db));
  let words = TY_WORDS.filter((w) => [...w].every((c) => ok.has(c)));
  if (words.length < 12) {
    // Too few real words yet: make pronounceable ones from the keys you have.
    const vowels = [...ok].filter((c) => 'aeiou'.includes(c));
    const cons = [...ok].filter((c) => !'aeiou,.'.includes(c));
    const pickc = (xs) => xs[Math.floor(Math.random() * xs.length)];
    for (let i = 0; i < 30; i++) {
      let w = '';
      const len = 2 + Math.floor(Math.random() * 4);
      for (let j = 0; j < len; j++) w += j % 2 ? pickc(vowels.length ? vowels : cons) : pickc(cons.length ? cons : vowels);
      words.push(w);
    }
  }
  const weak = tyWeakest(db);
  const withWeak = words.filter((w) => w.includes(weak));
  const out = [];
  while (out.join(' ').length < 44) {
    const src = withWeak.length && Math.random() < 0.5 ? withWeak : words;
    out.push(src[Math.floor(Math.random() * src.length)]);
  }
  TY.line = out.join(' ');
  TY.pos = 0;
  TY.wrongAt = -1;
  TY.lineStart = 0;
  TY.last = 0;
}

function startTyping() {
  stopRecording();
  const db = tyDb();
  Object.assign(TY, { on: true, right: 0, wrong: 0, streak: 0, best: 0, peek: db.peek, started: performance.now(), lines: 0 });
  tyNewLine();
  $('#app').classList.add('training');
  renderTyping(true);
}
/** Typed letters count towards today's practice (saved per line, not per key). */
function tyFlush(extraMs = 0) {
  if (!TY.pending && !extraMs) return;
  const db = trainDb();
  const day = (db.days[today()] ||= { ms: 0, answers: 0, ok: 0, real: 0 });
  day.typed = (day.typed || 0) + (TY.pending || 0);
  day.ms += extraMs;
  TY.pending = 0;
  trainSave(db);
  checkRankUp();
}
function typingStop() {
  if (!TY.on) return;
  TY.on = false;
  $('#app').classList.remove('training');
  tyFlush(TY.right ? performance.now() - TY.started : 0);
}

function tyKey(e) {
  if (!TY.on || S.view !== 'train') return;
  if (e.key === 'Escape') { e.preventDefault(); typingStop(); renderTrain(); return; }
  if (e.key.length !== 1 || e.ctrlKey || e.altKey || e.metaKey) return;
  e.preventDefault();
  e.stopPropagation();
  const want = TY.line[TY.pos];
  const got = e.key.toLowerCase();
  const now = performance.now();
  if (!TY.lineStart) TY.lineStart = now;
  const db = tyDb();
  if (got === want) {
    if (want !== ' ') {
      const s = (db.keys[want] ||= { n: 0, miss: 0, ms: 900 });
      s.n++;
      if (TY.last && TY.wrongAt !== TY.pos) s.ms = Math.round(s.ms * 0.8 + Math.min(3000, now - TY.last) * 0.2);
    }
    TY.right++;
    TY.streak++;
    TY.best = Math.max(TY.best, TY.streak);
    TY.pos++;
    TY.wrongAt = -1;
    TY.last = now;
    trainSounds.key();
    TY.pending = (TY.pending || 0) + 1;
    if (TY.pos >= TY.line.length) {
      tyFlush();
      const secs = (now - TY.lineStart) / 1000;
      const wpm = secs > 0 ? Math.round((TY.line.length / 5) / (secs / 60)) : 0;
      db.best = Math.max(db.best, wpm);
      TY.lines++;
      TY.lastWpm = wpm;
      sounds.mew();
      // a new key when every key so far is accurate and quick
      const ready = tyLetters(db).every((k) => { const s = db.keys[k]; return s && s.n >= 15 && s.miss / s.n <= 0.06 && s.ms <= TY_TARGET_MS; });
      if (ready && db.open < TY_ORDER.length) {
        db.open++;
        const k = TY_ORDER[db.open - 1];
        trainSounds.streak(3);
        toast(`New key: ${k.toUpperCase()}. Use your ${TY_FINGERS[TY_FINGER[k]]}.`, { ms: 6000 });
      }
      tySave(db);
      tyNewLine();
    } else tySave(db);
  } else {
    TY.wrong++;
    TY.streak = 0;
    if (TY.wrongAt !== TY.pos && want !== ' ') { const s = (db.keys[want] ||= { n: 0, miss: 0, ms: 900 }); s.miss++; s.n++; tySave(db); }
    TY.wrongAt = TY.pos;
    if (now - TY.sad > 700) { TY.sad = now; trainSounds.meow(); }
  }
  renderTyping();
}
document.addEventListener('keydown', tyKey, true);

function renderTyping(fresh = false) {
  const box = $('#train');
  if (!box || !TY.on) return;
  const db = tyDb();
  const open = new Set(tyLetters(db));
  if (fresh || !$('#ty-line')) {
    box.textContent = '';
    box.append(h('div', { class: 'tr-session ty' },
      h('div', { class: 'tr-head' },
        h('span', { class: 'pk-ico glyph', html: icon('keyboard') }),
        h('div', { class: 'txt' }, h('b', { class: 'display', text: 'Touch typing' }), h('span', { id: 'ty-sub', class: 'hint' })),
        h('div', { class: 'tr-score' }, h('b', { id: 'ty-wpm', class: 'display' }), h('span', { id: 'ty-acc', class: 'hint' })),
        h('span', { id: 'ty-streak', class: 'tr-streak' }),
        h('button', { class: `chipbtn${TY.peek ? '' : ' on'}`, id: 'ty-peek', type: 'button', html: `${icon('paw')}<span>No peeking</span>`, onclick: () => { TY.peek = !TY.peek; const d = tyDb(); d.peek = TY.peek; tySave(d); renderTyping(true); } }),
        h('button', { class: 'btn small ghost', type: 'button', html: `${icon('x')}<span>Stop</span>`, onclick: () => { typingStop(); renderTrain(); } })),
      h('div', { class: 'ty-card' }, h('div', { id: 'ty-line', class: 'ty-line', 'aria-live': 'off' }), h('div', { id: 'ty-hint', class: 'hint ty-hint' })),
      h('div', { id: 'ty-kb', class: `ty-kb${TY.peek ? '' : ' covered'}` }),
      h('input', { id: 'ty-input', class: 'sr', type: 'text', 'aria-label': 'Typing area', autocomplete: 'off', spellcheck: 'false' }),
      h('p', { class: 'hint', text: 'Eyes on the text, fingers on the bumps of F and J. Type the right key to move on. Esc stops.' })));
  }
  const line = $('#ty-line');
  line.textContent = '';
  [...TY.line].forEach((c, i) => line.append(h('span', { class: `ch${i < TY.pos ? ' done' : ''}${i === TY.pos ? ' cur' : ''}${i === TY.wrongAt ? ' bad' : ''}${c === ' ' ? ' sp' : ''}`, text: c === ' ' ? '·' : c })));
  const next = TY.line[TY.pos];
  $('#ty-hint').textContent = next && next !== ' ' ? `${next.toUpperCase()}: ${TY_FINGERS[TY_FINGER[next]]}` : next === ' ' ? 'Space: either thumb' : '';
  const secs = (performance.now() - TY.started) / 1000;
  $('#ty-wpm').textContent = `${TY.lastWpm || 0} wpm`;
  $('#ty-acc').textContent = `${TY.right + TY.wrong ? Math.round((100 * TY.right) / (TY.right + TY.wrong)) : 100}% right · best ${db.best} wpm`;
  $('#ty-sub').textContent = `${open.size} of ${TY_ORDER.length} keys · weakest: ${(tyWeakest(db) || '').toUpperCase()} · ${clock(secs * 1000)}`;
  const st = $('#ty-streak');
  st.textContent = TY.streak >= 10 ? `🔥 ${TY.streak}` : '';
  const kb = $('#ty-kb');
  kb.textContent = '';
  for (const row of ['qwertyuiop', 'asdfghjkl;', 'zxcvbnm,./']) {
    kb.append(h('div', { class: 'ty-row' }, ...[...row].map((k) => h('span', {
      class: `ty-k f${TY_FINGER[k] ?? 7}${open.has(k) ? '' : ' shut'}${k === next ? ' next' : ''}${k === 'f' || k === 'j' ? ' bump' : ''}`, text: k.toUpperCase() }))));
  }
  kb.append(h('div', { class: 'ty-row' }, h('span', { class: `ty-k space${next === ' ' ? ' next' : ''}`, text: '' })));
  if (!TY.peek) kb.append(h('div', { class: 'ty-cover' }, h('span', { text: `🐾 No peeking: ${cat()} is lying on it.` })));
  const inp = $('#ty-input');
  if (inp && document.activeElement !== inp) inp.focus({ preventScroll: true });
}

// ------------------------------------------------------------------ your data folder: everything in one place, easy to back up
//
// config.json, stats and the coach's counts are written by Rust. What the window keeps
// itself (training, typing, friends, rank) is mirrored to progress.json beside them, so
// the one folder really holds everything, and a backup of it brings everything back.

const PROGRESS_AT = 'catkey.progress.at';
let progressTimer = 0;
const readLS = (k) => { try { const v = localStorage.getItem(k); return v == null ? null : JSON.parse(v); } catch { return null; } };

function progressData() {
  return { format: 'catkey-progress', version: 1, savedAt: Date.now(), train: readLS(TRAIN_KEY), typing: readLS(TY_KEY), friends: readLS(FRIENDS_KEY),
    rank: readLS('catkey.rank'), trainSound: readLS('catkey.train.sound') };
}
function scheduleProgress() {
  clearTimeout(progressTimer);
  progressTimer = setTimeout(() => {
    const d = progressData();
    invoke('save_progress', { text: JSON.stringify(d) }).then(() => { try { localStorage.setItem(PROGRESS_AT, String(d.savedAt)); } catch { /* fine */ } }).catch(() => {});
  }, 2500);
}
/** On start: take progress.json when it came back from a backup, or is newer than what this window has. */
async function restoreProgress() {
  const p = await invoke('load_progress').catch(() => null);
  let d = null;
  try { d = p?.text ? JSON.parse(p.text) : null; } catch { d = null; }
  if (!d || d.format !== 'catkey-progress') { scheduleProgress(); return; }
  const localAt = Number(readLS(PROGRESS_AT) || 0);
  if (!p.restored && (d.savedAt || 0) <= localAt) return;
  const put = (k, v) => { try { if (v == null) localStorage.removeItem(k); else localStorage.setItem(k, JSON.stringify(v)); } catch { /* fine */ } };
  put(TRAIN_KEY, d.train);
  put(TY_KEY, d.typing);
  put(FRIENDS_KEY, d.friends);
  put('catkey.rank', d.rank);
  if (d.trainSound != null) put('catkey.train.sound', d.trainSound);
  put(PROGRESS_AT, d.savedAt || Date.now());
  TR.sound = readLS('catkey.train.sound') !== 0;
  if (p.restored) setTimeout(() => toast('Your backup is back: shortcuts, your cat, training, typing and stats.', { ms: 8000 }), 800);
}

const DATA_ABOUT = {
  'config.json': 'Your shortcuts, templates, packs, settings and your cat',
  'progress.json': 'Training, typing, friends and your rank',
  'coach.json': 'Shortcuts used and time saved or lost, per app',
  'stats.json': 'How often each of your shortcuts fired',
  'coach-dict.json': 'What apps call their commands (rebuilt by itself)',
  'catkey.log': 'A short log, for troubleshooting',
  'config.broken.json': 'A damaged settings file Cat Key set aside',
};
const fmtBytes = (n) => (n >= 1048576 ? `${(n / 1048576).toFixed(1)} MB` : n >= 1024 ? `${Math.round(n / 1024)} KB` : `${n} B`);

function dataSection() {
  const list = h('div', { class: 'datafiles' }, h('p', { class: 'hint', text: 'Reading the folder…' }));
  const path = h('code', { class: 'datapath-text', text: S.status.configDir || '%APPDATA%\\CatKey' });
  invoke('data_info').then((info) => {
    if (!info) return;
    path.textContent = info.folder;
    list.textContent = '';
    for (const f of info.files) {
      list.append(h('div', { class: 'datafile' },
        h('span', { class: 'nm' }, h('b', { text: f.name }), h('span', { class: 'hint', text: DATA_ABOUT[f.name] || '' })),
        h('span', { class: 'hint n', text: fmtBytes(f.bytes) }),
        h('span', { class: 'hint n', text: f.modified ? new Date(f.modified * 1000).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }) : '' })));
    }
    if (!info.files.length) list.append(h('p', { class: 'hint', text: 'Nothing saved yet.' }));
  }).catch(() => { list.textContent = ''; });
  return h('section', {}, h('h2', { text: 'Your data and backups' }),
    h('p', { class: 'hint', style: { margin: '0', fontSize: '13px' }, text: `Everything ${cat()} keeps about you lives in this one folder on your PC: shortcuts, templates, packs, settings, your cat, training and typing progress, friends and stats. All plain files: copy the folder anywhere to back it up.` }),
    h('div', { class: 'datapath' }, h('span', { html: icon('folder'), style: { display: 'inline-flex' } }), path,
      h('button', { class: 'tbtn', type: 'button', title: 'Copy the path', 'aria-label': 'Copy the folder path', html: icon('copy'),
        onclick: async () => { try { await navigator.clipboard.writeText(path.textContent); toast('Folder path copied.'); } catch { toast('Couldn’t copy the path.', { kind: 'warn' }); } } })),
    list,
    h('div', { class: 'grid2' },
      h('button', { class: 'btn strong', type: 'button', html: `${icon('folder')}<span>Open the folder</span>`, onclick: openDataFolder }),
      h('button', { class: 'btn', type: 'button', html: `${icon('up')}<span>Back up now…</span>`, onclick: backupNow }),
      h('button', { class: 'btn', type: 'button', html: `${icon('down')}<span>Restore from a backup…</span>`, onclick: restoreBackup })));
}

function openDataFolder() {
  invoke('open_data_folder').catch((e) => toast(String(e), { kind: 'warn' }));
}
async function backupNow() {
  clearTimeout(progressTimer);
  const d = progressData();
  await invoke('save_progress', { text: JSON.stringify(d) }).catch(() => {});
  try {
    const where = await invoke('backup_now');
    if (where) { sounds.saved(); toast(`Backed up to “${where}”.`, { ms: 8000 }); }
  } catch (e) { toast(String(e), { kind: 'err', ms: 8000 }); }
}
async function restoreBackup() {
  if (!(await confirmBox('Restore a backup?', `Pick a “Cat Key backup” folder. What you have now is kept in the data folder under “Before restore”, just in case. Then ${cat()} restarts with the backup.`, 'Pick the backup'))) return;
  clearTimeout(progressTimer);
  try {
    if (await invoke('restore_backup')) {
      toast('Restoring… Cat Key restarts in a moment.', { ms: 4000 });
      setTimeout(() => invoke('restart_app').catch(() => {}), 600);
    }
  } catch (e) { toast(String(e), { kind: 'err', ms: 9000 }); }
}
