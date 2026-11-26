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
  window.__mockEmit = (event, payload) => (listeners[event] || []).forEach((f) => f({ payload }));
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
              'explorer.exe|back': { app: 'explorer.exe', name: 'Back', keys: 'Alt+Left Arrow', kind: 'button', n: 7, last: 1 },
              'adobe premiere pro.exe|ripple delete': { app: 'adobe premiere pro.exe', name: 'Ripple Delete', keys: 'Shift+Delete', kind: 'menu', n: 5, last: 2 },
              '*|window-close': { app: 'notepad.exe', name: 'Close the window', keys: 'Alt+F4', kind: 'window', n: 4, last: 3 },
              '*|habit-backspace': { app: 'code.exe', name: 'Delete a whole word', keys: 'Ctrl+Backspace', kind: 'habit', n: 3, last: 4 },
            },
            savedPerShortcutMs: 1600, lostMenuMs: 2900, lostButtonMs: 1600,
          };
        }
        case 'set_coach_dict':
        case 'coach_demo':
        case 'coach_forget':
          return null;
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
