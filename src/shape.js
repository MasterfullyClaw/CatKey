// See-through windows clip themselves to what they show (Windows only). On some PCs the
// graphics driver draws a see-through window's empty parts as a solid box; clipped to the
// cat, the wheel or the tip, nothing else can show, and clicks around them go through.
// parts: [[selector, padding px, round?], …] (round: an ellipse around it, for a cat that
// isn't box-shaped). follow: keep tracking (things that move by themselves).

const T = window.__TAURI__;
const ON = !!T && /Windows/i.test(navigator.userAgent);

function shapesOf(parts) {
  const out = [];
  for (const [sel, pad = 0, round = false] of parts) {
    for (const el of document.querySelectorAll(sel)) {
      if (el.hidden || el.closest('[hidden]')) continue;
      const cs = getComputedStyle(el);
      if (cs.display === 'none' || cs.visibility === 'hidden' || Number(cs.opacity) < 0.02) continue;
      const r = el.getBoundingClientRect();
      if (r.width < 1 || r.height < 1) continue;
      const side = Math.min(r.width, r.height);
      const raw = cs.borderTopLeftRadius || '0';
      let rad = parseFloat(raw) || 0;
      if (raw.trim().endsWith('%')) rad = (side * rad) / 100;
      if (round) rad = Math.max(r.width, r.height);
      out.push({ x: r.left - pad, y: r.top - pad, w: r.width + 2 * pad, h: r.height + 2 * pad, r: Math.min(rad, side / 2) + pad });
    }
  }
  return out.map((s) => ({ x: Math.round(s.x), y: Math.round(s.y), w: Math.round(s.w), h: Math.round(s.h), r: Math.round(s.r) }));
}

// For checking a page in a browser: what the window would be clipped to.
window.__shapesOf = shapesOf;

export function autoShape(parts, { follow = false, fps = 30 } = {}) {
  if (!ON) return { now() {} };
  let last = '';
  let until = 0;
  let raf = 0;
  let prev = 0;
  const send = (shapes) => {
    const key = JSON.stringify(shapes);
    if (key === last) return;
    last = key;
    window.__shapes = shapes;
    T.core.invoke('shape_window', { shapes }).catch(() => {});
  };
  const frame = (t) => {
    raf = 0;
    if (t - prev >= 1000 / fps) {
      prev = t;
      if (!document.hidden) send(shapesOf(parts));
    }
    if (follow || performance.now() < until) raf = requestAnimationFrame(frame);
  };
  // Track for a moment after anything changes (transitions and spins settle within it).
  const kick = (ms = 900) => {
    until = Math.max(until, performance.now() + ms);
    if (!raf) raf = requestAnimationFrame(frame);
  };
  new MutationObserver(() => kick()).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['class', 'hidden', 'style'] });
  for (const ev of ['transitionend', 'animationend', 'resize']) window.addEventListener(ev, () => kick(300), true);
  document.addEventListener('visibilitychange', () => { last = ''; kick(); });
  kick();
  return {
    // Shape right now (and keep following for a moment).
    now(ms) { last = ''; kick(ms); },
  };
}
