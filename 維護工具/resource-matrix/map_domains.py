#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""地圖域、區域表與地圖頁盤點（G07.4，#337／T057）。

唯讀：只讀客戶端資源，輸出寫到本專案。內容只取決於客戶端資源與本工具規則，
沒有時間戳，同一份資源重跑會得到相同位元組。

  python -X utf8 維護工具/resource-matrix/map_domains.py              重產
  python -X utf8 維護工具/resource-matrix/map_domains.py --check      重算後與入庫輸出比對
  python -X utf8 維護工具/resource-matrix/map_domains.py --self-test  不讀客戶端的純邏輯自測
  python -X utf8 維護工具/resource-matrix/map_domains.py --pe <脫殼 PE>  另外核對地圖頁圖片表（只印結果，不寫檔）
"""
import argparse
import hashlib
import json
import os
import re
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import resource_matrix as rm  # noqa: E402

TOOL_VERSION = 1
REPO = rm.REPO
OUT_JSON = REPO / 'docs' / 'resource-matrix' / 'map-domains.json'
OUT_MD = REPO / 'docs' / 'MAP-DOMAINS.md'

# 伺服器 repo（唯讀參考）記錄「內容更新過」的 11 張經典地圖：
# lito-launcher-server origin/main 291474f2
# lito伺服器/LiTo381-Server/docs/maintenance/native-de-terrain-current.zh-TW.md 第 11 段。
CLASSIC_REFRESHED = [77, 78, 79, 80, 81, 82, 104, 105, 108, 109, 110]
# 同一份文件：504 在 CLASSIC 與 ARENA 兩個域的物理不同，呼叫端必須明確選域。
DUAL_DOMAIN_REFERENCE = 504
# Rust 客戶端唯一走經典域的地圖（native_world_scene.rs load_terrain：技能 131 的保留目的地）。
CLIENT_CLASSIC_MAPS = [4]
# 原版地圖頁圖片表：脫殼 PE 4271D92F… VA 0x0062F7BC 兩個 dword，後面緊接字串 "%d.alp"。
PAGE_TABLE_VA = 0x0062F7BC
PAGE_IMAGES = [6700, 6701]
PAGE_COUNT_WIRE = 18
PE_SHA256 = '4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
# native_mode_menu_skin.rs decode_img 的上限。
IMG_MAX_SIDE = 2048
IMG_MAX_PIXELS = 1_048_576
ZONE_TABLES = ['LTzone-c.tbl', 'LTzone.tbl', 'zone2-c.tbl']


def sha(data):
    return hashlib.sha256(data).hexdigest().upper()


def parse_zone_table(data):
    """LTzone 文字表。回傳 (dict, None) 或 (None, 原因碼)。

    第 1 列：總列數（原始註解寫「不含本列」；LTzone 兩張表符合，zone2-c.tbl 不符，
    所以這裡只回報規則 count_rule，不因列數不符拒絕）。
    第 2 列：色彩列數 N。接著 N 列 `r g b`，其餘非空列是 `"名稱" 色彩索引 地圖 x1 y1 x2 y2`。
    `;` 之後是註解。名稱保留原始位元組（各語系編碼不同），不在這裡解碼。
    """
    rows = []
    for raw in data.split(b'\n'):
        line = raw.rstrip(b'\r')
        quote = False
        cut = len(line)
        for i, b in enumerate(line):
            if b == 0x22:
                quote = not quote
            elif b == 0x3B and not quote:
                cut = i
                break
        line = line[:cut].strip()
        if line:
            rows.append(line)
    if len(rows) < 2:
        return None, 'zone.header_missing'
    try:
        declared = int(rows[0])
        colors_declared = int(rows[1])
    except ValueError:
        return None, 'zone.header_not_numeric'
    if not 0 <= colors_declared <= 64 or len(rows) < 2 + colors_declared:
        return None, 'zone.color_count_invalid'
    colors = []
    for line in rows[2:2 + colors_declared]:
        parts = line.split()
        if len(parts) != 3 or not all(p.isdigit() and int(p) <= 255 for p in parts):
            return None, 'zone.color_row_invalid'
        colors.append([int(p) for p in parts])
    zones = []
    for line in rows[2 + colors_declared:]:
        m = re.fullmatch(rb'"([^"]*)"\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)', line)
        if not m:
            return None, 'zone.zone_row_invalid'
        color, map_id, x1, y1, x2, y2 = (int(v) for v in m.groups()[1:])
        if color >= len(colors):
            return None, 'zone.color_index_out_of_range'
        zones.append({'name_hex': m.group(1).hex().upper(), 'name_ascii': m.group(1).decode('ascii', 'replace')
                      if all(32 <= b < 127 for b in m.group(1)) else None,
                      'color': color, 'map_id': map_id, 'bounds': [x1, y1, x2, y2],
                      'whole_map': (x1, y1, x2, y2) == (0, 0, 0, 0)})
    total = len(rows)
    count_rule = 'includes_header_row' if declared == total else \
        'excludes_header_row' if declared == total - 1 else 'mismatch'
    return {'declared_rows': declared, 'data_rows': total, 'count_rule': count_rule,
            'colors': colors, 'zones': zones}, None


def img_header(head, size):
    """IMG 表頭與 decode_img 上限的關係。"""
    if len(head) < 8:
        return {'verdict': 'img.truncated'}
    w, h, flag, _ = struct.unpack('<hhHH', head[:8])
    out = {'width': w, 'height': h, 'flag': flag, 'bytes': size}
    if w <= 0 or h <= 0:
        out['verdict'] = 'img.non_positive_dimensions'
    elif size != 8 + w * h * 2:
        out['verdict'] = 'img.payload_size_mismatch'
    elif w > IMG_MAX_SIDE or h > IMG_MAX_SIDE or w * h > IMG_MAX_PIXELS:
        out['verdict'] = 'img.exceeds_decode_budget'
    else:
        out['verdict'] = 'img.within_decode_budget'
    return out


def sector_files(directory):
    out = {}
    if not directory.is_dir():
        return None
    for entry in sorted(os.scandir(directory), key=lambda e: e.name.lower()):
        if entry.is_file():
            out[entry.name.lower()] = entry
    return out


def digest_dir(files):
    """目錄摘要：依檔名排序的 (檔名, 大小, SHA-256)。"""
    h = hashlib.sha256()
    total = 0
    hashes = {}
    for name in sorted(files):
        with open(files[name].path, 'rb') as f:
            data = f.read()
        hashes[name] = sha(data)
        total += len(data)
        h.update(('%s\t%d\t%s\n' % (name, len(data), hashes[name])).encode('ascii', 'replace'))
    return h.hexdigest().upper(), total, hashes


def scan(root):
    lt = root / 'LiTo'
    arena_root, classic_root = lt / 'LTmap', root / 'map'
    arena_ids = sorted(int(e.name) for e in os.scandir(arena_root) if e.is_dir() and e.name.isdigit())
    classic_ids = sorted(int(e.name) for e in os.scandir(classic_root) if e.is_dir() and e.name.isdigit())
    classic_set = set(classic_ids)

    def describe(directory):
        files = sector_files(directory)
        if files is None:
            return None, None
        s32 = {n: e for n, e in files.items() if n.endswith('.s32')}
        dir_sha, total, hashes = digest_dir(s32)
        return {'s32': len(s32), 'other_files': len(files) - len(s32), 's32_bytes': total,
                's32_set_sha256': dir_sha}, hashes

    arena = []
    for map_id in arena_ids:
        a, a_hashes = describe(arena_root / str(map_id))
        row = {'map_id': map_id, 'arena': a, 'classic': None}
        if map_id in classic_set:
            c, c_hashes = describe(classic_root / str(map_id))
            common = sorted(set(a_hashes) & set(c_hashes))
            row['classic'] = c
            row['same_name_sectors'] = len(common)
            row['same_name_identical'] = sum(a_hashes[n] == c_hashes[n] for n in common)
            row['arena_only_sectors'] = len(set(a_hashes) - set(c_hashes))
            row['classic_only_sectors'] = len(set(c_hashes) - set(a_hashes))
        arena.append(row)

    refreshed = []
    for map_id in CLASSIC_REFRESHED:
        c, _ = describe(classic_root / str(map_id))
        refreshed.append({'map_id': map_id, 'classic': c, 'in_arena_root': (arena_root / str(map_id)).is_dir()})

    idx = (lt / 'LTtext.idx').read_bytes()
    pak = (lt / 'LTtext.pak').read_bytes()
    members = {}
    for offset, name, size in rm.parse_index(idx, True):
        members.setdefault(rm.safe_name(name).lower(), []).append((offset, size))
    zones = []
    arena_set = set(arena_ids)
    for table in ZONE_TABLES:
        hits = members.get(table.lower(), [])
        row = {'member': table, 'archive': 'LiTo/LTtext', 'records': len(hits)}
        if len(hits) == 1:
            offset, size = hits[0]
            data = rm.l1_decode(pak[offset:offset + size])
            parsed, error = parse_zone_table(data)
            row.update({'bytes': size, 'decoded_sha256': sha(data), 'verdict': error or 'zone.ok'})
            if parsed:
                ids = [z['map_id'] for z in parsed['zones']]
                row.update({'declared_rows': parsed['declared_rows'], 'data_rows': parsed['data_rows'],
                            'count_rule': parsed['count_rule'], 'colors': parsed['colors'],
                            'zone_count': len(ids), 'whole_map_zones': sum(z['whole_map'] for z in parsed['zones']),
                            'zones_in_arena_root': sorted(set(ids) & arena_set),
                            'arena_maps_without_zone': sorted(arena_set - set(ids)),
                            'refreshed_classic_with_zone': sorted(set(ids) & set(CLASSIC_REFRESHED)),
                            'has_zone_for_504': DUAL_DOMAIN_REFERENCE in ids})
                if len(parsed['zones']) <= 8:
                    row['zones'] = parsed['zones']
        else:
            row['verdict'] = 'zone.member_missing' if not hits else 'zone.member_ambiguous'
        zones.append(row)

    surf_idx = (lt / 'LTsurf.idx').read_bytes()
    surf = {}
    for offset, name, size in rm.parse_index(surf_idx, False):
        surf.setdefault(rm.safe_name(name).lower(), []).append((offset, size))
    pages = []
    with open(lt / 'LTsurf.pak', 'rb') as f:
        for i, image in enumerate(PAGE_IMAGES):
            name = '%d.img' % image
            hits = surf.get(name, [])
            row = {'wire_page': i + 1, 'image': image, 'archive': 'LiTo/LTsurf', 'member': name, 'records': len(hits)}
            if len(hits) == 1:
                offset, size = hits[0]
                f.seek(offset)
                data = f.read(size)
                row.update(img_header(data[:8], len(data)))
                row['sha256'] = sha(data)
            else:
                row['verdict'] = 'img.member_missing' if not hits else 'img.member_ambiguous'
            pages.append(row)

    return {
        'tool': 'map_domains.py', 'tool_version': TOOL_VERSION,
        'sources': {
            'LiTo/LTtext.idx': sha(idx), 'LiTo/LTtext.pak': sha(pak), 'LiTo/LTsurf.idx': sha(surf_idx),
            'LiTo/LTmap directories': len(arena_ids), 'map directories': len(classic_ids),
        },
        'client_policy': {'classic_domain_maps': CLIENT_CLASSIC_MAPS,
                          'arena_domain': 'every other map id: LiTo/LTmap + LTtile, no fallback to map/'},
        'arena_maps': arena,
        'dual_domain_ids': [r['map_id'] for r in arena if r['classic']],
        'classic_refreshed': refreshed,
        'zone_tables': zones,
        'map_pages': {'pe_sha256': PE_SHA256, 'table_va': '0x%08X' % PAGE_TABLE_VA, 'table_entries': PAGE_IMAGES,
                      'wire_pages': PAGE_COUNT_WIRE, 'pages_without_image': list(range(len(PAGE_IMAGES) + 1,
                                                                                         PAGE_COUNT_WIRE + 1)),
                      'decode_budget': {'max_side': IMG_MAX_SIDE, 'max_pixels': IMG_MAX_PIXELS}, 'pages': pages},
    }


def render_markdown(doc):
    n = lambda v: format(v, ',')
    out = ['# MAP-DOMAINS — 地圖域、區域表與地圖頁（G07.4）', '',
           '本檔由 `維護工具/resource-matrix/map_domains.py` 產生（工具版本 %d，#337／T057），**不要手改**；'
           '要更新就重跑工具。內容只取決於客戶端資源與本工具的規則，沒有時間戳。' % doc['tool_version'], '',
           '機器可讀版：`docs/resource-matrix/map-domains.json`。客戶端怎麼用這些事實，見 '
           '`LiTo381-Modern/src/modern_lito/resource_diagnostics.rs`、`native_zone.rs`、`native_map_item.rs`。', '',
           '## 1. 來源', '', '| 來源 | SHA-256 或數量 |', '|---|---|']
    for key, value in doc['sources'].items():
        out.append('| `%s` | %s |' % (key, ('`%s`' % value) if isinstance(value, str) else n(value)))
    out += ['', '## 2. 兩個地圖域', '',
            '- **競技場域**：`LiTo/LTmap/<地圖>` 的 S32 ＋ `LTtile`。Rust 客戶端對地圖 %s 以外的所有地圖都只讀這個域，'
            '找不到就失敗，不會改讀 `map/`。' % '、'.join(str(v) for v in doc['client_policy']['classic_domain_maps']),
            '- **經典域**：共用 `map/<地圖>` 的 S32 ＋ `Tile`。客戶端只有地圖 %s 走這個域（寫死的契約，不是缺檔後備）。'
            % '、'.join(str(v) for v in doc['client_policy']['classic_domain_maps']),
            '- 兩個根目錄都有的地圖編號（**雙域**）：%s。同一個編號在兩個域是不同的資料，不能互相代替。'
            % ('、'.join(str(v) for v in doc['dual_domain_ids']) or '無'), '',
            '### 2.1 雙域地圖逐張比較', '',
            '| 地圖 | 競技場域 S32 | 競技場域 bytes | 經典域 S32 | 經典域 bytes | 同名區塊 | 同名且內容相同 | 只在競技場域 | 只在經典域 |',
            '|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    for row in doc['arena_maps']:
        if row['classic']:
            out.append('| %d | %d | %s | %d | %s | %d | %d | %d | %d |' % (
                row['map_id'], row['arena']['s32'], n(row['arena']['s32_bytes']), row['classic']['s32'],
                n(row['classic']['s32_bytes']), row['same_name_sectors'], row['same_name_identical'],
                row['arena_only_sectors'], row['classic_only_sectors']))
    out += ['', '### 2.2 競技場域全部地圖', '', '| 地圖 | S32 | bytes | S32 集合 SHA-256 | 經典域也有 |', '|---:|---:|---:|---|---|']
    for row in doc['arena_maps']:
        out.append('| %d | %d | %s | `%s` | %s |' % (row['map_id'], row['arena']['s32'], n(row['arena']['s32_bytes']),
                                                     row['arena']['s32_set_sha256'], '是' if row['classic'] else '否'))
    out += ['', '## 3. 伺服器更新過內容的 11 張經典地圖', '',
            '伺服器 repo 記錄 77–82、104、105、108–110 這 11 張經典地圖的通行資料更新過'
            '（`native-de-terrain-current.zh-TW.md` 第 11 段）。下表是這些編號在**客戶端資源**裡的狀態；'
            '伺服器的通行資料不在客戶端，本工具沒有比對。', '',
            '| 地圖 | 經典域 S32 | 其他檔 | bytes | S32 集合 SHA-256 | 競技場域有沒有 |', '|---:|---:|---:|---:|---|---|']
    for row in doc['classic_refreshed']:
        c = row['classic']
        if c:
            out.append('| %d | %d | %d | %s | `%s` | %s |' % (row['map_id'], c['s32'], c['other_files'], n(c['s32_bytes']),
                                                              c['s32_set_sha256'], '有' if row['in_arena_root'] else '沒有'))
        else:
            out.append('| %d | 0 | 0 | 0 | — | %s |' % (row['map_id'], '有' if row['in_arena_root'] else '沒有'))
    out += ['', '## 4. 區域表（LTzone）', '',
            '格式：第 1 列總列數、第 2 列色彩列數、色彩列 `r g b`、區域列 `"名稱" 色彩索引 地圖 x1 y1 x2 y2`'
            '（四個 0 代表整張地圖）。表裡只有名稱與顏色；**沒有任何欄位表示安全區或禁戰區，不能由顏色推導**。', '',
            '| 成員 | 判定 | 解碼後 SHA-256 | 色彩 | 區域 | 整張地圖 | 列數規則 | 有區域的競技場域地圖 |', '|---|---|---|---:|---:|---:|---|---|']
    for row in doc['zone_tables']:
        if 'colors' in row:
            out.append('| `%s` | `%s` | `%s` | %d | %d | %d | `%s` | %s |' % (
                row['member'], row['verdict'], row['decoded_sha256'], len(row['colors']), row['zone_count'],
                row['whole_map_zones'], row['count_rule'], '、'.join(str(v) for v in row['zones_in_arena_root']) or '無'))
        else:
            out.append('| `%s` | `%s` | %s | — | — | — | — | — |' % (
                row['member'], row['verdict'], ('`%s`' % row['decoded_sha256']) if 'decoded_sha256' in row else '—'))
    for row in doc['zone_tables']:
        if row.get('zones'):
            out += ['', '`%s` 的內容：' % row['member'], '', '| 名稱（原始位元組） | ASCII | 色彩索引 | RGB | 地圖 | 範圍 |', '|---|---|---:|---|---:|---|']
            for z in row['zones']:
                out.append('| `%s` | %s | %d | %s | %d | %s |' % (
                    z['name_hex'], z['name_ascii'] or '—', z['color'], ' '.join(str(v) for v in row['colors'][z['color']]),
                    z['map_id'], '整張地圖' if z['whole_map'] else ' '.join(str(v) for v in z['bounds'])))
    first = doc['zone_tables'][0]
    if 'arena_maps_without_zone' in first:
        out += ['', '`%s` 沒有涵蓋的競技場域地圖（%d 張，進這些地圖時沒有區域名稱可顯示）：%s。' % (
            first['member'], len(first['arena_maps_without_zone']),
            '、'.join(str(v) for v in first['arena_maps_without_zone']) or '無')]
    pages = doc['map_pages']
    out += ['', '## 5. 地圖頁（S_SHOW_MAP）', '',
            '- 伺服器頁碼是 1–%d；原版以「頁碼 − 1」當索引讀圖片表。' % pages['wire_pages'],
            '- 圖片表在脫殼 PE `%s…` 的 VA `%s`，只有 %d 個項目：%s。表的後面緊接字串 `%%d.alp`，所以沒有第 3 項。'
            % (pages['pe_sha256'][:8], pages['table_va'], len(pages['table_entries']),
               '、'.join(str(v) for v in pages['table_entries'])),
            '- 第 %d–%d 頁在原版沒有對應圖片。' % (pages['pages_without_image'][0], pages['pages_without_image'][-1]),
            '- Rust `decode_img` 的上限是單邊 %s、總像素 %s。' % (n(pages['decode_budget']['max_side']),
                                                         n(pages['decode_budget']['max_pixels'])), '',
            '| 頁碼 | 圖片 | 封存 | 寬 | 高 | bytes | SHA-256 | 判定 |', '|---:|---|---|---:|---:|---:|---|---|']
    for row in pages['pages']:
        if 'width' in row:
            out.append('| %d | `%s` | `%s` | %d | %d | %s | `%s` | `%s` |' % (
                row['wire_page'], row['member'], row['archive'], row['width'], row['height'], n(row['bytes']),
                row['sha256'], row['verdict']))
        else:
            out.append('| %d | `%s` | `%s` | — | — | — | — | `%s` |' % (row['wire_page'], row['member'], row['archive'],
                                                                    row['verdict']))
    return '\n'.join(out) + '\n'


def outputs(root):
    doc = scan(root)
    return {OUT_JSON: json.dumps(doc, ensure_ascii=False, indent=1) + '\n', OUT_MD: render_markdown(doc)}


def check_pe(path):
    raw = Path(path).read_bytes()
    digest = sha(raw)
    pe = struct.unpack_from('<I', raw, 60)[0]
    count = struct.unpack_from('<H', raw, pe + 6)[0]
    optional = pe + 24
    base = struct.unpack_from('<I', raw, optional + 28)[0]
    sections = optional + struct.unpack_from('<H', raw, pe + 20)[0]
    rva = PAGE_TABLE_VA - base
    for i in range(count):
        vsize, vaddr, rsize, rptr = struct.unpack_from('<IIII', raw, sections + i * 40 + 8)
        if vaddr <= rva < vaddr + max(vsize, rsize):
            offset = rptr + rva - vaddr
            entries = list(struct.unpack_from('<II', raw, offset))
            tail = raw[offset + 8:offset + 15]
            ok = digest == PE_SHA256 and entries == PAGE_IMAGES and tail == b'%d.alp\0'
            print('pe sha256 %s\ntable file offset 0x%X entries %s next %r\n%s' % (
                digest, offset, entries, tail, 'pe check ok' if ok else 'pe check FAILED'))
            return 0 if ok else 1
    print('pe check FAILED: VA not mapped')
    return 1


def self_test():
    table = (b'5 ;header\r\n2\r\n45 95 209 ; blue\r\n10 173 90 ; green \r\n'
             b'"LODUN" 0 500 0 0 0 0 \r\n"LENOA" 1 560 0 0 0 0\r\n\r\n')
    parsed, error = parse_zone_table(table)
    assert error is None and parsed['count_rule'] == 'excludes_header_row'
    assert parsed['colors'] == [[45, 95, 209], [10, 173, 90]]
    assert [(z['name_ascii'], z['color'], z['map_id'], z['whole_map']) for z in parsed['zones']] == \
        [('LODUN', 0, 500, True), ('LENOA', 1, 560, True)]
    assert parse_zone_table(table.replace(b'5 ;header', b'6'))[0]['count_rule'] == 'includes_header_row'
    assert parse_zone_table(table.replace(b'5 ;header', b'9'))[0]['count_rule'] == 'mismatch'
    assert parse_zone_table(b'1\r\n1\r\n1 2 3\r\n"A;B" 0 7 1 2 3 4 ; note\r\n')[0]['zones'][0] == {
        'name_hex': '413B42', 'name_ascii': 'A;B', 'color': 0, 'map_id': 7, 'bounds': [1, 2, 3, 4], 'whole_map': False}
    assert parse_zone_table(b'1\r\n1\r\n1 2 3\r\n"\xb7\xce" 0 7 0 0 0 0\r\n')[0]['zones'][0]['name_ascii'] is None
    assert parse_zone_table(b'')[1] == 'zone.header_missing'
    assert parse_zone_table(b'x\r\n1\r\n')[1] == 'zone.header_not_numeric'
    assert parse_zone_table(b'1\r\n2\r\n1 2 3\r\n')[1] == 'zone.color_count_invalid'
    assert parse_zone_table(b'1\r\n1\r\n1 2 256\r\n')[1] == 'zone.color_row_invalid'
    assert parse_zone_table(b'1\r\n1\r\n1 2 3\r\n"A" 1 7 0 0 0 0\r\n')[1] == 'zone.color_index_out_of_range'
    assert parse_zone_table(b'1\r\n1\r\n1 2 3\r\n"A" 0 7 0 0 0\r\n')[1] == 'zone.zone_row_invalid'
    assert img_header(struct.pack('<hhHH', 1536, 768, 0, 0), 8 + 1536 * 768 * 2)['verdict'] == 'img.exceeds_decode_budget'
    assert img_header(struct.pack('<hhHH', 1024, 1024, 0, 0), 8 + 1024 * 1024 * 2)['verdict'] == 'img.within_decode_budget'
    assert img_header(struct.pack('<hhHH', 2, 1, 0, 0), 11)['verdict'] == 'img.payload_size_mismatch'
    assert img_header(struct.pack('<hhHH', -2, 1, 0, 0), 8)['verdict'] == 'img.non_positive_dimensions'
    assert img_header(b'\0', 1)['verdict'] == 'img.truncated'
    # 本工具照抄的常數要還在 Rust 原始碼裡；Rust 端改了，這裡要跟著改。
    modules = rm.SRC / 'modern_lito'
    skin = (modules / 'native_mode_menu_skin.rs').read_text(encoding='utf-8')
    assert 'w<=2048&&h<=2048&&u64::from(w)*u64::from(h)<=1_048_576' in skin, 'decode_img 上限已變動'
    item = (modules / 'native_map_item.rs').read_text(encoding='utf-8')
    assert 'pub const NATIVE_PAGE_IMAGES: [u16; 2] = [6700, 6701];' in item, 'native_map_item.rs 的地圖頁圖片表已變動'
    diagnostics = (modules / 'resource_diagnostics.rs').read_text(encoding='utf-8')
    assert 'pub const CLASSIC_REFRESHED_MAPS: [u16; 11] = [77, 78, 79, 80, 81, 82, 104, 105, 108, 109, 110];' \
        in diagnostics, 'resource_diagnostics.rs 的經典地圖清單已變動'
    assert 'pub const CLASSIC_DOMAIN_MAPS: [u16; 1] = [4];' in diagnostics, 'resource_diagnostics.rs 的經典域清單已變動'
    scene = (modules / 'native_world_scene.rs').read_text(encoding='utf-8')
    assert 'if world.map_id!=Some(4) {' in scene, 'native_world_scene.rs 的選域規則已變動'
    print('self-test ok')


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--pe')
    args = parser.parse_args()
    assert sys.byteorder == 'little'
    if args.self_test:
        self_test()
        return 0
    if args.pe:
        return check_pe(args.pe)
    root = Path(os.environ.get('LITO_CLIENT_ROOT') or rm.DEFAULT_ROOT)
    if not (root / 'LiTo' / 'LTtext.idx').is_file():
        raise SystemExit('找不到客戶端資源：%s（可用 LITO_CLIENT_ROOT 指定）' % root)
    produced = outputs(root)
    if args.check:
        drift = [str(p.relative_to(REPO)) for p in produced
                 if not p.is_file() or p.read_text(encoding='utf-8') != produced[p]]
        print('drift: ' + ', '.join(drift) if drift else 'check ok: 入庫輸出與目前資源一致')
        return 1 if drift else 0
    for path, text in produced.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, 'w', encoding='utf-8', newline='\n') as f:
            f.write(text)
        print('wrote %s (%d bytes)' % (path.relative_to(REPO), len(text.encode('utf-8'))))
    return 0


if __name__ == '__main__':
    sys.exit(main())
