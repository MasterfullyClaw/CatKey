// The shortcut coach's tip window: shows the tip Rust sends, then fades it out.
const T = window.__TAURI__;
const tip = document.getElementById('tip');
let timer = 0, shown = -1;
function show(p) {
  if (!p || p.seq === shown) return;
  shown = p.seq;
  const keys = document.getElementById('keys');
  keys.replaceChildren(...String(p.keys).split('+').map((k) => { const s = document.createElement('span'); s.className = 'kc'; s.textContent = k.trim(); return s; }));
  document.getElementById('use').textContent = p.demo ? 'A tip looks like this:' : 'Next time:';
  document.getElementById('what').textContent = p.name;
  const secs = (p.savesMs / 1000).toFixed(1).replace(/\.0$/, '');
  document.getElementById('meta').textContent = p.demo
    ? 'Shown when you click something with a shortcut'
    : `${p.appName ? p.appName + ' · ' : ''}saves about ${secs} s${p.times > 1 ? ` · ${p.times}× so far` : ''}`;
  tip.classList.remove('in', 'out');
  void tip.offsetWidth;
  tip.classList.add('in');
  clearTimeout(timer);
  timer = setTimeout(() => { tip.classList.remove('in'); tip.classList.add('out'); }, 5000);
}
if (T) {
  T.event.listen('ck://coach-tip', (e) => show(e.payload));
  T.core.invoke('coach_last').then(show).catch(() => {});
}
