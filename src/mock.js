// Fake backend used only when the page is opened outside the app (design previews).
// The real app never loads this file.

const STARTER = [
  ['LAlt+W', 'LCtrl+W', 'Close tab'], ['LAlt+C', 'LCtrl+C', 'Copy'], ['LAlt+V', 'LCtrl+V', 'Paste'], ['LAlt+N', 'Ctrl+N', 'New'],
  ['LAlt+Q', 'LAlt+F4', 'Quit app'], ['LAlt+F', 'LCtrl+F', 'Find'], ['LAlt+A', 'LCtrl+A', 'Select all'],
  ['LAlt+LShift+N', 'LCtrl+LShift+N', 'New window (private)'], ['LAlt+S', 'LCtrl+S', 'Save'], ['LAlt+Z', 'LCtrl+Z', 'Undo'], ['LAlt+R', 'LCtrl+R', 'Reload'],
  ['LAlt+T', 'LCtrl+T', 'New tab'],
];
const MORE = [
  ['LAlt+X', 'LCtrl+X', 'Cut'], ['LAlt+LShift+Z', 'LCtrl+Y', 'Redo'], ['LAlt+LShift+T', 'LCtrl+LShift+T', 'Reopen closed tab'],
  ['LAlt+L', 'LCtrl+L', 'Address bar'], ['LAlt+P', 'LCtrl+P', 'Print'], ['LAlt+O', 'LCtrl+O', 'Open'], ['LAlt+K', 'LCtrl+K', 'Quick search'],
  ['LAlt+B', 'LCtrl+B', 'Bold'], ['LAlt+I', 'LCtrl+I', 'Italic'], ['LAlt+Equals', 'LCtrl+Equals', 'Zoom in'], ['LAlt+Minus', 'LCtrl+Minus', 'Zoom out'],
  ['LAlt+0', 'LCtrl+0', 'Reset zoom'], ['LAlt+LShift+RBracket', 'LCtrl+Tab', 'Next tab'], ['LAlt+LShift+LBracket', 'LCtrl+LShift+Tab', 'Previous tab'],
  ['LAlt+LBracket', 'LAlt+Left', 'Back'], ['LAlt+RBracket', 'LAlt+Right', 'Forward'], ['LAlt+Backspace', 'LCtrl+Backspace', 'Delete word'],
];
const rulesFrom = (list, prefix) => list.map(([trigger, keys, name], i) => ({ id: `${prefix}${i}`, enabled: true, name, trigger, action: { type: 'keys', keys }, scope: { mode: 'all', apps: [] } }));
const DEFAULT_RULES = rulesFrom(STARTER, 'd');
const TEMPLATES = [
  { id: 'builtin-starter', name: 'Your starter set', builtin: true, rules: rulesFrom(STARTER, 'builtin-starter-'),
    description: 'The shortcuts Cat Key came with: your PowerToys set plus Alt + T for a new tab. Alt works like ⌘ on a Mac.' },
  { id: 'builtin-mac-more', name: 'More Mac-style', builtin: true, rules: rulesFrom(MORE, 'builtin-mac-more-'),
    description: 'More of the ⌘ habits: cut, redo, reopen a closed tab, zoom, bold, switching tabs and more.' },
  { id: 'builtin-caps-esc', name: 'Caps Lock as Escape', builtin: true, rules: rulesFrom([['CapsLock', 'Esc', 'Escape']], 'builtin-caps-esc-'),
    description: 'Caps Lock becomes Escape: close pop-ups and menus without reaching. Loved by Vim users.' },
];

function mockStatus(params) {
  return {
    version: '1.2.1', admin: false, powertoysRunning: params.get('pt') !== '0', firstRun: params.get('first') === '1', configReset: false,
    configDir: 'C:\\Users\\me\\AppData\\Roaming\\CatKey', problems: {}, engineRunning: params.get('engine') !== '0',
    startup: params.get('startup') || 'on',
    platform: params.get('mac') === '1' ? 'mac' : 'windows',
    keyboardAllowed: params.get('allowed') !== '0',
    tempPlace: params.get('temp') === '1',
    screenAllowed: params.get('screen') !== '0',
  };
}

export function createMock() {
  const params = new URLSearchParams(location.search);
  let upgraded = params.get('upgraded') === '1';
  let keylogOn = false;
  let keylogSent = false;
  let config = {
    version: 2,
    enabled: params.get('nap') !== '1',
    rules: DEFAULT_RULES.concat(params.get('extras') === '1' ? [
      { id: 'x1', enabled: true, name: 'Address bar', trigger: 'LAlt+L', action: { type: 'keys', keys: 'LCtrl+L' }, scope: { mode: 'only', apps: ['chrome.exe'] } },
      { id: 'x2', enabled: false, name: 'Open Terminal', trigger: 'Ctrl+Alt+T', action: { type: 'launch', target: 'wt.exe', args: '' }, scope: { mode: 'all', apps: [] } },
      { id: 'x3', enabled: true, name: '', trigger: 'LAlt+M', action: { type: 'text', text: 'Thanks! I’ll get back to you today.' }, scope: { mode: 'all', apps: [] } },
    ] : []),
    settings: {
      launchAtStartup: true, adminMode: false, theme: params.get('theme') || 'dark', skin: params.get('skin') || 'mochi',
      catName: '', sounds: true, pauseHotkey: null, excludedApps: [], hidePowertoysTip: false,
      coach: true, coachChromium: false, coachMuted: [], purrVolume: 45,
    },
    templates: params.get('mytpl') === '1'
      ? [{ id: 'tmine', name: 'Work laptop', description: '3 shortcuts, saved Nov 20.', builtin: false, rules: rulesFrom(STARTER.slice(0, 3), 'tm') }]
      : [],
  };
  if (params.get('swapped') === '1') {
    config.rules = config.rules.map((r) => (r.id === 'd0' ? { ...r, name: 'Close tab (reversed)', trigger: 'LCtrl+W', action: { type: 'keys', keys: 'LAlt+W' } } : r));
  }
  const perRule = { d0: 187, d1: 1204, d2: 1031, d3: 96, d4: 58, d5: 143, d6: 312, d7: 21, d8: 455, d9: 602, d10: 77, d11: 3, x1: 64, x2: 12, x3: 9 };
  const apps = [
    { exe: 'chrome.exe', name: 'Google Chrome', path: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe', icon: null },
    { exe: 'code.exe', name: 'Visual Studio Code', path: 'C:\\Users\\me\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe', icon: null },
    { exe: 'explorer.exe', name: 'Windows Explorer', path: 'C:\\Windows\\explorer.exe', icon: null },
    { exe: 'spotify.exe', name: 'Spotify', path: 'C:\\Users\\me\\AppData\\Roaming\\Spotify\\Spotify.exe', icon: null },
    { exe: 'figma.exe', name: 'Figma', path: 'C:\\Users\\me\\AppData\\Local\\Figma\\Figma.exe', icon: null },
    { exe: 'discord.exe', name: 'Discord', path: 'C:\\Users\\me\\AppData\\Local\\Discord\\Discord.exe', icon: null },
  ];
  const listeners = {};
  let pendingAdjust = null;
  const nowS = Math.floor(Date.now() / 1000);
  window.__mockEmit = (event, payload) => (listeners[event] || []).forEach((f) => f({ payload }));
  window.__mockAdjust = (tip) => { pendingAdjust = tip; window.__mockEmit('ck://coach-adjust', tip); };
  return {
    listen: async (event, fn) => {
      (listeners[event] ||= []).push(fn);
      return () => {};
    },
    invoke: async (cmd, args = {}) => {
      switch (cmd) {
        case 'get_state': {
          const was = upgraded;
          upgraded = false;
          return {
            config: structuredClone(config),
            status: mockStatus(params),
            stats: { day: '', today: 42, total: 4200, perRule },
            templates: structuredClone(TEMPLATES),
            upgraded: was,
          };
        }
        case 'get_status':
          return mockStatus(params);
        case 'get_stats':
          return { day: '', today: 42, total: 4200, perRule };
        case 'save_config':
          config = structuredClone(args.config);
          return structuredClone(config);
        case 'set_enabled':
          config.enabled = args.enabled;
          return null;
        case 'running_apps':
          return apps;
        case 'mcp_info':
          return { claudeInstalled: params.get('claude') !== '0', claudeConnected: params.get('claude') === 'on', entry: '{\n  "mcpServers": {\n    "catkey": { "command": "C:\\\\Users\\\\me\\\\AppData\\\\Local\\\\Cat Key\\\\CatKey.exe", "args": ["--mcp"] }\n  }\n}', claudeCode: 'claude mcp add catkey -- "C:\\Users\\me\\AppData\\Local\\Cat Key\\CatKey.exe" --mcp', tempPlace: false };
        case 'mcp_connect':
          return null;
        case 'catgpt_info': {
          const svg = (c, t) => `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="640" height="400"><rect width="640" height="400" fill="${c}"/><rect x="40" y="40" width="560" height="40" rx="8" fill="#fff" opacity=".7"/><text x="320" y="240" font-size="44" font-family="sans-serif" text-anchor="middle" fill="#fff">${t}</text></svg>`)}`;
          window.__cgImgs = { s1: svg('#3B6FD9', 'Screenshot'), c1: svg('#7A3FD1', 'Canvas'), r1: svg('#1F8A5B', 'Recording') };
          const now = Date.now() / 1000;
          const items = params.get('basket') === '0' ? [] : [
            { id: 's1', kind: 'shot', name: 'Screenshot 2026-11-26 10-02-11.jpg', path: 'x', bytes: 182000, added: now - 60, w: 1920, h: 1080, sent: false, frames: [], text: '', secs: 0 },
            { id: 'n1', kind: 'note', name: 'Why does this button do nothing?', text: 'Why does this button do nothing?\nIt worked yesterday.', bytes: 60, added: now - 120, sent: false, frames: [], path: '', secs: 0 },
            { id: 'c1', kind: 'canvas', name: 'Canvas 2026-11-26 10-03-40.jpg', path: 'x', bytes: 240000, added: now - 300, w: 1920, h: 1080, sent: true, frames: [], text: '', secs: 0 },
            { id: 'r1', kind: 'rec', name: 'Recording 2026-11-26 09-58-00.mp4', path: 'x', bytes: 2400000, added: now - 900, w: 1920, h: 1080, sent: true, frames: ['f1'], text: '', secs: 21.5 },
            { id: 'f1', kind: 'file', name: 'report.pdf', path: 'x', bytes: 830000, added: now - 4000, sent: true, frames: [], text: '', secs: 0 },
          ];
          return { apps: [{ id: 'claude', name: 'Claude', installed: params.get('claudeapp') !== '0', deep: true }, { id: 'chatgpt', name: 'ChatGPT', installed: params.get('gptapp') !== '0', deep: true }],
            items, dir: params.get('mac') === '1' ? '/Users/me/Library/Application Support/CatKey/catgpt' : 'C:\\Users\\me\\AppData\\Roaming\\CatKey\\catgpt', bytes: 3652000, recording: params.get('rec') === '1', recStarted: Date.now() - 12000,
            canCapture: params.get('capture') !== '0' };
        }
        case 'catgpt_image':
          return (window.__cgImgs || {})[args.id] || null;
        case 'catgpt_recent':
          return args.app === 'chatgpt' ? [1, 2, 3].map((n) => ({ title: `Recent chat ${n}`, dest: `recent:${n}` })) : [];
        case 'catgpt_send':
          window.__cgSent = args.req;
          return 'Your words in the message box. Pick the model if you like, then press Enter.';
        case 'catgpt_wheel_last':
          return { seq: 1, recording: false, recSecs: 0, fresh: 2, apps: [{ id: 'claude', name: 'Claude', installed: true, deep: true }, { id: 'chatgpt', name: 'ChatGPT', installed: true, deep: true }] };
        case 'catgpt_canvas_bg':
          return { image: '', w: 0, h: 0 };
        case 'take_inbox':
          return params.get('inbox') === '1' && !window.__inboxTaken ? (window.__inboxTaken = true, [JSON.stringify({ format: 'catkey-pack', version: 1, name: 'REAPER', kind: 'cheatsheet', platform: 'both', app: 'REAPER', description: 'From Claude', apps: { windows: ['reaper.exe'] }, sections: [{ name: 'Editing', items: [{ name: 'Split items', keys: 'S', fav: true }, { name: 'Undo', keys: 'Mod+Z' }] }] })]) : [];
        case 'installed_apps':
          return params.get('scan') === '0' ? [] : [
            ...apps,
            { exe: 'photoshop.exe', name: 'Adobe Photoshop 2025', path: 'C:\\Program Files\\Adobe\\Adobe Photoshop 2025\\Photoshop.exe', title: '' },
            { exe: 'blender.exe', name: 'Blender 4.2', path: 'C:\\Program Files\\Blender Foundation\\Blender 4.2\\blender.exe', title: '' },
            { exe: 'ableton live 12 suite.exe', name: 'Ableton Live 12 Suite', path: 'C:\\ProgramData\\Ableton\\Live 12 Suite\\Program\\Ableton Live 12 Suite.exe', title: '' },
            { exe: 'reaper.exe', name: 'REAPER', path: 'C:\\Program Files\\REAPER (x64)\\reaper.exe', title: '' },
            { exe: 'obs64.exe', name: 'OBS Studio', path: 'C:\\Program Files\\obs-studio\\bin\\64bit\\obs64.exe', title: '' },
            { exe: 'krita.exe', name: 'Krita', path: 'C:\\Program Files\\Krita (x64)\\bin\\krita.exe', title: '' },
            { exe: 'notion.exe', name: 'Notion', path: 'C:\\Users\\me\\AppData\\Local\\Programs\\Notion\\Notion.exe', title: '' },
            { exe: 'affinity photo 2.exe', name: 'Affinity Photo 2', path: 'C:\\Program Files\\Affinity\\Photo 2\\Photo.exe', title: '' },
            { exe: 'houdini.exe', name: 'Houdini 20.5', path: 'C:\\Program Files\\Side Effects Software\\Houdini 20.5\\bin\\houdini.exe', title: '' },
          ];
        case 'restore_defaults':
          config.rules = structuredClone(DEFAULT_RULES);
          return structuredClone(config);
        case 'import_powertoys':
          return { found: 11, added: 0, config: structuredClone(config) };
        case 'key_log':
          keylogOn = !!args.on;
          keylogSent = false;
          return null;
        case 'key_log_take': {
          if (!keylogOn || keylogSent) return { on: keylogOn, lines: [] };
          keylogSent = true;
          return { on: true, lines: [
            '  0.000s  key log started · Cat Key 1.2.0 · hook on · watchdog on',
            '  2.114s  LAlt↓  from keyboard  held: -  → passed  [chrome.exe]',
            '  2.201s  W↓  from keyboard  held: LAlt  → swallowed  then send: LCtrl↓ mask LAlt↑ W↓  [chrome.exe]',
            '  2.201s     sent 5/5 in 0 ms',
            '  2.289s  W↑  from keyboard  held: LCtrl  → swallowed  then send: W↑  [chrome.exe]',
            '  2.290s     sent 1/1 in 0 ms',
            '  2.340s  LAlt↑  from keyboard  held: LCtrl  → passed  then send: LCtrl↑  [chrome.exe]',
          ] };
        }
        case 'set_admin_mode':
          throw 'cancelled';
        case 'clipboard_seq':
          return 1;
        case 'read_clipboard':
          return null;
        case 'coach_stats': {
          const d = new Date();
          const k = (n) => { const x = new Date(d); x.setDate(x.getDate() - n); return x.toLocaleDateString('en-CA'); };
          return {
            days: params.get('coach') === '0' ? {} : {
              [k(0)]: { 'adobe premiere pro.exe': { used: 182, caught: 9, lostMs: 20100 }, 'chrome.exe': { used: 240, caught: 0, lostMs: 0 }, 'explorer.exe': { used: 35, caught: 6, lostMs: 9600 } },
              [k(1)]: { 'adobe premiere pro.exe': { used: 150, caught: 12, lostMs: 30800 }, 'code.exe': { used: 410, caught: 2, lostMs: 3200 } },
              [k(3)]: { 'photoshop.exe': { used: 90, caught: 5, lostMs: 11000 } },
            },
            tips: params.get('coach') === '0' ? {} : {
              'explorer.exe|back': { app: 'explorer.exe', name: 'Back', keys: 'Alt+Left Arrow', kind: 'button', n: 7, last: nowS - 120 },
              'adobe premiere pro.exe|ripple delete': { app: 'adobe premiere pro.exe', name: 'Ripple Delete', keys: 'Shift+Delete', kind: 'menu', n: 5, last: nowS - 3 * 3600 },
              '*|window-close': { app: 'notepad.exe', name: 'Close the window', keys: 'Alt+F4', kind: 'window', n: 4, last: nowS - 26 * 3600 },
              '*|habit-backspace': { app: 'code.exe', name: 'Delete a whole word', keys: 'Ctrl+Backspace', kind: 'habit', n: 3, last: nowS - 4 * 86400 },
            },
            savedPerShortcutMs: 1600, lostMenuMs: 2900, lostButtonMs: 1600,
          };
        }
        case 'set_coach_dict':
        case 'coach_demo':
        case 'coach_forget':
        case 'coach_hide':
          (window.__coachCalls ||= []).push(cmd);
          return null;
        case 'coach_ignore': {
          const id = String(args.id).toLowerCase();
          const list = config.settings.coachMuted || (config.settings.coachMuted = []);
          if (!list.includes(id)) list.push(id);
          setTimeout(() => window.__mockEmit('ck://coach-muted', [...list]), 0);
          return [...list];
        }
        case 'coach_adjust':
          pendingAdjust = { id: args.id, app: 'explorer.exe', appName: 'File Explorer', name: 'Back', keys: 'Alt+Left Arrow', savesMs: 1600, times: 7, seq: 1, demo: false, anchor: 'bottom-right' };
          setTimeout(() => window.__mockEmit('ck://coach-adjust', pendingAdjust), 0);
          return null;
        case 'coach_take_adjust': {
          const t = pendingAdjust;
          pendingAdjust = null;
          return t;
        }
        case 'coach_last':
          return null;
        case 'purr_set':
          return !!args.on;
        case 'purr_state':
          return false;
        case 'data_info':
          return { folder: 'C:\\Users\\me\\AppData\\Roaming\\CatKey', files: [
            { name: 'config.json', bytes: 18420, modified: Math.floor(Date.now() / 1000) - 60 },
            { name: 'progress.json', bytes: 9120, modified: Math.floor(Date.now() / 1000) - 30 },
            { name: 'coach.json', bytes: 3311, modified: Math.floor(Date.now() / 1000) - 300 },
            { name: 'stats.json', bytes: 812, modified: Math.floor(Date.now() / 1000) - 900 },
            { name: 'coach-dict.json', bytes: 40210, modified: Math.floor(Date.now() / 1000) - 3600 },
            { name: 'catkey.log', bytes: 2100, modified: Math.floor(Date.now() / 1000) - 7200 }] };
        case 'open_data_folder':
        case 'save_progress':
        case 'load_progress':
          return null;
        case 'backup_now':
          return 'C:\\Users\\me\\Documents\\Cat Key backup 2026-11-26 0352';
        case 'restore_backup':
          return false;
        case 'tracked_counts':
          return {};
        case 'set_tracked':
        case 'focus_window':
          return null;
        case 'open_link':
          (window.__opened ||= []).push(args?.url);
          return null;
        case 'open_ai_app':
          (window.__opened ||= []).push(`app:${args?.app}:${args?.link || ''}`);
          return window.__mockAiApp || (args?.link ? 'filled' : 'paste');
        case 'save_text':
          console.log('[mock] save_text', args?.name, (args?.text || '').length);
          return true;
        case 'open_text_file':
          return null;
        case 'start_recording': {
          if (window.__mockNoKeys) return null;
          const keys = window.__mockKeys || 'LAlt+LShift+T';
          const parts = keys.split('+');
          parts.forEach((_, i) => setTimeout(() => window.__mockEmit('ck://record', { phase: 'update', shortcut: parts.slice(0, i + 1).join('+') }), 300 + i * 200));
          setTimeout(() => window.__mockEmit('ck://record', { phase: 'done', shortcut: keys }), 400 + parts.length * 200);
          return null;
        }
        default:
          return null;
      }
    },
  };
}
