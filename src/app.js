// Cat Key UI. Talks to the Rust side through `invoke` + events; everything the
// keyboard does happens in Rust, so this window can be closed at any time.

import * as K from './keys.js';
import { Mascot, SKINS, skinName } from './mascot.js';
import { sounds, setSoundsEnabled } from './sound.js';

const backend = window.__TAURI__
  ? { invoke: window.__TAURI__.core.invoke, listen: window.__TAURI__.event.listen }
  : (await import('./mock.js')).createMock();
const invoke = (cmd, args) => backend.invoke(cmd, args);

// ------------------------------------------------------------------ icons

const I = {
  gear: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
  minus: '<path d="M5 12h14"/>',
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
};
const FILLED = new Set(['paw', 'more', 'sparkle']);
const icon = (name, extra = '') =>
  `<svg class="i ${FILLED.has(name) ? 'fill' : ''} ${extra}" viewBox="0 0 24 24" aria-hidden="true">${I[name]}</svg>`;

// ------------------------------------------------------------------ DOM helpers

const $ = (sel, root = document) => root.querySelector(sel);
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
};
let hero = null; // hero Mascot
let REC = null; // active recording target
let E = null; // editor state

const cat = () => S.config?.settings?.catName || skinName(S.config?.settings?.skin);
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
const appNames = new Map(Object.entries(KNOWN_APPS));
function appName(exeOrPath) {
  if (!exeOrPath) return '';
  if (/^[a-z]+:\/\//i.test(exeOrPath)) return exeOrPath.replace(/^https?:\/\//i, '').replace(/\/$/, '');
  const file = exeOrPath.split(/[\\/]/).pop().toLowerCase();
  if (appNames.has(file)) return appNames.get(file);
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

// ------------------------------------------------------------------ boot

async function boot() {
  const st = await invoke('get_state');
  S.config = st.config;
  S.status = st.status;
  S.stats = st.stats || S.stats;
  setSoundsEnabled(S.config.settings.sounds);

  $('#btn-settings').innerHTML = icon('gear');
  $('#btn-min').innerHTML = icon('minus');
  $('#btn-close').innerHTML = icon('x');
  $('#ico-search').outerHTML = icon('search');
  $('#btn-new').innerHTML = `${icon('plus')}<span>New shortcut</span>`;
  $('.master .knob').innerHTML = icon('paw');

  hero = new Mascot($('#cushion'), { skin: S.config.settings.skin, size: 148, track: true, name: cat() });
  applyLook();
  renderAll();
  wire();
  await invoke('ui_ready').catch(() => {});

  if (S.status.firstRun) {
    setTimeout(() => toast(`${cat()} brought your ${rules().length} shortcuts over. They already work everywhere!`, { ms: 6000 }), 500);
  }
  if (S.status.configReset) {
    toast('Your settings file was damaged, so Cat Key started fresh (a copy was kept).', { kind: 'warn', ms: 8000 });
  }
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
  const key = (x) => JSON.stringify([x.powertoysRunning, x.admin, x.engineRunning, x.startup, x.problems]);
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
  for (const k of SKINS) app.classList.toggle(k.id, k.id === s.skin);
  hero?.setSkin(s.skin);
  hero?.setName(cat());
  E?.mini?.setSkin(s.skin);
}

function renderAll() {
  renderTitle();
  renderHero();
  renderBanners();
  renderFilters();
  renderList();
  renderSuggestions();
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
  $('#sub').textContent = on
    ? 'Alt now works like ⌘ on a Mac, in every app. Press one anywhere and ' + cat() + ' taps along.'
    : 'Every key works like normal until you wake ' + cat() + ' up.';
  const master = $('#master');
  master.setAttribute('aria-checked', String(on));
  master.querySelector('.switch').setAttribute('aria-checked', String(on));
  $('#master-label').textContent = on ? 'Awake' : 'Napping';
  $('#chip-today').innerHTML = `${icon('paw')}<b class="tick">${fmtCount(S.stats.today || 0)}</b> remaps today`;
  $('#chip-on').innerHTML = `${icon('keyboard')}${active} of ${rules().length} on`;
  $('#app').classList.toggle('napping', !on);
  hero.setMood(on ? 'awake' : 'nap');
}

function checkupRows() {
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
    rows.push({ level: s.hidePowertoysTip ? 'info' : 'warn', title: 'PowerToys Keyboard Manager is on', desc: 'Two remappers fight over the same keys. In PowerToys, open Keyboard Manager and switch it off. Your shortcuts already live here.', actions: [
      ['Open PowerToys', () => invoke('open_system_page', { page: 'powertoys' }).catch((e) => toast(String(e), { kind: 'warn' }))],
      ['Import its shortcuts', importPowerToys],
    ] });
  } else {
    rows.push({ level: 'ok', title: 'No other remapper is fighting for your keys', desc: 'Cat Key has the keyboard to itself.' });
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
    ? `${cat()} found ${issues.length === 1 ? 'one thing' : `${issues.length} things`} that need you.`
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
  const main = h('button', { class: 'card-main', type: 'button', 'aria-label': `Edit ${ruleName(r)}`, onclick: () => openEditor(r) },
    h('span', { class: 'from' }, capsEl(r.trigger)),
    h('span', { class: 'arrow', html: '<svg viewBox="0 0 40 16" width="40" height="16" aria-hidden="true"><circle class="dot" cx="4" cy="8" r="1.9" fill="currentColor" opacity=".45"/><circle class="dot" cx="11" cy="8" r="1.9" fill="currentColor" opacity=".75"/><path d="M17 8h17M29 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg>' }),
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
    new Mascot(m, { skin: S.config.settings.skin, size: 84 }).setMood(rules().length ? 'awake' : 'nap');
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

function renderSuggestions() {
  const box = $('#suggest');
  box.textContent = '';
  const have = rules().map((r) => r.trigger);
  const list = SUGGESTIONS.filter(([from]) => !have.some((t) => K.overlaps(t, from))).slice(0, 6);
  if (!list.length) return;
  box.append(h('h2', { html: `${icon('sparkle')}<span></span>` }));
  box.querySelector('h2 span').textContent = `${cat()} suggests`;
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
  $('#btn-min').onclick = () => invoke('window_minimize');
  $('#btn-close').onclick = () => invoke('window_close');
  $('#btn-new').onclick = () => openEditor(null);
  $('#master').onclick = () => setMaster(!S.config.enabled);
  $('#cushion').onclick = () => {
    if (!S.config.enabled) {
      setMaster(true);
      return;
    }
    hero.react(1500);
    sounds.mew();
    const lines = ['Mrrp?', 'Purr…', 'Hehe, that tickles!', `Need a new trick? Press New shortcut.`, '*happy tail wiggle*', 'Nya~'];
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
  $('#scrim').onclick = () => { if (E) closeEditor(); else if (SETTINGS_OPEN) closeSettings(); };

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
      if (SETTINGS_OPEN) { closeSettings(); return; }
    }
    const ctrl = e.ctrlKey && !e.altKey && !e.metaKey;
    if (ctrl && e.key.toLowerCase() === 'f' && !E && !SETTINGS_OPEN) { e.preventDefault(); search.focus(); search.select(); }
    if (ctrl && e.key.toLowerCase() === 'n' && !E && !SETTINGS_OPEN) { e.preventDefault(); openEditor(null); }
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

function openEditor(rule) {
  if (SETTINGS_OPEN) closeSettings();
  const d = rule ? clone(rule) : { id: newId(), enabled: true, name: '', trigger: '', action: { type: 'keys', keys: '' }, scope: { mode: 'all', apps: [] } };
  d.scope = d.scope || { mode: 'all', apps: [] };
  d.scope.apps = d.scope.apps || [];
  E = {
    d, isNew: !rule, err: {}, rec: null, live: '', manual: { trigger: false, keys: false }, apps: null,
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
  E.mini = new Mascot(mini, { skin: S.config.settings.skin, size: 76 });
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
    h('span', { class: 'hint', text: kind === 'trigger' ? 'Windows keeps Win + L and Ctrl + Alt + Del for itself.' : 'What Windows will see instead.' }),
    h('button', { class: 'link', type: 'button', 'aria-expanded': String(E.manual[kind]), text: E.manual[kind] ? 'Hide fine-tuning' : 'Fine-tune by hand',
      onclick: () => { E.manual[kind] = !E.manual[kind]; renderRecorderSection(kind); } })));
  if (E.manual[kind]) box.append(manualEl(kind));
  if (E.err[kind]) box.append(h('div', { class: 'err', role: 'alert', text: E.err[kind] }));
  E.justRecorded = null;
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
        toast('Add a regular key too, like Alt + C.', { kind: 'warn' });
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
  document.body.append(pop);
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

function ruleMenu(anchor, r) {
  const item = (ic, text, fn, cls = '') => h('button', { class: `item ${cls}`, type: 'button', role: 'menuitem', onclick: () => { closePop(); fn(); } }, h('span', { html: icon(ic) }), h('span', { class: 'nm', text }));
  showPop(anchor, h('div', { class: 'pop', role: 'menu' },
    item('edit', 'Edit', () => openEditor(r)),
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
    document.body.append(scrim, box);
    box.querySelector('.btn.ghost').focus();
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
    document.body.append(p);
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

  const skins = h('div', { class: 'skins', role: 'radiogroup', 'aria-label': 'Cat skin' });
  for (const k of SKINS) {
    const holder = h('span', {});
    const btn = h('button', { class: 'skin', type: 'button', role: 'radio', 'aria-checked': String(s.skin === k.id), onclick: () => {
      const oldDefault = skinName(s.skin);
      const patch = { skin: k.id };
      if (!s.catName || s.catName === oldDefault) patch.catName = k.name;
      sounds.mew();
      commitSettings(patch).then(() => { hero.react(1000); renderHero(); renderSuggestions(); });
    } }, holder, h('b', { text: k.name }), h('span', { text: k.fur }));
    skins.append(btn);
    settingMinis.push(new Mascot(holder, { skin: k.id, size: 74 }));
  }
  const nameInput = h('input', { class: 'field', type: 'text', maxlength: '24', value: s.catName || skinName(s.skin), 'aria-label': "Cat's name", style: { width: '180px', height: '40px' },
    onchange: (e) => { commitSettings({ catName: e.target.value.trim() || skinName(s.skin) }).then(() => { renderHero(); renderSuggestions(); }); } });
  const theme = h('div', { class: 'seg soft', role: 'group', 'aria-label': 'Theme' },
    ...[['auto', 'Match Windows'], ['light', 'Light'], ['dark', 'Dark']].map(([v, l]) => h('button', { type: 'button', 'aria-pressed': String(s.theme === v), text: l, onclick: () => commitSettings({ theme: v }) })));

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

  const adminDesc = S.status.admin
    ? 'On. Cat Key is running as administrator, so shortcuts also work in Task Manager and admin terminals.'
    : 'Runs as administrator so shortcuts also work in Task Manager and admin terminals. Windows asks you once.';

  d.append(
    h('div', { class: 'drawer-head' }, h('h1', { id: 'st-title', text: 'Settings' }), h('button', { class: 'tbtn', 'aria-label': 'Close settings', html: icon('x'), onclick: closeSettings })),
    h('div', { class: 'drawer-body' },
      h('section', {}, h('div', { class: 'row-between' }, h('h2', { text: 'Checkup' }),
        h('button', { class: 'link', type: 'button', text: 'Check again', onclick: () => { refreshStatusNow(); toast(`${cat()} checked everything again.`); } })),
        checkupEl(checkupRows())),
      h('section', {}, h('h2', { text: 'Your cat' }), skins,
        h('label', { class: 'row-between', style: { fontWeight: '800' } }, "Cat's name", nameInput),
        h('div', { class: 'row-between', style: { fontWeight: '800' } }, 'Theme', theme)),
      h('section', { style: { gap: '0' } }, h('h2', { text: 'How it behaves', style: { marginBottom: '4px' } }),
        settingRow('Start with Windows', 'Wakes up quietly in the tray when you sign in.', switchEl(s.launchAtStartup, 'Start with Windows', () => commitSettings({ launchAtStartup: !s.launchAtStartup }).then(() => setTimeout(refreshStatusNow, 900)))),
        settingRow('Work in admin apps', adminDesc, switchEl(s.adminMode, 'Work in admin apps', () => setAdmin(!s.adminMode))),
        settingRow('Cute sounds', 'Soft little blips when you flip a switch.', switchEl(s.sounds, 'Cute sounds', () => { commitSettings({ sounds: !s.sounds }); setSoundsEnabled(!s.sounds); sounds.on(); })),
        settingRow('Pause hotkey', 'One shortcut that pauses every remap. Handy for games.', pauseCtl)),
      h('section', {}, h('h2', { text: 'Nap zones' }),
        h('p', { class: 'hint', style: { margin: '0', fontSize: '13px' }, text: `${cat()} naps while these apps are in front, so every key works as normal there.` }), nap),
      h('section', {}, h('h2', { text: 'Your shortcuts' }),
        h('div', { class: 'grid2' },
          h('button', { class: 'btn', type: 'button', html: `${icon('down')}<span>Import from PowerToys</span>`, onclick: importPowerToys }),
          h('button', { class: 'btn', type: 'button', html: `${icon('folder')}<span>Import file…</span>`, onclick: async () => {
            try { const r = await invoke('import_file'); if (r) { S.config = r.config; renderAll(); toast(r.added ? `Added ${r.added} shortcut${r.added === 1 ? '' : 's'}.` : 'Nothing new in that file.'); } } catch (e) { toast(String(e), { kind: 'warn' }); }
          } }),
          h('button', { class: 'btn', type: 'button', html: `${icon('up')}<span>Export…</span>`, onclick: async () => { try { if (await invoke('export_file')) toast('Exported. Keep it somewhere safe.'); } catch (e) { toast(String(e), { kind: 'err' }); } } }),
          h('button', { class: 'btn', type: 'button', html: `${icon('refresh')}<span>Restore defaults</span>`, onclick: async () => {
            if (!(await confirmBox('Restore the starter shortcuts?', 'This replaces your shortcut list with the original 11 Alt shortcuts. Settings stay as they are.', 'Restore', true))) return;
            S.config = await invoke('restore_defaults');
            renderAll();
            toast('Starter shortcuts are back.');
          } })),
        h('p', { class: 'hint', style: { margin: '2px 0 0' } }, 'Everything is saved in ', h('b', { text: S.status.configDir || '%APPDATA%\\CatKey' }), '.'))),
    h('div', { class: 'drawer-foot' },
      h('span', {}, `Cat Key ${S.status.version || ''} · `, h('button', { class: 'link', type: 'button', text: 'MasterfullyClaw/CatKey', onclick: () => invoke('open_link', { url: 'https://github.com/MasterfullyClaw/CatKey' }) })),
      h('button', { class: 'btn small', type: 'button', text: 'Quit Cat Key', onclick: async () => {
        if (await confirmBox('Quit Cat Key?', 'Your shortcuts stop working until Cat Key starts again.', 'Quit', true)) invoke('quit_app');
      } })));
  const body = $('.drawer-body', d);
  if (body) body.scrollTop = scrollTop;
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
