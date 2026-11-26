// The cat walk: at the end of a focus timer, the cat walks along the bottom of the screen
// with a message, leaving paw prints. The window lets every click through.
import { Mascot } from './mascot.js';
import { sounds, setSoundsEnabled, setSoundParts } from './sound.js';

const T = window.__TAURI__;
const root = document.getElementById('walk');
const PAW = '<svg viewBox="0 0 24 24"><ellipse cx="12" cy="16" rx="5" ry="4"/><circle cx="6.5" cy="10.5" r="2.2"/><circle cx="10" cy="7" r="2.2"/><circle cx="14" cy="7" r="2.2"/><circle cx="17.5" cy="10.5" r="2.2"/></svg>';
let shown = 0, paws = 0;

function ui() { try { return JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { return {}; } }

function walk(p) {
  if (!p || p.seq === shown) return;
  shown = p.seq;
  const u = ui();
  root.className = `ck ${u.dark === false ? 'light' : 'dark'} ${u.skin || 'mochi'}`;
  root.textContent = '';
  setSoundsEnabled(u.sounds !== false);
  setSoundParts(u.soundParts || {});
  const W = window.innerWidth;
  const dur = Math.max(8, Math.min(16, W / 160));
  const cat = document.createElement('div');
  cat.className = 'wk-cat';
  cat.style.setProperty('--to', `${W + 40}px`);
  cat.style.setProperty('--dur', `${dur}s`);
  const body = document.createElement('div');
  body.className = 'wk-body';
  cat.append(body);
  const bubble = document.createElement('div');
  bubble.className = 'wk-bubble';
  bubble.textContent = p.text;
  cat.append(bubble);
  root.append(cat);
  const m = new Mascot(body, { size: 150, look: u.look || {} });
  m.setMood('happy');
  void cat.offsetWidth;
  cat.classList.add('wk-walking');
  sounds.caught();
  setTimeout(() => sounds.mew(), 900);
  // Little paw prints where the cat has been, fading away.
  clearInterval(paws);
  const start = performance.now();
  paws = setInterval(() => {
    const t = (performance.now() - start) / 1000 / dur;
    if (t > 1) { clearInterval(paws); return; }
    const x = -200 + t * (W + 240) + 60;
    const paw = document.createElement('span');
    paw.className = 'wk-paw';
    paw.innerHTML = PAW;
    paw.style.left = `${x + (Math.round(t * 40) % 2 ? 14 : -4)}px`;
    paw.style.transform = `rotate(90deg)`;
    root.append(paw);
    setTimeout(() => paw.remove(), 3300);
  }, 520);
  setTimeout(() => { root.classList.add('wk-out'); setTimeout(() => { root.classList.remove('wk-out'); root.textContent = ''; m.destroy(); }, 650); }, dur * 1000 + 600);
}

if (T) {
  T.event.listen('ck://walk', (e) => walk(e.payload));
  T.core.invoke('walk_last').then(walk).catch(() => {});
} else {
  window.__walk = walk;
}
