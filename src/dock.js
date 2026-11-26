// The Cat Dock: the cat ball on the screen edge and the little panel it opens into.
// Rust places the window, catches drops, watches the clipboard and runs the timer;
// this page shows it all, dressed like the cat you picked in Cat Key.

import { Mascot } from './mascot.js';
import { applyVars } from './theme.js';
import { sounds, setSoundsEnabled, setSoundParts } from './sound.js';

const T = window.__TAURI__;
const invoke = (cmd, args) => (T ? T.core.invoke(cmd, args) : (window.__dockMock?.(cmd, args) ?? Promise.resolve(null)));
const listen = (ev, fn) => (T ? T.event.listen(ev, fn) : window.__dockListen?.(ev, fn));
const $ = (s) => document.querySelector(s);
const root = $('#dock');

const I = {
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  pin: '<path d="M9 4h6l-1 6 4 3v2H6v-2l4-3z"/><path d="M12 15v6"/>',
  copy: '<rect x="8" y="8" width="12" height="12" rx="2.5"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>',
  folder: '<path d="M4 7a2 2 0 0 1 2-2h4l2 2h6a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z"/>',
  open: '<path d="M14 4h6v6M20 4l-9 9"/><path d="M18 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4"/>',
  trash: '<path d="M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3"/>',
  play: '<path d="M8 5v14l11-7z"/>',
  pause: '<path d="M7 5h3v14H7zM14 5h3v14h-3z"/>',
  move: '<path d="M12 3v12M7 10l5 5 5-5M4 19h16"/>',
  back: '<path d="M12 21V9M7 14l5-5 5 5M4 5h16"/>',
  bowl: '<path d="M3 11h18a9 9 0 0 1-18 0z"/><path d="M8 7c0-1.5 1-2 2-2M14 7c0-1.5 1-2 2-2"/>',
};
const FILLED = new Set(['play', 'pause']);
const icon = (n) => `<svg class="i${FILLED.has(n) ? ' fill' : ''}" viewBox="0 0 24 24" aria-hidden="true">${I[n]}</svg>`;

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

const D = { info: null, open: false, tab: 'bowl', q: '', vitals: null, hard: 0, name: 'Mochi', purring: false, volume: 45, settings: {} };

// ------------------------------------------------------------------ the cat, dressed like the app

const ballCat = new Mascot($('#ball-cat'), { size: 92 });
const panelCat = new Mascot($('#panel-cat'), { size: 46, track: true });
function readUi() {
  try { return JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { return {}; }
}
function applyUi() {
  const u = readUi();
  root.classList.toggle('dark', u.dark !== false);
  root.classList.toggle('light', u.dark === false);
  for (const s of ['mochi', 'kuro', 'tora', 'sora']) root.classList.toggle(s, (u.skin || 'mochi') === s);
  applyVars(root, u.vars);
  const look = u.look || { skin: 'mochi', gear: [] };
  ballCat.setLook(look);
  panelCat.setLook(look);
  D.name = u.name || 'Mochi';
  ballCat.setName(D.name);
  panelCat.setName(D.name);
  $('#dock-name').textContent = D.name;
  setSoundsEnabled(u.sounds !== false);
  setSoundParts(u.soundParts || {});
  // the paws that reach out to catch things wear the cat's colours
  const css = getComputedStyle(ballCat.svg);
  for (const v of ['--mk-fur', '--mk-line']) $('#ball').style.setProperty(v, css.getPropertyValue(v));
}
window.addEventListener('storage', (e) => { if (e.key === 'catkey.ui.now') applyUi(); });

let toastTimer = 0;
function toast(text) {
  const t = $('#toast');
  t.textContent = text;
  t.classList.add('on');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove('on'), 2200);
}
const fail = (e) => toast(String(e?.message || e || 'That didn’t work.'));
const catSound = () => sounds.caught();

// ------------------------------------------------------------------ the ball

const ball = $('#ball');
ball.addEventListener('pointerenter', () => { if (!D.open) invoke('dock_peek', { out: true }); });
ball.addEventListener('pointerleave', () => { if (!D.open) invoke('dock_peek', { out: false }); });
// Rust follows the pointer while the button is down: a drag moves the cat, a click opens it.
ball.addEventListener('pointerdown', (e) => { if (e.button === 0) { e.preventDefault(); invoke('dock_press'); } });
ball.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); invoke('dock_open', { open: true }); } });
ball.addEventListener('click', (e) => e.preventDefault());

function setOpen(open) {
  D.open = open;
  root.classList.toggle('open', open);
  $('#panel').hidden = !open;
  if (open) { renderPanel(); }
}

// Something is dragged over the cat: it reaches out, and bounces when it catches it.
function dragPhase(phase) {
  const wrap = $('#ball-wrap');
  wrap.classList.toggle('catch', phase === 'enter');
  $('#panel').classList.toggle('catch', phase === 'enter');
  if (phase === 'drop') {
    wrap.classList.remove('caught');
    void wrap.offsetWidth;
    wrap.classList.add('caught');
    ballCat.react(900);
    catSound();
    setTimeout(() => { wrap.classList.remove('caught'); if (!D.open) invoke('dock_peek', { out: false }); }, 900);
  }
  if (phase === 'leave' && !D.open) setTimeout(() => invoke('dock_peek', { out: false }), 300);
}

// ------------------------------------------------------------------ the panel

const TABS = [
  ['bowl', '🥣', 'Bowl'],
  ['litter', '📦', 'Litter box'],
  ['yarn', '🧶', 'Copied'],
  ['focus', '⏰', 'Focus'],
  ['pc', '🐾', 'PC'],
];
function renderTabs() {
  const items = D.info?.items || [];
  const counts = { bowl: items.filter((x) => !x.cut).length, litter: items.filter((x) => x.cut).length, yarn: (D.info?.clips || []).length };
  const nav = $('#tabs');
  nav.textContent = '';
  for (const [id, em, label] of TABS) {
    if (id === 'pc' && D.settings.vitals === false) continue;
    nav.append(h('button', { class: 'dk-tab', type: 'button', role: 'tab', 'aria-selected': String(D.tab === id), title: label,
      onclick: () => { D.tab = id; invoke('dock_tab', { tab: id }); renderPanel(); } },
      h('span', { class: 'dk-em', text: em }), h('span', { text: label }), counts[id] ? h('span', { class: 'dk-n', text: String(counts[id]) }) : null));
  }
}

function subText() {
  const t = D.info?.timer;
  if (t && t.state !== 'idle') return `${t.state === 'paused' ? 'Paused' : 'Focus'} · ${fmtClock(timerLeft(t))} left`;
  if (D.purring) return 'Purring ♪';
  return { bowl: 'Drop on me', litter: 'Moves when dragged', yarn: 'What you copied', focus: 'One thing at a time', pc: 'Your computer' }[D.tab] || '';
}
function renderHead() {
  $('#dock-sub').textContent = subText();
  const purr = $('#purr');
  purr.textContent = '';
  purr.append(
    h('button', { class: `dk-pp${D.purring ? ' on' : ''}`, type: 'button', title: D.purring ? 'Stop the purr' : 'Purr in the background', 'aria-label': D.purring ? 'Stop the purr' : 'Start the purr',
      html: icon(D.purring ? 'pause' : 'play'), onclick: () => invoke('purr_set', { on: !D.purring }).then((on) => { D.purring = !!on; paintPurr(); }).catch(fail) }),
    h('span', { class: 'dk-lbl', text: 'Purr' }),
    h('input', { type: 'range', min: '0', max: '100', step: '5', value: String(D.volume), 'aria-label': 'Purr volume',
      onchange: (e) => { D.volume = Number(e.target.value); invoke('dock_purr_volume', { volume: D.volume }); } }));
  $('#pin').innerHTML = icon('pin');
  $('#pin').setAttribute('aria-pressed', String(!!D.info?.pinned));
  $('#close').innerHTML = icon('x');
}
$('#pin').onclick = () => { const on = !D.info?.pinned; if (D.info) D.info.pinned = on; invoke('dock_pin', { pinned: on }); $('#pin').setAttribute('aria-pressed', String(on)); toast(on ? 'Stays open until you close it' : 'Tucks in when you click elsewhere'); };
$('#close').onclick = () => invoke('dock_open', { open: false });
document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && D.open) invoke('dock_open', { open: false }); });

function paintPurr() {
  root.classList.toggle('purring', D.purring);
  if (D.open) renderHead();
}

function renderPanel() {
  if (!D.open) return;
  renderHead();
  renderTabs();
  const body = $('#body');
  const keep = body.scrollTop;
  body.textContent = '';
  if (D.tab === 'bowl' || D.tab === 'litter') renderShelf(body, D.tab === 'litter');
  else if (D.tab === 'yarn') renderYarn(body);
  else if (D.tab === 'focus') renderFocus(body);
  else renderPc(body);
  body.scrollTop = keep;
}

// ------------------------------------------------------------------ the bowl and the litter box

const EXT = { png: '🖼️', jpg: '🖼️', jpeg: '🖼️', gif: '🖼️', webp: '🖼️', svg: '🖼️', heic: '🖼️', mp4: '🎬', mov: '🎬', mkv: '🎬', avi: '🎬', webm: '🎬',
  mp3: '🎵', wav: '🎵', flac: '🎵', m4a: '🎵', pdf: '📕', doc: '📄', docx: '📄', txt: '📄', md: '📄', rtf: '📄', xls: '📊', xlsx: '📊', csv: '📊', ppt: '📽️', pptx: '📽️',
  zip: '🗜️', rar: '🗜️', '7z': '🗜️', exe: '⚙️', msi: '⚙️', dmg: '💿', app: '🧩', psd: '🎨', ai: '🎨', fig: '🎨', blend: '🧊', prproj: '🎬', aep: '🎬' };
const baseName = (p) => String(p).replace(/[\\/]+$/, '').split(/[\\/]/).pop();
const folderOf = (p) => { const s = String(p).replace(/[\\/]+$/, ''); const i = Math.max(s.lastIndexOf('\\'), s.lastIndexOf('/')); return i > 0 ? s.slice(0, i) : s; };
const fileEmoji = (p) => EXT[baseName(p).split('.').pop().toLowerCase()] || (baseName(p).includes('.') ? '📄' : '📁');
function ago(sec) {
  const m = Math.max(0, Math.round((Date.now() / 1000 - sec) / 60));
  return m < 1 ? 'just now' : m < 60 ? `${m} min ago` : m < 1440 ? `${Math.round(m / 60)} h ago` : `${Math.round(m / 1440)} d ago`;
}
const iconCache = new Map();
function fileIcon(path, el) {
  if (D.info?.platform === 'mac') return;
  const k = path.toLowerCase();
  if (iconCache.has(k)) { const u = iconCache.get(k); if (u) el.replaceChildren(h('img', { src: u, alt: '' })); return; }
  iconCache.set(k, null);
  invoke('app_icon', { path }).then((u) => { if (u) { iconCache.set(k, u); el.replaceChildren(h('img', { src: u, alt: '' })); } }).catch(() => {});
}

function renderShelf(body, litter) {
  const items = (D.info?.items || []).filter((x) => !!x.cut === litter);
  const note = h('input', { class: 'dk-field', type: 'text', placeholder: litter ? 'Drop files here to move them later' : 'Type a note and press Enter',
    onkeydown: (e) => { if (e.key === 'Enter' && e.target.value.trim()) { invoke('dock_add_text', { text: e.target.value }); e.target.value = ''; catSound(); } } });
  body.append(h('div', { class: 'dk-zone' }, h('span', { class: 'dk-em', text: litter ? '📦' : '🥣' }),
    h('div', {}, h('b', { text: litter ? 'The litter box' : 'The food bowl' }),
      h('span', { class: 'dk-hint', text: litter
        ? 'Files you drop here, or send here from the bowl, move to wherever you drag them. Text leaves when you drag it out.'
        : `Drop files, text or links on ${D.name}. Drag them back out to copy them anywhere${D.info?.platform === 'mac' ? ' (text: paste it in the box below)' : ''}.` }))));
  if (!litter) body.append(note);
  if (!items.length) {
    body.append(h('div', { class: 'dk-empty' }, h('span', { class: 'dk-em', text: litter ? '🐾' : '🍽️' }), litter ? 'Nothing to move yet.' : `The bowl is empty. ${D.name} is hungry!`));
    return;
  }
  const list = h('div', { class: 'dk-list' });
  for (const it of items) list.append(shelfRow(it));
  body.append(list, h('div', { class: 'dk-row' }, h('span', { class: 'dk-sp' }),
    h('button', { class: 'dk-link', type: 'button', text: litter ? 'Empty the litter box' : 'Empty the bowl', onclick: () => invoke('dock_clear', { cut: litter }) })));
}

function shelfRow(it) {
  const files = it.kind === 'files';
  const first = files ? it.paths[0] : '';
  const ico = h('span', { class: 'dk-ico', text: files ? (it.paths.length > 1 ? '🗂️' : fileEmoji(first)) : it.kind === 'link' ? '🔗' : '📝' });
  if (files && it.paths.length === 1) fileIcon(first, ico);
  const title = files ? (it.paths.length > 1 ? `${baseName(first)} and ${it.paths.length - 1} more` : baseName(first)) : it.kind === 'link' ? it.text.replace(/^https?:\/\//, '').split('/')[0] : it.text.split('\n')[0];
  const sub = files ? folderOf(first) : it.kind === 'link' ? it.text : `${it.text.length.toLocaleString()} characters · ${ago(it.added)}`;
  const act = (ic, label, fn) => h('button', { class: 'dk-ib', type: 'button', title: label, 'aria-label': label, html: icon(ic), onclick: (e) => { e.stopPropagation(); fn(); } });
  const row = h('div', { class: 'dk-it dk-drag', title: files ? 'Drag it out to a folder or an app' : 'Drag it out into any app' },
    ico,
    h('div', { class: 'dk-txt' }, h('b', { text: title || '(empty)' }), h('span', { class: 'dk-sub', text: sub })),
    h('div', { class: 'dk-acts' },
      act('copy', 'Copy', () => invoke('dock_copy_item', { id: it.id }).then(() => toast(files ? (it.cut ? 'Copied: paste to move it' : 'Copied: paste it anywhere') : 'Copied')).catch(fail)),
      files ? act('folder', 'Show in folder', () => invoke('dock_reveal', { id: it.id }).catch(fail)) : null,
      files || it.kind === 'link' ? act('open', 'Open', () => invoke('dock_open_item', { id: it.id }).catch(fail)) : null,
      act(it.cut ? 'back' : 'move', it.cut ? 'Back to the bowl (copy)' : 'To the litter box (move)', () => invoke('dock_set_cut', { id: it.id, cut: !it.cut })),
      act('trash', 'Remove', () => invoke('dock_remove', { id: it.id }))));
  if (files) {
    // Files go out through the system's own drag, so folders and apps take them.
    let down = null;
    row.addEventListener('pointerdown', (e) => { if (e.button === 0 && !e.target.closest('.acts')) down = { x: e.clientX, y: e.clientY }; });
    row.addEventListener('pointermove', (e) => {
      if (!down || (Math.abs(e.clientX - down.x) < 5 && Math.abs(e.clientY - down.y) < 5)) return;
      down = null;
      invoke('dock_drag_out', { id: it.id }).then((dropped) => { if (dropped) toast(it.cut ? 'Moved' : 'Copied there'); }).catch(fail);
    });
    row.addEventListener('pointerup', () => { down = null; });
  } else {
    row.draggable = true;
    row.addEventListener('dragstart', (e) => {
      e.dataTransfer.setData('text/plain', it.text);
      if (it.kind === 'link') e.dataTransfer.setData('text/uri-list', it.text);
      e.dataTransfer.effectAllowed = 'copyMove';
    });
    row.addEventListener('dragend', (e) => { if (it.cut && e.dataTransfer.dropEffect !== 'none') invoke('dock_remove', { id: it.id }); });
  }
  row.addEventListener('dblclick', (e) => { if (!e.target.closest('.acts')) (files || it.kind === 'link' ? invoke('dock_open_item', { id: it.id }) : invoke('dock_copy_item', { id: it.id }).then(() => toast('Copied'))).catch(fail); });
  return row;
}

// ------------------------------------------------------------------ what you copied

function renderYarn(body) {
  const s = D.settings;
  if (s.clipboard === false) {
    body.append(h('div', { class: 'dk-empty' }, h('span', { class: 'dk-em', text: '🧶' }), 'Copy history is off. Turn it on in Cat Key → Settings → Cat Dock.'));
    return;
  }
  const paused = !!D.info?.clipsPaused;
  const q = h('input', { class: 'dk-field', type: 'search', placeholder: 'Find something you copied', value: D.q,
    oninput: (e) => { D.q = e.target.value; const pos = e.target.selectionStart; renderPanel(); const f = $('#body input[type="search"]'); if (f) { f.focus(); f.setSelectionRange(pos, pos); } } });
  body.append(q, h('div', { class: 'dk-row' },
    h('button', { class: 'dk-btn', type: 'button', html: `${icon(paused ? 'play' : 'pause')}<span>${paused ? 'Watch again' : 'Pause'}</span>`,
      onclick: () => invoke('clip_pause', { paused: !paused }).then(() => { D.info.clipsPaused = !paused; renderPanel(); }) }),
    h('span', { class: 'dk-sp' }),
    h('button', { class: 'dk-link', type: 'button', text: 'Clear (keeps pinned)', onclick: () => invoke('clip_clear', { all: false }) })));
  const needle = D.q.trim().toLowerCase();
  const clips = (D.info?.clips || []).filter((c) => !needle || c.text.toLowerCase().includes(needle) || c.paths.some((p) => p.toLowerCase().includes(needle)));
  if (!clips.length) {
    body.append(h('div', { class: 'dk-empty' }, h('span', { class: 'dk-em', text: '🧶' }), needle ? `Nothing copied matches “${D.q}”.` : paused ? 'Paused: nothing new is kept.' : 'Copy something and it shows up here.'));
  } else {
    const list = h('div', { class: 'dk-list' });
    for (const c of clips) {
      const files = c.kind === 'files';
      const act = (ic, label, fn) => h('button', { class: 'dk-ib', type: 'button', title: label, 'aria-label': label, html: icon(ic), onclick: (e) => { e.stopPropagation(); fn(); } });
      list.append(h('div', { class: `dk-it${c.pinned ? ' pinned' : ''}`, title: 'Click to copy it again', onclick: () => invoke('clip_copy', { id: c.id }).then(() => toast('Copied again')).catch(fail) },
        h('span', { class: 'dk-ico', text: files ? '🗂️' : /^https?:\/\/\S+$/.test(c.text.trim()) ? '🔗' : '📋' }),
        h('div', { class: 'dk-txt' },
          files ? h('b', { text: c.paths.length > 1 ? `${baseName(c.paths[0])} and ${c.paths.length - 1} more` : baseName(c.paths[0]) }) : h('span', { class: 'dk-clip', text: c.text.slice(0, 400) }),
          h('span', { class: 'dk-sub', text: `${ago(c.at)}${files ? ` · ${folderOf(c.paths[0])}` : ''}` })),
        c.pinned ? h('span', { class: 'dk-pinmark', text: '●' }) : null,
        h('div', { class: 'dk-acts' },
          act('bowl', 'Keep it in the bowl', () => { if (files) invoke('dock_keep_clip', { id: c.id }).then(() => toast(`In ${D.name}’s bowl`)); else invoke('dock_add_text', { text: c.text }).then(() => toast(`In ${D.name}’s bowl`)); }),
          act('pin', c.pinned ? 'Unpin' : 'Pin', () => invoke('clip_pin', { id: c.id, pinned: !c.pinned })),
          act('trash', 'Forget it', () => invoke('clip_delete', { id: c.id })))));
    }
    body.append(list);
  }
  body.append(h('p', { class: 'dk-hint', text: `Kept only on this computer${s.keepClips ? '' : ', and only until Cat Key closes'}. What password managers copy is never kept.` }));
}

// ------------------------------------------------------------------ focus timer

const fmtClock = (ms) => { const s = Math.max(0, Math.ceil(ms / 1000)); const m = Math.floor(s / 60); return `${m}:${String(s % 60).padStart(2, '0')}`; };
function timerLeft(t = D.info?.timer) {
  if (!t || t.state === 'idle') return 0;
  return t.state === 'running' ? Math.max(0, t.endsAt - Date.now()) : t.leftMs;
}
function renderFocus(body) {
  const t = D.info?.timer || { state: 'idle' };
  const C = 2 * Math.PI * 74;
  const left = timerLeft(t);
  const frac = t.state === 'idle' ? 0 : 1 - left / Math.max(1, t.totalMs);
  const dial = h('div', { class: 'dk-dial' });
  dial.innerHTML = `<svg viewBox="0 0 168 168"><circle class="dk-track" cx="84" cy="84" r="74"/><circle class="dk-done" id="focus-ring" cx="84" cy="84" r="74" stroke-dasharray="${C}" stroke-dashoffset="${C * (1 - frac)}"/></svg>`;
  dial.append(h('div', { class: 'dk-mid' }, h('b', { id: 'focus-left', text: t.state === 'idle' ? '25:00' : fmtClock(left) }),
    h('span', { text: t.state === 'idle' ? 'Ready when you are' : t.label || (t.state === 'paused' ? 'Paused' : 'Focusing') })));
  body.append(h('div', { class: 'dk-clock' }, dial));
  if (t.state === 'idle') {
    const label = h('input', { class: 'dk-field', type: 'text', id: 'focus-label', placeholder: 'What are you working on? (optional)', maxlength: '60' });
    const start = (m) => invoke('timer_start', { minutes: m, label: label.value.trim() }).then(() => { toast(`${m} minutes. ${D.name} keeps watch.`); catSound(); });
    const custom = h('input', { class: 'dk-field', type: 'number', min: '1', max: '600', placeholder: 'Minutes', style: { width: '92px' },
      onkeydown: (e) => { if (e.key === 'Enter' && Number(e.target.value) > 0) start(Number(e.target.value)); } });
    body.append(label,
      h('div', { class: 'dk-presets' }, ...[[15, 'quick'], [25, 'classic'], [45, 'deep'], [60, 'long']].map(([m, w]) =>
        h('button', { class: 'dk-btn', type: 'button', onclick: () => start(m) }, `${m}`, h('small', { text: w })))),
      h('div', { class: 'dk-row' }, custom, h('button', { class: 'dk-btn', type: 'button', text: 'Start', onclick: () => { if (Number(custom.value) > 0) start(Number(custom.value)); } })),
      endings());
  } else {
    body.append(h('div', { class: 'dk-row', style: { justifyContent: 'center' } },
      t.state === 'running'
        ? h('button', { class: 'dk-btn', type: 'button', html: `${icon('pause')}<span>Pause</span>`, onclick: () => invoke('timer_pause') })
        : h('button', { class: 'dk-btn dk-strong', type: 'button', html: `${icon('play')}<span>Go on</span>`, onclick: () => invoke('timer_resume') }),
      h('button', { class: 'dk-btn', type: 'button', html: `${icon('x')}<span>Stop</span>`, onclick: () => invoke('timer_stop') })));
  }
}

/** When a timer ends: the cat walk, the spinning cat, or a different one each time; motion and sound. */
const END_DEFAULT = { mode: 'walk', walk: true, spin: true, motion: true, sound: true };
function endings() {
  const end = { ...END_DEFAULT, ...(D.settings.end || {}) };
  const save = (patch) => {
    const next = { ...end, ...patch };
    D.settings = { ...D.settings, end: next };
    renderPanel();
    return invoke('dock_set_end', { end: next }).catch(fail);
  };
  const chip = (on, label, title, fn) => h('button', { class: `dk-chip${on ? ' on' : ''}`, type: 'button', 'aria-pressed': String(on), title, text: label, onclick: fn });
  const say = { walk: `${D.name} walks along the bottom of your screen with a message.`, spin: `${D.name} spins in the middle of your screen, singing, then stretches.`,
    shuffle: 'A different ending each time, from the ones you tick.' }[end.mode] || '';
  return h('div', { class: 'dk-ends' },
    h('div', { class: 'dk-ends-head' }, h('b', { text: 'When it ends' }), h('span', { class: 'dk-sp' }),
      h('button', { class: 'dk-link', type: 'button', text: 'Try it', title: 'Play the ending now', onclick: () => invoke('walk_test', { kind: end.mode === 'shuffle' ? null : end.mode }) })),
    h('div', { class: 'dk-chips', role: 'radiogroup', 'aria-label': 'Ending' },
      chip(end.mode === 'walk', '🐾 Cat walk', 'The cat walks across the bottom of the screen', () => save({ mode: 'walk' })),
      chip(end.mode === 'spin', '🌀 Spinning cat', 'The cat spins in the middle of the screen', () => save({ mode: 'spin' })),
      chip(end.mode === 'shuffle', '🎲 Shuffle', 'A different one each time', () => save({ mode: 'shuffle' }))),
    end.mode === 'shuffle' ? h('div', { class: 'dk-chips small' }, h('span', { class: 'dk-sub', text: 'In the shuffle:' }),
      chip(end.walk, 'Cat walk', 'Include the cat walk', () => save({ walk: !end.walk })),
      chip(end.spin, 'Spinning cat', 'Include the spinning cat', () => save({ spin: !end.spin }))) : null,
    h('div', { class: 'dk-chips small' },
      chip(end.motion, end.motion ? 'Motion on' : 'Motion off', 'Off: the cat just shows up with its message', () => save({ motion: !end.motion })),
      chip(end.sound, end.sound ? 'Sound on' : 'Sound off', 'The ending’s sound (Cat Key’s sounds need to be on too)', () => save({ sound: !end.sound }))),
    h('p', { class: 'dk-hint', text: `${say} Clicks go straight through.` }));
}

// ------------------------------------------------------------------ the computer's vitals, and the cat's face

function vitalRow(name, v, note = '') {
  const pct = v == null ? null : Math.round(v);
  const cls = pct == null ? '' : pct >= 85 ? ' hard' : pct >= 55 ? ' busy' : '';
  return h('div', { class: `dk-vit${cls}` },
    h('div', { class: 'dk-top' }, h('span', { text: name }), h('b', { text: pct == null ? '—' : `${pct}%` })),
    h('div', { class: 'dk-bar' }, h('span', { style: { width: `${pct || 0}%` } })),
    note ? h('span', { class: 'dk-hint', text: note }) : null);
}
const HEAT = ['Cool', 'Warm', 'Hot', 'Very hot'];
function moodText() {
  const v = D.vitals;
  if (!v) return `${D.name} is checking…`;
  if (D.hard >= 2 || (v.heat ?? 0) >= 2) return `${D.name} is serious: your computer is working very hard.`;
  if ((v.cpu ?? 0) >= 55 || (v.gpu ?? 0) >= 60 || (v.heat ?? 0) === 1) return `${D.name} is focused: your computer is busy.`;
  return `${D.name} is relaxed: your computer takes it easy.`;
}
function renderPc(body) {
  const v = D.vitals || {};
  const mini = h('span', { class: 'dk-mini' });
  body.append(h('div', { class: 'dk-mood' }, mini, h('span', { text: moodText() })));
  const m = new Mascot(mini, { size: 42, look: readUi().look || {} });
  m.setMood(D.hard >= 2 ? 'serious' : 'awake');
  m.setHot(D.hard >= 3 || (v.heat ?? 0) >= 2);
  body.append(vitalRow('Processor', v.cpu), vitalRow('Memory', v.mem));
  if (D.info?.platform !== 'mac') body.append(vitalRow('Graphics', v.gpu));
  if (v.heat != null) {
    body.append(h('div', { class: `dk-vit${v.heat >= 2 ? ' hard' : v.heat === 1 ? ' busy' : ''}` },
      h('div', { class: 'dk-top' }, h('span', { text: 'Heat' }), h('b', { text: HEAT[v.heat] || '—' })),
      h('div', { class: 'dk-bar' }, h('span', { style: { width: `${[18, 50, 80, 100][v.heat] || 0}%` } }))));
  }
  body.append(h('p', { class: 'dk-hint', text: 'Measured every 2 seconds with the system’s own counters: nothing you’d notice. Storage isn’t counted.' }));
}
function onVitals(v) {
  D.vitals = v;
  const busy = (v.cpu ?? 0) >= 80 || (v.gpu ?? 0) >= 85 || (v.heat ?? 0) >= 2;
  // A short spike doesn't count: it takes a few busy readings in a row.
  D.hard = busy ? Math.min(D.hard + 1, 4) : Math.max(D.hard - 1, 0);
  const serious = D.hard >= 2;
  ballCat.setMood(serious ? 'serious' : 'awake');
  ballCat.setHot(D.hard >= 3 || (v.heat ?? 0) >= 2);
  panelCat.setMood(serious ? 'serious' : 'awake');
  panelCat.setHot(D.hard >= 3 || (v.heat ?? 0) >= 2);
  if (D.open && D.tab === 'pc') renderPanel();
}

// ------------------------------------------------------------------ the timer on the ball

function paintTimer() {
  const t = D.info?.timer;
  const on = t && t.state !== 'idle';
  root.classList.toggle('timing', !!on);
  const badge = $('#ball-badge');
  badge.hidden = !on;
  if (!on) return;
  const left = timerLeft(t);
  badge.textContent = t.state === 'paused' ? '❚❚' : left >= 60000 ? `${Math.ceil(left / 60000)}m` : `${Math.ceil(left / 1000)}s`;
  $('#ring').style.strokeDashoffset = String(207.3 * (left / Math.max(1, t.totalMs)));
  if (D.open) $('#dock-sub').textContent = subText();
  if (D.open && D.tab === 'focus') {
    const l = $('#focus-left');
    if (l) l.textContent = fmtClock(left);
    const r = $('#focus-ring');
    if (r) r.style.strokeDashoffset = String(2 * Math.PI * 74 * (left / Math.max(1, t.totalMs)));
  }
}
setInterval(paintTimer, 1000);

// ------------------------------------------------------------------ start

async function boot() {
  applyUi();
  D.info = await invoke('dock_info').catch(() => null);
  if (D.info) {
    D.settings = D.info.settings || {};
    D.tab = D.info.tab || 'bowl';
    D.purring = !!D.info.purring;
    D.volume = D.info.purrVolume ?? 45;
    root.classList.toggle('left', D.settings.edge === 'left');
    setOpen(!!D.info.open);
  }
  paintPurr();
  paintTimer();
  listen('ck://dock-open', (e) => setOpen(!!e.payload));
  listen('ck://dock-drag', (e) => dragPhase(e.payload));
  listen('ck://dock-dragging', (e) => root.classList.toggle('dragging', !!e.payload));
  listen('ck://dock-items', (e) => { if (D.info) D.info.items = e.payload || []; renderPanel(); });
  listen('ck://dock-clips', (e) => { if (D.info) D.info.clips = e.payload || []; if (D.tab === 'yarn' || D.open) renderPanel(); });
  listen('ck://dock-clips-paused', (e) => { if (D.info) D.info.clipsPaused = !!e.payload; renderPanel(); });
  listen('ck://dock-settings', (e) => { D.settings = e.payload || {}; if (D.info) D.info.settings = D.settings; root.classList.toggle('left', D.settings.edge === 'left'); renderPanel(); });
  listen('ck://settings', (e) => { if (e.payload?.purrVolume != null) D.volume = e.payload.purrVolume; if (e.payload?.dock) { D.settings = e.payload.dock; root.classList.toggle('left', D.settings.edge === 'left'); } });
  listen('ck://timer', (e) => { if (D.info) D.info.timer = e.payload; paintTimer(); if (D.tab === 'focus') renderPanel(); else if (D.open) $('#dock-sub').textContent = subText(); });
  listen('ck://purr', (e) => { D.purring = !!e.payload; paintPurr(); });
  listen('ck://vitals', (e) => onVitals(e.payload || {}));
}
boot();
