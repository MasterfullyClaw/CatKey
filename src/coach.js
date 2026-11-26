// The shortcut coach's tip window: shows the tip Rust sends, waits while the pointer is
// on it, and hides itself. Ignore = never again, Dismiss = for now, Adjust = make your own.
import * as K from './keys.js';
import { paletteVars, applyVars } from './theme.js';
import { autoShape } from './shape.js';
// Only the tip shows; around it the window is clipped away.
autoShape([['.tip', 2]]);

// The tip wears the cat's colours (always its dark side: tips float over anything).
function dress() {
  let u = {};
  try { u = JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { /* default colours */ }
  const v = paletteVars(u.pal, true);
  applyVars(document.documentElement, v);
  document.documentElement.style.setProperty('--edge', v ? v['--line'] : '');
  if (!v) document.documentElement.style.removeProperty('--edge');
}
dress();
window.addEventListener('storage', (e) => { if (e.key === 'catkey.ui.now') dress(); });

const T = window.__TAURI__;
// On a Mac the keys wear their Mac names (⌘ ⌥ ⌃ ⇧).
K.setMac(/Mac/.test(navigator.userAgent));
const tip = document.getElementById('tip');
const LIFE_MS = 6000;
let timer = 0, shown = -1, cur = null, left = LIFE_MS, started = 0;
const call = (cmd, args) => (T ? T.core.invoke(cmd, args).catch(() => {}) : Promise.resolve());

function hide(now = false) {
  clearTimeout(timer);
  tip.classList.remove('in', 'hold');
  tip.classList.add('out');
  setTimeout(() => call('coach_hide'), now ? 0 : 330);
}
function run(ms) {
  clearTimeout(timer);
  started = Date.now();
  left = ms;
  timer = setTimeout(() => hide(), ms);
}
function show(p) {
  if (!p || p.seq === shown) return;
  shown = p.seq;
  cur = p;
  if (p.sound) import('./sound.js').then((m) => { m.setSoundParts({ tips: true }); m.sounds.tip(); }).catch(() => {});
  document.body.dataset.anchor = p.anchor || 'bottom-right';
  const keys = document.getElementById('keys');
  keys.replaceChildren(...String(p.keys).split('+').map((k) => { const s = document.createElement('span'); s.className = 'kc'; s.textContent = K.keyLabel(k.trim()); return s; }));
  document.getElementById('use').textContent = p.demo ? 'A tip looks like this:' : 'Next time:';
  document.getElementById('what').textContent = p.name;
  const secs = (p.savesMs / 1000).toFixed(1).replace(/\.0$/, '');
  document.getElementById('meta').textContent = p.demo
    ? 'Shown when you click something with a shortcut'
    : `${p.appName ? p.appName + ' · ' : ''}saves about ${secs} s${p.times > 1 ? ` · ${p.times}× so far` : ''}`;
  for (const b of document.querySelectorAll('.acts button')) b.disabled = !!p.demo && b.id !== 'b-dismiss';
  tip.style.setProperty('--life', `${LIFE_MS}ms`);
  tip.classList.remove('in', 'out', 'hold');
  void tip.offsetWidth;
  tip.classList.add('in');
  run(LIFE_MS);
}
// The pointer on the tip keeps it there; leaving gives it its remaining time.
tip.addEventListener('mouseenter', () => {
  if (!tip.classList.contains('in')) return;
  clearTimeout(timer);
  left = Math.max(1500, left - (Date.now() - started));
  tip.classList.add('hold');
});
tip.addEventListener('mouseleave', () => {
  if (!tip.classList.contains('in')) return;
  tip.classList.remove('hold');
  run(left);
});
document.getElementById('b-dismiss').onclick = () => hide(true);
document.getElementById('b-ignore').onclick = () => { if (cur && !cur.demo) call('coach_ignore', { id: cur.id }); hide(true); };
document.getElementById('b-adjust').onclick = () => { if (cur && !cur.demo) call('coach_adjust', { id: cur.id }); hide(true); };
if (T) {
  T.event.listen('ck://coach-tip', (e) => show(e.payload));
  T.core.invoke('coach_last').then(show).catch(() => {});
}
