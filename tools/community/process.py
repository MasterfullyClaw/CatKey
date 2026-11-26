"""The Cat Key store and leaderboard, kept by a GitHub Action.

People post from inside Cat Key: it opens a GitHub issue form, filled in, in their
browser (Cat Key never sees a login). This script reads those issues:

* "Pack: …"      → checks the pack, saves it in library/community/packs/ and lists it
                   in library/community/index.json with the poster's GitHub name;
* "Cat Card: …"  → adds the card to leaderboard/index.json (one line per person);
* --refresh      → counts the ❤️ on each pack's issue and sorts the store by them.

Everything it reads from an issue is untrusted: only known fields of known shapes
are kept, and nothing from an issue is ever run.
"""
import base64, datetime, json, os, re, sys, urllib.request, zlib

REPO = os.environ.get('GITHUB_REPOSITORY', 'MasterfullyClaw/CatKey')
TOKEN = os.environ.get('GITHUB_TOKEN', '')
ROOT = os.environ.get('GITHUB_WORKSPACE', os.getcwd())
STORE = os.path.join(ROOT, 'library', 'community')
BOARD = os.path.join(ROOT, 'leaderboard', 'index.json')
KEY_RE = re.compile(r'^[A-Za-z0-9+]{1,40}$')


def api(path, method='GET', body=None):
    req = urllib.request.Request(f'https://api.github.com/repos/{REPO}{path}', method=method,
                                 data=json.dumps(body).encode() if body is not None else None,
                                 headers={'Authorization': f'Bearer {TOKEN}', 'Accept': 'application/vnd.github+json', 'User-Agent': 'catkey-store'})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read() or b'null')


def load(path, default):
    try:
        with open(path, encoding='utf-8') as f:
            return json.load(f)
    except Exception:
        return default


def save(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=1)
        f.write('\n')


def decode(text, prefix):
    m = re.search(re.escape(prefix) + r'([A-Za-z0-9_-]{8,})', text or '')
    if not m:
        return None
    raw = m.group(1)
    raw += '=' * (-len(raw) % 4)
    data = zlib.decompress(base64.urlsafe_b64decode(raw), -15)
    if len(data) > 600_000:
        raise ValueError('too big')
    return json.loads(data)


def s(x, n):
    return str(x or '').strip()[:n]


def clean_pack(src):
    if not isinstance(src, dict):
        raise ValueError('not a pack')
    kind = 'cheatsheet' if src.get('kind') == 'cheatsheet' else 'remap'
    sections, count = [], 0
    for sec in (src.get('sections') or [])[:30]:
        items = []
        for it in (sec.get('items') or [])[:200] if isinstance(sec, dict) else []:
            if not isinstance(it, dict):
                continue
            x = {'name': s(it.get('name'), 80)}
            fields = ('keys', 'mac') if kind == 'cheatsheet' else ('press', 'does')
            ok = True
            for k in fields:
                v = it.get(k)
                if v is None:
                    continue
                if not isinstance(v, str) or not KEY_RE.match(v):
                    ok = False
                    break
                x[k] = v
            if not ok or (kind == 'cheatsheet' and 'keys' not in x) or (kind == 'remap' and 'press' not in x):
                continue
            if kind == 'remap' and 'does' not in x:
                continue  # the store keeps key-to-key remaps only (no programs or text)
            for k in ('fav', 'off'):
                if it.get(k) is True:
                    x[k] = True
            if it.get('note'):
                x['note'] = s(it['note'], 140)
            demo = it.get('demo')
            if isinstance(demo, str) and len(demo) <= 20000 and '<svg' in demo and not re.search(r'<script|<foreignObject|javascript:', demo, re.I):
                x['demo'] = demo  # Cat Key cleans it again before drawing it
            for k in ('only', 'except'):
                if isinstance(it.get(k), list):
                    x[k] = [s(a, 120) for a in it[k][:10]]
            items.append(x)
        if items:
            sections.append({'name': s(sec.get('name'), 60) or 'Shortcuts', 'items': items})
            count += len(items)
    if not count:
        raise ValueError('no usable shortcuts')
    pack = {'format': 'catkey-pack', 'version': 1, 'name': s(src.get('name'), 60) or 'Shared pack', 'kind': kind,
            'platform': src.get('platform') if src.get('platform') in ('windows', 'mac', 'both') else 'both',
            'description': s(src.get('description'), 300), 'sections': sections}
    if src.get('app'):
        pack['app'] = s(src['app'], 60)
    apps = src.get('apps')
    if isinstance(apps, dict):
        pack['apps'] = {p: [s(a, 120) for a in (apps.get(p) or [])[:10] if isinstance(a, str)] for p in ('windows', 'mac')}
    srcs = [x for x in (src.get('sources') or []) if isinstance(x, dict) and str(x.get('url', '')).startswith('https://')]
    if srcs:
        pack['sources'] = [{'title': s(x.get('title') or x['url'], 120), 'url': s(x['url'], 400)} for x in srcs[:12]]
    return pack, count


def slug(name):
    return re.sub(r'[^a-z0-9]+', '-', name.lower()).strip('-')[:40] or 'pack'


def reply(number, text, close=True):
    api(f'/issues/{number}/comments', 'POST', {'body': text})
    if close:
        api(f'/issues/{number}', 'PATCH', {'state': 'closed', 'state_reason': 'completed'})


def handle_issue(issue):
    title, body, number = issue.get('title') or '', issue.get('body') or '', issue['number']
    login = (issue.get('user') or {}).get('login', '')
    today = datetime.date.today().isoformat()
    if title.lower().startswith('pack:'):
        try:
            raw = decode(body, 'CK1:')
            if raw is None:
                m = re.search(r'\{[\s\S]*\}', body)
                raw = json.loads(m.group(0)) if m else None
            pack, count = clean_pack(raw)
        except Exception as e:  # noqa: BLE001 - tell the poster, never crash
            reply(number, f"Cat Key couldn't read this pack ({e}). In Cat Key, open your pack and press **Post to the store** again.", close=False)
            return False
        pid = f'{slug(pack["name"])}-{number}'
        pack['author'] = login
        save(os.path.join(STORE, 'packs', f'{pid}.json'), pack)
        index = load(os.path.join(STORE, 'index.json'), {'version': 1, 'packs': []})
        index['packs'] = [p for p in index.get('packs', []) if p.get('issue') != number]
        index['packs'].append({'id': pid, 'name': pack['name'], 'description': pack['description'], 'app': pack.get('app', ''),
                               'kind': pack['kind'], 'platform': pack['platform'], 'count': count, 'file': f'packs/{pid}.json',
                               'author': login, 'issue': number, 'likes': 0, 'created': today})
        save(os.path.join(STORE, 'index.json'), index)
        reply(number, f'Listed in the store as **{pack["name"]}** ({count} shortcuts), by @{login}. ❤️ this issue to like it: the most loved packs rise to the top. Thanks for sharing!')
        return True
    if title.lower().startswith('cat card:'):
        try:
            card = decode(body, 'CKC1:')
            if not isinstance(card, dict) or card.get('format') != 'catkey-card':
                raise ValueError('no card code')
        except Exception as e:  # noqa: BLE001
            reply(number, f"Cat Key couldn't read this card ({e}). In Cat Key, open Cat Warrior and press **Post to the leaderboard** again.", close=False)
            return False
        num = lambda v, top: max(0, min(int(v) if isinstance(v, (int, float)) else 0, top))  # noqa: E731
        entry = {'login': login, 'name': s(card.get('name'), 24), 'cat': s(card.get('cat'), 24), 'rank': s(card.get('rank'), 30),
                 'xp': num(card.get('xp'), 10_000_000), 'learned': num(card.get('learned'), 100_000), 'savedMin': num(card.get('savedMin'), 10_000_000),
                 'streak': num(card.get('streak'), 10_000), 'at': today, 'issue': number}
        board = load(BOARD, {'version': 1, 'players': []})
        board['players'] = [p for p in board.get('players', []) if p.get('login') != login] + [entry]
        board['players'].sort(key=lambda p: -p['xp'])
        save(BOARD, board)
        place = next(i for i, p in enumerate(board['players']) if p['login'] == login) + 1
        reply(number, f'@{login} is on the board: **{entry["rank"]}**, {entry["xp"]:,} XP, number {place} of {len(board["players"])}. Post again any time to update your line.')
        return True
    return False


def refresh_likes():
    index = load(os.path.join(STORE, 'index.json'), {'version': 1, 'packs': []})
    changed = False
    for p in index.get('packs', []):
        if not p.get('issue'):
            continue
        try:
            hearts = int((api(f'/issues/{p["issue"]}').get('reactions') or {}).get('heart', 0))
        except Exception:  # noqa: BLE001
            continue
        if hearts != p.get('likes'):
            p['likes'] = hearts
            changed = True
    index['packs'].sort(key=lambda p: (-int(p.get('likes') or 0), p.get('created', '')))
    save(os.path.join(STORE, 'index.json'), index)
    return changed


if __name__ == '__main__':
    if '--refresh' in sys.argv:
        print('likes changed' if refresh_likes() else 'likes unchanged')
    else:
        event = load(os.environ['GITHUB_EVENT_PATH'], {})
        issue = event.get('issue')
        print('handled' if issue and handle_issue(issue) else 'not a store issue')
