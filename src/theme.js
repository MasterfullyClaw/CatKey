// Colours for the whole app from the cat you picked. The house cats keep their classic
// themes (classes in styles.css); every other cat brings a palette: an accent colour and
// the hue of the surfaces. Each window (main, Dock, switcher, walk, CatGPT) paints itself
// from the same variables, so they all change together.

const HEX = /^#[0-9a-f]{6}$/i;
const rgb = (hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
const toHex = (c) => `#${c.map((v) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0')).join('')}`;

export function mix(a, b, t) {
  const x = rgb(a), y = rgb(b);
  return toHex(x.map((v, i) => v + (y[i] - v) * t));
}

function hsl(h, s, l) {
  s /= 100; l /= 100;
  const k = (n) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return toHex([f(0) * 255, f(8) * 255, f(4) * 255]);
}

function lum(hex) {
  const c = rgb(hex).map((v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; });
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}
export function contrast(a, b) {
  const x = lum(a), y = lum(b);
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}
/** Move a colour toward `to` until it reads on `bg` (at least `min`:1). */
function readable(c, bg, min, to) {
  let out = c;
  for (let t = 0; t <= 1 && contrast(out, bg) < min; t += 0.05) out = mix(c, to, t);
  return out;
}
function rgba(hex, a) {
  const [r, g, b] = rgb(hex);
  return `rgba(${r},${g},${b},${a})`;
}

/** The variables that dress a window for a cat's palette, in light or dark. Null for the house themes. */
export function paletteVars(pal, dark) {
  if (!pal || !HEX.test(pal.ac || '')) return null;
  const h = (((pal.h ?? 0) % 360) + 360) % 360;
  const s = Math.max(0, Math.min(60, pal.s ?? 22));
  const ac = pal.ac;
  const v = dark
    ? {
      '--bg': hsl(h, s, 8.5), '--s1': hsl(h, s, 12.5), '--s2': hsl(h, s, 16), '--s3': hsl(h, s, 19.5), '--line': hsl(h, s * 0.9, 22.5),
      '--tx': hsl(h, Math.min(40, s + 12), 96), '--mu': hsl(h, Math.min(24, s), 72), '--cap': hsl(h, s, 17.5), '--capb': hsl(h, s, 6),
    }
    : {
      '--bg': hsl(h, Math.min(70, s * 3), 97.5), '--s1': '#FFFFFF', '--s2': hsl(h, Math.min(60, s * 2.5), 95), '--s3': hsl(h, Math.min(50, s * 2.2), 91.5),
      '--line': hsl(h, Math.min(45, s * 2), 87.5), '--tx': hsl(h, Math.min(30, s), 15), '--mu': hsl(h, Math.min(14, s * 0.7), 40),
      '--cap': '#FFFFFF', '--capb': hsl(h, Math.min(30, s * 1.5), 82),
    };
  v['--ac'] = ac;
  v['--acs'] = rgba(ac, dark ? 0.17 : 0.16);
  v['--act'] = dark ? readable(mix(ac, '#ffffff', 0.4), v['--s1'], 6, '#ffffff') : readable(mix(ac, '#000000', 0.3), '#FFFFFF', 4.6, '#000000');
  v['--aci'] = contrast(ac, '#FFFFFF') >= 4.5 ? '#FFFFFF' : readable(mix(ac, '#000000', 0.78), ac, 5, '#000000');
  return v;
}

export const PALETTE_KEYS = ['--bg', '--s1', '--s2', '--s3', '--line', '--tx', '--mu', '--cap', '--capb', '--ac', '--acs', '--act', '--aci'];

/** Put a palette on an element (or take it off again with null). */
export function applyVars(el, vars) {
  if (!el) return;
  for (const k of PALETTE_KEYS) {
    if (vars && vars[k]) el.style.setProperty(k, vars[k]);
    else el.style.removeProperty(k);
  }
}
