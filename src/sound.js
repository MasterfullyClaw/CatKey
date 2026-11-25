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
