// Tiny synthesized UI sounds (no audio files). Off unless "Cute sounds" is on.

let ctx = null;
let enabled = false;

export function setSoundsEnabled(on) {
  enabled = !!on;
}

function audio() {
  if (!ctx) {
    try {
      ctx = new AudioContext();
    } catch {
      return null;
    }
  }
  if (ctx.state === 'suspended') ctx.resume();
  return ctx;
}

function blip(from, to, ms, type = 'sine', gain = 0.06, delay = 0) {
  const a = audio();
  if (!a) return;
  const t0 = a.currentTime + delay;
  const osc = a.createOscillator();
  const g = a.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(from, t0);
  osc.frequency.exponentialRampToValueAtTime(to, t0 + ms / 1000);
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(gain, t0 + 0.012);
  g.gain.exponentialRampToValueAtTime(0.0001, t0 + ms / 1000);
  osc.connect(g).connect(a.destination);
  osc.start(t0);
  osc.stop(t0 + ms / 1000 + 0.02);
}

export const sounds = {
  on() {
    if (enabled) {
      blip(520, 880, 90);
      blip(880, 1180, 70, 'sine', 0.04, 0.07);
    }
  },
  off() {
    if (enabled) blip(700, 380, 120, 'sine', 0.05);
  },
  pop() {
    if (enabled) blip(380, 760, 70, 'triangle', 0.05);
  },
  mew() {
    if (enabled) {
      blip(660, 990, 110, 'sine', 0.05);
      blip(990, 740, 160, 'sine', 0.045, 0.11);
    }
  },
  saved() {
    if (enabled) {
      blip(660, 660, 70, 'sine', 0.045);
      blip(880, 880, 70, 'sine', 0.045, 0.08);
      blip(1320, 1320, 110, 'sine', 0.04, 0.16);
    }
  },
};

// ------------------------------------------------------------------ training sounds
// Train yourself has its own switch: these are the point of the game, so they play
// even when the everyday "Cute sounds" are off.

let trainOn = true;
export function setTrainSounds(on) {
  trainOn = !!on;
}

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
  osc.connect(g).connect(a.destination);
  osc.start(t0);
  osc.stop(end + 0.03);
}

export const trainSounds = {
  /** Right answer: a springy little "boing". */
  boing() {
    if (!trainOn) return;
    wobble(190, 560, 230, { gain: 0.08, vib: 60, vibHz: 22 });
    wobble(380, 1120, 160, { gain: 0.02, delay: 0.01 });
  },
  /** Wrong answer: a soft, short "mew" (never a buzzer). */
  meow() {
    if (!trainOn) return;
    wobble(560, 820, 140, { type: 'triangle', gain: 0.045, vib: 18, vibHz: 9 });
    wobble(820, 470, 230, { type: 'triangle', gain: 0.04, vib: 14, vibHz: 7, delay: 0.13 });
  },
  /** Every few in a row: a sparkly little run. */
  streak(level = 1) {
    if (!trainOn) return;
    const base = 660 * Math.pow(1.06, Math.min(level, 6));
    [1, 1.25, 1.5, 2].forEach((m, i) => wobble(base * m, base * m * 1.01, 120, { gain: 0.035, delay: 0.06 * i }));
  },
  /** End of a round. */
  done() {
    if (!trainOn) return;
    [523, 659, 784, 1046].forEach((f, i) => wobble(f, f, 160, { gain: 0.04, delay: 0.09 * i }));
  },
};
