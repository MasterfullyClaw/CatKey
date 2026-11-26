// The Cat Key mascot: one original chibi cat drawn in SVG.
// Fur colours are CSS variables (see styles.css, "Mascot"), outfits are SVG pieces
// shown or hidden, and moods are classes, so changing any of them is instant.

/** The four house cats. Each also gives the app its accent colour. */
export const SKINS = [
  { id: 'mochi', name: 'Mochi', fur: 'Cream' },
  { id: 'kuro', name: 'Kuro', fur: 'Midnight' },
  { id: 'tora', name: 'Tora', fur: 'Tabby' },
  { id: 'sora', name: 'Sora', fur: 'Sky' },
];

export function skinName(id) {
  return (SKINS.find((s) => s.id === id) || CATS.find((c) => c.id === id) || SKINS[0]).name;
}

const C = (fur, belly, line, ear, eye, eyed, blush, nose, collar, gold, stripe, stripeOp, tip, whisker, muzzle, muzzleOp, accent, zz) =>
  ({ fur, belly, line, ear, eye, eyed, blush, nose, collar, gold, stripe, stripeOp, tip, whisker, muzzle, muzzleOp, accent, zz });

/**
 * Cats to pick from. The house cats use their skin; the meme cats (the kittens of
 * "Men:") bring their own fur and their signature outfit.
 */
export const CATS = [
  { id: 'mochi', name: 'Mochi', full: 'Mochi', tag: 'Cream', skin: 'mochi', theme: 'mochi' },
  { id: 'kuro', name: 'Kuro', full: 'Kuro', tag: 'Midnight', skin: 'kuro', theme: 'kuro' },
  { id: 'tora', name: 'Tora', full: 'Tora', tag: 'Tabby', skin: 'tora', theme: 'tora' },
  { id: 'sora', name: 'Sora', full: 'Sora', tag: 'Sky', skin: 'sora', theme: 'sora' },
  { id: 'yaqub', name: "Ya'qub", full: "Ya'qub Qamar Ad-Din Dibiazah", tag: 'YQD', meme: true, theme: 'tora', gear: ['turban-white'],
    colors: C('#F6B26B', '#FFF6EC', '#7A4416', '#FFC6B3', '#F0B429', '#3A2400', '#FF9E8A', '#FF8A80', '#C0392B', '#FFD24D', '#E08A3C', 1, '#FFF6EC', '#8A4A1C', '#FFF6EC', 1, '#C0392B', '#FFA963') },
  { id: 'khalid', name: 'Khalid', full: 'Khalid Kashmiri', tag: 'KK', meme: true, theme: 'kuro', gear: ['wrap'],
    colors: C('#8E8E96', '#B9B9C2', '#2F2F38', '#E8A0B4', '#F2C94C', '#2A2100', '#E58FA8', '#D97A92', '#C9A227', '#FFD666', '#6E6E78', 0, '#75757E', '#D0D0D8', '#A9A9B3', 0.6, '#C9A227', '#B9B9C2') },
  { id: 'khidir', name: 'Khidir', full: 'Khidir Karawita', tag: 'KKa', meme: true, theme: 'sora', gear: ['band-blue', 'shield'],
    colors: C('#A7A9AE', '#F4F4F6', '#3A3D45', '#F5B5C8', '#5DADE2', '#0E2A4F', '#F7A8C0', '#F08CA8', '#2E86C1', '#E8EEF5', '#7D8089', 1, '#F4F4F6', '#5A5D66', '#F4F4F6', 1, '#2E86C1', '#8FC3EA') },
  { id: 'ismail', name: 'Ismail', full: 'Ismail Ahmad Kanabawi', tag: 'IAK', meme: true, theme: 'mochi', gear: ['cape-red'],
    colors: C('#FBEFDC', '#FFFFFF', '#6E5640', '#FFC2C9', '#4CAF50', '#10361A', '#FFA9B6', '#FF8FA3', '#B71C1C', '#F4C542', '#EADBC4', 0, '#EFDCC0', '#A08C78', '#FFFFFF', 0, '#B71C1C', '#F4C542') },
  { id: 'usman', name: 'Usman', full: 'Usman Abdul Jalil Sisha', tag: 'UAJS', meme: true, theme: 'tora', gear: ['turban-orange', 'glasses'],
    colors: C('#FFF9F0', '#FFFFFF', '#6B5A4C', '#FFC4CF', '#64B5F6', '#0D2B4A', '#FFB0C4', '#FF9BB5', '#E67E22', '#FFD24D', '#EADFD2', 0, '#F2E6D8', '#A9998B', '#FFFFFF', 0, '#E67E22', '#FFB25B') },
  { id: 'sonbol', name: 'Sonbol', full: 'Sonbol', tag: 'The helmet', meme: true, theme: 'tora', gear: ['helmet'],
    colors: C('#F39C4A', '#FFE3C4', '#6A3410', '#FFB8A0', '#66BB6A', '#0B3D14', '#FF8E7A', '#F47C6A', '#7B8794', '#C9D1D9', '#C8691C', 1, '#C8691C', '#7A3E12', '#FFE3C4', 1, '#7B8794', '#FFA963') },
  { id: 'jasmine', name: 'Jasmine', full: 'Jasmine', tag: 'The kitten', meme: true, theme: 'mochi', gear: ['flower'], ears: 'fold',
    colors: C('#9EA3AA', '#FFFFFF', '#3D4047', '#F3B3C6', '#D9A441', '#2E2410', '#F7A6BF', '#F18CA9', '#E91E63', '#FFF3B0', '#5D6168', 1, '#5D6168', '#F0F0F3', '#FFFFFF', 1, '#E91E63', '#F7A6BF') },
];

/** Outfit pieces. `rank` is the rank (index in RANKS) that unlocks it. */
export const GEAR = [
  { id: 'belt-white', name: 'White belt', slot: 'belt', rank: 0, color: '#F5F5F5' },
  { id: 'belt-yellow', name: 'Yellow belt', slot: 'belt', rank: 1, color: '#F4D03F' },
  { id: 'flower', name: 'Jasmine flower', slot: 'head', rank: 1 },
  { id: 'belt-orange', name: 'Orange belt', slot: 'belt', rank: 2, color: '#F39C12' },
  { id: 'band-red', name: 'Red headband', slot: 'head', rank: 2 },
  { id: 'belt-green', name: 'Green belt', slot: 'belt', rank: 3, color: '#27AE60' },
  { id: 'glasses', name: 'Round glasses', slot: 'face', rank: 3 },
  { id: 'belt-blue', name: 'Blue belt', slot: 'belt', rank: 4, color: '#2E86C1' },
  { id: 'band-black', name: 'Ninja band', slot: 'head', rank: 4 },
  { id: 'band-blue', name: 'Blue band', slot: 'head', rank: 4 },
  { id: 'belt-purple', name: 'Purple belt', slot: 'belt', rank: 5, color: '#8E44AD' },
  { id: 'helmet', name: 'Steel helmet', slot: 'head', rank: 5, hat: true },
  { id: 'wrap', name: 'Desert wrap', slot: 'head', rank: 5, hat: true },
  { id: 'belt-brown', name: 'Brown belt', slot: 'belt', rank: 6, color: '#8B5A2B' },
  { id: 'cape-black', name: 'Shadow cape', slot: 'back', rank: 6, color: '#26222E', line: '#0C0A10' },
  { id: 'belt-black', name: 'Black belt', slot: 'belt', rank: 7, color: '#1F1F24' },
  { id: 'cape-red', name: 'War cape', slot: 'back', rank: 7, color: '#B71C1C', line: '#5E0E0E' },
  { id: 'shield', name: 'Round shield', slot: 'hand', rank: 7 },
  { id: 'belt-gold', name: 'Golden belt', slot: 'belt', rank: 8, color: '#FFD24D' },
  { id: 'turban-white', name: 'White turban', slot: 'head', rank: 8, hat: true },
  { id: 'turban-orange', name: 'Saffron turban', slot: 'head', rank: 8, hat: true },
  { id: 'aura', name: 'Warrior glow', slot: 'aura', rank: 8 },
  { id: 'cape-purple', name: 'Royal cape', slot: 'back', rank: 9, color: '#6C3483', line: '#3B1A49' },
  { id: 'crown', name: 'Crown', slot: 'head', rank: 9 },
];
export const gearById = (id) => GEAR.find((g) => g.id === id);

/** Cat Warrior ranks. XP: see app.js (training, real shortcut use, practice days). */
export const RANKS = [
  { name: 'Kitten', xp: 0 },
  { name: 'House Cat', xp: 100 },
  { name: 'Alley Cat', xp: 300 },
  { name: 'Street Cat', xp: 700 },
  { name: 'Ninja Cat', xp: 1500 },
  { name: 'Samurai Cat', xp: 3000 },
  { name: 'Shadow Cat', xp: 6000 },
  { name: 'Keyboard Warrior', xp: 12000 },
  { name: 'Cat Warrior', xp: 25000 },
  { name: 'Nine Lives Legend', xp: 50000 },
];
export function rankFor(xp) {
  let i = 0;
  while (i + 1 < RANKS.length && xp >= RANKS[i + 1].xp) i++;
  const next = RANKS[i + 1];
  return { index: i, ...RANKS[i], next: next || null, pct: next ? Math.min(100, Math.round(((xp - RANKS[i].xp) / (next.xp - RANKS[i].xp)) * 100)) : 100 };
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

const BAND = (id, fill, line, tailsRight = true) => `
  <g data-gear="${id}">
    <path d="M33 72C60 57 140 57 167 72L165 84C140 70 60 70 35 84Z" fill="${fill}" stroke="${line}" stroke-width="2" stroke-linejoin="round"/>
    ${tailsRight
    ? `<path d="M165 76C180 72 190 62 197 54M165 80C182 82 191 90 194 100" fill="none" stroke="${fill}" stroke-width="7" stroke-linecap="round"/>`
    : `<path d="M35 76C20 72 10 62 3 54M35 80C18 82 9 90 6 100" fill="none" stroke="${fill}" stroke-width="7" stroke-linecap="round"/>`}
  </g>`;

const TURBAN = (id, cloth, fold, line, jewel) => `
  <g data-gear="${id}">
    <path d="M30 78C26 42 58 12 100 10C142 12 174 42 170 78C150 67 126 63 100 63C74 63 50 67 30 78Z" fill="${cloth}" stroke="${line}" stroke-width="2.5" stroke-linejoin="round"/>
    <path d="M37 64C62 44 138 32 165 58M35 50C70 24 130 22 160 42M54 30C80 17 120 15 146 25" fill="none" stroke="${fold}" stroke-width="3.2" stroke-linecap="round"/>
    <path class="plume" d="M101 50C104 30 115 10 133 -8C129 14 116 34 106 52Z" fill="#fff" stroke="${jewel}" stroke-width="1.8" stroke-linejoin="round"/>
    <circle cx="101" cy="53" r="7.5" fill="${jewel}" stroke="#F2C14E" stroke-width="3"/>
  </g>`;

let uid = 0;

function svg({ keyboard = false } = {}) {
  const n = ++uid;
  const petals = [0, 72, 144, 216, 288].map((a) => `<ellipse rx="5.2" ry="8.5" fill="#fff" stroke="#D8C7A8" stroke-width="1" transform="rotate(${a}) translate(0 -7)"/>`).join('');
  return `<svg class="mk" viewBox="0 0 200 190" role="img" aria-label="">
  <defs>
    <linearGradient id="mk-steel-${n}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#EEF2F5"/><stop offset="1" stop-color="#7F8B97"/></linearGradient>
    <radialGradient id="mk-aura-${n}"><stop offset=".35" stop-color="#FFD24D" stop-opacity=".55"/><stop offset="1" stop-color="#FFD24D" stop-opacity="0"/></radialGradient>
  </defs>
  <circle class="aura" data-gear="aura" cx="100" cy="112" r="98" fill="url(#mk-aura-${n})"/>
  <ellipse cx="100" cy="186" rx="58" ry="4.5" fill="#000" opacity=".16"/>
  <g data-gear="cape"><path d="M58 126C40 150 33 172 29 188L171 188C167 172 160 150 142 126Z" fill="var(--cape, #B71C1C)" stroke="var(--cape-line, #5E0E0E)" stroke-width="2.5" stroke-linejoin="round"/></g>
  <g class="tail">
    <path class="s-line" fill="none" stroke-width="17" stroke-linecap="round" d="M146 168C180 170 191 140 178 120C172 111 161 114 165 123"/>
    <path class="s-fur" fill="none" stroke-width="12" stroke-linecap="round" d="M146 168C180 170 191 140 178 120C172 111 161 114 165 123"/>
    <path class="s-tip" fill="none" stroke-width="12" stroke-linecap="round" d="M176 117C170 111 162 115 165 123"/>
  </g>
  <ellipse class="f-fur s-line" stroke-width="2.5" cx="100" cy="152" rx="50" ry="33"/>
  <ellipse class="f-belly" cx="100" cy="161" rx="27" ry="18"/>
  <g data-gear="belt">
    <path d="M55 163Q100 180 145 163" fill="none" stroke="var(--mk-line)" stroke-width="11" stroke-linecap="round"/>
    <path d="M55 163Q100 180 145 163" fill="none" stroke="var(--belt, #F5F5F5)" stroke-width="7.5" stroke-linecap="round"/>
    <path d="M97 175l-6 13M103 175l6 13" stroke="var(--mk-line)" stroke-width="7" stroke-linecap="round"/>
    <path d="M97 175l-6 13M103 175l6 13" stroke="var(--belt, #F5F5F5)" stroke-width="4" stroke-linecap="round"/>
    <rect x="93" y="167" width="14" height="11" rx="3.5" fill="var(--belt, #F5F5F5)" stroke="var(--mk-line)" stroke-width="2"/>
  </g>
  ${keyboard ? `<g class="kb">
    <rect x="30" y="169" width="140" height="24" rx="7" fill="var(--mk-kb, #2C2437)" stroke="var(--mk-line)" stroke-width="2"/>
    ${[0, 1, 2, 3, 4, 5, 6, 7, 8].map((i) => `<rect class="key k${i}" x="${37 + i * 14.2}" y="173.5" width="11.5" height="7" rx="2" fill="var(--mk-key, #F7F0FB)" opacity=".9"/>`).join('')}
    <rect class="key ksp" x="62" y="184" width="76" height="5.5" rx="2" fill="var(--mk-key, #F7F0FB)" opacity=".9"/>
  </g>` : ''}
  <g class="paw paw-l">
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="80" cy="179" rx="14" ry="9"/>
    <path class="s-line" stroke-width="1.8" stroke-linecap="round" opacity=".5" d="M76 176v6M84 176v6"/>
  </g>
  <g class="paw paw-r">
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="120" cy="179" rx="14" ry="9"/>
    <path class="s-line" stroke-width="1.8" stroke-linecap="round" opacity=".5" d="M116 176v6M124 176v6"/>
  </g>
  <g data-gear="shield">
    <circle cx="38" cy="160" r="24" fill="#8D6E63" stroke="#4E342E" stroke-width="3"/>
    <circle cx="38" cy="160" r="16" fill="none" stroke="#C9A227" stroke-width="3"/>
    <path d="M38 137v8M38 175v8M15 160h8M53 160h8" stroke="#C9A227" stroke-width="2.6" stroke-linecap="round"/>
    <circle cx="38" cy="160" r="6" fill="#C9A227" stroke="#7A5C10" stroke-width="2"/>
  </g>
  <g class="head">
    <g class="ear ear-l"><path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M38 76 44 22Q46 13 54 17L88 42Z"/><path class="f-ear" d="M47 64 50 30Q51 25 56 28L76 44Z"/></g>
    <g class="ear ear-r"><path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M162 76 156 22Q154 13 146 17L112 42Z"/><path class="f-ear" d="M153 64 150 30Q149 25 144 28L124 44Z"/></g>
    <path class="f-fur s-line" stroke-width="2.5" d="M28 94C28 54 60 34 100 34C140 34 172 54 172 94C172 130 140 148 100 148C60 148 28 130 28 94Z"/>
    <g class="ears-fold">
      <path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M42 60C44 40 60 30 76 35C71 44 62 53 49 63Z"/>
      <path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M158 60C156 40 140 30 124 35C129 44 138 53 151 63Z"/>
    </g>
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
    <g data-gear="glasses" fill="none" stroke="#C9A227" stroke-width="3">
      <circle cx="70" cy="98" r="19" fill="#C8E6FF" fill-opacity=".16"/><circle cx="130" cy="98" r="19" fill="#C8E6FF" fill-opacity=".16"/>
      <path d="M89 96Q100 90 111 96M51 93 33 87M149 93 167 87" stroke-linecap="round"/>
    </g>
    ${BAND('band-red', '#D7263D', '#7B1020', false)}
    ${BAND('band-blue', '#2E86C1', '#1B4F72', true)}
    <g data-gear="band-black">
      <path d="M33 72C60 57 140 57 167 72L165 84C140 70 60 70 35 84Z" fill="#24202B" stroke="#0C0A10" stroke-width="2" stroke-linejoin="round"/>
      <rect x="86" y="60" width="28" height="13" rx="3" fill="#C9D1D9" stroke="#3E4A56" stroke-width="1.6"/>
      <path d="M35 76C20 72 10 62 3 54M35 80C18 82 9 90 6 100" fill="none" stroke="#24202B" stroke-width="7" stroke-linecap="round"/>
    </g>
    <g data-gear="wrap">
      <path d="M23 124C17 72 46 18 100 16C154 18 183 72 177 124C169 98 161 82 151 75C130 64 70 64 49 75C39 82 31 98 23 124Z" fill="#EFE3C6" stroke="#9C8B66" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M44 56l12 7M64 44l11 8M88 37l9 9M112 37l9-1M134 44l11-3M154 56l10-5" stroke="#C0392B" stroke-width="2.4" stroke-linecap="round" opacity=".5"/>
      <path d="M35 71C70 55 130 55 165 71" fill="none" stroke="#1E1E24" stroke-width="7" stroke-linecap="round"/>
      <path d="M37 80C70 64 130 64 163 80" fill="none" stroke="#1E1E24" stroke-width="5" stroke-linecap="round"/>
    </g>
    ${TURBAN('turban-white', '#F7F3EA', '#D9D0BE', '#A89D88', '#D7263D')}
    ${TURBAN('turban-orange', '#F2A33A', '#D9822B', '#8A4B12', '#16A085')}
    <g data-gear="helmet">
      <path d="M97 16 100 -6 103 16Z" fill="#9AA5B1" stroke="#3E4A56" stroke-width="2" stroke-linejoin="round"/>
      <circle cx="100" cy="-8" r="4" fill="#C9D1D9" stroke="#3E4A56" stroke-width="2"/>
      <path d="M30 80C28 40 60 14 100 14C140 14 172 40 170 80Z" fill="url(#mk-steel-${n})" stroke="#3E4A56" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M100 18V74" stroke="#3E4A56" stroke-width="2" opacity=".45"/>
      <path d="M30 80C60 69 140 69 170 80L168 89C140 78 60 78 32 89Z" fill="#C9A227" stroke="#7A5C10" stroke-width="2" stroke-linejoin="round"/>
      <rect x="97" y="80" width="6" height="28" rx="3" fill="#9AA5B1" stroke="#3E4A56" stroke-width="1.6"/>
    </g>
    <g data-gear="crown">
      <path d="M64 44 60 13 80 30 100 5 120 30 140 13 136 44Z" fill="#FFD24D" stroke="#9A6B00" stroke-width="2.5" stroke-linejoin="round"/>
      <circle cx="100" cy="31" r="4.5" fill="#D7263D"/><circle cx="76" cy="37" r="3" fill="#2E86C1"/><circle cx="124" cy="37" r="3" fill="#2E86C1"/>
    </g>
    <g data-gear="flower" transform="translate(60 50)">${petals}<circle r="4" fill="#F7C948"/></g>
  </g>
  <g class="bang"><circle class="f-accent" cx="176" cy="30" r="13"/><rect fill="#fff" x="174.3" y="20.5" width="3.4" height="11" rx="1.7"/><circle fill="#fff" cx="176" cy="36.5" r="2"/></g>
  <g class="zz"><text class="f-zz z1" x="160" y="44" font-size="17">z</text><text class="f-zz z2" x="172" y="26" font-size="24">Z</text></g>
  <path class="spark f-gold s-line" stroke-width="1" d="M26 34l2.2 6.2 6.2 2.2-6.2 2.2L26 51l-2.2-6.2-6.2-2.2 6.2-2.2zM174 40l1.6 4.6 4.6 1.6-4.6 1.6L174 52.4l-1.6-4.6-4.6-1.6 4.6-1.6z"/>
</svg>`;
}

const MOODS = ['awake', 'happy', 'listen', 'nap'];
const VAR = { fur: 'fur', belly: 'belly', line: 'line', ear: 'ear', eye: 'eye', eyed: 'eyed', blush: 'blush', nose: 'nose', collar: 'collar', gold: 'gold',
  stripe: 'stripe', stripeOp: 'stripe-op', tip: 'tip', whisker: 'whisker', muzzle: 'muzzle', muzzleOp: 'muzzle-op', accent: 'accent', zz: 'zz' };

export class Mascot {
  constructor(host, { skin = 'mochi', size = 148, track = false, name = '', keyboard = false, look = null } = {}) {
    this.host = host;
    host.innerHTML = svg({ keyboard });
    this.svg = host.firstElementChild;
    this.svg.style.width = `${size}px`;
    this.svg.style.height = `${Math.round(size * 0.95)}px`;
    this.looks = [...this.svg.querySelectorAll('.look')];
    this.base = 'awake';
    this.name = name;
    this.setSkin(skin);
    this.setMood('awake');
    this.setLook(look || { gear: [] });
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
    this.svg.classList.add(`skin-${SKINS.some((s) => s.id === skin) ? skin : 'mochi'}`);
    for (const k of Object.values(VAR)) this.svg.style.removeProperty(`--mk-${k}`);
    this.skin = skin;
    this.label();
  }

  /**
   * Fur, ears and outfit at once: { skin } or { colors }, { ears: 'fold' }, { gear: [ids] }.
   * Colours come from CATS or a custom cat (already checked by the caller).
   */
  setLook(look = {}) {
    if (look.colors) {
      this.setSkin('custom');
      for (const c of [...this.svg.classList]) if (c.startsWith('skin-')) this.svg.classList.remove(c);
      this.svg.classList.add('skin-custom');
      for (const [k, v] of Object.entries(look.colors)) if (VAR[k] != null && v != null) this.svg.style.setProperty(`--mk-${VAR[k]}`, String(v));
    } else if (look.skin) this.setSkin(look.skin);
    const gear = new Set(look.gear || []);
    this.svg.classList.toggle('fold', look.ears === 'fold');
    this.svg.classList.toggle('hat', [...gear].some((g) => gearById(g)?.hat));
    const belt = [...gear].map(gearById).find((g) => g?.slot === 'belt');
    const cape = [...gear].map(gearById).find((g) => g?.slot === 'back');
    for (const el of this.svg.querySelectorAll('[data-gear]')) {
      const id = el.dataset.gear;
      const on = id === 'belt' ? !!belt : id === 'cape' ? !!cape : gear.has(id);
      el.style.display = on ? 'inline' : 'none';
    }
    if (belt) this.svg.style.setProperty('--belt', belt.color);
    if (cape) {
      this.svg.style.setProperty('--cape', cape.color);
      this.svg.style.setProperty('--cape-line', cape.line);
    }
    this.gear = [...gear];
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

  /** Making biscuits: the paws knead the keyboard, keys press, the cat is happy. */
  knead(ms = 1800) {
    if (this.base === 'nap') return;
    this.apply('happy');
    this.svg.classList.remove('knead');
    void this.svg.getBBox();
    this.svg.classList.add('knead');
    clearTimeout(this.timer);
    clearTimeout(this.kneadTimer);
    this.kneadTimer = setTimeout(() => this.svg.classList.remove('knead'), ms);
    this.timer = setTimeout(() => this.apply(this.base), ms + 200);
  }

  destroy() {
    if (this.onMove) window.removeEventListener('pointermove', this.onMove);
    clearTimeout(this.timer);
    clearTimeout(this.kneadTimer);
    this.host.innerHTML = '';
  }
}
