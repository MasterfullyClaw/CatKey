// Cat Key's sounds, all made right here (no audio files): a soft paw tap and a whole
// choir of little cat voices (mews, chirps, trills, "mrrp", a happy "nya" and purrs).
// A voice is a buzzing tone shaped by mouth-like resonances that glide from "ee" to
// "ow", the way a real meow does. On by default; one switch in Settings.

let ctx = null;
let out = null;
let enabled = true;
let noiseBuf = null;
let purrBuf = null;
let lastVoice = 0;
let lastPick = '';

/**
 * Each part of Cat Key has its own sound switch (Settings → Sounds); the main switch
 * mutes them all. ui: paw taps on clicks · cat: the cat's mews, chirps and purrs ·
 * training: the game · tips: the coach's note popping up · tools: the Cat Dock catching
 * things and the focus timer's walk.
 */
export const PART_DEFAULTS = { ui: true, cat: true, training: true, tips: false, tools: true };
const parts = { ...PART_DEFAULTS };
export function setSoundParts(p) {
  for (const k of Object.keys(PART_DEFAULTS)) parts[k] = typeof p?.[k] === 'boolean' ? p[k] : PART_DEFAULTS[k];
}
/** Whether a part makes sound right now. */
export const soundOn = (part) => enabled && parts[part] !== false;

export function setSoundsEnabled(on) {
  enabled = !!on;
}

function audio() {
  if (!ctx) {
    try {
      ctx = new AudioContext();
      const comp = ctx.createDynamicsCompressor();
      comp.threshold.value = -18;
      comp.ratio.value = 4;
      out = ctx.createGain();
      out.gain.value = 0.9;
      out.connect(comp).connect(ctx.destination);
      loadSamples();
    } catch {
      return null;
    }
  }
  if (ctx.state === 'suspended' && !(typeof OfflineAudioContext !== 'undefined' && ctx instanceof OfflineAudioContext)) ctx.resume();
  return ctx;
}

function noise(a) {
  if (!noiseBuf) {
    noiseBuf = a.createBuffer(1, a.sampleRate, a.sampleRate);
    const d = noiseBuf.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
  }
  return noiseBuf;
}

/** Glide an AudioParam through [[0..1 of the duration, value], …]. */
function glide(param, pts, t0, dur) {
  param.setValueAtTime(pts[0][1], t0);
  for (const [x, v] of pts.slice(1)) param.exponentialRampToValueAtTime(Math.max(1, v), t0 + x * dur);
}

/**
 * One cat voice. f0: the pitch contour; f1/f2: the two mouth resonances (they open
 * from "ee" towards "ah" and close to "ow"). trill: a rolled "rrr" (Hz).
 */
function voice({ dur, f0, f1, f2, gain = 0.09, vib = 6.5, vibDepth = 12, breath = 0.035, trill = 0, delay = 0 }) {
  const a = audio();
  if (!a) return;
  const t0 = a.currentTime + 0.01 + delay;
  const t1 = t0 + dur;
  const src = a.createOscillator();
  src.type = 'sawtooth';
  const jitter = 1 + (Math.random() - 0.5) * 0.08; // no two meows quite alike
  glide(src.frequency, f0.map(([x, v]) => [x, v * jitter]), t0, dur);
  const lfo = a.createOscillator();
  lfo.frequency.value = vib;
  const lfoGain = a.createGain();
  lfoGain.gain.value = vibDepth;
  lfo.connect(lfoGain).connect(src.frequency);

  const mouth = a.createGain();
  for (const [curve, q, level] of [[f1, 6, 1.0], [f2, 8, 0.55], [[[0, 3600], [1, 3300]], 10, 0.22]]) {
    const bp = a.createBiquadFilter();
    bp.type = 'bandpass';
    bp.Q.value = q;
    glide(bp.frequency, curve, t0, dur);
    const g = a.createGain();
    g.gain.value = level * q * 0.6;
    src.connect(bp).connect(g).connect(mouth);
  }
  if (breath) {
    const n = a.createBufferSource();
    n.buffer = noise(a);
    n.loop = true;
    const nf = a.createBiquadFilter();
    nf.type = 'bandpass';
    nf.frequency.value = 3200;
    nf.Q.value = 0.9;
    const ng = a.createGain();
    ng.gain.value = breath;
    n.connect(nf).connect(ng).connect(mouth);
    n.start(t0, Math.random() * 0.5);
    n.stop(t1 + 0.02);
  }
  const env = a.createGain();
  env.gain.setValueAtTime(0.0001, t0);
  env.gain.exponentialRampToValueAtTime(gain, t0 + Math.min(0.05, dur * 0.25));
  env.gain.setValueAtTime(gain, t0 + dur * 0.62);
  env.gain.exponentialRampToValueAtTime(0.0001, t1);
  let tail = mouth;
  if (trill) {
    // A rolled "rrr": the voice flutters on and off many times a second.
    const am = a.createGain();
    am.gain.value = 0.55;
    const flutter = a.createOscillator();
    flutter.type = 'square';
    flutter.frequency.value = trill;
    const depth = a.createGain();
    depth.gain.setValueAtTime(0.45, t0);
    depth.gain.linearRampToValueAtTime(0.05, t1);
    flutter.connect(depth).connect(am.gain);
    mouth.connect(am);
    tail = am;
    flutter.start(t0);
    flutter.stop(t1 + 0.02);
  }
  // The "m" of a meow: the sound starts muffled and opens up, then closes again.
  const lips = a.createBiquadFilter();
  lips.type = 'lowpass';
  lips.Q.value = 0.7;
  lips.frequency.setValueAtTime(800, t0);
  lips.frequency.exponentialRampToValueAtTime(5200, t0 + Math.min(0.11, dur * 0.3));
  lips.frequency.setValueAtTime(5200, t0 + dur * 0.7);
  lips.frequency.exponentialRampToValueAtTime(1400, t1);
  tail.connect(env).connect(lips).connect(out);
  src.start(t0);
  src.stop(t1 + 0.03);
  lfo.start(t0);
  lfo.stop(t1 + 0.03);
}

// ------------------------------------------------------------------ real cat recordings
// When the app ships recordings (sounds/*.mp3), real meows and a real purr play; the
// made-up voices below fill in while they load, or when there are none.

const SAMPLE_FILES = { meow: [1, 2, 3, 4, 5].map((n) => `sounds/meow${n}.mp3`), purr: ['sounds/purr.mp3'] };
const samples = { meow: [], purr: [] };
let samplesLoading = null;
function loadSamples() {
  if (samplesLoading || !ctx || (typeof OfflineAudioContext !== 'undefined' && ctx instanceof OfflineAudioContext)) return samplesLoading;
  const a = ctx;
  samplesLoading = Promise.all(Object.entries(SAMPLE_FILES).flatMap(([kind, files]) => files.map(async (f) => {
    try {
      const r = await fetch(f);
      if (!r.ok) return;
      samples[kind].push(await a.decodeAudioData(await r.arrayBuffer()));
    } catch { /* no recording here: the made-up voices play */ }
  })));
  return samplesLoading;
}
/** Play a recording (a random one of its kind). False if there's none. */
function sample(kind, { gain = 0.4, rate = 1, delay = 0 } = {}) {
  const list = samples[kind];
  if (!list.length) return false;
  const a = audio();
  if (!a) return false;
  const src = a.createBufferSource();
  src.buffer = list[Math.floor(Math.random() * list.length)];
  src.playbackRate.value = rate;
  const g = a.createGain();
  g.gain.value = gain;
  src.connect(g).connect(out);
  src.start(a.currentTime + 0.01 + delay);
  return true;
}
const rnd = (a, b) => a + Math.random() * (b - a);

const VOICES = {
  mew: (g = 1, d = 0) => voice({ dur: 0.26, f0: [[0, 720], [0.4, 990], [1, 770]], f1: [[0, 1100], [0.4, 1550], [1, 1000]], f2: [[0, 2900], [0.4, 2450], [1, 1950]], gain: 0.119 * g, delay: d }),
  meow: (g = 1, d = 0) => voice({ dur: 0.62, f0: [[0, 500], [0.3, 770], [0.7, 650], [1, 430]], f1: [[0, 900], [0.35, 1450], [1, 760]], f2: [[0, 2700], [0.35, 2100], [1, 1250]], gain: 0.126 * g, delay: d }),
  mrrp: (g = 1, d = 0) => voice({ dur: 0.32, f0: [[0, 430], [0.55, 480], [1, 740]], f1: [[0, 800], [1, 1250]], f2: [[0, 1700], [1, 2300]], gain: 0.119 * g, trill: 27, delay: d }),
  chirp: (g = 1, d = 0) => voice({ dur: 0.12, f0: [[0, 830], [1, 1360]], f1: [[0, 1200], [1, 1650]], f2: [[0, 2500], [1, 3000]], gain: 0.098 * g, vibDepth: 4, delay: d }),
  trill: (g = 1, d = 0) => voice({ dur: 0.44, f0: [[0, 560], [0.5, 650], [1, 900]], f1: [[0, 1000], [1, 1420]], f2: [[0, 2200], [1, 2800]], gain: 0.112 * g, trill: 23, delay: d }),
  nya: (g = 1, d = 0) => voice({ dur: 0.34, f0: [[0, 650], [0.25, 1010], [1, 830]], f1: [[0, 1300], [0.3, 1650], [1, 1200]], f2: [[0, 2950], [0.3, 2600], [1, 2300]], gain: 0.119 * g, delay: d }),
  sad: (g = 1, d = 0) => voice({ dur: 0.5, f0: [[0, 700], [0.25, 760], [1, 440]], f1: [[0, 1100], [1, 720]], f2: [[0, 2500], [1, 1300]], gain: 0.091 * g, vibDepth: 18, delay: d }),
};

/** A short, warm purr (for petting and kneading), built once. */
function purr(ms = 1200, gain = 0.3) {
  const a = audio();
  if (!a) return;
  if (!purrBuf) {
    const sr = a.sampleRate;
    const len = Math.round(sr * 1.6);
    purrBuf = a.createBuffer(1, len, sr);
    const d = purrBuf.getChannelData(0);
    let t = 0;
    while (t < 1.55) {
      t += 1 / (25.5 * (1 + (Math.random() - 0.5) * 0.04));
      const i0 = Math.round(t * sr);
      for (let k = 0; k < sr * 0.012 && i0 + k < len; k++) d[i0 + k] += (Math.random() * 2 - 1) * Math.exp(-k / (sr * 0.004)) + Math.sin((2 * Math.PI * 50 * k) / sr) * Math.exp(-k / (sr * 0.01)) * 1.4;
    }
    // Warm it up: two gentle low-pass passes.
    for (let pass = 0; pass < 2; pass++) {
      let y = 0;
      for (let i = 0; i < len; i++) d[i] = y += 0.09 * (d[i] - y);
    }
    let peak = 0;
    for (let i = 0; i < len; i++) peak = Math.max(peak, Math.abs(d[i]));
    for (let i = 0; i < len; i++) d[i] /= peak || 1;
  }
  const t0 = a.currentTime + 0.01;
  const end = t0 + Math.min(ms, 1500) / 1000;
  const src = a.createBufferSource();
  src.buffer = purrBuf;
  const g = a.createGain();
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(gain, t0 + 0.12);
  g.gain.setValueAtTime(gain, end - 0.25);
  g.gain.exponentialRampToValueAtTime(0.0001, end);
  src.connect(g).connect(out);
  src.start(t0);
  src.stop(end + 0.02);
}

/** A soft paw on a cushion (one at a time: a click and its button's own sound don't double up). */
let lastTap = 0;
function tap(gain = 0.05) {
  const a = audio();
  if (!a) return;
  if (!(typeof OfflineAudioContext !== 'undefined' && a instanceof OfflineAudioContext)) {
    const now = performance.now();
    if (now - lastTap < 45) return;
    lastTap = now;
  }
  const t0 = a.currentTime + 0.005;
  const pitch = 1 + (Math.random() - 0.5) * 0.16;
  const o = a.createOscillator();
  o.type = 'sine';
  o.frequency.setValueAtTime(230 * pitch, t0);
  o.frequency.exponentialRampToValueAtTime(120 * pitch, t0 + 0.05);
  const g = a.createGain();
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(gain, t0 + 0.006);
  g.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.07);
  o.connect(g).connect(out);
  o.start(t0);
  o.stop(t0 + 0.08);
  const n = a.createBufferSource();
  n.buffer = noise(a);
  const lp = a.createBiquadFilter();
  lp.type = 'lowpass';
  lp.frequency.value = 1400;
  const ng = a.createGain();
  ng.gain.setValueAtTime(gain * 0.5, t0);
  ng.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.025);
  n.connect(lp).connect(ng).connect(out);
  n.start(t0, Math.random() * 0.5);
  n.stop(t0 + 0.03);
}

function pick(names) {
  const choices = names.filter((n) => n !== lastPick);
  const name = choices[Math.floor(Math.random() * choices.length)];
  lastPick = name;
  return VOICES[name];
}

/** Every click: a paw tap, and now and then a little voice (never two at once). */
export function clickSound() {
  if (soundOn('ui')) tap(0.045);
  if (!soundOn('cat')) return;
  const now = performance.now();
  if (now - lastVoice > 900 && Math.random() < 0.3) {
    lastVoice = now;
    // a little kitten mew (a real meow, higher and quicker), or one of the made-up chirps
    if (Math.random() < 0.6 && sample('meow', { gain: 0.2, rate: rnd(1.35, 1.65), delay: 0.02 })) return;
    pick(['mew', 'chirp', 'mrrp', 'nya', 'trill'])(0.6, 0.02);
  }
}

export const sounds = {
  on() {
    if (soundOn('ui')) tap(0.05);
    if (soundOn('cat')) VOICES.chirp(0.9, 0.03);
  },
  off() {
    if (soundOn('ui')) tap(0.05);
    if (soundOn('cat')) VOICES.mrrp(0.6, 0.03);
  },
  pop() {
    if (soundOn('ui')) tap(0.05);
  },
  mew() {
    if (!soundOn('cat')) return;
    lastVoice = performance.now();
    if (Math.random() < 0.75 && sample('meow', { gain: 0.32, rate: rnd(0.95, 1.2) })) return;
    pick(['mew', 'nya', 'mrrp', 'meow', 'trill'])();
  },
  saved() {
    if (soundOn('cat')) VOICES.trill(0.9);
  },
  /** Petting the cat: making biscuits on the keyboard. */
  knead() {
    if (!soundOn('cat')) return;
    if (!sample('purr', { gain: 0.55, rate: rnd(0.95, 1.05) })) purr(1700, 0.3);
    for (let i = 0; i < 4; i++) setTimeout(() => tap(0.03), 60 + i * 230);
    if (Math.random() < 0.6 && !sample('meow', { gain: 0.26, rate: rnd(1.05, 1.3), delay: 1.0 })) pick(['mew', 'nya', 'mrrp', 'trill'])(0.7, 0.9);
  },
  /** The Cat Dock caught something (or the timer's walk begins): a happy trill. */
  caught() {
    if (soundOn('tools')) VOICES.trill(0.85);
  },
  /** A coach tip popping up: one soft, short mrrp. */
  tip() {
    if (soundOn('tips')) VOICES.mrrp(0.45, 0.02);
  },
  /**
   * The spinning cat's song (end of a focus timer): our own round "oo-ee-ah" sung
   * over and over while it spins, a little higher each time round, and a whoosh.
   */
  spin(secs = 4) {
    if (!soundOn('tools')) return;
    const beat = 0.4;
    const n = Math.max(2, Math.floor(secs / beat));
    for (let i = 0; i < n; i++) {
      const lift = 1 + (i % 4) * 0.045;
      voice({ dur: 0.34, delay: i * beat, gain: 0.1, vib: 7.5, vibDepth: 16, breath: 0.02,
        f0: (i % 2 ? [[0, 610], [0.4, 860], [1, 560]] : [[0, 540], [0.35, 780], [1, 620]]).map(([x, v]) => [x, v * lift]),
        f1: [[0, 330], [0.3, 300], [0.65, 760], [1, 860]],
        f2: [[0, 780], [0.3, 2350], [0.65, 1350], [1, 1180]] });
    }
    const a = audio();
    if (!a) return;
    const t0 = a.currentTime + 0.02;
    const n2 = a.createBufferSource();
    n2.buffer = noise(a);
    n2.loop = true;
    const bp = a.createBiquadFilter();
    bp.type = 'bandpass';
    bp.Q.value = 1.4;
    bp.frequency.setValueAtTime(500, t0);
    const g = a.createGain();
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.exponentialRampToValueAtTime(0.035, t0 + 0.3);
    g.gain.setValueAtTime(0.035, t0 + secs - 0.4);
    g.gain.exponentialRampToValueAtTime(0.0001, t0 + secs);
    // the whoosh swells and fades on each turn
    const lfo = a.createOscillator();
    lfo.frequency.value = 1 / beat;
    const lg = a.createGain();
    lg.gain.value = 900;
    lfo.connect(lg).connect(bp.frequency);
    n2.connect(bp).connect(g).connect(out);
    n2.start(t0);
    n2.stop(t0 + secs + 0.05);
    lfo.start(t0);
    lfo.stop(t0 + secs + 0.05);
  },
};

// ------------------------------------------------------------------ training sounds

const trainOn = () => soundOn('training');

function wobble(from, to, ms, { type = 'sine', gain = 0.07, vib = 0, vibHz = 0, delay = 0 } = {}) {
  const a = audio();
  if (!a) return;
  const t0 = a.currentTime + delay;
  const end = t0 + ms / 1000;
  const osc = a.createOscillator();
  const g = a.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(from, t0);
  osc.frequency.exponentialRampToValueAtTime(to, end);
  if (vib) {
    const lfo = a.createOscillator();
    const lg = a.createGain();
    lfo.frequency.value = vibHz;
    lg.gain.setValueAtTime(vib, t0);
    lg.gain.exponentialRampToValueAtTime(0.01, end);
    lfo.connect(lg).connect(osc.frequency);
    lfo.start(t0);
    lfo.stop(end + 0.02);
  }
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(gain, t0 + 0.015);
  g.gain.exponentialRampToValueAtTime(0.0001, end);
  osc.connect(g).connect(out);
  osc.start(t0);
  osc.stop(end + 0.03);
}

let rightRun = 0;
export const trainSounds = {
  /** Right answer: a springy "boing", and every so often a happy chirp. */
  boing() {
    if (!trainOn()) return;
    wobble(190, 560, 230, { gain: 0.08, vib: 60, vibHz: 22 });
    wobble(380, 1120, 160, { gain: 0.02, delay: 0.01 });
    if (++rightRun % 3 === 0 && !sample('meow', { gain: 0.24, rate: rnd(1.25, 1.45), delay: 0.2 })) pick(['chirp', 'nya', 'mrrp'])(0.7, 0.2);
  },
  /** Wrong answer: a soft, short sad mew (never a buzzer). */
  meow() {
    if (!trainOn()) return;
    rightRun = 0;
    if (!sample('meow', { gain: 0.26, rate: rnd(0.78, 0.86) })) VOICES.sad(0.85);
  },
  /** Every few in a row: a trill and a sparkly run. */
  streak(level = 1) {
    if (!trainOn()) return;
    VOICES.trill(0.8);
    const base = 660 * Math.pow(1.06, Math.min(level, 6));
    [1, 1.25, 1.5, 2].forEach((m, i) => wobble(base * m, base * m * 1.01, 120, { gain: 0.03, delay: 0.3 + 0.06 * i }));
  },
  /** End of a round: "mrrp… nya!" */
  done() {
    if (!trainOn()) return;
    if (!sample('meow', { gain: 0.3, rate: 1.15 })) {
      VOICES.mrrp(0.9);
      VOICES.nya(0.9, 0.38);
    }
    [523, 659, 784, 1046].forEach((f, i) => wobble(f, f, 160, { gain: 0.03, delay: 0.75 + 0.09 * i }));
  },
  /** A key typed right in the typing trainer: the softest tap. */
  key() {
    if (trainOn()) tap(0.03);
  },
};

/** For tests: render one sound offline (44.1 kHz mono) so its shape can be checked. */
export async function _render(name, secs = 1) {
  const off = new OfflineAudioContext(1, Math.round(44100 * secs), 44100);
  const saved = [ctx, out, noiseBuf, purrBuf];
  ctx = off;
  out = off.createGain();
  out.connect(off.destination);
  noiseBuf = null;
  purrBuf = null;
  try {
    if (name === 'purr') purr(1000);
    else if (name === 'tap') tap(0.05);
    else VOICES[name]();
    const buf = await off.startRendering();
    return Array.from(buf.getChannelData(0));
  } finally {
    [ctx, out, noiseBuf, purrBuf] = saved;
  }
}
export const _voices = [...Object.keys(VOICES), 'purr', 'tap'];
