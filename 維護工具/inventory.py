"""Confined, non-following inventory; no writes outside the reconstruction root."""
import argparse
import hashlib
import json
import os
from collections import defaultdict
from pathlib import Path
import stat
import zipfile

ROOT = Path(__file__).absolute().parent.parent
OUT = ROOT / '驗證紀錄' / '20261012'
REPARSE = getattr(stat, 'FILE_ATTRIBUTE_REPARSE_POINT', 0x400)

def safe(path):
    path = Path(os.path.abspath(path))
    if path != ROOT and ROOT not in path.parents:
        raise ValueError(f'Outside reconstruction root: {path}')
    for part in [path, *path.parents]:
        if part.exists() and part.lstat().st_file_attributes & REPARSE:
            raise ValueError(f'Reparse point refused: {part}')
        if part == ROOT:
            break
    if ROOT.resolve() != ROOT:
        raise ValueError('Root identity changed')
    return path

def walk(root):
    pending = [safe(root)]
    while pending:
        parent = pending.pop()
        with os.scandir(parent) as entries:
            for item in entries:
                info = item.stat(follow_symlinks=False)
                path = Path(item.path)
                if info.st_file_attributes & REPARSE:
                    yield path, info, 'link'
                elif item.is_dir(follow_symlinks=False):
                    pending.append(path)
                else:
                    yield path, info, 'file'

def digest(path):
    with safe(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def save(name, value):
    OUT.mkdir(parents=True, exist_ok=True)
    safe(OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

def inventory():
    if (OUT / 'inventory-before.json').exists() or (OUT / 'file-inventory-before.json').exists():
        raise ValueError('Initial inventory already exists; preserve immutable cleanup evidence')
    groups = defaultdict(lambda: {'files': 0, 'bytes': 0})
    target_groups = defaultdict(lambda: {'files': 0, 'bytes': 0})
    links, instructions, incremental = [], [], defaultdict(lambda: {'files': 0, 'bytes': 0})
    files = []
    for path, info, kind in walk(ROOT):
        rel = path.relative_to(ROOT)
        if kind == 'link':
            links.append(str(rel)); continue
        parts = rel.parts
        group = '/'.join(parts[:2]) if len(parts) > 2 else parts[0]
        groups[group]['files'] += 1; groups[group]['bytes'] += info.st_size
        if path.name == 'AGENTS.md':
            instructions.append(str(rel))
        if parts[:2] == ('LiTo381-Modern', 'target') and len(parts) > 3:
            sub = parts[2]
            target_groups[sub]['files'] += 1; target_groups[sub]['bytes'] += info.st_size
            if 'incremental' in parts:
                idx = parts.index('incremental')
                inc = ROOT.joinpath(*parts[:idx+1])
                incremental[str(inc)]['files'] += 1; incremental[str(inc)]['bytes'] += info.st_size
        files.append({'path': str(rel), 'bytes': info.st_size, 'mtime_ns': info.st_mtime_ns})
    result = {'root': str(ROOT), 'files': len(files), 'bytes': sum(x['bytes'] for x in files),
              'groups': groups, 'target_groups': target_groups, 'links_skipped': links,
              'instructions': instructions, 'incremental_candidates': incremental}
    save('inventory-before.json', result)
    save('file-inventory-before.json', files)
    print(json.dumps({k: result[k] for k in ('files','bytes','links_skipped','instructions')}, ensure_ascii=False))
    print('Largest target groups:', sorted(target_groups.items(), key=lambda x: x[1]['bytes'], reverse=True)[:12])
    print('Rebuildable incremental bytes:', sum(x['bytes'] for x in incremental.values()))

def archive():
    """Preserve current sources/docs/queues and ALL historical Rust source snapshots."""
    destination = safe(ROOT / '封存' / 'before-cleanup-20261012.zip')
    destination.parent.mkdir(exist_ok=True)
    if destination.exists():
        raise ValueError('Archive already exists; preserve it')
    manifest, blobs = [], set()
    with zipfile.ZipFile(destination, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        rows = json.loads((OUT / 'file-inventory-before.json').read_text(encoding='utf-8'))
        rows.sort(key=lambda row: '/target/' in row['path'].replace('\\','/'))
        for row in rows:
            path = ROOT / row['path']
            rel = Path(row['path'])
            parts = rel.parts
            if parts[0] in ('封存', '驗證紀錄'):
                continue
            preserve = (len(parts) == 1 or parts[0] == '整理紀錄' or
                (parts[0] == 'LiTo381-Modern' and len(parts) > 1 and
                 parts[1] not in ('target','logs','dist') ) or
                (parts[:2] == ('LiTo381-Modern','target') and path.suffix in ('.rs','.toml','.lock','.ps1','.py','.java','.md','.txt')
                 and '.fingerprint' not in parts and 'incremental' not in parts))
            # Large runtime JSON is not a source snapshot; keep it in place.
            if not preserve or row['bytes'] > 32 * 1024 * 1024:
                continue
            info = safe(path).stat()
            sha = digest(path)
            blob = 'sha256/' + sha
            if blob not in blobs:
                data = path.read_bytes()
                if hashlib.sha256(data).hexdigest() != sha:
                    raise ValueError(f'Source changed during archive: {path}')
                z.writestr(blob, data); blobs.add(blob)
            manifest.append({'path': str(rel), 'bytes': info.st_size, 'sha256': sha, 'blob': blob})
        z.writestr('manifest.json', json.dumps(manifest, ensure_ascii=False, indent=2))
    with zipfile.ZipFile(destination) as z:
        for blob in blobs:
            if hashlib.sha256(z.read(blob)).hexdigest() != blob.split('/')[1]:
                raise ValueError(f'Archive hash mismatch: {blob}')
    receipt = {'path': str(destination), 'sha256': digest(destination), 'bytes': destination.stat().st_size,
               'paths_preserved': len(manifest), 'unique_blobs': len(blobs), 'verified': True}
    save('archive-receipt.json', receipt)
    print(json.dumps(receipt, ensure_ascii=False))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=('inventory','archive'))
    args = parser.parse_args()
    {'inventory': inventory, 'archive': archive}[args.action]()
