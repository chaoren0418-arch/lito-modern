#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""G12 稽核（T037 第 2 張，Issue #325）：核對 docs/G01-G12.md 引用的收據。

唯讀工具。只讀 <專案根> 內的檔案，輸出寫到 <輸出目錄>；不刪、不改任何檔。

用法（在 repo 根目錄；-X utf8 讓中文路徑與輸出正常）：

  python -I -X utf8 維護工具/g12-audit-evidence-check.py receipts  docs/G01-G12.md <專案根> <輸出目錄>
  python -I -X utf8 維護工具/g12-audit-evidence-check.py cited-exe <專案根> <輸出目錄>

receipts：
  逐項（46 項）取出原文引用的本專案路徑（驗證紀錄／LiTo381-Modern／維護工具／交付／封存），
  檢查是否仍在；原文同時給了 sha256 的，重算比對。
  輸出 evidence-summary.tsv（每項一列）與 evidence-detail.tsv（每個路徑一列）。

cited-exe（要先跑 receipts；會讀完 target 下所有 .exe，約十幾 GiB）：
  從仍存在的收據 JSON 取出全部 64 位十六進位值，再對 LiTo381-Modern/target 下的 .exe 逐檔算
  SHA256，列出雜湊被收據引用的 exe。用來判斷 #324 的 B／Bexe 類哪些不能當冗餘刪除。
  輸出 cited-exe.tsv。
"""
import hashlib
import io
import os
import re
import sys

PREFIX = 'D:/lineage開發/3-lito開發/lito重建版/'
PATH_RE = re.compile(
    r'"((?:' + re.escape(PREFIX) + r')?(?:驗證紀錄|LiTo381-Modern|維護工具|交付|封存)/[^"\n]*?)"')
PAIR_RE = re.compile(r'"(\w+)":\s*"([^"\n]+)",\s*"(\w*sha256)":\s*"([0-9A-Fa-f]{64})"')
HEX_RE = re.compile(r'\b[0-9A-Fa-f]{64}\b')


def rel(path):
    return path[len(PREFIX):] if path.startswith(PREFIX) else path


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, 'rb') as handle:
        for chunk in iter(lambda: handle.read(1 << 22), b''):
            digest.update(chunk)
    return digest.hexdigest().upper()


def receipts(md_path, root, out_dir):
    text = io.open(md_path, encoding='utf-8').read()
    sections = re.split(r'(?m)^## ', text)[3:]  # 前三段是標題、來源說明、索引
    os.makedirs(out_dir, exist_ok=True)
    summary = io.open(os.path.join(out_dir, 'evidence-summary.tsv'), 'w', encoding='utf-8', newline='\n')
    detail = io.open(os.path.join(out_dir, 'evidence-detail.tsv'), 'w', encoding='utf-8', newline='\n')
    summary.write('item\tpaths\texist\tmissing\tsha_pairs\tsha_ok\tsha_bad\tsha_not_a_file\n')
    detail.write('status\titem\tpath\texpected_sha256\tactual_sha256\n')
    total = [0] * 7
    for section in sections:
        head, _, body = section.partition('\n')
        item = head.split(' ')[0]
        body = body.replace('\\\\', '/').replace('\\', '/')
        paths = {rel(m.group(1)) for m in PATH_RE.finditer(body)}
        exist = {p for p in paths if os.path.exists(os.path.join(root, p))}
        pairs = {(rel(m.group(2)), m.group(4).upper()) for m in PAIR_RE.finditer(body)}
        pairs = {pair for pair in pairs if pair[0] in paths}
        ok = bad = skipped = 0
        hashed = {}
        for path, expected in sorted(pairs):
            full = os.path.join(root, path)
            if not os.path.isfile(full):
                skipped += 1
                continue
            actual = sha256_file(full)
            hashed[path] = (expected, actual)
            if actual == expected:
                ok += 1
            else:
                bad += 1
        for path in sorted(paths):
            if path in hashed:
                expected, actual = hashed[path]
                status = 'SHA_OK' if expected == actual else 'SHA_MISMATCH'
                detail.write('%s\t%s\t%s\t%s\t%s\n' % (status, item, path, expected, actual))
            else:
                status = 'EXISTS' if path in exist else 'MISSING'
                detail.write('%s\t%s\t%s\t\t\n' % (status, item, path))
        row = (len(paths), len(exist), len(paths) - len(exist), len(pairs), ok, bad, skipped)
        summary.write('%s\t%s\n' % (item, '\t'.join(str(v) for v in row)))
        total = [a + b for a, b in zip(total, row)]
    summary.write('TOTAL\t%s\n' % '\t'.join(str(v) for v in total))
    summary.close()
    detail.close()
    print('items=%d paths=%d exist=%d missing=%d sha_pairs=%d ok=%d bad=%d not_a_file=%d'
          % ((len(sections),) + tuple(total)))


def cited_exe(root, out_dir):
    cited = {}
    receipt_files = 0
    for line in io.open(os.path.join(out_dir, 'evidence-detail.tsv'), encoding='utf-8'):
        fields = line.rstrip('\n').split('\t')
        if fields[0] not in ('EXISTS', 'SHA_OK', 'SHA_MISMATCH'):
            continue
        full = os.path.join(root, fields[2])
        if not (os.path.isfile(full) and full.lower().endswith('.json')):
            continue
        if os.path.getsize(full) > 64 * 1024 * 1024:
            continue
        receipt_files += 1
        content = open(full, 'rb').read().decode('utf-8', 'replace')
        for value in set(HEX_RE.findall(content)):
            cited.setdefault(value.upper(), set()).add(fields[1])
    target = os.path.join(root, 'LiTo381-Modern', 'target')
    scanned = size = 0
    hits = []
    for folder, _dirs, names in os.walk(target):
        for name in names:
            if not name.lower().endswith('.exe'):
                continue
            full = os.path.join(folder, name)
            try:
                digest = sha256_file(full)
                length = os.path.getsize(full)
            except OSError:
                continue
            scanned += 1
            size += length
            if digest in cited:
                hits.append((os.path.relpath(full, root), length, digest, ','.join(sorted(cited[digest]))))
    with io.open(os.path.join(out_dir, 'cited-exe.tsv'), 'w', encoding='utf-8', newline='\n') as out:
        out.write('# receipts_json=%d cited_hashes=%d exe_scanned=%d exe_bytes=%d exe_cited=%d distinct_cited_exe_hashes=%d\n'
                  % (receipt_files, len(cited), scanned, size, len(hits), len({h[2] for h in hits})))
        out.write('path\tbytes\tsha256\tcited_by_items\n')
        for hit in sorted(hits):
            out.write('%s\t%d\t%s\t%s\n' % hit)
    print('receipts_json=%d cited_hashes=%d exe_scanned=%d exe_cited=%d'
          % (receipt_files, len(cited), scanned, len(hits)))


def main(argv):
    if len(argv) == 5 and argv[1] == 'receipts':
        receipts(argv[2], argv[3], argv[4])
    elif len(argv) == 4 and argv[1] == 'cited-exe':
        cited_exe(argv[2], argv[3])
    else:
        sys.stderr.write(__doc__)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
