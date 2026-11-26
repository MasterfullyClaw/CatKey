// The app switcher's panel. Rust watches the pointer, lists the windows and places this
// window at the corner; this page shows the list, dressed like the cat you picked, and
// jumps to the one you click (or pick with the keys, when a shortcut opened it).

import { Mascot } from './mascot.js';

const T = window.__TAURI__;
const invoke = (cmd, args) => (T ? T.core.invoke(cmd, args) : (window.__swMock?.(cmd, args) ?? Promise.resolve(null)));
const listen = (ev, fn) => (T ? T.event.listen(ev, fn) : window.__swListen?.(ev, fn));
const $ = (s) => document.querySelector(s);
const root = $('#sw');
const list = $('#list');
const search = $('#search');

function h(tag, props, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'text') el.textContent = v;
    else if (k === 'html') el.innerHTML = v;
    else if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2).toLowerCase(), v);
    else if (k === 'style' && typeof v === 'object') Object.assign(el.style, v);
    else el.setAttribute(k, v === true ? '' : String(v));
  }
  for (const kid of kids.flat(Infinity)) if (kid != null && kid !== false) el.append(kid instanceof Node ? kid : document.createTextNode(String(kid)));
  return el;
}

const PAW = '<svg class="sw-paw" viewBox="0 0 24 24" aria-hidden="true"><ellipse cx="12" cy="16" rx="5" ry="4"/><circle cx="6.5" cy="10.5" r="2.2"/><circle cx="10" cy="7" r="2.2"/><circle cx="14" cy="7" r="2.2"/><circle cx="17.5" cy="10.5" r="2.2"/></svg>';
const X = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18"/></svg>';
const PALETTE = ['#4F6FE0', '#0F8A7A', '#C0500C', '#7C3AED', '#C0266D', '#0A6CA8', '#4D7C0F', '#A85B07'];
const colorFor = (s) => {
  let n = 0;
  for (const c of s) n = (n * 31 + c.charCodeAt(0)) >>> 0;
  return PALETTE[n % PALETTE.length];
};

const S = { gen: 0, how: 'spot', at: 'top-left', items: [], icons: {}, platform: 'windows', sel: -1, q: '', shown: [] };

// ------------------------------------------------------------------ the cat, dressed like the app

const cat = new Mascot($('#cat'), { size: 46, track: true });
function readUi() {
  try { return JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { return {}; }
}
function applyUi() {
  const u = readUi();
  root.classList.toggle('dark', u.dark !== false);
  root.classList.toggle('light', u.dark === false);
  for (const s of ['mochi', 'kuro', 'tora', 'sora']) root.classList.toggle(s, (u.skin || 'mochi') === s);
  cat.setLook(u.look || { skin: 'mochi', gear: [] });
  cat.setName(u.name || 'Mochi');
}
window.addEventListener('storage', (e) => { if (e.key === 'catkey.ui.now') applyUi(); });
applyUi();

// ------------------------------------------------------------------ what each row says

const plain = (s) => (s || '').replace(/[​-‏⁠﻿]/g, '').trim();
/** "Inbox - Outlook" next to the app's own name is just "Inbox". */
function shortTitle(title, app) {
  const t = plain(title);
  const a = plain(app);
  if (!a || t.toLowerCase() === a.toLowerCase()) return t;
  for (const sep of [' - ', ' — ', ' – ', ' | ']) {
    const tail = sep + a;
    if (t.toLowerCase().endsWith(tail.toLowerCase())) {
      const rest = t.slice(0, t.length - tail.length).trim();
      if (rest) return rest;
    }
  }
  return t;
}
const iconKey = (it) => (it.path ? it.path.toLowerCase() : `w:${it.id}`);
const isMac = () => S.platform === 'mac';

function lines(it) {
  const app = plain(it.app) || 'Window';
  const title = shortTitle(it.title, app);
  const main = title && title.toLowerCase() !== app.toLowerCase() ? title : app;
  const bits = [];
  if (main !== app) bits.push(app);
  if (it.min) bits.push(isMac() ? 'not on screen' : 'minimised');
  return { main, sub: bits.join(' · ') };
}

function iconEl(it) {
  const url = S.icons[iconKey(it)];
  if (url) return h('img', { class: 'sw-ico', src: url, alt: '' });
  const name = plain(it.app) || plain(it.title) || '?';
  return h('span', { class: 'sw-ico letter', 'data-key': iconKey(it), style: { background: colorFor(name.toLowerCase()) }, text: name[0].toUpperCase() });
}

function matches(it, q) {
  if (!q) return true;
  const hay = `${plain(it.title)} ${plain(it.app)}`.toLowerCase();
  return q.toLowerCase().split(/\s+/).filter(Boolean).every((w) => hay.includes(w));
}

function marked(text, q) {
  const words = q.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return [text];
  const low = text.toLowerCase();
  const out = [];
  let i = 0;
  while (i < text.length) {
    let hit = null;
    for (const w of words) if (low.startsWith(w, i) && (!hit || w.length > hit.length)) hit = w;
    if (hit) { out.push(h('mark', { text: text.slice(i, i + hit.length) })); i += hit.length; } else {
      let j = i + 1;
      while (j < text.length && !words.some((w) => low.startsWith(w, j))) j++;
      out.push(text.slice(i, j));
      i = j;
    }
  }
  return out;
}

// ------------------------------------------------------------------ drawing

function render() {
  const keys = S.how === 'keys';
  root.dataset.how = keys ? 'keys' : 'spot';
  $('#search-wrap').hidden = !keys;
  $('#title').textContent = isMac() ? 'Your apps' : 'Your windows';
  const shown = S.items.filter((it) => matches(it, S.q));
  S.shown = shown;
  $('#count').textContent = String(shown.length);
  $('#sub').textContent = keys
    ? (S.q ? `${shown.length} found` : 'Pick one, or type to find it')
    : (isMac() ? 'Click one to bring it to the front' : 'Click one to jump to it');
  if (S.sel >= shown.length) S.sel = shown.length - 1;
  list.textContent = '';
  if (!shown.length) {
    list.append(h('div', { class: 'sw-empty' },
      h('b', { text: S.q ? 'Nothing by that name' : (isMac() ? 'No other apps open' : 'No other windows open') }),
      h('span', { text: S.q ? 'Try another word.' : 'Everything is tidy. Purr.' })));
  }
  shown.forEach((it, i) => {
    const { main, sub } = lines(it);
    const row = h('div', {
      class: `sw-row${i === S.sel ? ' sel' : ''}${it.min ? ' min' : ''}`, role: 'option', 'aria-selected': String(i === S.sel),
      'data-id': it.id, style: { '--i': Math.min(i, 10) }, title: plain(it.title) || plain(it.app),
      onclick: () => jump(it), onmouseenter: () => select(i, false),
      onauxclick: (e) => { if (e.button === 1 && it.canClose) { e.preventDefault(); closeIt(it, row); } },
    },
    iconEl(it),
    h('span', { class: 'sw-tx' }, h('b', {}, marked(main, S.q)), sub ? h('small', {}, marked(sub, S.q)) : null),
    it.front ? h('span', { class: 'sw-badge', text: 'now' }) : null,
    keys && i < 9 && !S.q ? h('span', { class: 'sw-key', text: String(i + 1) }) : null,
    h('span', { html: PAW, style: { display: 'contents' } }),
    it.canClose ? h('button', {
      class: 'sw-x', type: 'button', html: X, 'aria-label': isMac() ? `Quit ${plain(it.app)}` : `Close ${main}`,
      title: isMac() ? 'Quit the app' : 'Close the window',
      onclick: (e) => { e.stopPropagation(); closeIt(it, row); },
    }) : null);
    list.append(row);
  });
  const foot = $('#foot');
  foot.textContent = '';
  if (keys) {
    foot.append(h('kbd', { text: '↑' }), h('kbd', { text: '↓' }), ' pick ·', h('kbd', { text: 'Enter' }), ' jump ·', h('kbd', { text: 'Esc' }), ' close');
  } else {
    foot.append(isMac() ? 'Click to bring it forward · move away to close' : 'Click to jump · middle-click closes · move away to close');
  }
}

function select(i, scroll = true) {
  if (i === S.sel) return;
  S.sel = i;
  list.querySelectorAll('.sw-row').forEach((r, n) => {
    r.classList.toggle('sel', n === i);
    r.setAttribute('aria-selected', String(n === i));
  });
  if (scroll) list.querySelectorAll('.sw-row')[i]?.scrollIntoView({ block: 'nearest' });
}

function origin(at) {
  const x = at.includes('left') ? '0%' : at.includes('right') ? '100%' : '50%';
  const y = at.startsWith('top') ? '0%' : at.startsWith('bottom') ? '100%' : at === 'center' ? '30%' : '50%';
  root.style.setProperty('--ox', x);
  root.style.setProperty('--oy', y);
  root.dataset.side = x === '0%' ? 'left' : x === '100%' ? 'right' : 'center';
}

// ------------------------------------------------------------------ open, update, close

function open(p) {
  if (!p) return;
  S.gen = p.gen;
  S.how = p.how;
  S.at = p.at || 'top-left';
  S.items = p.items || [];
  S.icons = { ...S.icons, ...(p.icons || {}) };
  S.platform = p.platform || 'windows';
  S.q = '';
  search.value = '';
  // With the keys, start on the window before this one (like Alt+Tab); with the mouse, on none.
  S.sel = S.how === 'keys' ? Math.max(0, S.items.findIndex((it) => !it.front)) : -1;
  if (S.how === 'keys' && !S.items.length) S.sel = -1;
  origin(S.at);
  root.classList.remove('on', 'off');
  render();
  void root.offsetWidth;
  root.classList.add('on');
  cat.setMood('happy');
  setTimeout(() => cat.setMood('awake'), 900);
  invoke('switch_ready', { gen: p.gen });
  if (S.how === 'keys') setTimeout(() => search.focus(), 30);
}

function update(p) {
  if (!p) return;
  S.items = p.items || [];
  S.icons = { ...S.icons, ...(p.icons || {}) };
  render();
}

function addIcons(map) {
  Object.assign(S.icons, map || {});
  for (const [key, url] of Object.entries(map || {})) {
    list.querySelectorAll(`.sw-ico.letter[data-key="${CSS.escape(key)}"]`).forEach((e) => e.replaceWith(h('img', { class: 'sw-ico', src: url, alt: '' })));
  }
}

function close() {
  root.classList.remove('on');
  root.classList.add('off');
}

function jump(it) {
  if (!it) return;
  close();
  invoke('switch_to', { id: it.id, reopen: !!(isMac() && it.min) });
}

function closeIt(it, row) {
  row?.classList.add('closing');
  invoke('switch_close_window', { id: it.id });
}

// ------------------------------------------------------------------ the keys (opened with a shortcut)

search.addEventListener('input', () => {
  S.q = search.value.trim();
  S.sel = 0;
  render();
});

document.addEventListener('keydown', (e) => {
  if (S.how !== 'keys') return;
  const n = S.shown.length;
  if (e.key === 'Escape') {
    e.preventDefault();
    if (S.q) { search.value = ''; S.q = ''; S.sel = 0; render(); } else { close(); invoke('switch_hide'); }
  } else if (e.key === 'ArrowDown' || (e.key === 'Tab' && !e.shiftKey)) {
    e.preventDefault();
    if (n) select((S.sel + 1) % n);
  } else if (e.key === 'ArrowUp' || (e.key === 'Tab' && e.shiftKey)) {
    e.preventDefault();
    if (n) select((S.sel - 1 + n) % n);
  } else if (e.key === 'Enter') {
    e.preventDefault();
    jump(S.shown[Math.max(0, S.sel)]);
  } else if (/^[1-9]$/.test(e.key) && !S.q) {
    e.preventDefault();
    jump(S.shown[Number(e.key) - 1]);
  } else if (e.key === 'Delete' && S.shown[S.sel]?.canClose && !S.q) {
    e.preventDefault();
    closeIt(S.shown[S.sel], list.querySelectorAll('.sw-row')[S.sel]);
  }
});

// ------------------------------------------------------------------ from Rust

listen('ck://switch-open', (e) => open(e.payload));
listen('ck://switch-items', (e) => update(e.payload));
listen('ck://switch-icons', (e) => addIcons(e.payload));
listen('ck://switch-close', () => close());
invoke('switch_state').then((p) => { if (p) { open(p); } });
