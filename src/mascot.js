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

function shade(hex, to, t) {
  const p = (x, i) => parseInt(x.slice(1 + i * 2, 3 + i * 2), 16);
  return `#${[0, 1, 2].map((i) => Math.round(p(hex, i) + (p(to, i) - p(hex, i)) * t).toString(16).padStart(2, '0')).join('')}`;
}

/** A whole set of fur colours from the few that make a cat itself. */
function kit(o) {
  const eye = o.eye || '#9B6BFF';
  const stripe = o.stripe || shade(o.fur, '#000000', 0.3);
  const c = C(o.fur, o.belly || '#FFFFFF', o.line || shade(o.fur, '#1a0f1f', 0.62), o.ear || '#FFB8D2', eye, o.eyed || shade(eye, '#000000', 0.72),
    o.blush || '#FF9EC4', o.nose || '#FF8FAF', o.collar || '#FF6FAE', o.gold || '#FFD24D', stripe, o.stripes ? 1 : 0,
    o.tip || (o.stripes ? stripe : shade(o.fur, '#000000', 0.08)), o.whisker || shade(o.fur, '#000000', 0.45), o.belly || '#FFFFFF', o.muzzle ? 1 : 0,
    o.accent || o.collar || '#FF6FAE', o.zz || '#B79CFF');
  if (o.patch) c.patch = o.patch;
  if (o.aura) c.aura = o.aura;
  return c;
}

/**
 * The meme cats: our own cartoon cats, each wearing the look or holding the prop that
 * made its meme famous, so you know it at a glance. No weapons; the photos they come
 * from stay where they are (they belong to the people who took them).
 * `pal` colours the whole app while the cat is yours: accent + the hue of the surfaces.
 */
const M = (id, name, tag, o, pal) => ({ id, name, full: name, tag, meme: true, group: 'meme', theme: o.theme || 'mochi', face: o.face, gear: o.gear || [],
  patches: o.patches || [], pose: o.pose, ears: o.ears, colors: kit(o), pal });
const MEME_CATS = [
  M('teary', 'Teary', 'Crying thumbs-up', { fur: '#F6F2EC', line: '#5C4F45', eye: '#C9A066', ear: '#F7B9C6', collar: '#8FD3FF', patch: '#9A8573', patches: ['top', 'ears'], face: 'cry', gear: ['thumbsup'], theme: 'sora' }, { ac: '#8FD3FF', h: 200, s: 24 }),
  M('salad', 'Salad', 'The dinner-table cat', { fur: '#FBFAF6', line: '#6A6258', eye: '#D9B34E', ear: '#F6B8C4', collar: '#A3E635', face: 'smudge', gear: ['plate'] }, { ac: '#A3E635', h: 95, s: 18 }),
  M('ace', 'Ace', 'We’re so back', { fur: '#F2A65E', stripes: true, stripe: '#C86C25', belly: '#FFE5C8', muzzle: true, eye: '#5FA86B', collar: '#FF8A3D', gear: ['pilot'], theme: 'tora' }, { ac: '#FF9A4D', h: 22, s: 26 }),
  M('boss', 'Boss', 'The cat in a suit', { fur: '#F3C690', belly: '#FFF3E3', muzzle: true, eye: '#79A86B', collar: '#E3B341', gear: ['suit'], theme: 'tora' }, { ac: '#E3B341', h: 40, s: 10 }),
  M('cashmere', 'Cashmere', 'The money cat', { fur: '#B49A80', stripes: true, stripe: '#7A5E45', belly: '#EFE3D6', muzzle: true, eye: '#7BC47F', collar: '#2FBF71', gear: ['shades', 'cash'] }, { ac: '#3DD68C', h: 150, s: 20 }),
  M('nerd', 'Nerd', 'The glasses kitten', { fur: '#A39C94', stripes: true, stripe: '#5F5850', belly: '#EDE8E2', muzzle: true, eye: '#5C8FD6', collar: '#8C9EFF', gear: ['nerd-glasses'], theme: 'kuro' }, { ac: '#8C9EFF', h: 230, s: 24 }),
  M('tutu', 'Tutu', 'The ballerina cat', { fur: '#F4CDA2', belly: '#FFF5EA', muzzle: true, eye: '#69AEDB', collar: '#1F1C22', gear: ['tutu'] }, { ac: '#FF7EB6', h: 330, s: 26 }),
  M('frost', 'Frost', 'The fur-hat cat', { fur: '#A98666', stripes: true, stripe: '#6B4C33', belly: '#EBDCCB', muzzle: true, eye: '#E3B341', collar: '#7CC8F0', gear: ['ushanka'], theme: 'sora' }, { ac: '#7CC8F0', h: 200, s: 26 }),
  M('duke', 'Duke', 'The top-hat cat', { fur: '#EBA463', stripes: true, stripe: '#B8662A', belly: '#FFE8CF', muzzle: true, eye: '#7FB36A', collar: '#8E2C48', gear: ['tophat', 'monocle'], theme: 'tora' }, { ac: '#E0567A', h: 345, s: 22 }),
  M('kufi', 'Kufi', 'Kufi and shades', { fur: '#F7F4EF', eye: '#8BB5DC', ear: '#F5BDC8', collar: '#D9B27C', gear: ['kufi', 'round-shades'] }, { ac: '#D9B27C', h: 36, s: 20 }),
  M('paladin', 'Sir Paws', 'The knight cat', { fur: '#A6A6AD', stripes: true, stripe: '#6E6E76', belly: '#E9E9EE', muzzle: true, eye: '#79B8E8', collar: '#9DB4D6', gear: ['knight', 'armor', 'cape-red'], theme: 'sora' }, { ac: '#9DB4D6', h: 215, s: 16 }),
  M('wrench', 'Wrench', 'The mechanic cat', { fur: '#8F939A', stripes: true, stripe: '#5E6168', belly: '#E3E5E8', muzzle: true, eye: '#E0B040', collar: '#FFCB47', gear: ['cap-mech', 'wrench'], theme: 'tora' }, { ac: '#FFCB47', h: 210, s: 14 }),
  M('yell', 'Yell', 'The screaming cat', { fur: '#F7F5F2', eye: '#BFA36A', ear: '#F5B5C4', collar: '#FF6B6B', patch: '#8F8983', patches: ['top', 'ears'], face: 'scream' }, { ac: '#FF6B6B', h: 0, s: 22 }),
  M('math', 'Math', 'The confused math cat', { fur: '#F1EFEA', eye: '#9BC4B4', patch: '#C9C4BD', patches: ['eye'], collar: '#4FD1C5', face: 'confused', gear: ['mathfx'], theme: 'sora' }, { ac: '#4FD1C5', h: 172, s: 22 }),
  M('getin', 'Get In', 'No time to explain', { fur: '#A9A398', stripes: true, stripe: '#6A6459', belly: '#ECE8E1', muzzle: true, eye: '#D9B44A', collar: '#FFD23F', gear: ['wheel'], theme: 'tora' }, { ac: '#FFD23F', h: 48, s: 18 }),
  M('typer', 'Typer', 'The cat at the computer', { fur: '#9B9792', stripes: true, stripe: '#5F5B56', belly: '#E8E5E1', muzzle: true, eye: '#69C47A', collar: '#4ADE80', gear: ['laptop'] }, { ac: '#4ADE80', h: 140, s: 16 }),
  M('hood', 'Consequences', 'Pay in treats', { fur: '#A48C6F', stripes: true, stripe: '#6C553C', belly: '#EADFD1', muzzle: true, eye: '#E8C24A', collar: '#B48CFF', face: 'judge', gear: ['hood'], theme: 'kuro' }, { ac: '#B48CFF', h: 268, s: 24 }),
  M('burrito', 'Burrito', 'The blanket kitten', { fur: '#2B2A30', line: '#0E0D12', belly: '#F6F6F6', muzzle: true, eye: '#E9C46A', ear: '#F0A3B8', collar: '#F2A65A', gear: ['blanket'], theme: 'kuro' }, { ac: '#F2A65A', h: 28, s: 24 }),
  M('bricky', 'Bricky', 'The toy-brick cat', { fur: '#A0A5AB', belly: '#EEF0F2', eye: '#7FA6D6', collar: '#FF5A4F', face: 'shock', gear: ['brick'] }, { ac: '#FF5A4F', h: 4, s: 20 }),
  M('patch', 'Patch', 'The smug one', { fur: '#FBFBFA', line: '#2A2A30', eye: '#8F9AA6', patch: '#1D1D22', patches: ['top', 'eye', 'ears'], collar: '#D4D4D8', face: 'smug', theme: 'kuro' }, { ac: '#D4D4D8', h: 240, s: 5 }),
  M('polite', 'Polite', 'The polite cat', { fur: '#27232D', belly: '#38323F', line: '#0B090E', eye: '#F2C94C', eyed: '#3A2A00', ear: '#E99AB8', collar: '#A78BFA', face: 'joy', pose: 'polite', theme: 'kuro' }, { ac: '#A78BFA', h: 258, s: 22 }),
  M('gasp', 'Gasp', 'The shocked cat', { fur: '#BBB6AC', stripes: true, stripe: '#7C766B', belly: '#F0EEE9', muzzle: true, eye: '#9CC16B', collar: '#FFE45C', face: 'shock' }, { ac: '#FFE45C', h: 52, s: 16 }),
  M('judge', 'Judge', 'He judges you', { fur: '#918B85', stripes: true, stripe: '#5C5650', belly: '#E2DFDB', muzzle: true, eye: '#D5B04C', collar: '#7AA2F7', face: 'judge', theme: 'sora' }, { ac: '#7AA2F7', h: 225, s: 20 }),
  M('pout', 'Pout', 'The sad kitten', { fur: '#9C7D62', stripes: true, stripe: '#5F4733', belly: '#EADCCD', muzzle: true, eye: '#6E94BC', collar: '#93C5FD', face: 'sad', theme: 'sora' }, { ac: '#93C5FD', h: 212, s: 22 }),
  M('grin', 'Grin', 'The smiling cat', { fur: '#FCFBF9', eye: '#A7C7E7', ear: '#F7B2C4', collar: '#FBBF24', face: 'grin', theme: 'tora' }, { ac: '#FBBF24', h: 42, s: 22 }),
  M('neon', 'Neon', 'The glowing cat', { fur: '#D63A8E', belly: '#F48BC1', line: '#4A0B2C', eye: '#FFE8F4', eyed: '#5A0A35', ear: '#FF9FD0', blush: '#FF6FB5', nose: '#FFC2E1', collar: '#FF4FD8', aura: '#FF4FD8', gear: ['aura'] }, { ac: '#FF4FD8', h: 312, s: 34 }),
  M('stare', 'Stare', 'The stare', { fur: '#AC9172', stripes: true, stripe: '#6E5336', belly: '#EEE2D2', muzzle: true, eye: '#C8C24A', collar: '#F59E0B', face: 'stare', theme: 'tora' }, { ac: '#F59E0B', h: 32, s: 20 }),
  M('happy', 'Happy', 'Happy happy happy', { fur: '#F5E8D8', eye: '#7CC4A4', collar: '#34D399', face: 'joy', pose: 'bounce' }, { ac: '#34D399', h: 158, s: 22 }),
  M('beanie', 'Beanie', 'The beanie cat', { fur: '#E9A35F', stripes: true, stripe: '#B8682A', belly: '#FFE6CB', muzzle: true, eye: '#5FA86B', collar: '#A3B18A', gear: ['beanie'], theme: 'tora' }, { ac: '#A3B18A', h: 80, s: 14 }),
];

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
  ...MEME_CATS,
];

/** Which shelf a cat sits on in the pickers. */
export const catGroup = (c) => (c.custom ? 'yours' : c.group === 'meme' ? 'meme' : c.meme ? 'warrior' : 'house');
export const CAT_GROUPS = [['house', 'House cats'], ['meme', 'Meme cats'], ['warrior', 'The warriors'], ['yours', 'Your cats']];

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
  // The meme cats' signature pieces: theirs from the start, everyone else's by rank.
  { id: 'tutu', name: 'Tutu', slot: 'body', rank: 1 },
  { id: 'blanket', name: 'Blanket burrito', slot: 'body', rank: 1, paws: false, tail: false },
  { id: 'thumbsup', name: 'Thumbs up', slot: 'hand', rank: 1 },
  { id: 'kufi', name: 'Knitted cap', slot: 'head', rank: 2, hat: true },
  { id: 'beanie', name: 'Beanie', slot: 'head', rank: 2, hat: true },
  { id: 'nerd-glasses', name: 'Big black glasses', slot: 'face', rank: 2 },
  { id: 'laptop', name: 'Laptop', slot: 'front', rank: 2, paws: false },
  { id: 'plate', name: 'Salad plate', slot: 'front', rank: 2, paws: false },
  { id: 'ushanka', name: 'Fur hat', slot: 'head', rank: 3, hat: true },
  { id: 'round-shades', name: 'Round shades', slot: 'face', rank: 3 },
  { id: 'cap-mech', name: 'Work cap', slot: 'head', rank: 3, hat: true },
  { id: 'wrench', name: 'Wrench', slot: 'hand', rank: 3 },
  { id: 'shades', name: 'Dark shades', slot: 'face', rank: 4 },
  { id: 'wheel', name: 'Steering wheel', slot: 'front', rank: 4, paws: false },
  { id: 'mathfx', name: 'Maths in the air', slot: 'fx', rank: 4 },
  { id: 'hood', name: 'The hood', slot: 'head', rank: 5, hat: true },
  { id: 'brick', name: 'Toy brick', slot: 'body', rank: 5, paws: false, tail: false },
  { id: 'pilot', name: 'Pilot helmet', slot: 'head', rank: 6, hat: true },
  { id: 'monocle', name: 'Monocle', slot: 'face', rank: 6 },
  { id: 'armor', name: 'Armour', slot: 'body', rank: 6 },
  { id: 'tophat', name: 'Top hat', slot: 'head', rank: 7, hat: true },
  { id: 'knight', name: 'Knight helm', slot: 'head', rank: 7, hat: true },
  { id: 'suit', name: 'Suit and tie', slot: 'body', rank: 7, collar: false },
  { id: 'cash', name: 'A fan of cash', slot: 'hand', rank: 8 },
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
      <ellipse class="f-eyed pupil" cx="${cx}" cy="100.5" rx="6.5" ry="9.5"/>
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
  const dots = [[30, [64, 76, 88, 100, 112, 124, 136]], [41, [54, 66, 78, 90, 102, 114, 126, 138, 150]], [52, [48, 60, 72, 84, 96, 108, 120, 132, 144, 156]]]
    .map(([y, xs]) => xs.map((x) => `<circle cx="${x}" cy="${y}" r="2.1"/>`).join('')).join('');
  const bills = [-36, -22, -8, 6].map((a) => `<g transform="rotate(${a} 44 168)"><rect x="30" y="116" width="28" height="52" rx="3" fill="#9BDCA9" stroke="#2E7D4F" stroke-width="2"/><circle cx="44" cy="138" r="7" fill="none" stroke="#2E7D4F" stroke-width="1.6"/><text x="44" y="142.5" font-size="11" text-anchor="middle" fill="#2E7D4F" font-weight="800" font-family="Nunito, sans-serif">$</text></g>`).join('');
  const petals = [0, 72, 144, 216, 288].map((a) => `<ellipse rx="5.2" ry="8.5" fill="#fff" stroke="#D8C7A8" stroke-width="1" transform="rotate(${a}) translate(0 -7)"/>`).join('');
  return `<svg class="mk" viewBox="0 0 200 190" role="img" aria-label="">
  <defs>
    <linearGradient id="mk-steel-${n}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#EEF2F5"/><stop offset="1" stop-color="#7F8B97"/></linearGradient>
    <radialGradient id="mk-aura-${n}"><stop offset=".35" style="stop-color:var(--mk-aura,#FFD24D)" stop-opacity=".55"/><stop offset="1" style="stop-color:var(--mk-aura,#FFD24D)" stop-opacity="0"/></radialGradient>
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
  <g data-gear="armor">
    <path d="M56 138C62 126 80 122 100 122C120 122 138 126 144 138C148 152 146 168 138 180L62 180C54 168 52 152 56 138Z" fill="url(#mk-steel-${n})" stroke="#3E4A56" stroke-width="2.5" stroke-linejoin="round"/>
    <path d="M100 148V178M66 162Q100 172 134 162" fill="none" stroke="#3E4A56" stroke-width="2" opacity=".5"/>
  </g>
  <g data-gear="suit">
    <path d="M52 140C56 128 76 120 100 120C124 120 144 128 148 140C152 154 150 172 140 184L60 184C50 172 48 154 52 140Z" fill="#26242B" stroke="#0E0D10" stroke-width="2.5" stroke-linejoin="round"/>
    <path d="M84 124L100 172L116 124C110 121 90 121 84 124Z" fill="#FAFAFA" stroke="#C9C9CF" stroke-width="1.5"/>
    <path d="M95 134H105L102.5 141L106.5 160L100 168L93.5 160L97.5 141Z" fill="#B3263A" stroke="#6E1020" stroke-width="1.5" stroke-linejoin="round"/>
    <path d="M84 124L95 152L79 144ZM116 124L105 152L121 144Z" fill="#3A3740" stroke="#0E0D10" stroke-width="1.5" stroke-linejoin="round"/>
    <circle cx="100" cy="176" r="2.4" fill="#0E0D10"/>
  </g>
  <g data-gear="tutu">
    <path d="M50 160Q56 150 66 156Q72 146 82 154Q90 144 100 152Q110 144 118 154Q128 146 134 156Q144 150 150 160Q158 170 146 176Q138 186 124 180Q114 190 100 182Q86 190 76 180Q62 186 54 176Q42 170 50 160Z" fill="#FF9CC8" stroke="#C2457F" stroke-width="2" stroke-linejoin="round"/>
    <path d="M58 166Q70 160 82 166Q94 158 100 166Q108 158 118 166Q130 160 142 166" fill="none" stroke="#FFD1E6" stroke-width="3" stroke-linecap="round"/>
  </g>
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
  <g data-gear="blanket">
    <path d="M44 150C40 128 66 118 100 118C134 118 160 128 156 150C160 170 150 188 100 188C50 188 40 170 44 150Z" fill="#D9B48A" stroke="#7A5A3A" stroke-width="2.5" stroke-linejoin="round"/>
    <path d="M56 138C80 154 120 154 144 138M50 160C76 176 124 176 150 160M70 130C84 142 92 152 96 174" fill="none" stroke="#B48C62" stroke-width="3" stroke-linecap="round"/>
    <path d="M60 124C76 118 124 118 140 124" fill="none" stroke="#F2D9BC" stroke-width="3" stroke-linecap="round"/>
  </g>
  <g data-gear="brick">
    <ellipse cx="54" cy="146" rx="9" ry="4.5" fill="#F7C531" stroke="#9A7400" stroke-width="2"/><ellipse cx="146" cy="146" rx="9" ry="4.5" fill="#F7C531" stroke="#9A7400" stroke-width="2"/>
    <path d="M45 146v-5h18v5M137 146v-5h18v5" fill="#F7C531" stroke="#9A7400" stroke-width="2"/>
    <rect x="40" y="146" width="120" height="44" rx="6" fill="#F7C531" stroke="#9A7400" stroke-width="2.5"/>
    <path d="M47 153H153" stroke="#FFE07A" stroke-width="3" stroke-linecap="round" opacity=".8"/>
    <path d="M40 182H160" stroke="#D9A60F" stroke-width="2" opacity=".7"/>
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
    <g class="patches">
      <path class="pt pt-top f-patch" d="M31.5 84C35 54 64 36.5 100 36.5C136 36.5 165 54 168.5 84C154 74 134 70 116 80C104 66 88 64 76 74C62 68 44 72 31.5 84Z"/>
      <path class="pt pt-eye f-patch" d="M110 80C124 66 156 68 166 92C168.5 112 152 124 132 121C116 119 106 102 110 80Z"/>
    </g>
    <g class="ears-fold">
      <path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M42 60C44 40 60 30 76 35C71 44 62 53 49 63Z"/>
      <path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M158 60C156 40 140 30 124 35C129 44 138 53 151 63Z"/>
    </g>
    <ellipse class="f-muzzle muzzle" cx="100" cy="122" rx="30" ry="19"/>
    <path class="s-stripe stripes" fill="none" stroke-width="3.6" stroke-linecap="round" d="M91 43l2 12M100 40v14M109 43l-2 12M31 92l11 3M30 100l11 1M169 92l-11 3M170 100l-11 1"/>
    <g class="eyes-open">${EYE(70)}${EYE(130)}</g>
    <path class="eyes-happy s-eyed" fill="none" stroke-width="4.4" stroke-linecap="round" d="M57 101Q70 85 83 101M117 101Q130 85 143 101"/>
    <path class="eyes-nap s-eyed" fill="none" stroke-width="4" stroke-linecap="round" d="M57 97Q70 107 83 97M117 97Q130 107 143 97"/>
    <path class="brows s-line" fill="none" stroke-width="3.6" stroke-linecap="round" d="M55 78Q69 79 84 86M145 78Q131 79 116 86"/>
    <g class="faces">
      <g class="fx fx-lids"><path class="f-fur s-line" stroke-width="2.2" d="M53 98C55 78 85 78 87 98C79 93 61 93 53 98Z"/><path class="f-fur s-line" stroke-width="2.2" d="M113 98C115 78 145 78 147 98C139 93 121 93 113 98Z"/></g>
      <path class="fx fx-lid-l f-fur s-line" stroke-width="2.2" d="M53 99C55 78 85 78 87 99C79 94 61 94 53 99Z"/>
      <path class="fx fx-squeeze s-eyed" fill="none" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" d="M58 87 81 98 58 109M142 87 119 98 142 109"/>
      <g class="fx fx-tears"><ellipse cx="70" cy="112" rx="11" ry="3.6" fill="#8FD3FF" opacity=".95"/><ellipse cx="130" cy="112" rx="11" ry="3.6" fill="#8FD3FF" opacity=".95"/>
        <path d="M61 113C58 123 57 133 59 145M139 113C142 123 143 133 141 145" fill="none" stroke="#8FD3FF" stroke-width="6.5" stroke-linecap="round"/>
        <path d="M62 116C60 123 59.5 130 60.5 138M138 116C140 123 140.5 130 139.5 138" fill="none" stroke="#fff" stroke-width="1.8" stroke-linecap="round" opacity=".85"/></g>
      <path class="fx fx-tearline" fill="#8FD3FF" opacity=".9" d="M59 111Q70 116 81 111Q70 113.5 59 111ZM119 111Q130 116 141 111Q130 113.5 119 111Z"/>
      <path class="fx fx-brows-sad s-line" fill="none" stroke-width="3.4" stroke-linecap="round" d="M57 82Q69 74 83 74M143 82Q131 74 117 74"/>
      <path class="fx fx-brows-up s-line" fill="none" stroke-width="3.4" stroke-linecap="round" d="M56 72Q69 63 83 70M144 72Q131 63 117 70"/>
      <path class="fx fx-brows-angry s-line" fill="none" stroke-width="3.6" stroke-linecap="round" d="M56 78 84 88M118 79Q131 70 144 75"/>
      <path class="fx fx-brows-confused s-line" fill="none" stroke-width="3.4" stroke-linecap="round" d="M56 75Q69 64 83 72M118 82Q131 84 144 79"/>
      <path class="fx fx-mouth-sad s-line" fill="none" stroke-width="2.6" stroke-linecap="round" d="M90 128Q100 119 110 128"/>
      <path class="fx fx-mouth-flat s-line" fill="none" stroke-width="2.6" stroke-linecap="round" d="M91 125H109"/>
      <path class="fx fx-mouth-smirk s-line" fill="none" stroke-width="2.4" stroke-linecap="round" d="M89 123Q101 130 112 120"/>
      <ellipse class="fx fx-mouth-o s-line" fill="#5A1A2A" stroke-width="1.8" cx="100" cy="127" rx="5" ry="6"/>
      <g class="fx fx-mouth-yell"><path class="s-line" fill="#5A1A2A" stroke-width="2.2" stroke-linejoin="round" d="M87 121Q100 117 113 121Q113 139 100 140Q87 139 87 121Z"/><path fill="#F07A9A" d="M92 134Q100 128 108 134Q104 138 100 138Q96 138 92 134Z"/><path fill="#fff" d="M89.5 122 92.5 127 95 121ZM110.5 122 107.5 127 105 121Z"/></g>
      <g class="fx fx-mouth-scream"><path class="s-line" fill="#5A1A2A" stroke-width="2.4" stroke-linejoin="round" d="M80 118Q100 112 120 118Q124 147 100 151Q76 147 80 118Z"/><path fill="#F07A9A" d="M87 141Q100 130 113 141Q107 149 100 149Q93 149 87 141Z"/><path fill="#fff" d="M84 118.5 88 128 92 117ZM116 118.5 112 128 108 117Z"/></g>
      <g class="fx fx-mouth-grin"><path class="s-line" fill="#5A1A2A" stroke-width="2.2" stroke-linejoin="round" d="M77 118Q100 147 123 118Q100 126 77 118Z"/><path fill="#fff" d="M81 119.5Q100 127 119 119.5L117.5 123.5Q100 131 82.5 123.5Z"/></g>
    </g>
    <path class="sweat" fill="#9ED8FF" stroke="#4A9BD6" stroke-width="1.6" stroke-linejoin="round" d="M163 60Q170 72 163 79Q156 72 163 60Z"/>
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
    <circle class="f-fur charm-hole" cx="100" cy="153.5" r="2"/>
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
    <g data-gear="kufi">
      <path d="M38 70C38 34 66 20 100 20C134 20 162 34 162 70C144 62 124 58 100 58C76 58 56 62 38 70Z" fill="#F4F1EA" stroke="#B5AE9F" stroke-width="2.5" stroke-linejoin="round"/>
      <g fill="#D6CFBF">${dots}</g>
    </g>
    <g data-gear="beanie">
      <path d="M34 68C30 32 62 14 100 14C138 14 170 32 166 68Z" fill="#232127" stroke="#0C0B0E" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M31 66C31 54 60 48 100 48C140 48 169 54 169 66C169 74 140 77 100 77C60 77 31 74 31 66Z" fill="#2D2A33" stroke="#0C0B0E" stroke-width="2.5"/>
      <path d="M44 54v20M58 51v24M72 49v27M86 48v28M100 48v29M114 48v28M128 49v27M142 51v24M156 54v20" stroke="#403B48" stroke-width="3"/>
    </g>
    <g data-gear="ushanka">
      <path d="M30 76C28 36 62 14 100 14C138 14 172 36 170 76Z" fill="#6B4A32" stroke="#3B2718" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M26 74C24 56 46 46 100 46C154 46 176 56 174 74C172 80 160 82 100 80C40 82 28 80 26 74Z" fill="#B38B65" stroke="#3B2718" stroke-width="2.5"/>
      <path d="M22 78C13 94 15 122 29 134C38 139 45 131 43 119L41 80Z" fill="#B38B65" stroke="#3B2718" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M178 78C187 94 185 122 171 134C162 139 155 131 157 119L159 80Z" fill="#B38B65" stroke="#3B2718" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M44 62l6 6M64 56l5 7M86 52l3 8M114 52l-3 8M136 56l-5 7M156 62l-6 6M24 100l7 3M176 100l-7 3M26 116l7 1M174 116l-7 1" stroke="#8A6646" stroke-width="2" stroke-linecap="round"/>
    </g>
    <g data-gear="cap-mech">
      <path d="M38 72C36 36 64 20 100 20C136 20 164 36 162 72C140 64 120 62 100 62C80 62 60 64 38 72Z" fill="#2F6FB5" stroke="#173B63" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M41 70C26 69 9 73 2 82C19 85 36 81 47 76Z" fill="#245A94" stroke="#173B63" stroke-width="2.5" stroke-linejoin="round"/>
      <circle cx="100" cy="22" r="4" fill="#245A94" stroke="#173B63" stroke-width="2"/>
      <path d="M86 42h28" stroke="#fff" stroke-width="5" stroke-linecap="round" opacity=".85"/>
    </g>
    <g data-gear="pilot">
      <path d="M26 92C22 46 58 12 100 12C142 12 178 46 174 92C160 80 136 74 100 74C64 74 40 80 26 92Z" fill="#7C8A6E" stroke="#3E4636" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M40 58C58 42 142 42 160 58L158 69C140 55 60 55 42 69Z" fill="#2F3A46" stroke="#1C232B" stroke-width="2" stroke-linejoin="round"/>
      <path d="M50 55C70 46 130 46 150 55" fill="none" stroke="#8FB4D9" stroke-width="2" opacity=".7"/>
      <path d="M100 14V42" stroke="#3E4636" stroke-width="2" opacity=".4"/>
      <rect x="16" y="78" width="22" height="36" rx="10" fill="#5F6B55" stroke="#3E4636" stroke-width="2"/>
      <rect x="162" y="78" width="22" height="36" rx="10" fill="#5F6B55" stroke="#3E4636" stroke-width="2"/>
      <path d="M29 112C25 126 32 138 44 140" fill="none" stroke="#3E4636" stroke-width="4" stroke-linecap="round"/>
      <path d="M38 128C38 119 55 117 59 125C61 136 55 144 47 144C41 144 38 136 38 128Z" fill="#5F6B55" stroke="#3E4636" stroke-width="2"/>
      <path d="M49 145C51 156 60 162 71 164" fill="none" stroke="#3E4636" stroke-width="5.5" stroke-linecap="round" stroke-dasharray="1.5 4.5"/>
    </g>
    <g data-gear="knight">
      <path class="plume" d="M101 18C97 0 110 -14 130 -18C121 -6 119 6 109 20Z" fill="#D7263D" stroke="#7B1020" stroke-width="2" stroke-linejoin="round"/>
      <path d="M32 80C28 40 60 14 100 14C140 14 172 40 168 80C150 73 128 70 100 70C72 70 50 73 32 80Z" fill="url(#mk-steel-${n})" stroke="#3E4A56" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M100 16V68M44 58H156" stroke="#3E4A56" stroke-width="2" opacity=".45"/>
      <path d="M58 46h10M76 42h10M114 42h10M132 46h10" stroke="#3E4A56" stroke-width="3" stroke-linecap="round" opacity=".6"/>
    </g>
    <g data-gear="tophat">
      <ellipse cx="100" cy="48" rx="56" ry="10" fill="#1E1B22" stroke="#000" stroke-width="2"/>
      <path d="M66 48V-6C66 -12 134 -12 134 -6V48Z" fill="#26222B" stroke="#000" stroke-width="2.5" stroke-linejoin="round"/>
      <rect x="66" y="28" width="68" height="12" fill="#8E2C48"/>
      <path d="M76 -4V26" stroke="#fff" stroke-opacity=".14" stroke-width="7" stroke-linecap="round"/>
    </g>
    <g data-gear="hood">
      <path d="M100 6C46 6 14 50 18 104C20 132 34 150 52 156L60 150C46 134 40 116 42 96C44 64 68 44 100 44C132 44 156 64 158 96C160 116 154 134 140 150L148 156C166 150 180 132 182 104C186 50 154 6 100 6Z" fill="#17151B" stroke="#000" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M50 32C68 17 98 12 120 16" fill="none" stroke="#fff" stroke-opacity=".22" stroke-width="5" stroke-linecap="round"/>
      <path d="M100 7C104 -5 116 -9 122 -3" fill="none" stroke="#17151B" stroke-width="6" stroke-linecap="round"/>
    </g>
    <g data-gear="shades">
      <path d="M47 86H92Q95 86 94 90L91 106Q89 115 78 115H60Q49 115 47 104Z" fill="#101014" stroke="#000" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M153 86H108Q105 86 106 90L109 106Q111 115 122 115H140Q151 115 153 104Z" fill="#101014" stroke="#000" stroke-width="2.5" stroke-linejoin="round"/>
      <path d="M92 90Q100 86 108 90" fill="none" stroke="#000" stroke-width="3"/>
      <path d="M47 88 29 84M153 88 171 84" stroke="#000" stroke-width="3" stroke-linecap="round"/>
      <path d="M55 92h11M113 92h11" stroke="#fff" stroke-width="2.4" stroke-linecap="round" opacity=".55"/>
    </g>
    <g data-gear="round-shades">
      <circle cx="70" cy="99" r="16" fill="#121214" stroke="#000" stroke-width="2.5"/><circle cx="130" cy="99" r="16" fill="#121214" stroke="#000" stroke-width="2.5"/>
      <path d="M86 97Q100 91 114 97" fill="none" stroke="#000" stroke-width="3"/>
      <path d="M54 95 36 89M146 95 164 89" stroke="#000" stroke-width="2.5" stroke-linecap="round"/>
      <path d="M62 92q5-4 10-2M122 92q5-4 10-2" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" opacity=".6"/>
    </g>
    <g data-gear="nerd-glasses" fill="none" stroke="#141216" stroke-width="5.5">
      <circle cx="70" cy="99" r="20" fill="#E8F4FF" fill-opacity=".14"/><circle cx="130" cy="99" r="20" fill="#E8F4FF" fill-opacity=".14"/>
      <path d="M90 97Q100 91 110 97M50 94 32 88M150 94 168 88" stroke-linecap="round"/>
    </g>
    <g data-gear="monocle">
      <circle cx="130" cy="98" r="18" fill="#C8E6FF" fill-opacity=".18" stroke="#C9A227" stroke-width="3"/>
      <path d="M147 106C157 122 158 140 150 158" fill="none" stroke="#C9A227" stroke-width="1.6" stroke-dasharray="3 2"/>
    </g>
    <g data-gear="crown">
      <path d="M64 44 60 13 80 30 100 5 120 30 140 13 136 44Z" fill="#FFD24D" stroke="#9A6B00" stroke-width="2.5" stroke-linejoin="round"/>
      <circle cx="100" cy="31" r="4.5" fill="#D7263D"/><circle cx="76" cy="37" r="3" fill="#2E86C1"/><circle cx="124" cy="37" r="3" fill="#2E86C1"/>
    </g>
    <g data-gear="flower" transform="translate(60 50)">${petals}<circle r="4" fill="#F7C948"/></g>
  </g>
  <g data-gear="plate">
    <ellipse cx="100" cy="182" rx="66" ry="12" fill="#FFFFFF" stroke="#B8BCC4" stroke-width="2.5"/>
    <ellipse cx="100" cy="180" rx="46" ry="7" fill="#EEF1F4"/>
    <path d="M66 178C70 166 84 164 90 172C86 180 74 182 66 178Z" fill="#6BBF59" stroke="#3E7B33" stroke-width="1.5"/>
    <path d="M90 177C96 162 116 162 121 172C114 182 98 182 90 177Z" fill="#86D16E" stroke="#3E7B33" stroke-width="1.5"/>
    <path d="M118 178C124 168 136 168 140 176C134 182 124 182 118 178Z" fill="#5DAE4B" stroke="#3E7B33" stroke-width="1.5"/>
    <circle cx="108" cy="169" r="5" fill="#E5484D" stroke="#9B1C22" stroke-width="1.2"/><circle cx="82" cy="173" r="4.5" fill="#CFE8A9" stroke="#4E8B3A" stroke-width="1.4"/>
  </g>
  <g data-gear="wheel">
    <circle cx="100" cy="168" r="34" fill="none" stroke="#2A2730" stroke-width="10"/>
    <circle cx="100" cy="168" r="34" fill="none" stroke="#F4C430" stroke-width="6"/>
    <path d="M68 168H132M100 168V200" stroke="#2A2730" stroke-width="8" stroke-linecap="round"/>
    <circle cx="100" cy="168" r="9" fill="#3A3640" stroke="#1C1A20" stroke-width="2"/>
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="70" cy="150" rx="12" ry="9"/><ellipse class="f-fur s-line" stroke-width="2.5" cx="130" cy="150" rx="12" ry="9"/>
  </g>
  <g data-gear="laptop">
    <path d="M46 140H154L146 186H54Z" fill="#C9CED6" stroke="#4B5563" stroke-width="2.5" stroke-linejoin="round"/>
    <path d="M40 187H160" stroke="#4B5563" stroke-width="6" stroke-linecap="round"/>
    <g fill="#9AA3AE" transform="translate(100 164)"><ellipse cx="0" cy="4" rx="7" ry="5.5"/><circle cx="-7" cy="-4" r="2.6"/><circle cx="-2.5" cy="-8" r="2.6"/><circle cx="2.5" cy="-8" r="2.6"/><circle cx="7" cy="-4" r="2.6"/></g>
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="64" cy="139" rx="12" ry="8"/><ellipse class="f-fur s-line" stroke-width="2.5" cx="136" cy="139" rx="12" ry="8"/>
  </g>
  <g data-gear="thumbsup">
    <path class="f-fur s-line" stroke-width="2.5" stroke-linejoin="round" d="M154 136C152 124 154 112 162 108C168 106 171 112 169 118L167 134"/>
    <path class="f-fur s-line" stroke-width="2.5" d="M146 150C146 140 152 134 160 134L168 134C176 134 180 140 178 148L176 160C174 168 168 170 160 170L150 170C146 170 144 166 146 160Z"/>
    <path class="s-line" stroke-width="1.6" opacity=".45" d="M150 146h22M150 154h22M150 162h20"/>
  </g>
  <g data-gear="cash">${bills}<ellipse class="f-fur s-line" stroke-width="2.5" cx="46" cy="166" rx="12" ry="9"/></g>
  <g data-gear="wrench" transform="rotate(-28 158 150)">
    <rect x="152" y="112" width="12" height="62" rx="5" fill="#B9C1CA" stroke="#4B5563" stroke-width="2"/>
    <path d="M145 113C145 97 171 97 171 113H164.5V104.5H151.5V113Z" fill="#B9C1CA" stroke="#4B5563" stroke-width="2" stroke-linejoin="round"/>
    <ellipse class="f-fur s-line" stroke-width="2.5" cx="158" cy="160" rx="12" ry="9"/>
  </g>
  <g data-gear="mathfx" class="mathfx" font-family="Fredoka, Nunito, sans-serif" font-weight="700">
    <text class="f-accent m1" x="2" y="42" font-size="21">π</text><text class="f-accent m2" x="166" y="36" font-size="18">√x</text>
    <text class="f-accent m3" x="0" y="134" font-size="16">x²</text><text class="f-accent m4" x="176" y="130" font-size="19">∑</text>
    <text class="f-accent m5" x="150" y="8" font-size="17">?</text>
  </g>
  <g class="bang"><circle class="f-accent" cx="176" cy="30" r="13"/><rect fill="#fff" x="174.3" y="20.5" width="3.4" height="11" rx="1.7"/><circle fill="#fff" cx="176" cy="36.5" r="2"/></g>
  <g class="zz"><text class="f-zz z1" x="160" y="44" font-size="17">z</text><text class="f-zz z2" x="172" y="26" font-size="24">Z</text></g>
  <path class="spark f-gold s-line" stroke-width="1" d="M26 34l2.2 6.2 6.2 2.2-6.2 2.2L26 51l-2.2-6.2-6.2-2.2 6.2-2.2zM174 40l1.6 4.6 4.6 1.6-4.6 1.6L174 52.4l-1.6-4.6-4.6-1.6 4.6-1.6z"/>
</svg>`;
}

const MOODS = ['awake', 'happy', 'listen', 'nap', 'serious'];
const VAR = { fur: 'fur', belly: 'belly', line: 'line', ear: 'ear', eye: 'eye', eyed: 'eyed', blush: 'blush', nose: 'nose', collar: 'collar', gold: 'gold',
  stripe: 'stripe', stripeOp: 'stripe-op', tip: 'tip', whisker: 'whisker', muzzle: 'muzzle', muzzleOp: 'muzzle-op', accent: 'accent', zz: 'zz', patch: 'patch', aura: 'aura' };
const FACES = ['cry', 'sad', 'scream', 'smudge', 'judge', 'smug', 'shock', 'stare', 'confused', 'grin', 'joy'];

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
    const pieces = [...gear].map(gearById).filter(Boolean);
    this.svg.classList.toggle('fold', look.ears === 'fold');
    this.svg.classList.toggle('hat', pieces.some((g) => g.hat));
    this.svg.classList.toggle('no-collar', pieces.some((g) => g.collar === false));
    this.svg.classList.toggle('no-paws', pieces.some((g) => g.paws === false));
    this.svg.classList.toggle('no-tail', pieces.some((g) => g.tail === false));
    for (const f of FACES) this.svg.classList.toggle(`face-${f}`, look.face === f);
    const patches = new Set(look.patches || []);
    for (const p of ['top', 'eye', 'ears']) this.svg.classList.toggle(`p-${p}`, patches.has(p));
    for (const p of ['polite', 'bounce']) this.svg.classList.toggle(`pose-${p}`, look.pose === p);
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
    const words = { awake: 'awake', happy: 'happy', listen: 'listening', nap: 'napping', serious: 'working hard' };
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

  /** Warm: a little sweat drop (the computer is working hard or getting hot). */
  setHot(on) {
    this.svg.classList.toggle('hot', !!on);
  }

  destroy() {
    if (this.onMove) window.removeEventListener('pointermove', this.onMove);
    clearTimeout(this.timer);
    clearTimeout(this.kneadTimer);
    this.host.innerHTML = '';
  }
}
