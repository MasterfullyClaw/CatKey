// CatGPT's wheel: a ring of choices that spins out around the pointer. Resting the pointer
// on one picks it (a little ring fills first), a click picks it at once, Esc closes.
import { Mascot } from './mascot.js';
import { applyVars } from './theme.js';
import { sounds, setSoundsEnabled, setSoundParts } from './sound.js';

const T = window.__TAURI__;
const invoke = (cmd, args) => (T ? T.core.invoke(cmd, args) : (window.__mock ? window.__mock(cmd, args) : Promise.resolve(null)));
const $ = (s) => document.querySelector(s);
const root = $('#cg');
const wheel = $('#wheel');
const ring = $('#ring');
const hub = $('#hub');
const card = $('#card');
const cat = new Mascot($('#cat'), { size: 52, track: true });
const DWELL = 420;
const W = { p: null, mode: 'main', app: null, hot: null, start: 0, raf: 0, busy: false };
const APP_NAME = { claude: 'Claude', chatgpt: 'ChatGPT' };

export function prefs() {
  const d = { app: 'claude', now: false, video: false, use: { claude: true, chatgpt: true } };
  try { return { ...d, ...JSON.parse(localStorage.getItem('catkey.catgpt') || '{}') }; } catch { return d; }
}
function setPrefs(patch) {
  try { localStorage.setItem('catkey.catgpt', JSON.stringify({ ...prefs(), ...patch })); } catch { /* not kept */ }
}

function ui() { try { return JSON.parse(localStorage.getItem('catkey.ui.now') || 'null') || {}; } catch { return {}; } }
function dress() {
  const u = ui();
  root.className = `ck ${u.dark === false ? 'light' : 'dark'} ${u.skin || 'mochi'}`;
  applyVars(root, u.vars);
  cat.setLook(u.look || { skin: 'mochi', gear: [] });
  setSoundsEnabled(u.sounds !== false);
  setSoundParts(u.soundParts || {});
}
window.addEventListener('storage', (e) => { if (e.key === 'catkey.ui.now') dress(); });

const el = (tag, cls, text) => { const d = document.createElement(tag); if (cls) d.className = cls; if (text != null) d.textContent = text; return d; };

function apps() {
  const use = prefs().use || {};
  return (W.p?.apps || []).filter((a) => a.installed && use[a.id] !== false);
}

function hubText(title, sub) {
  $('#hub-title').textContent = title;
  $('#hub-sub').textContent = sub;
}

// ------------------------------------------------------------------ the ring

function stopDwell() {
  cancelAnimationFrame(W.raf);
  if (W.hot) { W.hot.classList.remove('hot'); W.hot.style.setProperty('--p', 0); }
  W.hot = null;
}

function petals(list) {
  stopDwell();
  ring.textContent = '';
  const n = list.length;
  list.forEach((p, i) => {
    const b = el('button', `cg-pet${p.cls ? ` ${p.cls}` : ''}${p.off ? ' off' : ''}`);
    b.type = 'button';
    b.setAttribute('role', 'menuitem');
    b.setAttribute('aria-label', p.title || p.lb);
    b.title = p.title || p.lb;
    b.style.setProperty('--a', `${-90 + (360 / n) * i}deg`);
    b.style.setProperty('--d', `${i * 0.03}s`);
    b.append(el('span', 'em', p.em), el('span', 'lb', p.lb));
    if (p.badge) b.append(el('span', 'badge', String(p.badge)));
    const go = () => { if (!p.off && !W.busy) { stopDwell(); sounds.pop(); p.act(); } };
    b.addEventListener('click', go);
    b.addEventListener('pointerenter', () => {
      if (p.off || W.busy) return;
      stopDwell();
      W.hot = b;
      b.classList.add('hot');
      hubText(p.lb, p.say || '');
      W.start = performance.now();
      const tick = (t) => {
        const k = Math.min(1, (t - W.start) / (p.dwell || DWELL));
        b.style.setProperty('--p', (k * 100).toFixed(1));
        if (k >= 1) { go(); return; }
        W.raf = requestAnimationFrame(tick);
      };
      W.raf = requestAnimationFrame(tick);
    });
    b.addEventListener('pointerleave', () => { if (W.hot === b) { stopDwell(); restHub(); } });
    ring.append(b);
  });
  wheel.classList.remove('spin');
  void wheel.offsetWidth;
  wheel.classList.add('spin');
}

function restHub() {
  if (W.mode === 'app') { hubText(APP_NAME[W.app], sendSay(W.app)); return; }
  const f = W.p?.fresh || 0;
  const a = APP_NAME[prefs().app] || 'Claude';
  hubText('CatGPT', f ? `Click: send ${f} new to ${a}` : `Click: show ${a} your screen`);
}

function sendSay(app) {
  const f = W.p?.fresh || 0;
  const what = f ? `${f} new thing${f > 1 ? 's' : ''} from the basket` : 'a screenshot of this screen';
  return `${what}${prefs().now ? ', sent at once' : ', left as a draft'}`;
}

function mainRing() {
  W.mode = 'main';
  card.hidden = true;
  const list = [
    { em: '📸', lb: 'Screenshot', say: 'Into the basket', act: () => run('catgpt_shot', {}, null) },
    { em: '✏️', lb: 'Draw', say: 'Turn the screen into a canvas', act: () => run('catgpt_draw', { blank: false }, null) },
    { em: W.p?.recording ? '⏹️' : '🎥', lb: W.p?.recording ? 'Stop' : 'Record', cls: W.p?.recording ? 'rec' : '', say: 'A very light video of this screen',
      act: () => run('catgpt_record', { on: !W.p?.recording }, null) },
    { em: '📝', lb: 'Note', say: 'Type something for the AI', act: noteCard },
    { em: '📎', lb: 'Add a file', say: 'Pick files for the basket', act: addFiles },
    { em: '🧺', lb: 'Basket', badge: W.p?.fresh || 0, say: 'See everything in Cat Key', act: openBasket },
  ];
  for (const a of apps()) list.push({ em: a.id === 'claude' ? '🧡' : '💚', lb: a.name, say: 'New chat, this chat or a recent one', act: () => appRing(a.id) });
  if (!apps().length) list.push({ em: '🔌', lb: 'Connect', say: 'Claude or ChatGPT: in Cat Key → CatGPT', act: openBasket });
  petals(list);
  restHub();
}

function appRing(app) {
  W.mode = 'app';
  W.app = app;
  card.hidden = true;
  const now = prefs().now;
  const list = [
    { em: '✨', lb: 'New chat', say: sendSay(app), act: () => send(app, 'new') },
    { em: '💬', lb: 'This chat', say: 'The chat that’s open in the app', act: () => send(app, 'open') },
    { em: '🕘', lb: 'Recent', say: 'Pick one of your recent chats', act: () => recentCard(app), dwell: 520 },
    { em: now ? '⚡' : '✍️', lb: now ? 'Sends at once' : 'Stays a draft', say: now ? 'Rest here to leave it as a draft instead' : 'Rest here to send it right away', dwell: 650,
      act: () => { setPrefs({ now: !prefs().now }); appRing(app); } },
    { em: '↩️', lb: 'Back', say: 'The other choices', act: mainRing, dwell: 520 },
  ];
  petals(list);
  hubText(APP_NAME[app], sendSay(app));
}

// ------------------------------------------------------------------ cards in the middle

function showCard(...kids) {
  stopDwell();
  card.textContent = '';
  card.append(...kids);
  card.hidden = false;
}

function noteCard() {
  W.mode = 'note';
  const ta = el('textarea');
  ta.placeholder = 'Type a note: what you want to ask, or anything the AI should know…';
  const add = (thenSend) => {
    const t = ta.value.trim();
    if (!t) { ta.focus(); return; }
    invoke('catgpt_note', { text: t }).then(() => {
      W.p.fresh = (W.p.fresh || 0) + 1;
      if (thenSend) send(prefs().app, 'new');
      else done('📝 Note in the basket');
    }).catch(fail);
  };
  ta.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); add(false); }
    if (e.key === 'Escape') { e.stopPropagation(); mainRing(); }
  });
  const b1 = btn('Add to the basket', 'btn primary', () => add(false));
  const b2 = btn(`Add and send to ${APP_NAME[prefs().app] || 'Claude'}`, 'btn', () => add(true));
  const h3 = el('h3', null, '📝 A note');
  const hint = el('p', 'hint', 'Enter adds it · Shift+Enter for a new line · Esc goes back');
  const row = el('div', 'row');
  row.append(b1, b2);
  showCard(h3, ta, row, hint);
  setTimeout(() => ta.focus(), 50);
}

function recentCard(app) {
  W.mode = 'recent';
  const list = el('div', 'cg-list');
  const h3 = el('h3', null, `🕘 ${APP_NAME[app]}: recent chats`);
  const back = btn('Back', 'btn', () => appRing(app));
  const row = el('div', 'row');
  row.append(back);
  showCard(h3, el('p', 'hint', 'Looking…'), row);
  invoke('catgpt_recent', { app }).then((r) => {
    const items = r || [];
    if (!items.length) {
      showCard(h3, el('p', 'hint', app === 'claude' ? 'Cat Key couldn’t read Claude’s recent chats here. “This chat” sends to the one that’s open.' : 'No recent chats found.'), row);
      return;
    }
    for (const it of items) list.append(btn(it.title, '', () => send(app, it.dest)));
    showCard(h3, list, row);
  }).catch(fail);
}

function btn(text, cls, fn) {
  const b = el('button', cls, text);
  b.type = 'button';
  b.addEventListener('click', fn);
  return b;
}

function done(text, ms = 1100) {
  stopDwell();
  const d = el('div', 'cg-done', text);
  wheel.append(d);
  setTimeout(() => { d.remove(); close(false); }, ms);
}

function fail(e) {
  W.busy = false;
  const msg = String(e?.message || e || 'That didn’t work.');
  showCard(el('h3', null, '🙀 Not this time'), el('p', null, msg), btn('Close', 'btn primary', () => close(true)));
}

// ------------------------------------------------------------------ doing things

function run(cmd, args) {
  W.busy = true;
  invoke(cmd, args).then(() => { W.busy = false; }).catch(fail);
}

function addFiles() {
  W.busy = true;
  invoke('catgpt_pick_files').then((list) => {
    W.busy = false;
    const n = (list || []).length;
    if (!n) { mainRing(); return; }
    W.p.fresh = (W.p.fresh || 0) + n;
    done(`📎 ${n} file${n > 1 ? 's' : ''} in the basket`);
  }).catch(fail);
}

function openBasket() {
  if (T) T.event.emit('ck://catgpt-open', null).catch(() => {});
  invoke('focus_window').catch(() => {});
  close(false);
}

function send(app, dest) {
  W.busy = true;
  const p = prefs();
  ring.textContent = '';
  card.hidden = true;
  hubText(APP_NAME[app], `Opening ${APP_NAME[app]}…`);
  invoke('catgpt_send', { req: { app, dest, ids: [], text: '', now: !!p.now, video: !!p.video } })
    .then((msg) => { W.busy = false; done(`✓ ${msg || 'Done'}`, 1800); sounds.caught?.(); })
    .catch(fail);
}

function close(back) {
  stopDwell();
  wheel.classList.remove('open');
  W.busy = false;
  setTimeout(() => invoke('catgpt_wheel_hide', { back }).catch(() => {}), 120);
}

hub.addEventListener('click', () => {
  if (W.busy) return;
  if (W.mode === 'app') send(W.app, 'new');
  else if (W.mode === 'main') send(prefs().app, 'new');
});
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') { if (W.mode === 'main' || W.mode === 'app') close(true); else mainRing(); }
  if (W.mode === 'main' || W.mode === 'app') {
    const n = Number(e.key);
    if (n >= 1 && n <= 9) ring.children[n - 1]?.click();
  }
});
// A click on the empty corners closes it.
root.addEventListener('pointerdown', (e) => {
  if (e.target === root || e.target === wheel || e.target === ring) close(true);
});
// Clicking another app puts it away.
window.addEventListener('blur', () => { if (!W.busy && wheel.classList.contains('open')) close(false); });

function open(p) {
  if (!p || p.seq === W.seq) return;
  W.seq = p.seq;
  W.p = p;
  W.busy = false;
  dress();
  card.hidden = true;
  wheel.classList.remove('open');
  mainRing();
  requestAnimationFrame(() => wheel.classList.add('open'));
  cat.react(900);
}

if (T) {
  T.event.listen('ck://wheel', (e) => open(e.payload));
  invoke('catgpt_wheel_last').then(open).catch(() => {});
} else {
  window.__wheel = open;
}
