// The end of a focus timer: the cat walks along the bottom of the screen with a message,
// leaving paw prints, or spins in the middle of the screen singing its own little song.
// Motion and sound each have a switch (Cat Dock → Focus). The window lets every click through.
import { Mascot } from './mascot.js';
import { applyVars } from './theme.js';
import { sounds, setSoundsEnabled, setSoundParts } from './sound.js';

const T = window.__TAURI__;
const root = document.getElementById('walk');
const PAW = '<svg viewBox="0 0 24 24"><ellipse cx="12" cy="16" rx="5" ry="4"/><circle cx="6.5" cy="10.5" r="2.2"/><circle cx="10" cy="7" r="2.2"/><circle cx="14" cy="7" r="2.2"/><circle cx="17.5" cy="10.5" r="2.2"/></svg>';
let shown = 0, paws = 0, cleanup = 0;

function ui() { try { return JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { return {}; } }
const el = (cls, parent) => { const d = document.createElement('div'); d.className = cls; if (parent) parent.append(d); return d; };

function dress(p) {
  const u = ui();
  root.className = `ck ${u.dark === false ? 'light' : 'dark'} ${u.skin || 'mochi'} kind-${p.kind || 'walk'}${p.motion === false ? ' still' : ''}`;
  applyVars(root, u.vars);
  root.textContent = '';
  clearInterval(paws);
  clearTimeout(cleanup);
  setSoundsEnabled(u.sounds !== false && p.sound !== false);
  setSoundParts(u.soundParts || {});
  return u;
}

function finish(m, after) {
  cleanup = setTimeout(() => { root.classList.add('wk-out'); setTimeout(() => { root.classList.remove('wk-out'); root.textContent = ''; m.destroy(); }, 650); }, after);
}

function walk(p, u) {
  const W = window.innerWidth;
  const still = p.motion === false;
  const dur = Math.max(8, Math.min(16, W / 160));
  const cat = el('wk-cat', root);
  cat.style.setProperty('--to', `${W + 40}px`);
  cat.style.setProperty('--dur', `${dur}s`);
  const body = el('wk-body', cat);
  const bubble = el('wk-bubble', cat);
  bubble.textContent = p.text;
  const m = new Mascot(body, { size: 150, look: u.look || {} });
  m.setMood('happy');
  void cat.offsetWidth;
  cat.classList.add(still ? 'wk-here' : 'wk-walking');
  sounds.caught();
  setTimeout(() => sounds.mew(), 900);
  if (!still) {
    // Little paw prints where the cat has been, fading away.
    const start = performance.now();
    paws = setInterval(() => {
      const t = (performance.now() - start) / 1000 / dur;
      if (t > 1) { clearInterval(paws); return; }
      const x = -200 + t * (W + 240) + 60;
      const paw = document.createElement('span');
      paw.className = 'wk-paw';
      paw.innerHTML = PAW;
      paw.style.left = `${x + (Math.round(t * 40) % 2 ? 14 : -4)}px`;
      paw.style.transform = 'rotate(90deg)';
      root.append(paw);
      setTimeout(() => paw.remove(), 3300);
    }, 520);
  }
  finish(m, (still ? 8 : dur) * 1000 + 600);
}

function spin(p, u) {
  const still = p.motion === false;
  const stage = el('sp-stage', root);
  el('sp-glow', stage);
  const box = el('sp-box', stage);
  const rings = el('sp-rings', box);
  for (let i = 0; i < 3; i++) el(`sp-ring r${i}`, rings);
  const turn = el('sp-turn', box);
  const body = el('sp-body', turn);
  const bubble = el('wk-bubble sp-bubble', stage);
  bubble.textContent = p.text;
  const m = new Mascot(body, { size: Math.round(Math.min(260, window.innerHeight * 0.32)), look: u.look || {} });
  m.setMood('happy');
  void stage.offsetWidth;
  stage.classList.add('in');
  const SPIN = 4.8;
  if (!still) {
    setTimeout(() => { stage.classList.add('spinning'); sounds.spin(SPIN); }, 450);
    setTimeout(() => { stage.classList.remove('spinning'); stage.classList.add('done'); m.knead(1600); sounds.mew(); }, 450 + SPIN * 1000);
  } else {
    sounds.caught();
    setTimeout(() => { stage.classList.add('done'); sounds.mew(); }, 500);
  }
  finish(m, (still ? 6.5 : 8.6) * 1000);
}

function show(p) {
  if (!p || p.seq === shown) return;
  shown = p.seq;
  const u = dress(p);
  if (p.kind === 'spin') spin(p, u);
  else walk(p, u);
}

if (T) {
  T.event.listen('ck://walk', (e) => show(e.payload));
  T.core.invoke('walk_last').then(show).catch(() => {});
} else {
  window.__walk = show;
}
