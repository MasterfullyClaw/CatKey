// Fake backend used only when the page is opened outside the app (design previews).
// The real app never loads this file.

const DEFAULT_RULES = [
  ['LAlt+W', 'LCtrl+W', 'Close tab'], ['LAlt+C', 'LCtrl+C', 'Copy'], ['LAlt+V', 'LCtrl+V', 'Paste'], ['LAlt+N', 'Ctrl+N', 'New'],
  ['LAlt+Q', 'LAlt+F4', 'Quit app'], ['LAlt+F', 'LCtrl+F', 'Find'], ['LAlt+A', 'LCtrl+A', 'Select all'],
  ['LAlt+LShift+N', 'LCtrl+LShift+N', 'New window (private)'], ['LAlt+S', 'LCtrl+S', 'Save'], ['LAlt+Z', 'LCtrl+Z', 'Undo'], ['LAlt+R', 'LCtrl+R', 'Reload'],
].map(([trigger, keys, name], i) => ({ id: `d${i}`, enabled: true, name, trigger, action: { type: 'keys', keys }, scope: { mode: 'all', apps: [] } }));

function mockStatus(params) {
  return {
    version: '1.0.1', admin: false, powertoysRunning: params.get('pt') !== '0', firstRun: params.get('first') === '1', configReset: false,
    configDir: 'C:\\Users\\me\\AppData\\Roaming\\CatKey', problems: {}, engineRunning: params.get('engine') !== '0',
    startup: params.get('startup') || 'on',
  };
}

export function createMock() {
  const params = new URLSearchParams(location.search);
  let config = {
    version: 1,
    enabled: params.get('nap') !== '1',
    rules: DEFAULT_RULES.concat(params.get('extras') === '1' ? [
      { id: 'x1', enabled: true, name: 'Address bar', trigger: 'LAlt+L', action: { type: 'keys', keys: 'LCtrl+L' }, scope: { mode: 'only', apps: ['chrome.exe'] } },
      { id: 'x2', enabled: false, name: 'Open Terminal', trigger: 'Ctrl+Alt+T', action: { type: 'launch', target: 'wt.exe', args: '' }, scope: { mode: 'all', apps: [] } },
      { id: 'x3', enabled: true, name: '', trigger: 'LAlt+M', action: { type: 'text', text: 'Thanks! I’ll get back to you today.' }, scope: { mode: 'all', apps: [] } },
    ] : []),
    settings: {
      launchAtStartup: true, adminMode: false, theme: params.get('theme') || 'dark', skin: params.get('skin') || 'mochi',
      catName: '', sounds: false, pauseHotkey: null, excludedApps: [], hidePowertoysTip: false,
    },
  };
  const perRule = { d0: 187, d1: 1204, d2: 1031, d3: 96, d4: 58, d5: 143, d6: 312, d7: 21, d8: 455, d9: 602, d10: 77, x1: 64, x2: 12, x3: 9 };
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
        case 'get_state':
          return {
            config: structuredClone(config),
            status: mockStatus(params),
            stats: { day: '', today: 42, total: 4200, perRule },
          };
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
        case 'set_admin_mode':
          throw 'cancelled';
        case 'start_recording':
          setTimeout(() => window.__mockEmit('ck://record', { phase: 'update', shortcut: 'LAlt' }), 500);
          setTimeout(() => window.__mockEmit('ck://record', { phase: 'update', shortcut: 'LAlt+LShift' }), 800);
          setTimeout(() => window.__mockEmit('ck://record', { phase: 'update', shortcut: 'LAlt+LShift+T' }), 1100);
          setTimeout(() => window.__mockEmit('ck://record', { phase: 'done', shortcut: 'LAlt+LShift+T' }), 1500);
          return null;
        default:
          return null;
      }
    },
  };
}
