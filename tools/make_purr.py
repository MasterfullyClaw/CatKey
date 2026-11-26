# Synthesize a seamless cat-purr loop (16 kHz mono 16-bit WAV).
# A purr is a train of ~25 Hz laryngeal pulses on both the out-breath (louder,
# a little slower) and the in-breath (softer, a little faster), shaped by the
# throat and chest (resonances around 150, 400 and 900 Hz).
import numpy as np, wave, sys
from scipy.signal import butter, sosfilt, iirpeak, lfilter

SR = 16000
rng = np.random.default_rng(7)

def breath_cycle(exhale_s, inhale_s, gap_s):
    segs = []
    for dur, rate, amp, kind in ((exhale_s, 24.5, 1.0, 'ex'), (gap_s, 0, 0, 'gap'), (inhale_s, 27.0, 0.62, 'in'), (gap_s * 1.6, 0, 0, 'gap')):
        n = int(dur * SR)
        if kind == 'gap':
            segs.append((np.zeros(n), np.zeros(n)))
            continue
        clicks = np.zeros(n)
        t = 0.0
        while True:
            period = 1.0 / (rate * (1 + rng.normal(0, 0.018)))
            t += period
            i = int(t * SR)
            if i >= n - 200:
                break
            # each pulse: a short decaying noise burst plus a soft low thump
            L = 160
            env = np.exp(-np.arange(L) / (SR * 0.0045))
            burst = rng.normal(0, 1, L) * env
            thump = np.sin(2 * np.pi * (48 + rng.normal(0, 3)) * np.arange(L) / SR) * np.exp(-np.arange(L) / (SR * 0.012)) * 1.6
            clicks[i:i + L] += (burst + thump) * (0.85 + 0.3 * rng.random())
        x = np.linspace(0, 1, n)
        # smooth swell: quick-ish rise, long fall
        shape = np.sin(np.pi * np.clip(x, 0, 1)) ** 0.7
        shape *= (1 - 0.25 * x) if kind == 'ex' else (0.8 + 0.2 * x)
        air = rng.normal(0, 1, n) * shape * 0.05
        segs.append((clicks * shape * amp, air * amp))
    pulses = np.concatenate([s[0] for s in segs])
    air = np.concatenate([s[1] for s in segs])
    return pulses, air

cycles = []
total = 0.0
while total < 26.0:
    ex = 1.55 + rng.normal(0, 0.12)
    inh = 1.12 + rng.normal(0, 0.08)
    gap = 0.06 + abs(rng.normal(0, 0.02))
    cycles.append(breath_cycle(ex, inh, gap))
    total += ex + inh + gap * 2.6
pulses = np.concatenate([c[0] for c in cycles])
air = np.concatenate([c[1] for c in cycles])

def peak(sig, f, q):
    b, a = iirpeak(f, q, fs=SR)
    return lfilter(b, a, sig)

body = 1.0 * peak(pulses, 150, 1.4) + 0.65 * peak(pulses, 380, 2.2) + 0.28 * peak(pulses, 900, 3.0) + 0.12 * pulses
air_f = sosfilt(butter(2, [350, 1600], btype='band', fs=SR, output='sos'), air)
sig = body + air_f * 0.8
sig = sosfilt(butter(4, 2200, btype='low', fs=SR, output='sos'), sig)
sig = sosfilt(butter(2, 30, btype='high', fs=SR, output='sos'), sig)
# gentle loop seam: the file starts and ends in a breath gap; fade 40 ms each side anyway
f = int(0.04 * SR)
sig[:f] *= np.linspace(0, 1, f)
sig[-f:] *= np.linspace(1, 0, f)
sig = sig / np.max(np.abs(sig))
k = 3.2
sig = np.tanh(sig * k) / np.tanh(k)
sig = sosfilt(butter(4, 2200, btype='low', fs=SR, output='sos'), sig)
sig = sig / np.max(np.abs(sig)) * (10 ** (-4 / 20))
pcm = (sig * 32767).astype('<i2')
out = sys.argv[1]
with wave.open(out, 'wb') as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR); w.writeframes(pcm.tobytes())
print(out, len(pcm) / SR, 's', len(pcm) * 2 // 1024, 'KB')
