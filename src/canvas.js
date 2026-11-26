// CatGPT's canvas: the screen as it was a moment ago (or blank paper), to draw, write and
// point on. "Add to the basket" keeps it as a picture; then make another canvas and go on.
import { applyVars } from './theme.js';

const T = window.__TAURI__;
const invoke = (cmd, args) => (T ? T.core.invoke(cmd, args) : (window.__mock ? window.__mock(cmd, args) : Promise.resolve(null)));
const $ = (s) => document.querySelector(s);
const root = $('#cv');
const bg = $('#bg');
const ink = $('#ink');
const texts = $('#texts');
const ctx = ink.getContext('2d');

const COLORS = ['#FF3B5C', '#FFC233', '#2ECC71', '#3B82F6', '#FFFFFF', '#111111'];
const SIZES = [3, 6, 12];
const TOOLS = [['pen', '✏️', 'Pen'], ['marker', '🖍️', 'Highlighter'], ['arrow', '↗️', 'Arrow'], ['box', '⬜', 'Box'], ['text', '🔤', 'Text'], ['eraser', '🧽', 'Eraser']];
const S = { tool: 'pen', color: COLORS[0], size: 1, items: [], redo: [], cur: null, dirty: false, bgSize: null, blank: false, armedClose: 0 };
const PREF = 'catkey.catgpt';
const appPref = () => { try { return JSON.parse(localStorage.getItem(PREF) || '{}'); } catch { return {}; } };

function dress() {
  let u = {};
  try { u = JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { /* default look */ }
  root.className = `ck ${u.dark === false ? 'light' : 'dark'} ${u.skin || 'mochi'}${S.blank ? ' blank' : ''}`;
  applyVars(root, u.vars);
}

function el(tag, cls, html) { const d = document.createElement(tag); if (cls) d.className = cls; if (html != null) d.innerHTML = html; return d; }
function toast(text, ms = 2200) {
  const t = $('#toast');
  t.textContent = text;
  t.classList.add('on');
  clearTimeout(toast.t);
  toast.t = setTimeout(() => t.classList.remove('on'), ms);
}

// ------------------------------------------------------------------ the bars

function bars() {
  const tools = $('#tools');
  tools.textContent = '';
  for (const [id, em, label] of TOOLS) {
    const b = el('button', S.tool === id ? 'on' : '', `<span class="em">${em}</span>`);
    b.title = label; b.setAttribute('aria-label', label); b.setAttribute('aria-pressed', String(S.tool === id));
    b.onclick = () => { commitText(); S.tool = id; bars(); };
    tools.append(b);
  }
  tools.append(el('span', 'sep'));
  for (const c of COLORS) {
    const b = el('button', `sw${S.color === c ? ' on' : ''}`);
    b.style.background = c; b.title = 'Colour'; b.setAttribute('aria-label', `Colour ${c}`);
    b.onclick = () => { S.color = c; bars(); };
    tools.append(b);
  }
  tools.append(el('span', 'sep'));
  SIZES.forEach((px, i) => {
    const b = el('button', S.size === i ? 'on' : '', `<span class="dot" style="width:${4 + i * 4}px;height:${4 + i * 4}px"></span>`);
    b.title = ['Thin', 'Medium', 'Thick'][i]; b.setAttribute('aria-label', b.title);
    b.onclick = () => { S.size = i; bars(); };
    tools.append(b);
  });
  tools.append(el('span', 'sep'));
  const act = (em, label, fn, title) => { const b = el('button', '', `<span class="em">${em}</span>${label ? `<span>${label}</span>` : ''}`); b.title = title || label; b.onclick = fn; return b; };
  tools.append(act('↶', '', undo, 'Undo'), act('↷', '', redo, 'Redo'), act('🗑️', '', () => { if (S.items.length) { S.redo = []; S.items = []; S.dirty = true; draw(); } }, 'Clear the drawing'));

  const acts = $('#acts');
  acts.textContent = '';
  const wrap = el('span');
  wrap.style.position = 'relative';
  const nb = act('📄', 'New canvas', () => {
    const open = wrap.querySelector('.cv-menu');
    if (open) { open.remove(); return; }
    const m = el('div', 'cv-menu');
    m.append(act('🖥️', 'This screen again', () => fresh(false)), act('📃', 'Blank page', () => fresh(true)));
    wrap.append(m);
  });
  wrap.append(nb);
  acts.append(wrap, act('🧺', 'Add to the basket', () => keep()), el('span', 'sep'));
  const p = appPref();
  const use = p.use || {};
  for (const [id, em, name] of [['claude', '🧡', 'Claude'], ['chatgpt', '💚', 'ChatGPT']]) {
    if (use[id] === false) continue;
    const b = act(em, `Send to ${name}`, () => keep({ app: id }));
    if ((p.app || 'claude') === id) b.classList.add('primary');
    acts.append(b);
  }
  acts.append(el('span', 'sep'), act('✕', 'Close', () => closeCanvas(), 'Close (Esc)'));
}

// ------------------------------------------------------------------ drawing

function fit() {
  const dpr = window.devicePixelRatio || 1;
  ink.width = Math.round(innerWidth * dpr);
  ink.height = Math.round(innerHeight * dpr);
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  draw();
}
addEventListener('resize', fit);

const width = (it) => SIZES[it.size] * (it.type === 'marker' ? 3.2 : it.type === 'eraser' ? 4 : 1);

function paint(c, it, k = 1) {
  c.save();
  c.lineCap = 'round';
  c.lineJoin = 'round';
  c.strokeStyle = it.color;
  c.fillStyle = it.color;
  c.lineWidth = width(it) * k;
  if (it.type === 'marker') c.globalAlpha = 0.38;
  if (it.type === 'eraser') c.globalCompositeOperation = 'destination-out';
  if (it.type === 'pen' || it.type === 'marker' || it.type === 'eraser') {
    const pts = it.pts;
    c.beginPath();
    c.moveTo(pts[0][0] * k, pts[0][1] * k);
    if (pts.length === 1) c.lineTo(pts[0][0] * k + 0.1, pts[0][1] * k);
    for (let i = 1; i < pts.length - 1; i++) {
      const mx = (pts[i][0] + pts[i + 1][0]) / 2, my = (pts[i][1] + pts[i + 1][1]) / 2;
      c.quadraticCurveTo(pts[i][0] * k, pts[i][1] * k, mx * k, my * k);
    }
    if (pts.length > 1) c.lineTo(pts.at(-1)[0] * k, pts.at(-1)[1] * k);
    c.stroke();
  } else if (it.type === 'box') {
    const [x1, y1] = it.a, [x2, y2] = it.b;
    c.beginPath();
    c.roundRect(Math.min(x1, x2) * k, Math.min(y1, y2) * k, Math.abs(x2 - x1) * k, Math.abs(y2 - y1) * k, 8 * k);
    c.stroke();
  } else if (it.type === 'arrow') {
    const [x1, y1] = it.a, [x2, y2] = it.b;
    const ang = Math.atan2(y2 - y1, x2 - x1);
    const head = Math.max(14, width(it) * 4) * k;
    c.beginPath();
    c.moveTo(x1 * k, y1 * k);
    c.lineTo(x2 * k, y2 * k);
    c.stroke();
    c.beginPath();
    c.moveTo(x2 * k, y2 * k);
    c.lineTo(x2 * k - head * Math.cos(ang - 0.45), y2 * k - head * Math.sin(ang - 0.45));
    c.lineTo(x2 * k - head * Math.cos(ang + 0.45), y2 * k - head * Math.sin(ang + 0.45));
    c.closePath();
    c.fill();
  } else if (it.type === 'text') {
    const fs = (16 + SIZES[it.size] * 2.2) * k;
    c.font = `800 ${fs}px Nunito, 'Segoe UI', sans-serif`;
    c.textBaseline = 'top';
    // a soft outline keeps text readable on any background
    c.lineWidth = Math.max(2, fs / 7);
    c.strokeStyle = it.color === '#111111' ? 'rgba(255,255,255,.85)' : 'rgba(0,0,0,.55)';
    it.text.split('\n').forEach((line, i) => {
      c.strokeText(line, it.x * k, it.y * k + i * fs * 1.25);
      c.fillText(line, it.x * k, it.y * k + i * fs * 1.25);
    });
  }
  c.restore();
}

function draw() {
  ctx.clearRect(0, 0, innerWidth, innerHeight);
  for (const it of S.items) paint(ctx, it);
  if (S.cur) paint(ctx, S.cur);
}

function pos(e) { return [e.clientX, e.clientY]; }

ink.addEventListener('pointerdown', (e) => {
  if (e.button !== 0) return;
  closeMenus();
  if (S.tool === 'text') { e.preventDefault(); startText(...pos(e)); return; }
  commitText();
  ink.setPointerCapture(e.pointerId);
  const p = pos(e);
  S.cur = ['pen', 'marker', 'eraser'].includes(S.tool)
    ? { type: S.tool, color: S.color, size: S.size, pts: [p] }
    : { type: S.tool, color: S.color, size: S.size, a: p, b: p };
  draw();
});
ink.addEventListener('pointermove', (e) => {
  if (!S.cur) return;
  const p = pos(e);
  if (S.cur.pts) {
    const last = S.cur.pts.at(-1);
    if (Math.hypot(p[0] - last[0], p[1] - last[1]) < 1.5) return;
    S.cur.pts.push(p);
  } else S.cur.b = p;
  draw();
});
const finish = () => {
  if (!S.cur) return;
  const it = S.cur;
  S.cur = null;
  const tiny = it.a && Math.hypot(it.b[0] - it.a[0], it.b[1] - it.a[1]) < 4;
  if (!tiny) { S.items.push(it); S.redo = []; S.dirty = true; }
  draw();
};
ink.addEventListener('pointerup', finish);
ink.addEventListener('pointercancel', finish);

function undo() { commitText(); if (S.items.length) { S.redo.push(S.items.pop()); S.dirty = true; draw(); } }
function redo() { if (S.redo.length) { S.items.push(S.redo.pop()); S.dirty = true; draw(); } }

// ------------------------------------------------------------------ text

function startText(x, y) {
  commitText();
  const ta = document.createElement('textarea');
  ta.style.left = `${x}px`;
  ta.style.top = `${y - 6}px`;
  ta.style.color = S.color;
  ta.style.fontSize = `${16 + SIZES[S.size] * 2.2}px`;
  ta.rows = 1;
  ta.placeholder = 'Type…';
  ta.dataset.x = x;
  ta.dataset.y = y;
  const grow = () => { ta.style.height = 'auto'; ta.style.height = `${ta.scrollHeight}px`; ta.style.width = 'auto'; ta.style.width = `${Math.max(140, ta.scrollWidth + 8)}px`; };
  ta.addEventListener('input', grow);
  ta.addEventListener('keydown', (e) => {
    e.stopPropagation();
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); commitText(); }
    if (e.key === 'Escape') { ta.remove(); }
  });
  texts.append(ta);
  setTimeout(() => ta.focus(), 0);
}

function commitText() {
  for (const ta of [...texts.querySelectorAll('textarea')]) {
    const t = ta.value.replace(/\s+$/, '');
    if (t) {
      S.items.push({ type: 'text', color: ta.style.color ? rgbHex(ta.style.color) : S.color, size: S.size, x: Number(ta.dataset.x), y: Number(ta.dataset.y), text: t });
      S.redo = [];
      S.dirty = true;
    }
    ta.remove();
  }
  draw();
}
function rgbHex(c) {
  const m = c.match(/\d+/g);
  return m ? `#${m.slice(0, 3).map((v) => Number(v).toString(16).padStart(2, '0')).join('')}` : c;
}

function closeMenus() { for (const m of document.querySelectorAll('.cv-menu')) m.remove(); }

// ------------------------------------------------------------------ keeping it

/** The canvas as a picture: the screen at its own resolution, the drawing on top. */
async function picture() {
  commitText();
  const W = S.bgSize?.w || Math.round(innerWidth * (devicePixelRatio || 1));
  const H = S.bgSize?.h || Math.round(innerHeight * (devicePixelRatio || 1));
  const k = W / innerWidth;
  const c = document.createElement('canvas');
  c.width = W; c.height = H;
  const x = c.getContext('2d');
  if (S.blank || !bg.src) { x.fillStyle = '#FFFFFF'; x.fillRect(0, 0, W, H); } else x.drawImage(bg, 0, 0, W, H);
  // the drawing goes on its own layer first, so the eraser only rubs out ink
  const layer = document.createElement('canvas');
  layer.width = W; layer.height = H;
  const l = layer.getContext('2d');
  for (const it of S.items) paint(l, it, k);
  x.drawImage(layer, 0, 0);
  return { data: c.toDataURL('image/jpeg', 0.86), w: W, h: H };
}

let saving = false;
async function keep(sendTo = null) {
  if (saving) return;
  saving = true;
  try {
    const pic = await picture();
    await invoke('catgpt_canvas_save', pic);
    S.dirty = false;
    if (sendTo) {
      const p = appPref();
      await invoke('catgpt_canvas_close', { back: false });
      await invoke('catgpt_send', { req: { app: sendTo.app, dest: 'new', ids: [], text: '', now: !!p.now, video: !!p.video } });
      reset();
    } else {
      toast('In the basket. Make another canvas, or close this one.', 3200);
    }
  } catch (e) {
    toast(String(e?.message || e || 'That didn’t work.'), 4500);
  } finally {
    saving = false;
  }
}

function fresh(blank) {
  closeMenus();
  if (S.dirty && !confirmLose()) return;
  invoke('catgpt_canvas_new', { blank }).catch((e) => toast(String(e), 4000));
}

function confirmLose() {
  const now = Date.now();
  if (now - S.armedClose < 3000) return true;
  S.armedClose = now;
  toast('This drawing isn’t in the basket yet. Do it again to leave it.', 3000);
  return false;
}

function closeCanvas() {
  commitText();
  if (S.dirty && !confirmLose()) return;
  invoke('catgpt_canvas_close', { back: true }).catch(() => {});
  reset();
}

function reset() {
  S.items = []; S.redo = []; S.cur = null; S.dirty = false;
  texts.textContent = '';
  draw();
}

document.addEventListener('keydown', (e) => {
  const mod = e.ctrlKey || e.metaKey;
  if (e.key === 'Escape') { e.preventDefault(); closeCanvas(); }
  else if (mod && e.key.toLowerCase() === 'z') { e.preventDefault(); if (e.shiftKey) redo(); else undo(); }
  else if (mod && e.key.toLowerCase() === 'y') { e.preventDefault(); redo(); }
  else if (!mod && /^[1-6]$/.test(e.key)) { S.tool = TOOLS[Number(e.key) - 1][0]; bars(); }
});

async function load() {
  reset();
  const b = await invoke('catgpt_canvas_bg').catch(() => null);
  S.blank = !b?.image;
  S.bgSize = b?.image ? { w: b.w, h: b.h } : null;
  if (b?.image) {
    bg.src = b.image;
    await bg.decode().catch(() => {});
  } else bg.removeAttribute('src');
  dress();
  bars();
  fit();
}

if (T) T.event.listen('ck://canvas', () => load());
load();
window.addEventListener('storage', (e) => { if (e.key === 'catkey.ui.now') dress(); });
