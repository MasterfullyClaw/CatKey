// Paw prints: a little print just beside the pointer where you click, a trail when you drag,
// tiny steps when you scroll. Never under the pointer, gone in a second.
import { autoShape } from './shape.js';

const T = window.__TAURI__;
const root = document.getElementById('paws');
const SKIN = { mochi: ['#FFF7F0', '#6B4A5E', '#FFB8D2'], kuro: ['#2D2638', '#0E0A14', '#FF8FBF'], tora: ['#FFB36B', '#7A3E12', '#FFC1B0'], sora: ['#CFDBF1', '#43506E', '#FFC0D9'] };
let svg = '';
let side = 1;

function paint() {
  let u = {};
  try { u = JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { /* default cat */ }
  const look = u.look || {};
  const base = SKIN[look.skin] || SKIN[u.skin] || SKIN.mochi;
  const c = look.colors || {};
  const line = c.line || base[1];
  const bean = c.ear || base[2];
  const toe = (x, y) => `<ellipse cx="${x}" cy="${y}" rx="2.3" ry="2.6" fill="${bean}" stroke="${line}" stroke-width="1.3"/>`;
  svg = `<svg viewBox="0 0 24 24"><ellipse cx="12" cy="15.6" rx="5.3" ry="4.4" fill="${bean}" stroke="${line}" stroke-width="1.4"/>${toe(6.4, 10.1)}${toe(9.8, 6.9)}${toe(14.2, 6.9)}${toe(17.6, 10.1)}</svg>`;
}
paint();
window.addEventListener('storage', (e) => { if (e.key === 'catkey.ui.now') paint(); });

function put(cls, x, y, rot, extra = {}) {
  const el = document.createElement('span');
  el.className = `pp ${cls}`;
  el.innerHTML = svg;
  el.style.left = `${x}px`;
  el.style.top = `${y}px`;
  el.style.setProperty('--r', `rotate(${rot}deg)`);
  for (const [k, v] of Object.entries(extra)) el.style.setProperty(k, v);
  el.addEventListener('animationend', () => el.remove());
  root.append(el);
  while (root.childElementCount > 40) root.firstElementChild.remove();
}

function paw(p) {
  if (!p) return;
  // Just beside the pointer, down and to the right, so it never covers what you clicked.
  const x = p.x + 16;
  const y = p.y + 14;
  if (p.kind === 'click') put('click', x, y, -12 + Math.random() * 10);
  else if (p.kind === 'right') put('right', x, y, 18);
  else if (p.kind === 'scroll') {
    for (const i of [0, 1]) put('scroll', x + i * 7, y + (p.a > 0 ? -i * 9 : i * 9), p.a > 0 ? 180 : 0, { '--dy': `${p.a * 22}px`, 'animation-delay': `${i * 90}ms` });
  } else if (p.kind === 'trail') {
    side = -side;
    const rad = (p.a * Math.PI) / 180;
    // left and right steps, a little to the side of the path
    put('trail', p.x + Math.cos(rad + Math.PI / 2) * 7 * side, p.y + Math.sin(rad + Math.PI / 2) * 7 * side, p.a + 90);
  }
}

// Only the prints show, followed while they fade; the rest of the screen is clipped away.
autoShape([['.pp', 3]], { follow: true });

if (T) T.event.listen('ck://paw', (e) => paw(e.payload));
else window.__paw = paw;
