# -*- coding: utf-8 -*-
"""G12 稽核（T037 第 1 張，Issue #324）：專案產物盤點與冗餘產物清單。

  python -I g12-artifact-inventory.py scan  <ROOT> <OUT>            唯讀：列出 ROOT 全部檔案並分類
  python -I g12-artifact-inventory.py clean <ROOT> <OUT>            預演：依 OUT/tierA-files.tsv 檢查，不刪
  python -I g12-artifact-inventory.py clean <ROOT> <OUT> --apply    實際刪除（不可逆，需使用者同意後才可執行）

ROOT 是專案目錄（含 LiTo381-Modern/）；OUT 必須在 ROOT 的 scratch/ 之下或 ROOT 之外。
scan 不進入 .git 與 scratch/wt（worktree 由調度器管理），也不跟隨 junction／symlink。

分類（只有 LiTo381-Modern/target 之下才可能是 A）：
  A  cargo 可重建的中間產物：路徑含 <debug|release>/<deps|incremental|.fingerprint>/，
     deps 內只認 .rlib .rmeta .pdb .o .d .lib .exp .ilk；而且硬連結數為 1。
  AH 條件同 A，但硬連結數大於 1（不動）。
  X  cargo 版面內的其他檔（exe、dll、build script 輸出等，不動）。
  E  target 內、不在 cargo 版面的檔（各輪留存的 rlib／exe、來源快照、證據，不動）。
  K  target 以外（不動）。
clean 只處理 A，逐檔重新檢查：路徑在 ROOT/LiTo381-Modern/target 內、不是 reparse point、
硬連結數 1、大小與清單一致、重新分類仍是 A。任何一項不符就略過並記錄。只刪檔案，不刪目錄。
"""
import os, sys, stat, time

S = os.sep
T = 'LiTo381-Modern' + S + 'target' + S
PROFILES = ('debug', 'release')
WHOLE_DIRS = ('incremental', '.fingerprint')
DEPS_EXT = {'.rlib', '.rmeta', '.pdb', '.o', '.d', '.lib', '.exp', '.ilk'}


def classify(rel, nlink):
    """回傳 (類別, cargo profile 目錄或 None)。rel 用 os.sep。"""
    if not rel.startswith(T):
        return 'K', None
    parts = rel[len(T):].split(S)
    for i in range(1, len(parts) - 1):
        if parts[i - 1] in PROFILES and parts[i] in WHOLE_DIRS + ('deps', 'build'):
            ext = os.path.splitext(parts[-1])[1].lower()
            ok = parts[i] in WHOLE_DIRS or (parts[i] == 'deps' and ext in DEPS_EXT)
            profile = T + S.join(parts[:i])
            if not ok:
                return 'X', profile
            return ('A' if nlink == 1 else 'AH'), profile
    return 'E', None


def is_reparse(st):
    return bool(getattr(st, 'st_file_attributes', 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT)


def scan(root, out):
    skip = {os.path.join(root, '.git').lower(), os.path.join(root, 'scratch', 'wt').lower()}
    rows, reparse, errors = [], [], []

    def walk(d):
        try:
            entries = list(os.scandir(d))
        except OSError as ex:
            errors.append('%s\t%s' % (d, ex))
            return
        for e in entries:
            try:
                st = os.lstat(e.path)
            except OSError as ex:
                errors.append('%s\t%s' % (e.path, ex))
                continue
            if is_reparse(st):
                reparse.append(os.path.relpath(e.path, root))
            elif stat.S_ISDIR(st.st_mode):
                if e.path.lower() not in skip:
                    walk(e.path)
            else:
                rows.append((os.path.relpath(e.path, root), st.st_size, st.st_nlink, int(st.st_mtime)))

    walk(root)
    os.makedirs(out, exist_ok=True)
    tot, roots = {}, {}
    with open(os.path.join(out, 'files.tsv'), 'w', encoding='utf-8', newline='\n') as f, \
            open(os.path.join(out, 'tierA-files.tsv'), 'w', encoding='utf-8', newline='\n') as a:
        for rel, size, nlink, mt in rows:
            c, profile = classify(rel, nlink)
            f.write('%s\t%d\t%d\t%d\t%s\n' % (rel, size, nlink, mt, c))
            t = tot.setdefault(c, [0, 0]); t[0] += size; t[1] += 1
            if profile:
                r = roots.setdefault(profile, {'A': [0, 0], 'AH': [0, 0], 'X': [0, 0]})[c]
                r[0] += size; r[1] += 1
            if c == 'A':
                a.write('%s\t%d\n' % (rel, size))
    with open(os.path.join(out, 'tierA-roots.tsv'), 'w', encoding='utf-8', newline='\n') as o:
        o.write('profile_dir\tA_bytes\tA_files\thardlinked_bytes\thardlinked_files\tkept_other_bytes\tkept_other_files\n')
        for p in sorted(roots):
            v = roots[p]
            o.write('%s\t%d\t%d\t%d\t%d\t%d\t%d\n' % (p.replace(S, '/'), v['A'][0], v['A'][1], v['AH'][0], v['AH'][1], v['X'][0], v['X'][1]))
    with open(os.path.join(out, 'summary.txt'), 'w', encoding='utf-8', newline='\n') as o:
        o.write('root\t%s\nscanned\t%s\nfiles\t%d\nbytes\t%d\n' % (root, time.strftime('%Y-%m-%d %H:%M:%S'), len(rows), sum(r[1] for r in rows)))
        for c in ('A', 'AH', 'X', 'E', 'K'):
            v = tot.get(c, [0, 0])
            o.write('%s\tbytes=%d\tfiles=%d\n' % (c, v[0], v[1]))
        o.write('reparse_points\t%d\nerrors\t%d\n' % (len(reparse), len(errors)))
        for r in reparse:
            o.write('reparse\t%s\n' % r)
        for e in errors:
            o.write('error\t%s\n' % e)
    print(open(os.path.join(out, 'summary.txt'), encoding='utf-8').read())


def clean(root, out, apply):
    base = os.path.join(os.path.abspath(root), T)
    done = [0, 0]
    skipped = []
    with open(os.path.join(out, 'tierA-files.tsv'), encoding='utf-8') as f:
        for line in f:
            rel, size = line.rstrip('\n').split('\t')
            p = os.path.abspath(os.path.join(root, rel))
            why = None
            try:
                st = os.lstat(p)
                if not p.startswith(base) or '..' in rel.split(S):
                    why = 'outside-target'
                elif is_reparse(st) or not stat.S_ISREG(st.st_mode):
                    why = 'not-regular-file'
                elif st.st_nlink != 1:
                    why = 'hardlinked'
                elif st.st_size != int(size):
                    why = 'size-changed'
                elif classify(rel, st.st_nlink)[0] != 'A':
                    why = 'not-tier-A'
                elif any(is_reparse(os.lstat(os.path.join(root, S.join(rel.split(S)[:n])))) for n in range(1, len(rel.split(S)))):
                    why = 'under-reparse-point'
            except OSError as ex:
                why = 'stat-failed:%s' % ex.__class__.__name__
            if why is None and apply:
                try:
                    os.remove(p)
                except OSError as ex:
                    why = 'remove-failed:%s' % ex.__class__.__name__
            if why:
                skipped.append('%s\t%s' % (why, rel))
            else:
                done[0] += int(size); done[1] += 1
    name = 'clean-applied.txt' if apply else 'clean-dryrun.txt'
    with open(os.path.join(out, name), 'w', encoding='utf-8', newline='\n') as o:
        o.write('%s\tfiles=%d\tbytes=%d\tskipped=%d\n' % ('deleted' if apply else 'would-delete', done[1], done[0], len(skipped)))
        o.write('\n'.join(skipped) + ('\n' if skipped else ''))
    print('%s files=%d bytes=%d skipped=%d' % ('deleted' if apply else 'would-delete', done[1], done[0], len(skipped)))


if __name__ == '__main__':
    if len(sys.argv) < 4 or sys.argv[1] not in ('scan', 'clean'):
        sys.exit(__doc__)
    if sys.argv[1] == 'scan':
        scan(os.path.abspath(sys.argv[2]), sys.argv[3])
    else:
        clean(sys.argv[2], sys.argv[3], '--apply' in sys.argv[4:])
