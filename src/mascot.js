// The Cat Key mascot: one original chibi cat drawn in SVG.
// Skins and moods are CSS classes (see styles.css, "Mascot"), so switching is instant.

export const SKINS = [
  { id: 'mochi', name: 'Mochi', fur: 'Cream' },
  { id: 'kuro', name: 'Kuro', fur: 'Midnight' },
  { id: 'tora', name: 'Tora', fur: 'Tabby' },
  { id: 'sora', name: 'Sora', fur: 'Sky' },
];

export function skinName(id) {
  return (SKINS.find((s) => s.id === id) || SKINS[0]).name;
}

const EYE = (cx) => `
  <g class="eye">
    <ellipse class="f-eyed eye-rim" cx="${cx}" cy="97" rx="15" ry="18"/>
    <g class="look">
      <ellipse class="f-eye" cx="${cx}" cy="99" rx="12.5" ry="15"/>
      <path class="f-eyed" opacity=".55" d="M${cx - 12.5} 93Q${cx} 80 ${cx + 12.5} 93Q${cx} 88 ${cx - 12.5} 93Z"/>
      <ellipse class="f-eyed" cx="${cx}" cy="100.5" rx="6.5" ry="9.5"/>
      <circle fill="#fff" cx="${cx - 6}" cy="90" r="5.4"/>
      <circle fill="#fff" cx="${cx + 6.5}" cy="107" r="2.5"/>
      <circle fill="#fff" opacity=".9" cx="${cx + 7.5}" cy="89" r="1.5"/>
    </g>
  </g>`;

function svg() {
  return `<svg class="mk" viewBox="0 0 200 190" role="img" aria-label="">
  <ellipse cx="100" cy="186" rx="58" ry="4.5" fill="#000" opacity=".16"/>
  <g class="tail">
    <path class="s-line" fill="none" stroke-width="17" stroke-linecap="round" d="M146 168C180 170 191 140 178 120C172 111 161 114 165 123"/>
    <path class="s-fur" fill="none" stroke-width="12" stroke-linecap="round" d="M146 168C180 170 191 140 178 120C172 111 161 114 165 123"/>
    <path class="s-tip" fill="none" stroke-width="12" stroke-linecap="round" d="M176 117C170 111 162 115 165 123"/>
  </g>
  <ellipse class="f-fur s-line" stroke-width="2.5" cx="100" cy="152" rx="50" ry="33"/>
  <ellipse class="f-belly" cx="100" cy="161" rx="27" ry="18"/>
  <g class="paws">
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="80" cy="179" rx="14" ry="9"/>
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="120" cy="179" rx="14" ry="9"/>
    <path class="s-line" stroke-width="1.8" stroke-linecap="round" opacity=".5" d="M76 176v6M84 176v6M116 176v6M124 176v6"/>
  </g>
  <g class="head">
    <g class="ear ear-l"><path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M38 76 44 22Q46 13 54 17L88 42Z"/><path class="f-ear" d="M47 64 50 30Q51 25 56 28L76 44Z"/></g>
    <g class="ear ear-r"><path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M162 76 156 22Q154 13 146 17L112 42Z"/><path class="f-ear" d="M153 64 150 30Q149 25 144 28L124 44Z"/></g>
    <path class="f-fur s-line" stroke-width="2.5" d="M28 94C28 54 60 34 100 34C140 34 172 54 172 94C172 130 140 148 100 148C60 148 28 130 28 94Z"/>
    <ellipse class="f-muzzle muzzle" cx="100" cy="122" rx="30" ry="19"/>
    <path class="s-stripe stripes" fill="none" stroke-width="3.6" stroke-linecap="round" d="M91 43l2 12M100 40v14M109 43l-2 12M31 92l11 3M30 100l11 1M169 92l-11 3M170 100l-11 1"/>
    <g class="eyes-open">${EYE(70)}${EYE(130)}</g>
    <path class="eyes-happy s-eyed" fill="none" stroke-width="4.4" stroke-linecap="round" d="M57 101Q70 85 83 101M117 101Q130 85 143 101"/>
    <path class="eyes-nap s-eyed" fill="none" stroke-width="4" stroke-linecap="round" d="M57 97Q70 107 83 97M117 97Q130 107 143 97"/>
    <ellipse class="f-blush" opacity=".6" cx="50" cy="118" rx="10.5" ry="5.5"/>
    <ellipse class="f-blush" opacity=".6" cx="150" cy="118" rx="10.5" ry="5.5"/>
    <path class="f-nose" d="M95 113.5Q100 109.5 105 113.5Q102.6 118.5 100 119.5Q97.4 118.5 95 113.5Z"/>
    <path class="mouth-plain s-line" fill="none" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" d="M90 121.5Q95 128.5 100 121.5Q105 128.5 110 121.5"/>
    <path class="mouth-smile s-line" fill="#E8577E" stroke-width="2.2" stroke-linejoin="round" d="M91 121Q100 136 109 121Q100 125 91 121Z"/>
    <path class="s-whisker" stroke-width="1.8" stroke-linecap="round" opacity=".75" d="M42 110 20 106M42 116 19 118M158 110 180 106M158 116 181 118"/>
    <path class="s-collar" fill="none" stroke-width="7.5" stroke-linecap="round" d="M66 141Q100 157 134 141"/>
    <g class="f-gold s-line charm" stroke-width="1.2">
      <circle cx="100" cy="153.5" r="5.2"/><rect x="98.6" y="157.5" width="2.8" height="11" rx="1"/>
      <rect x="101" y="163.2" width="4" height="2.4" rx=".8"/><rect x="101" y="166.6" width="3" height="2.2" rx=".8"/>
    </g>
    <circle class="f-fur" cx="100" cy="153.5" r="2"/>
  </g>
  <g class="bang"><circle class="f-accent" cx="176" cy="30" r="13"/><rect fill="#fff" x="174.3" y="20.5" width="3.4" height="11" rx="1.7"/><circle fill="#fff" cx="176" cy="36.5" r="2"/></g>
  <g class="zz"><text class="f-zz z1" x="160" y="44" font-size="17">z</text><text class="f-zz z2" x="172" y="26" font-size="24">Z</text></g>
  <path class="spark f-gold s-line" stroke-width="1" d="M26 34l2.2 6.2 6.2 2.2-6.2 2.2L26 51l-2.2-6.2-6.2-2.2 6.2-2.2zM174 40l1.6 4.6 4.6 1.6-4.6 1.6L174 52.4l-1.6-4.6-4.6-1.6 4.6-1.6z"/>
</svg>`;
}

const MOODS = ['awake', 'happy', 'listen', 'nap'];

export class Mascot {
  constructor(host, { skin = 'mochi', size = 148, track = false, name = '' } = {}) {
    this.host = host;
    host.innerHTML = svg();
    this.svg = host.firstElementChild;
    this.svg.style.width = `${size}px`;
    this.svg.style.height = `${Math.round(size * 0.95)}px`;
    this.looks = [...this.svg.querySelectorAll('.look')];
    this.base = 'awake';
    this.name = name;
    this.setSkin(skin);
    this.setMood('awake');
    if (track) {
      this.onMove = (e) => {
        this.px = e.clientX;
        this.py = e.clientY;
        if (!this.raf) this.raf = requestAnimationFrame(() => this.look());
      };
      window.addEventListener('pointermove', this.onMove, { passive: true });
    }
  }

  look() {
    this.raf = 0;
    if (this.mood === 'nap' || !this.svg.isConnected) return;
    const r = this.svg.getBoundingClientRect();
    const dx = this.px - (r.left + r.width / 2);
    const dy = this.py - (r.top + r.height * 0.5);
    const dist = Math.hypot(dx, dy) || 1;
    const k = Math.min(dist / 90, 1) * 2.6;
    const t = `translate(${((dx / dist) * k).toFixed(2)}px, ${((dy / dist) * k * 0.8).toFixed(2)}px)`;
    for (const g of this.looks) g.style.transform = t;
  }

  setSkin(skin) {
    for (const c of [...this.svg.classList]) if (c.startsWith('skin-')) this.svg.classList.remove(c);
    this.svg.classList.add(`skin-${skin}`);
    this.skin = skin;
    this.label();
  }

  setName(name) {
    this.name = name;
    this.label();
  }

  label() {
    const n = this.name || skinName(this.skin);
    const words = { awake: 'awake', happy: 'happy', listen: 'listening', nap: 'napping' };
    this.svg.setAttribute('aria-label', `${n} the cat, ${words[this.mood] || 'awake'}`);
  }

  setMood(mood) {
    if (!MOODS.includes(mood)) return;
    if (mood === this.base && (this.mood === mood || this.mood === 'happy')) return;
    if (mood !== 'happy') this.base = mood;
    clearTimeout(this.timer);
    this.apply(mood);
  }

  apply(mood) {
    for (const m of MOODS) this.svg.classList.toggle(`mood-${m}`, m === mood);
    this.mood = mood;
    if (mood === 'nap') for (const g of this.looks) g.style.transform = '';
    this.label();
  }

  /** A shortcut fired (or someone poked the cat): happy face and a paw tap. */
  react(ms = 1200) {
    if (this.base === 'nap') return;
    this.apply('happy');
    this.svg.classList.remove('tap');
    void this.svg.getBBox();
    this.svg.classList.add('tap');
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.svg.classList.remove('tap');
      this.apply(this.base);
    }, ms);
  }

  destroy() {
    if (this.onMove) window.removeEventListener('pointermove', this.onMove);
    clearTimeout(this.timer);
    this.host.innerHTML = '';
  }
}
