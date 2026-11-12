# -*- coding: utf-8 -*-
"""G07.1 資源命名空間與格式矩陣（#336／T057）。唯讀、可重跑、綁資源 SHA。

用法（在 repo 根目錄）：
  python -X utf8 維護工具/resource-matrix/resource_matrix.py            # 重產 docs/ 與 scratch/ 的輸出
  python -X utf8 維護工具/resource-matrix/resource_matrix.py --check    # 重算後與已入庫的 docs/ 輸出比對
  python -X utf8 維護工具/resource-matrix/resource_matrix.py --self-test # 純邏輯自測（不讀客戶端，CI 用）

客戶端根目錄：環境變數 LITO_CLIENT_ROOT，未設時為 D:\\lineage開發\\0-客戶端。
本工具只開檔讀取，不寫入、不改名、不刪除客戶端的任何檔案。

「逐成員覆蓋」的定義：客戶端每一個封存成員（idx 的每一筆記錄）與每一個散檔
都恰好落在矩陣的一列（命名空間 × 容器 × 格式 × 判定），沒有「其他／未分類」的餘數；
工具在結尾以總數核對（列加總 == 列舉總數），不相等就失敗。

判定（verdict）是把 Rust 解碼器的「容器層關卡」照抄成 Python 後逐成員跑出來的結果，
不是 Rust 解碼器本身的執行結果；SPR／TIL 的 RLE 像素本體沒有在這裡解。Rust 端逐成員
實跑由 LiTo381-Modern/tests/resource_matrix_exhaustive.rs（#[ignore]）負責。
"""
import argparse
import collections
import hashlib
import json
import os
import re
import struct
import sys
from array import array
from pathlib import Path

TOOL_VERSION = 1
REPO = Path(__file__).resolve().parents[2]
SRC = REPO / 'LiTo381-Modern' / 'src'
DEFAULT_ROOT = r'D:\lineage開發\0-客戶端'
OUT_MD = REPO / 'docs' / 'RESOURCE-MATRIX.md'
OUT_JSON = REPO / 'docs' / 'resource-matrix' / 'matrix.json'
OUT_EXC = REPO / 'docs' / 'resource-matrix' / 'exceptions.tsv'
OUT_MEMBERS = REPO / 'scratch' / 'resource-matrix' / 'members.tsv'

# --------------------------------------------------------------------------
# L1 保護索引／Text 內容的解碼（照 src/modern_lito/resource_cipher.rs 移植）
# --------------------------------------------------------------------------
_IB = [39, 7, 47, 15, 55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30, 37, 5, 45, 13, 53, 21, 61, 29,
       36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27, 34, 2, 42, 10, 50, 18, 58, 26,
       33, 1, 41, 9, 49, 17, 57, 25, 32, 0, 40, 8, 48, 16, 56, 24]
_FB = [57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3, 61, 53, 45, 37, 29, 21, 13, 5, 63,
       55, 47, 39, 31, 23, 15, 7, 56, 48, 40, 32, 24, 16, 8, 0, 58, 50, 42, 34, 26, 18, 10, 2, 60, 52,
       44, 36, 28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6]
_RB = [8, 16, 22, 30, 12, 27, 1, 17, 23, 15, 29, 5, 25, 19, 9, 0, 7, 13, 24, 2, 3, 28, 10, 18, 31, 11,
       21, 6, 4, 26, 14, 20]
_RK = bytes.fromhex(
    '509CA6168A2580AE2030409EC03E36C53087E4B640A622E942D662329B47E8D1'
    '521685B224C34B4D2D412351136AE0581E5249D28C6B0B51580E9F180C49CD19'
    '757013690969C820092DA1C06C1ED12C8DAD129C51A6809152E3F0282C796B04')
_SB = [
    [14, 0, 4, 15, 13, 7, 1, 4, 2, 14, 15, 2, 11, 13, 8, 1, 3, 10, 10, 6, 6, 12, 12, 11, 5, 9, 9, 5, 0, 3, 7, 8,
     4, 15, 1, 12, 14, 8, 8, 2, 13, 4, 6, 9, 2, 1, 11, 7, 15, 5, 12, 11, 9, 3, 7, 14, 3, 10, 10, 0, 5, 6, 0, 13],
    [15, 3, 1, 13, 8, 4, 14, 7, 6, 15, 11, 2, 3, 8, 4, 14, 9, 12, 7, 0, 2, 1, 13, 10, 12, 6, 0, 9, 5, 11, 10, 5,
     0, 13, 14, 8, 7, 10, 11, 1, 10, 3, 4, 15, 13, 4, 1, 2, 5, 11, 8, 6, 12, 7, 6, 12, 9, 0, 3, 5, 2, 14, 15, 9],
    [10, 13, 0, 7, 9, 0, 14, 9, 6, 3, 3, 4, 15, 6, 5, 10, 1, 2, 13, 8, 12, 5, 7, 14, 11, 12, 4, 11, 2, 15, 8, 1,
     13, 1, 6, 10, 4, 13, 9, 0, 8, 6, 15, 9, 3, 8, 0, 7, 11, 4, 1, 15, 2, 14, 12, 3, 5, 11, 10, 5, 14, 2, 7, 12],
    [7, 13, 13, 8, 14, 11, 3, 5, 0, 6, 6, 15, 9, 0, 10, 3, 1, 4, 2, 7, 8, 2, 5, 12, 11, 1, 12, 10, 4, 14, 15, 9,
     10, 3, 6, 15, 9, 0, 0, 6, 12, 10, 11, 1, 7, 13, 13, 8, 15, 9, 1, 4, 3, 5, 14, 11, 5, 12, 2, 7, 8, 2, 4, 14],
    [2, 14, 12, 11, 4, 2, 1, 12, 7, 4, 10, 7, 11, 13, 6, 1, 8, 5, 5, 0, 3, 15, 15, 10, 13, 3, 0, 9, 14, 8, 9, 6,
     4, 11, 2, 8, 1, 12, 11, 7, 10, 1, 13, 14, 7, 2, 8, 13, 15, 6, 9, 15, 12, 0, 5, 9, 6, 10, 3, 4, 0, 5, 14, 3],
    [12, 10, 1, 15, 10, 4, 15, 2, 9, 7, 2, 12, 6, 9, 8, 5, 0, 6, 13, 1, 3, 13, 4, 14, 14, 0, 7, 11, 5, 3, 11, 8,
     9, 4, 14, 3, 15, 2, 5, 12, 2, 9, 8, 5, 12, 15, 3, 10, 7, 11, 0, 14, 4, 1, 10, 7, 1, 6, 13, 0, 11, 8, 6, 13],
    [4, 13, 11, 0, 2, 11, 14, 7, 15, 4, 0, 9, 8, 1, 13, 10, 3, 14, 12, 3, 9, 5, 7, 12, 5, 2, 10, 15, 6, 8, 1, 6,
     1, 6, 4, 11, 11, 13, 13, 8, 12, 1, 3, 4, 7, 10, 14, 7, 10, 9, 15, 5, 6, 0, 8, 15, 0, 14, 5, 2, 9, 3, 2, 12],
    [13, 1, 2, 15, 8, 13, 4, 8, 6, 10, 15, 3, 11, 7, 1, 4, 10, 12, 9, 5, 3, 6, 14, 11, 5, 0, 0, 14, 12, 9, 7, 2,
     7, 2, 11, 1, 4, 14, 1, 7, 9, 4, 12, 10, 14, 8, 2, 13, 0, 15, 6, 12, 10, 9, 13, 0, 15, 3, 3, 5, 5, 6, 8, 11]]


def _nibble_table(dest):
    table = [[0] * 16 for _ in range(16)]
    for nibble in range(16):
        for value in range(16):
            for bit in range(4):
                if value & (1 << (3 - bit)):
                    table[nibble][value] |= 1 << (63 - dest[nibble * 4 + bit])
    return table


_IT, _FT = _nibble_table(_IB), _nibble_table(_FB)
_RT = [[0] * 256 for _ in range(4)]
for _by in range(4):
    for _v in range(256):
        for _b in range(8):
            if _v & (128 >> _b):
                _RT[_by][_v] |= 1 << (31 - _RB[_by * 8 + _b])
_PS = [[_RT[p][(_SB[p * 2][i >> 6] << 4) | _SB[p * 2 + 1][i & 63]] for i in range(4096)] for p in range(4)]
_PK = [[(_RK[r * 6] << 4) | (_RK[r * 6 + 1] >> 4), ((_RK[r * 6 + 1] & 15) << 8) | _RK[r * 6 + 2],
        (_RK[r * 6 + 3] << 4) | (_RK[r * 6 + 4] >> 4), ((_RK[r * 6 + 4] & 15) << 8) | _RK[r * 6 + 5]]
       for r in range(16)]


def _permute(x, table):
    r = 0
    for i in range(16):
        r |= table[i][(x >> (60 - 4 * i)) & 15]
    return r


def _round(x, k):
    a = x >> 24
    b = (x >> 16) & 255
    c = (x >> 8) & 255
    d = x & 255
    e0 = (d << 7 | ((a & 249 | a >> 2 & 6) >> 1)) & 255
    e1 = (((a & 1 | a << 2) << 3) | ((b >> 2 | b & 135) >> 3)) & 255
    e2 = (c >> 7 | ((b & 31 | (b & 248) << 2) << 1)) & 255
    e3 = (b << 7 | ((c & 249 | c >> 2 & 6) >> 1)) & 255
    e4 = (((c & 1 | c << 2) << 3) | ((d >> 2 | d & 135) >> 3)) & 255
    e5 = (a >> 7 | ((d & 31 | (d & 248) << 2) << 1)) & 255
    return (_PS[0][(e0 << 4 | e1 >> 4) ^ k[0]] | _PS[1][((e1 & 15) << 8 | e2) ^ k[1]]
            | _PS[2][(e3 << 4 | e4 >> 4) ^ k[2]] | _PS[3][((e4 & 15) << 8 | e5) ^ k[3]])


def l1_decode(buf):
    """整塊 8 bytes 解碼；結尾不足 8 bytes 的部分原樣保留（與 Rust 相同）。"""
    out = bytearray(buf)
    for o in range(0, len(buf) - len(buf) % 8, 8):
        s = _permute(int.from_bytes(buf[o:o + 8], 'big'), _IT)
        left, right = s >> 32, s & 0xffffffff
        for rd in range(15, -1, -1):
            left, right = right, left ^ _round(right, _PK[rd])
        out[o:o + 8] = _permute((right << 32) | left, _FT).to_bytes(8, 'big')
    return bytes(out)


L1_KNOWN_ANSWER = bytes.fromhex('48DBEF8CDA28479F3725EE7E1A9792EAAFF0A5BA7B6088A5A0B0AE0A5692D542')

# --------------------------------------------------------------------------
# 容器層關卡（各自照抄 Rust 解碼器在解像素前的檢查；代碼即診斷用語）
# --------------------------------------------------------------------------
SPR_EMPTY_SENTINELS = {
    (192, 120, -128, -120, 0xffff, 0), (192, 120, -128, -120, 0x0f0f, 0xf1f1),
    (432, 132, -368, -468, 0xffff, 0), (352, 120, -288, -360, 0xffff, 0),
    (544, 384, -480, -384, 0xffff, 0)}


def gate_spr(d, markers=None):
    """sprite.rs decode_impl 到區塊偏移表為止（SpriteLimits::default）。"""
    n = len(d)
    if n > 64 << 20:
        return 'spr.limit_input_bytes'
    if n == 0:
        return 'spr.truncated'
    if d[:3] in (b'SPX', b'SP2') or d[0] == 0xfe:
        return 'spr.sp2_spx_or_fe_marker'
    palette = d[0] == 255
    pos = 1
    if palette:
        if n < 2:
            return 'spr.truncated'
        pos = 2 + (d[1] or 256) * 2
        if pos >= n:
            return 'spr.truncated'
        frames = d[pos]
        pos += 1
    else:
        frames = d[0]
    if frames == 0:
        return 'spr.zero_frame_count'
    refs = 0
    max_id = -1
    for _ in range(frames):
        if pos + 14 > n:
            return 'spr.truncated'
        left, top, right, bottom, u1, u2, count = struct.unpack_from('<hhhhHHH', d, pos)
        pos += 14
        if (right < left or bottom < top) and not (
                count == 0 and (left, top, right, bottom, u1, u2) in SPR_EMPTY_SENTINELS):
            if count:
                return 'spr.reversed_frame_bounds_with_blocks'
            if markers is not None:
                markers[(left, top, right, bottom, u1, u2)] += 1
            return 'spr.unlisted_empty_frame_marker'
        refs += count
        if refs > 65536:
            return 'spr.limit_block_references'
        end = pos + count * 5
        if end > n:
            return 'spr.truncated'
        if count:
            max_id = max(max_id, max(map(lambda lo, hi: lo | hi << 8, d[pos + 3:end:5], d[pos + 4:end:5])))
        pos = end
    if pos + 4 > n:
        return 'spr.truncated'
    blocks = struct.unpack_from('<I', d, pos)[0]
    pos += 4
    if blocks > 16384:
        return 'spr.limit_block_count'
    if pos + blocks * 4 + 4 > n:
        return 'spr.truncated'
    offsets = array('I')
    offsets.frombytes(d[pos:pos + blocks * 4])
    pos += blocks * 4 + 4
    body = n - pos
    if blocks == 0 and body > 0:
        return 'spr.data_without_blocks'
    if max_id >= blocks:
        return 'spr.block_reference_outside_table'
    if blocks:
        if min(offsets) != 0:
            return 'spr.prefix_before_first_block'
        if max(offsets) >= body:
            return 'spr.block_offset_outside_body'
    return 'spr.container_ok_palette' if palette else 'spr.container_ok_rgb555'


TIL_TYPES = {0, 1, 2, 3, 6, 7, 8, 9, 16, 17, 18, 19, 22, 23, 34, 35}


def gate_til(d, types=None):
    """tile.rs ValidatedTileInput::validate ＋ 每個區塊第一個 byte 的型別檢查。"""
    n = len(d)
    if n > 64 << 20:
        return 'til.limit_input_bytes'
    if n < 4:
        return 'til.truncated'
    count = struct.unpack_from('<I', d)[0]
    if count > 65536 and d[0] in (0x78, 0x5b, 0x1b):
        return 'til.compressed_container'
    if count == 0:
        return 'til.zero_tile_count'
    if count > 16384:
        return 'til.limit_tile_count'
    table = (count + 1) * 4
    if 4 + table > n:
        return 'til.truncated'
    offsets = array('I')
    offsets.frombytes(d[4:4 + count * 4])
    end = struct.unpack_from('<I', d, 4 + count * 4)[0]
    start = 4 + table
    body = n - start
    if end > body:
        return 'til.end_offset_exceeds_body'
    if end != body:
        return 'til.trailing_data_after_end_offset'
    if max(offsets) > end:
        return 'til.block_offset_outside_body'
    empty_tail = False
    if end in offsets:
        first = offsets.index(end)
        if first == 0 or any(o != end for o in offsets[first:]):
            return 'til.empty_slots_not_terminal_suffix'
        empty_tail = True
    distinct = sorted(set(o for o in offsets if o < end))
    if not distinct or distinct[0] != 0:
        return 'til.prefix_before_first_block'
    bad = set()
    for o in distinct:
        kind = d[start + o]
        if types is not None:
            types[kind] += 1
        if kind not in TIL_TYPES:
            bad.add(kind)
    if bad:
        return 'til.unsupported_block_type_' + '_'.join(str(k) for k in sorted(bad))
    return 'til.container_ok_empty_tail' if empty_tail else 'til.container_ok'


def gate_tbt(d):
    """inventory_icons.rs decode_tbt 的完整走訪（TBT／LBT 共用）。"""
    n = len(d)
    if n > 256 * 1024:
        return 'tbt.payload_too_large'
    if n < 4:
        return 'tbt.truncated'
    height, width = d[3], d[2]
    if width == 0 or height == 0:
        return 'tbt.empty_image'
    cursor = 4
    for _ in range(height):
        if cursor >= n:
            return 'tbt.truncated'
        spans = d[cursor]
        cursor += 1
        x = 0
        for _ in range(spans):
            if cursor + 2 > n:
                return 'tbt.truncated'
            skip, count = d[cursor], d[cursor + 1]
            cursor += 2
            if skip & 1:
                return 'tbt.odd_byte_skip'
            x += skip // 2
            end = x + count
            if end > 512:
                return 'tbt.dimension_limit'
            width = max(width, end)
            stop = cursor + count * 2
            if stop > n:
                return 'tbt.truncated'
            if count and max(d[cursor + 1:stop:2]) & 0x80:
                return 'tbt.pixel_high_bit'
            cursor = stop
            x = end
    if cursor != n:
        return 'tbt.trailing_bytes'
    return 'tbt.ok'


def gate_img(d):
    """native_mode_menu_skin.rs decode_img（最嚴格的通用 IMG 讀法）。"""
    n = len(d)
    if n < 8:
        return 'img.header_truncated'
    w, h, flag = struct.unpack_from('<hhH', d)
    if w <= 0 or h <= 0:
        return 'img.non_positive_dimensions'
    if w > 2048 or h > 2048 or w * h > 1_048_576:
        return 'img.decode_budget'
    if flag > 1:
        return 'img.unknown_flag_%d' % flag
    if n != 8 + w * h * 2:
        return 'img.payload_size_mismatch'
    if max(d[9::2]) & 0x80:
        return 'img.rgb555_high_bit'
    return 'img.ok_flag%d' % flag


def gate_png(d):
    if d[:8] != b'\x89PNG\r\n\x1a\n':
        return 'png.bad_signature_' + d[:4].hex()
    if len(d) < 33 or d[12:16] != b'IHDR':
        return 'png.missing_ihdr'
    return 'png.signature_ok'


def gate_wav(d):
    """audio.rs parse_pcm_wav（預設 limits 的 chunk／frame 上限未照抄）。"""
    n = len(d)
    if n > 16 << 20:
        return 'wav.too_large'
    if n < 12:
        return 'wav.truncated'
    if d[:4] != b'RIFF':
        return 'wav.not_riff'
    if d[8:12] != b'WAVE':
        return 'wav.not_wave'
    if struct.unpack_from('<I', d, 4)[0] + 8 != n:
        return 'wav.riff_size_mismatch'
    pos = 12
    fmt = None
    data = None
    unpadded = False
    while pos < n:
        if pos + 8 > n:
            return 'wav.truncated'
        cid = d[pos:pos + 4]
        size = struct.unpack_from('<I', d, pos + 4)[0]
        start = pos + 8
        end = start + size
        if end > n:
            return 'wav.truncated'
        if size & 1 and end == n and cid == b'data':
            unpadded = True
            nxt = end
        else:
            if end + (size & 1) > n:
                return 'wav.truncated'
            nxt = end + (size & 1)
        if cid == b'fmt ':
            if fmt is not None:
                return 'wav.duplicate_fmt'
            if size < 16:
                return 'wav.truncated'
            enc, ch, rate, brate, align, bits = struct.unpack_from('<HHIIHH', d, start)
            if enc != 1:
                return 'wav.unsupported_encoding_%d' % enc
            if size != 16 and not (size == 18 and d[start + 16:start + 18] == b'\0\0'):
                return 'wav.fmt_size_invalid'
            if ch not in (1, 2) or bits not in (8, 16):
                return 'wav.unsupported_pcm_%dch_%dbit' % (ch, bits)
            if not 1 <= rate <= 192000:
                return 'wav.sample_rate_invalid'
            if align != ch * (bits // 8):
                return 'wav.block_align_mismatch'
            if brate != rate * align:
                return 'wav.byte_rate_mismatch'
            fmt = align
        elif cid == b'data':
            if data is not None:
                return 'wav.duplicate_data'
            if fmt is None:
                return 'wav.data_precedes_fmt'
            data = size
        pos = nxt
    if fmt is None:
        return 'wav.missing_fmt'
    if data is None:
        return 'wav.missing_data'
    if data % fmt:
        return 'wav.partial_frame'
    return 'wav.ok_terminal_unpadded_data' if unpadded else 'wav.ok'


def gate_s32(d, native):
    """map.rs parse_s32_format（native=True 為 parse_native_s32 的 portal 尾碼）。"""
    n = len(d)
    if n > 8 << 20:
        return 's32.too_large'
    pos = 0
    layer = 1

    class Stop(Exception):
        pass

    def need(count):
        if pos + count > n:
            raise Stop('s32.truncated_layer%d' % layer)

    try:
        need(8192 * 4)
        pos += 8192 * 4
        layer = 2
        need(2)
        count = struct.unpack_from('<H', d, pos)[0]
        pos += 2
        need(count * 6)
        pos += count * 6
        layer = 3
        need(4096 * 4)
        pos += 4096 * 4
        layer = 4
        need(4)
        groups = struct.unpack_from('<i', d, pos)[0]
        pos += 4
        if groups < 0:
            return 's32.negative_count_layer4'
        if groups > 65536:
            return 's32.too_many_records_layer4'
        need(groups * 4)
        total = 0
        for _ in range(groups):
            need(4)
            count = struct.unpack_from('<H', d, pos + 2)[0]
            pos += 4
            total += count
            if total > 262144:
                return 's32.too_many_records_layer4'
            need(count * 7)
            pos += count * 7
        if pos == n:
            return 's32.ok_layers1to4'
        layer = 5
        need(4)
        count = struct.unpack_from('<i', d, pos)[0]
        pos += 4
        if count < 0:
            return 's32.negative_count_layer5'
        if count > 262144:
            return 's32.too_many_records_layer5'
        need(count * 5)
        pos += count * 5
        if pos == n:
            return 's32.ok_layers1to5'
        layer = 6
        need(4)
        count = struct.unpack_from('<i', d, pos)[0]
        pos += 4
        if count < 0:
            return 's32.negative_count_layer6'
        if count > 262144:
            return 's32.too_many_records_layer6'
        need(count * 4)
        pos += count * 4
        if pos == n:
            return 's32.ok_layers1to6'
        layer = 7
        need(2)
        count = struct.unpack_from('<H', d, pos)[0]
        pos += 2
        need(count * 9)
        for _ in range(count):
            need(1)
            name = d[pos]
            pos += 1
            need(name + 8)
            pos += name + 8
            if native and pos < n and 100 <= d[pos] <= 102:
                pos += 1
        if pos == n:
            return 's32.ok_layers1to7'
        layer = 8
        need(2)
        packed = struct.unpack_from('<H', d, pos)[0]
        pos += 2
        count = packed & 0x7fff
        size = 10 if packed & 0x8000 else 6
        need(count * size)
        pos += count * size
        return 's32.ok_layers1to8' if pos == n else 's32.ok_layers1to8_unknown_trailing'
    except Stop as stop:
        return stop.args[0]


# --------------------------------------------------------------------------
# 格式支援表：格式 → Rust 端的對應（路徑相對 LiTo381-Modern/src/modern_lito）
#   decoder  ：有通用解碼器；逐成員判定為「容器層通過」或「被拒＋原因碼」
#   selected ：只有指名的成員有消費端，沒有通用解析器
#   none     ：Rust 端沒有這個格式的解碼器
#   junk     ：不是遊戲資源（開發殘留、版控、壓縮檔）
# --------------------------------------------------------------------------
FORMATS = {
    'spr': ('decoder', ['sprite.rs'], 'SPR 影格圖（RGB555 或調色盤，24×24 RLE 區塊）'),
    'til': ('decoder', ['tile.rs', 'tile_reader.rs'], 'TIL 地磚（型別 0–3、6–9、16–19、22–23、34–35）'),
    'tbt': ('decoder', ['inventory_icons.rs'], 'TBT 圖示（列 span、RGB555）'),
    'lbt': ('decoder', ['native_inventory_icons.rs', 'inventory_icons.rs'], 'LBT 圖示（與 TBT 同一個解碼器）'),
    'img': ('decoder', ['native_mode_menu_skin.rs', 'battle_hud.rs', 'native_skill_layers.rs'],
            'IMG 介面圖（8 byte 表頭＋RGB555）'),
    'wav': ('decoder', ['audio.rs'], 'RIFF/WAVE PCM8／PCM16 單聲道或立體聲'),
    's32': ('decoder', ['map.rs', 'world_map.rs'], 'S32 地圖區塊（Layer1–8）'),
    'list_spr_text': ('selected', ['sprite_actions.rs', 'shared_action_catalog.rs'], 'list.spr 動作目錄（文字）'),
    'ico_list': ('selected', ['inventory_icons.rs'], 'list.ico 圖示範圍表（文字）'),
    'tbl': ('selected', ['descriptions.rs', 'native_inventory_names.rs'], 'TBL 文字表'),
    'html': ('selected', ['native_npc_document.rs'], 'NPC 對話 HTML'),
    'text_misc': ('none', [], '其他文字檔（txt／def／mcf）'),
    'png': ('none', [], 'PNG 圖（png crate 只用在輸出截圖，沒有資源消費端）'),
    'xml_tile': ('none', [], 'Tile.pak 內的 XML（首 4 bytes 為 58 3F 78 6D，非明文 `<?xm`）'),
    'mp3': ('none', [], 'MP3 音樂（audio.rs 明確拒絕）'),
    'mid': ('none', [], 'MIDI（audio.rs 明確拒絕）'),
    'seg': ('none', [], 'SEG 地圖區塊（map.rs 明確不解）'),
    'bmp': ('none', [], 'BMP（原登入器外框圖）'),
    'fnt': ('none', [], 'FNT 點陣字型'),
    'unknown': ('none', [], '未知格式（副檔名不在表內）'),
    'junk': ('junk', [], '非遊戲資源（開發殘留／版控／壓縮檔）'),
    'loose_other': ('none', [], '其他散檔'),
}
JUNK_EXT = {'py', 'db', 'scc', 'rar', 'bak', 'tmp'}
TEXT_HTML_TRUNCATED = {'', 'h', 'ht', 'htm'}

ARCHIVES = [('SHARED', 'Sprite', 'sprite', False)] + \
    [('SHARED', 'Sprite%02d' % i, 'sprite', False) for i in range(16)] + [
    ('SHARED', 'Tile', 'tile', False), ('SHARED', 'Text', 'text', True),
    ('NATIVE_LITO', 'LiTo/LTicon', 'lticon', False), ('NATIVE_LITO', 'LiTo/LTsprite', 'sprite', False),
    ('NATIVE_LITO', 'LiTo/LTsurf', 'ltsurf', False), ('NATIVE_LITO', 'LiTo/LTtext', 'text', True),
    ('NATIVE_LITO', 'LiTo/LTtile', 'tile', False)]

# 散檔命名空間：(路徑前綴小寫, 命名空間, 是否納入資源集合雜湊)
LOOSE_NAMESPACES = [
    ('map/', 'SHARED/map', True), ('sound/', 'SHARED/Sound', True), ('font/', 'SHARED/Font', True),
    ('lito/ltmap/', 'NATIVE_LITO/LTmap', True), ('lito/ltsound/', 'NATIVE_LITO/LTsound', True),
    ('lito/ltsurf/', 'NATIVE_LITO/LTSurf', True), ('lito/', 'NATIVE_LITO/loose', True)]
# 根目錄與 launcher/ 只納入指名的資料檔；執行檔、設定檔、紀錄會變動，不屬於資源集合。
ROOT_BOUND = {'shade.dat', 'emblem769.dat', 'tw13081901.txt'}
LAUNCHER_BOUND = {'launcher/tw13081901.pak'}


def split_ext(name):
    return name.rsplit('.', 1)[1] if '.' in name else None


def classify_member(kind, name):
    """回傳 (格式, 旗標)。name 為小寫；kind 為 sprite／tile／text／lticon／ltsurf。"""
    ext = split_ext(name)
    flags = []
    truncated = len(name) == 20
    if kind == 'text':
        if ext == 'html' or (truncated and (ext in TEXT_HTML_TRUNCATED)):
            if ext != 'html':
                flags.append('name_truncated_20')
            return 'html', flags
        if ext == 'spr':
            return 'list_spr_text', flags
        if ext == 'tbl':
            return 'tbl', flags
        if ext in ('txt', 'def', 'mcf'):
            return 'text_misc', flags
    else:
        if ext == 'spr' or (truncated and ext in ('sp', 's')):
            if ext != 'spr':
                flags.append('name_truncated_20')
            return 'spr', flags
        if ext in ('til', 'tbt', 'lbt', 'img', 'png'):
            return ext, flags
        if ext == 'ico':
            return 'ico_list', flags
        if ext == 'tbl':
            return 'tbl', flags
        if ext == 'xml' and kind == 'tile':
            return 'xml_tile', flags
    if ext in JUNK_EXT:
        return 'junk', flags
    if truncated:
        flags.append('name_truncated_20')
    return 'unknown', flags


def classify_loose(rel_lower):
    ext = split_ext(rel_lower.rsplit('/', 1)[-1])
    if ext in ('s32', 'seg', 'wav', 'mp3', 'mid', 'img', 'bmp', 'fnt'):
        return ext
    if ext in JUNK_EXT:
        return 'junk'
    return 'loose_other'


GATES = {'spr': gate_spr, 'tbt': gate_tbt, 'lbt': gate_tbt, 'img': gate_img, 'png': gate_png, 'wav': gate_wav}


def text_sniff(head):
    if head[:2] == b'\xff\xfe':
        return 'text.utf16le_bom'
    if head[:3] == b'\xef\xbb\xbf':
        return 'text.utf8_bom'
    if head and all(b in (9, 10, 13) or 32 <= b < 127 for b in head):
        return 'text.ascii_head'
    return 'text.non_ascii_head'


def status_of(fmt, verdict):
    kind = FORMATS[fmt][0]
    if kind == 'decoder':
        ok = '.ok' in verdict or 'container_ok' in verdict
        return 'supported_container_ok' if ok else 'rejected_by_decoder_gate'
    return {'selected': 'selected_members_only', 'none': 'no_decoder', 'junk': 'not_game_resource'}[kind]


# --------------------------------------------------------------------------
# 掃描
# --------------------------------------------------------------------------
def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        while True:
            chunk = f.read(8 << 20)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest().upper()


def parse_index(raw, protected):
    count = struct.unpack_from('<I', raw)[0]
    body = l1_decode(raw[4:]) if protected else raw[4:]
    if len(body) != count * 28:
        raise SystemExit('old28 索引長度與筆數不符')
    return [(off, name.split(b'\0')[0], size) for off, name, size in struct.iter_unpack('<I20sI', body)]


def safe_name(raw):
    return ''.join(chr(b) if 33 <= b < 127 and b != 92 else '\\x%02x' % b for b in raw)


def log(message):
    print(message, file=sys.stderr, flush=True)


def scan(root):
    members = []   # (namespace, container, name, bytes, sha256, fmt, verdict, status, flags)
    sources = []
    til_types = collections.defaultdict(collections.Counter)
    spr_markers = collections.Counter()
    index_facts = {}
    names_by_container = {}
    for namespace, stem, kind, protected in ARCHIVES:
        idx_path = root / (stem + '.idx')
        pak_path = root / (stem + '.pak')
        log('archive ' + stem)
        raw = idx_path.read_bytes()
        entries = parse_index(raw, protected)
        pak_size = pak_path.stat().st_size
        sources.append({'path': stem + '.idx', 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest().upper()})
        sources.append({'path': stem + '.pak', 'bytes': pak_size, 'sha256': sha256_file(pak_path)})
        seen = collections.Counter(name.lower() for _, name, _ in entries)
        facts = collections.Counter()
        table = {}
        shard = int(stem[-2:]) if stem.startswith('Sprite') and len(stem) == 8 else None
        with open(pak_path, 'rb') as pak:
            for offset, raw_name, size in sorted(entries):
                name = safe_name(raw_name.lower())
                fmt, flags = classify_member(kind, name)
                if seen[raw_name.lower()] > 1:
                    flags.append('duplicate_name_in_index')
                    facts['duplicate_name_records'] += 1
                if any(not (48 <= b <= 57 or 65 <= b <= 90 or 97 <= b <= 122 or b in b'._-') for b in raw_name):
                    flags.append('name_outside_ascii_alnum_dot_underscore_dash')
                    facts['unusual_name_records'] += 1
                if 'name_truncated_20' in flags:
                    facts['name_truncated_20_records'] += 1
                if shard is not None:
                    routed = sum(raw_name) & 15
                    facts['shard_route_match' if routed == shard else 'shard_route_mismatch'] += 1
                    if routed != shard:
                        flags.append('shard_route_expected_%02d' % routed)
                if offset + size > pak_size:
                    data = b''
                    verdict = 'index.entry_out_of_bounds'
                    digest = '-'
                else:
                    pak.seek(offset)
                    data = pak.read(size)
                    digest = hashlib.sha256(data).hexdigest().upper()
                    if size == 0:
                        verdict = 'index.zero_size'
                    elif fmt == 'til':
                        verdict = gate_til(data, til_types[stem])
                    elif fmt == 'spr':
                        verdict = gate_spr(data, spr_markers)
                    elif fmt in GATES:
                        verdict = GATES[fmt](data)
                    elif protected:
                        verdict = text_sniff(l1_decode(data[:16]))
                    else:
                        verdict = 'magic.' + data[:4].hex()
                        if fmt in ('unknown', 'junk', 'tbl', 'ico_list'):
                            verdict = 'not_inspected'
                status = status_of(fmt, verdict)
                if verdict.startswith('index.'):
                    status = 'rejected_by_decoder_gate'
                members.append((namespace, stem, name, size, digest, fmt, verdict, status, ','.join(flags)))
                table.setdefault(name, []).append(digest)
        facts['records'] = len(entries)
        facts['unique_names'] = len(seen)
        index_facts[stem] = dict(sorted(facts.items()))
        names_by_container[stem] = table

    # 散檔
    log('loose files')
    trees = collections.defaultdict(list)
    unbound = collections.Counter()
    empty_dirs = []
    archive_files = {(stem + ext).lower() for _, stem, _, _ in ARCHIVES for ext in ('.idx', '.pak')}
    for current, dirs, files in os.walk(root):
        dirs.sort(key=str.lower)
        rel_dir = Path(current).relative_to(root).as_posix()
        rel_dir = '' if rel_dir == '.' else rel_dir + '/'
        if not dirs and not files:
            empty_dirs.append(rel_dir)
        for file_name in sorted(files, key=str.lower):
            rel = rel_dir + file_name
            low = rel.lower()
            if low in archive_files:
                continue
            path = Path(current) / file_name
            namespace = None
            for prefix, ns, _ in LOOSE_NAMESPACES:
                if low.startswith(prefix):
                    namespace = ns
                    break
            if namespace is None:
                if low in ROOT_BOUND:
                    namespace = 'SHARED/root'
                elif low in LAUNCHER_BOUND:
                    namespace = 'LAUNCHER'
                else:
                    top = rel.split('/')[0] if '/' in rel else '(root)'
                    unbound[(top, split_ext(file_name.lower()) or '')] += 1
                    continue
            data = path.read_bytes()
            digest = hashlib.sha256(data).hexdigest().upper()
            fmt = classify_loose(low)
            flags = []
            if not data:
                verdict = 'file.zero_size'
            elif fmt == 's32':
                verdict = gate_s32(data, namespace == 'NATIVE_LITO/LTmap')
            elif fmt in GATES:
                verdict = GATES[fmt](data)
            elif fmt == 'bmp':
                verdict = 'bmp.signature_ok' if data[:2] == b'BM' else 'bmp.bad_signature'
            elif fmt == 'mp3':
                verdict = 'mp3.id3' if data[:3] == b'ID3' else ('mp3.frame_sync' if data[0] == 0xff else 'mp3.other_head')
            elif fmt == 'mid':
                verdict = 'mid.mthd' if data[:4] == b'MThd' else 'mid.other_head'
            else:
                verdict = 'not_inspected'
            status = status_of(fmt, verdict)
            if verdict == 'file.zero_size' and FORMATS[fmt][0] == 'decoder':
                status = 'rejected_by_decoder_gate'
            parts = rel.split('/')
            container = '/'.join(parts[:-1]) or '(root)'
            members.append((namespace, container, safe_name(file_name.lower().encode('utf-8', 'replace'))
                            if file_name.isascii() else file_name.lower(),
                            len(data), digest, fmt, verdict, status, ','.join(flags)))
            trees[namespace].append('%s\t%d\t%s' % (low, len(data), digest))
    for namespace in sorted(trees):
        lines = sorted(trees[namespace])
        sources.append({'path': '[tree] ' + namespace, 'files': len(lines),
                        'bytes': sum(int(line.split('\t')[1]) for line in lines),
                        'sha256': hashlib.sha256('\n'.join(lines).encode('utf-8')).hexdigest().upper()})
    return {'members': members, 'sources': sources, 'til_types': til_types, 'spr_markers': spr_markers, 'index_facts': index_facts,
            'names_by_container': names_by_container, 'unbound': unbound, 'empty_dirs': sorted(empty_dirs)}


# --------------------------------------------------------------------------
# 彙整與輸出
# --------------------------------------------------------------------------
def overlap(tables, left, right):
    """left 與 right（容器名稱清單）之間同名成員：同名數、內容相同數。"""
    names = collections.defaultdict(lambda: [set(), set()])
    for side, containers in enumerate((left, right)):
        for container in containers:
            for name, digests in tables[container].items():
                names[name][side].update(digests)
    both = [(a, b) for a, b in names.values() if a and b]
    return {'same_name': len(both), 'identical_content': sum(1 for a, b in both if a & b)}


def build(scanned):
    members = sorted(scanned['members'])
    rows = collections.OrderedDict()
    for ns, container, name, size, digest, fmt, verdict, status, flags in members:
        group = container
        if ns == 'SHARED/map' and container != 'map':
            group = 'map/*'
        elif ns == 'NATIVE_LITO/LTmap' and container != 'LiTo/LTmap':
            group = 'LiTo/LTmap/*'
        key = (ns, group, fmt, status, verdict)
        row = rows.setdefault(key, [0, 0])
        row[0] += 1
        row[1] += size
    matrix = [{'namespace': k[0], 'container': k[1], 'format': k[2], 'status': k[3], 'verdict': k[4],
               'members': v[0], 'bytes': v[1]} for k, v in sorted(rows.items())]
    assert sum(r['members'] for r in matrix) == len(members), '逐成員覆蓋核對失敗'

    tables = scanned['names_by_container']
    shards = ['Sprite%02d' % i for i in range(16)]
    sprite_family = collections.Counter()
    by_name = collections.defaultdict(list)
    for container in ['Sprite'] + shards:
        for name, digests in tables[container].items():
            by_name[name].append((container, digests))
    for name, where in by_name.items():
        if len(where) > 1:
            ext = split_ext(name) or ''
            same = len(set(d for _, ds in where for d in ds)) == 1
            sprite_family[(ext, 'identical' if same else 'different')] += 1
    namespace_facts = {
        'shared_sprite_family_names_in_multiple_archives': [
            {'ext': e, 'content': c, 'names': n} for (e, c), n in sorted(sprite_family.items())],
        'shared_sprite_base_vs_shards': overlap(tables, ['Sprite'], shards),
        'native_ltsprite_vs_shared_sprite_family': overlap(tables, ['LiTo/LTsprite'], ['Sprite'] + shards),
        'native_ltsurf_vs_shared_sprite_family': overlap(tables, ['LiTo/LTsurf'], ['Sprite'] + shards),
        'native_lttile_vs_shared_tile': overlap(tables, ['LiTo/LTtile'], ['Tile']),
        'native_lttext_vs_shared_text': overlap(tables, ['LiTo/LTtext'], ['Text']),
        'index_facts': scanned['index_facts'],
        'empty_directories': scanned['empty_dirs'],
    }
    maps = collections.OrderedDict()
    for ns, container, name, size, digest, fmt, verdict, status, flags in members:
        if ns in ('SHARED/map', 'NATIVE_LITO/LTmap') and fmt in ('s32', 'seg'):
            entry = maps.setdefault((ns, container), collections.Counter())
            entry[fmt] += 1
            entry[fmt + '_bytes'] += size
            if fmt == 's32':
                entry['verdict:' + verdict] += 1
    map_rows = [dict(namespace=k[0], directory=k[1], **dict(sorted(v.items()))) for k, v in maps.items()]

    exceptions = [m for m in members if m[7] == 'rejected_by_decoder_gate' or m[5] in ('unknown', 'junk')
                  or 'duplicate_name_in_index' in m[8] or 'shard_route_expected' in m[8]
                  or 'name_outside_ascii' in m[8] or (m[5] == 'spr' and 'name_truncated_20' in m[8])
                  or m[6].startswith('png.bad') or m[6].endswith('unknown_trailing')]
    member_lines = ['\t'.join(str(x) for x in m) for m in members]
    header = 'namespace\tcontainer\tname\tbytes\tsha256\tformat\tverdict\tstatus\tflags'
    members_text = header + '\n' + '\n'.join(member_lines) + '\n'
    exceptions_text = header.replace('\tsha256', '') + '\n' + '\n'.join(
        '\t'.join(str(x) for x in m[:4] + m[5:]) for m in exceptions) + '\n'
    sources = scanned['sources']
    set_digest = hashlib.sha256('\n'.join(
        '%s\t%d\t%s' % (s['path'], s['bytes'], s['sha256']) for s in sources).encode('utf-8')).hexdigest().upper()
    document = {
        'tool': 'resource_matrix.py', 'tool_version': TOOL_VERSION, 'issue': 336, 'goal': 'G07.1',
        'resource_set_sha256': set_digest,
        'sources': sources,
        'members_total': len(members),
        'members_tsv_sha256': hashlib.sha256(members_text.encode('utf-8')).hexdigest().upper(),
        'exceptions_total': len(exceptions),
        'formats': {k: {'support': v[0], 'rust_modules': v[1], 'title': v[2]} for k, v in FORMATS.items()},
        'matrix': matrix,
        'til_block_types': {k: {str(t): c for t, c in sorted(v.items())} for k, v in sorted(scanned['til_types'].items())},
        'spr_unlisted_empty_frame_markers': [
            {'left': k[0], 'top': k[1], 'right': k[2], 'bottom': k[3], 'unknown1': k[4], 'unknown2': k[5], 'members': v}
            for k, v in sorted(scanned['spr_markers'].items(), key=lambda kv: (-kv[1], kv[0]))],
        'namespace_facts': namespace_facts,
        'map_directories': map_rows,
        'unbound_files': [{'top': t, 'ext': e, 'files': c} for (t, e), c in sorted(scanned['unbound'].items())],
    }
    return document, members_text, exceptions_text


def fmt_bytes(n):
    return '{:,}'.format(n)


def render_markdown(doc):
    out = []
    add = out.append
    add('# RESOURCE-MATRIX — 資源命名空間與格式矩陣（G07.1）')
    add('')
    add('本檔由 `維護工具/resource-matrix/resource_matrix.py` 產生（工具版本 %d，#336／T057），**不要手改**；'
        '要更新就重跑工具。內容只取決於客戶端資源與本工具的規則，沒有時間戳，同一份資源重跑會得到相同位元組。' % doc['tool_version'])
    add('')
    add('- 資源集合雜湊（下表所有來源的路徑／大小／SHA-256 串接後再取 SHA-256）：`%s`' % doc['resource_set_sha256'])
    add('- 列舉成員總數：**%s**（封存記錄＋納入雜湊的散檔），每一個都落在 §2 的某一列；工具以加總核對。' % fmt_bytes(doc['members_total']))
    add('- 逐成員清單（含每個成員的 SHA-256，約 48 MB）不入庫，重跑後在 `scratch/resource-matrix/members.tsv`；其 SHA-256 應為 `%s`。' % doc['members_tsv_sha256'])
    add('- 例外清單（被關卡拒絕、未知格式、非資源、同名重複、分片路由不符等）：`docs/resource-matrix/exceptions.tsv`（%s 筆）。' % fmt_bytes(doc['exceptions_total']))
    add('- 機器可讀版：`docs/resource-matrix/matrix.json`。')
    add('')
    add('## 0. 怎麼讀這份矩陣')
    add('')
    add('| 欄位 | 意思 |')
    add('|---|---|')
    add('| 命名空間 | `SHARED`＝天堂共用資源（客戶端根目錄）；`NATIVE_LITO`＝LiTo 專屬（`LiTo/`）；與 `native_resources.rs` 的 `ResourceNamespace` 對應。兩者之間**沒有**自動 fallback。 |')
    add('| 狀態 `supported_container_ok` | Rust 有這個格式的通用解碼器，而且這個成員通過「容器層關卡」。 |')
    add('| 狀態 `rejected_by_decoder_gate` | Rust 有解碼器，但這個成員會被關卡拒絕；判定欄是原因碼（G07.4 診斷要用的來源）。 |')
    add('| 狀態 `selected_members_only` | Rust 只消費指名的成員（例如特定 tbl／html），沒有通用解析器。 |')
    add('| 狀態 `no_decoder` | Rust 沒有這個格式的解碼器。 |')
    add('| 狀態 `not_game_resource` | 開發殘留、版控檔、壓縮檔等，不是遊戲資源。 |')
    add('')
    add('**界線**：判定是把 Rust 解碼器在解像素前的檢查照抄成 Python 逐成員跑的結果（已驗證的是「Python 關卡的結果」）。'
        'SPR／TIL 的 RLE 像素本體、影格尺寸與總像素上限沒有在這裡驗；「Rust 解碼器實際逐成員跑過」仍是待驗證，'
        '由 `LiTo381-Modern/tests/resource_matrix_exhaustive.rs`（`#[ignore]`）負責。')
    add('')
    add('## 1. 來源綁定（SHA-256）')
    add('')
    add('| 來源 | 大小（bytes） | SHA-256 |')
    add('|---|---:|---|')
    for s in doc['sources']:
        label = s['path'] + ('（%s 檔）' % fmt_bytes(s['files']) if 'files' in s else '')
        add('| `%s` | %s | `%s` |' % (label, fmt_bytes(s['bytes']), s['sha256']))
    add('')
    add('`[tree]` 列是散檔命名空間：把每個檔的「相對路徑（小寫）／大小／SHA-256」排序後串接再取 SHA-256。')
    add('')
    add('## 2. 格式矩陣（命名空間 × 容器 × 格式 × 狀態）')
    add('')
    add('共用 Sprite 分片（Sprite00–15）在這裡合併成一列，逐分片的數字在 `matrix.json`；地圖目錄合併為 `map/*`、`LiTo/LTmap/*`，逐目錄見 §4.4。')
    add('')
    merged = collections.OrderedDict()
    for r in doc['matrix']:
        container = r['container']
        if re.fullmatch(r'Sprite\d\d', container):
            container = 'Sprite00–15'
        key = (r['namespace'], container, r['format'], r['status'])
        entry = merged.setdefault(key, [0, 0, collections.Counter()])
        entry[0] += r['members']
        entry[1] += r['bytes']
        entry[2][r['verdict']] += r['members']
    add('| 命名空間 | 容器 | 格式 | 狀態 | 成員數 | bytes | 判定（成員數） |')
    add('|---|---|---|---|---:|---:|---|')
    for (ns, container, fmt, status), (count, size, verdicts) in merged.items():
        items = verdicts.most_common()
        shown = '；'.join('`%s` %s' % (v, fmt_bytes(c)) for v, c in items[:6])
        if len(items) > 6:
            shown += '；另 %d 種判定共 %s' % (len(items) - 6, fmt_bytes(sum(c for _, c in items[6:])))
        add('| %s | `%s` | %s | %s | %s | %s | %s |' % (ns, container, fmt, status, fmt_bytes(count), fmt_bytes(size), shown))
    add('')
    add('### 2.1 依狀態加總')
    add('')
    totals = collections.Counter()
    for r in doc['matrix']:
        totals[r['status']] += r['members']
    add('| 狀態 | 成員數 |')
    add('|---|---:|')
    for status, count in sorted(totals.items()):
        add('| %s | %s |' % (status, fmt_bytes(count)))
    add('| 合計 | %s |' % fmt_bytes(sum(totals.values())))
    add('')
    add('### 2.2 被解碼器關卡拒絕的原因碼')
    add('')
    rejected = collections.Counter()
    for r in doc['matrix']:
        if r['status'] == 'rejected_by_decoder_gate':
            rejected[(r['format'], r['verdict'])] += r['members']
    if rejected:
        add('| 格式 | 原因碼 | 成員數 |')
        add('|---|---|---:|')
        for (fmt, verdict), count in sorted(rejected.items(), key=lambda kv: (kv[0][0], -kv[1], kv[0][1])):
            add('| %s | `%s` | %s |' % (fmt, verdict, fmt_bytes(count)))
    else:
        add('沒有成員被拒絕。')
    add('')
    add('逐成員名稱見 `docs/resource-matrix/exceptions.tsv`。')
    add('')
    add('- `til.container_ok_empty_tail` 列在「通過」，但只有 `decode_classic_til_selected_with_empty_tail` 這個明確選用的入口會接受；一般的 `decode_classic_til` 會拒絕。')
    add('- `til.limit_tile_count` 的兩個成員是 `list.til`（4～5 bytes 的文字），不是地磚。')
    markers = doc['spr_unlisted_empty_frame_markers']
    if markers:
        add('- `spr.unlisted_empty_frame_marker`：影格宣告的邊界是反的（right < left 或 bottom < top）而且沒有區塊，也就是「不畫的空影格」；'
            '`sprite.rs` 的 `empty_sentinel` 目前只認得 5 組數值，其餘一律當成格式錯誤。資源裡第一個遇到的未列數值共有 **%d 種**，前 12 種如下（全部在 `matrix.json` 的 `spr_unlisted_empty_frame_markers`）：' % len(markers))
        add('')
        add('| left | top | right | bottom | unknown1 | unknown2 | 成員數 |')
        add('|---:|---:|---:|---:|---|---|---:|')
        for m in markers[:12]:
            add('| %d | %d | %d | %d | %04X | %04X | %s |' % (m['left'], m['top'], m['right'], m['bottom'], m['unknown1'], m['unknown2'], fmt_bytes(m['members'])))
        add('')
    add('## 3. 格式支援表')
    add('')
    add('| 格式 | 支援方式 | Rust 模組（`src/modern_lito/`） | 說明 |')
    add('|---|---|---|---|')
    for name, info in doc['formats'].items():
        add('| %s | %s | %s | %s |' % (name, info['support'], '、'.join('`%s`' % m for m in info['rust_modules']) or '—', info['title']))
    add('')
    add('## 4. 命名空間事實')
    add('')
    facts = doc['namespace_facts']
    add('### 4.1 索引層')
    add('')
    add('| 容器 | 記錄數 | 不重複名稱 | 同名重複記錄 | 名稱滿 20 bytes（疑似截斷） | 非常規字元名稱 | 分片路由相符 | 分片路由不符 |')
    add('|---|---:|---:|---:|---:|---:|---:|---:|')
    for container, f in facts['index_facts'].items():
        add('| `%s` | %s | %s | %s | %s | %s | %s | %s |' % (
            container, fmt_bytes(f['records']), fmt_bytes(f['unique_names']), fmt_bytes(f.get('duplicate_name_records', 0)),
            fmt_bytes(f.get('name_truncated_20_records', 0)), f.get('unusual_name_records', 0),
            fmt_bytes(f['shard_route_match']) if 'shard_route_match' in f else '—',
            (f.get('shard_route_mismatch', 0) if 'shard_route_match' in f else '—')))
    add('')
    add('- 索引是 old28（4 bytes 筆數＋每筆 `offset u32｜name 20 bytes｜size u32`）；`Text`、`LTtext` 的索引本體與成員內容另有 L1 保護（`resource_cipher.rs`）。')
    add('- 「分片路由」是檔名 ASCII bytes 加總後取低 4 bits（`inventory_icons.rs` 引用的 `sub_43FFC0`）；這裡對 Sprite00–15 的**每一個**成員都算一次。')
    add('- 名稱欄位只有 20 bytes：剛好 20 bytes 且副檔名是 `.html`／`.spr` 前綴（`.htm`、`.ht`、`.h`、`.`、`.sp`）的成員，矩陣依截斷前的格式歸類並加旗標 `name_truncated_20`。查這些成員要用截斷後的名稱。')
    add('- `Text` 有同名重複記錄：多個較長的原始檔名被截成同一個 20 bytes 名稱（例如 `mercenaryarrange-j.h`）。`resources.rs` 的 `SharedArchive::entry` 對這些名稱回 `AmbiguousEntry`，不會自動挑一筆。')
    add('')
    add('### 4.2 跨封存同名')
    add('')
    add('| 比較 | 同名成員 | 其中內容相同 |')
    add('|---|---:|---:|')
    for key, title in (('shared_sprite_base_vs_shards', '`Sprite` 對 `Sprite00–15`'),
                       ('native_ltsprite_vs_shared_sprite_family', '`LTsprite` 對共用 Sprite 全家'),
                       ('native_ltsurf_vs_shared_sprite_family', '`LTsurf` 對共用 Sprite 全家'),
                       ('native_lttile_vs_shared_tile', '`LTtile` 對 `Tile`'),
                       ('native_lttext_vs_shared_text', '`LTtext` 對 `Text`')):
        add('| %s | %s | %s |' % (title, fmt_bytes(facts[key]['same_name']), fmt_bytes(facts[key]['identical_content'])))
    add('')
    add('共用 Sprite 全家（17 個封存）裡出現在一個以上封存的名稱：')
    add('')
    add('| 副檔名 | 內容 | 名稱數 |')
    add('|---|---|---:|')
    for item in facts['shared_sprite_family_names_in_multiple_archives']:
        add('| .%s | %s | %s |' % (item['ext'], item['content'], fmt_bytes(item['names'])))
    add('')
    add('同名不同內容代表「讀哪一個封存」會改變畫面；`actor_assets.rs` 的規則是必須指名封存或全家唯一，不挑贏家。`Text` 的比較用的是保護狀態下的原始 bytes。')
    add('')
    add('### 4.3 TIL 區塊型別（每個不重複區塊偏移算一次）')
    add('')
    kinds = sorted({int(t) for v in doc['til_block_types'].values() for t in v})
    add('| 容器 | ' + ' | '.join('型別 %d%s' % (k, '' if k in TIL_TYPES else '（不支援）') for k in kinds) + ' |')
    add('|---|' + '---:|' * len(kinds))
    for container, counts in doc['til_block_types'].items():
        add('| `%s` | ' % container + ' | '.join(fmt_bytes(counts.get(str(k), 0)) for k in kinds) + ' |')
    add('')
    add('只統計通過偏移表檢查的 TIL；型別取自區塊第一個 byte。')
    add('')
    add('### 4.4 地圖目錄')
    add('')
    shared = [m for m in doc['map_directories'] if m['namespace'] == 'SHARED/map']
    native = [m for m in doc['map_directories'] if m['namespace'] == 'NATIVE_LITO/LTmap']
    add('- `map/`（SHARED）：%s 個地圖目錄，S32 %s 個、SEG %s 個。逐目錄數字在 `matrix.json` 的 `map_directories`。' % (
        fmt_bytes(len(shared)), fmt_bytes(sum(m.get('s32', 0) for m in shared)), fmt_bytes(sum(m.get('seg', 0) for m in shared))))
    only_seg = [m['directory'] for m in shared if not m.get('s32')]
    add('- 只有 SEG、沒有 S32 的共用地圖目錄：%s 個%s。' % (len(only_seg), ('（' + '、'.join('`%s`' % d for d in only_seg[:40]) + ('…' if len(only_seg) > 40 else '') + '）') if only_seg else ''))
    add('- `LiTo/LTmap/`（NATIVE_LITO）：%s 個地圖目錄，以 `parse_native_s32` 的 portal 尾碼規則檢查：' % len(native))
    add('')
    add('| 目錄 | S32 數 | bytes | 判定 |')
    add('|---|---:|---:|---|')
    for m in native:
        verdicts = '；'.join('`%s` %d' % (k[8:], v) for k, v in m.items() if k.startswith('verdict:'))
        add('| `%s` | %s | %s | %s |' % (m['directory'], m.get('s32', 0), fmt_bytes(m.get('s32_bytes', 0)), verdicts))
    add('')
    add('### 4.5 空目錄與未納入雜湊的檔案')
    add('')
    add('- 空目錄：%s。' % ('、'.join('`%s`' % d for d in facts['empty_directories']) or '無'))
    add('- 下列檔案不是資源命名空間的一部分（執行檔、設定、紀錄、工具包、登入器），只列數量、不納入資源集合雜湊。根目錄只納入 `shade.dat`、`Emblem769.dat`、`TW13081901.txt`，`launcher/` 只納入 `TW13081901.pak`：')
    add('')
    add('| 位置 | 副檔名 | 檔數 |')
    add('|---|---|---:|')
    for item in doc['unbound_files']:
        add('| `%s` | %s | %s |' % (item['top'], '.' + item['ext'] if item['ext'] else '（無）', item['files']))
    add('')
    add('## 5. 重跑與核對')
    add('')
    add('```powershell')
    add('python -X utf8 維護工具/resource-matrix/resource_matrix.py           # 重產')
    add('python -X utf8 維護工具/resource-matrix/resource_matrix.py --check   # 重算並與入庫版本比對，不同則 exit 1')
    add('python -X utf8 維護工具/resource-matrix/resource_matrix.py --self-test')
    add('```')
    add('')
    add('`--check` 失敗代表客戶端資源或工具規則變了；重產後在 PR 說明差異來源。')
    add('')
    return '\n'.join(out)


def outputs(root):
    document, members_text, exceptions_text = build(scan(root))
    return {
        OUT_JSON: json.dumps(document, ensure_ascii=False, indent=1) + '\n',
        OUT_MD: render_markdown(document),
        OUT_EXC: exceptions_text,
        OUT_MEMBERS: members_text,
    }


# --------------------------------------------------------------------------
# 自測（不讀客戶端）
# --------------------------------------------------------------------------
def self_test():
    assert l1_decode(bytes(range(32))) == L1_KNOWN_ANSWER
    assert l1_decode(bytes(range(11)))[8:] == bytes(range(8, 11))

    def spr(frames, blocks, body, palette=b''):
        out = bytearray(palette)
        out.append(len(frames))
        for bounds, refs in frames:
            out += struct.pack('<hhhhHHH', *bounds, len(refs))
            for a, b, t, i in refs:
                out += struct.pack('<bbBH', a, b, t, i)
        out += struct.pack('<I', len(blocks))
        for o in blocks:
            out += struct.pack('<I', o)
        return bytes(out + struct.pack('<I', 0) + body)

    frame = ((0, 0, 23, 23, 0, 0), [(0, 0, 0, 0)])
    assert gate_spr(spr([frame], [0], b'\0')) == 'spr.container_ok_rgb555'
    assert gate_spr(spr([frame], [0], b'\0', b'\xff\x01\x00\x00')) == 'spr.container_ok_palette'
    assert gate_spr(spr([frame], [1], b'\0\0')) == 'spr.prefix_before_first_block'
    assert gate_spr(spr([frame], [0], b'')) == 'spr.block_offset_outside_body'
    assert gate_spr(spr([((0, 0, 23, 23, 0, 0), [(0, 0, 0, 3)])], [0], b'\0')) == 'spr.block_reference_outside_table'
    assert gate_spr(spr([((5, 0, 1, 23, 0, 0), [])], [], b'')) == 'spr.unlisted_empty_frame_marker'
    assert gate_spr(spr([((5, 0, 1, 23, 0, 0), [(0, 0, 0, 0)])], [0], b'\0')) == 'spr.reversed_frame_bounds_with_blocks'
    assert gate_spr(spr([((192, 120, -128, -120, 0xffff, 0), [])], [], b'')) == 'spr.container_ok_rgb555'
    assert gate_spr(spr([frame], [], b'\0')) == 'spr.data_without_blocks'
    assert gate_spr(b'\0') == 'spr.zero_frame_count'
    assert gate_spr(b'SP2x') == 'spr.sp2_spx_or_fe_marker' and gate_spr(b'\xfe') == 'spr.sp2_spx_or_fe_marker'
    assert gate_spr(spr([frame], [0], b'\0')[:9]) == 'spr.truncated'

    def til(offsets, end, body):
        return struct.pack('<I', len(offsets)) + b''.join(struct.pack('<I', o) for o in offsets) + struct.pack('<I', end) + body

    counter = collections.Counter()
    assert gate_til(til([0, 2], 4, bytes([0, 9, 3, 9])), counter) == 'til.container_ok' and counter == {0: 1, 3: 1}
    assert gate_til(til([0, 2], 4, bytes([0, 9, 5, 9]))) == 'til.unsupported_block_type_5'
    assert gate_til(til([0, 4], 4, bytes([1, 9, 9, 9]))) == 'til.container_ok_empty_tail'
    assert gate_til(til([4, 0], 4, bytes(4))) == 'til.empty_slots_not_terminal_suffix'
    assert gate_til(til([1], 4, bytes(4))) == 'til.prefix_before_first_block'
    assert gate_til(til([0], 3, bytes(4))) == 'til.trailing_data_after_end_offset'
    assert gate_til(til([0], 9, bytes(4))) == 'til.end_offset_exceeds_body'
    assert gate_til(struct.pack('<I', 0)) == 'til.zero_tile_count'
    assert gate_til(b'\x78\x9c\xff\xff') == 'til.compressed_container'

    assert gate_tbt(bytes([0, 0, 2, 1, 1, 0, 2, 1, 0, 2, 0])) == 'tbt.ok'
    assert gate_tbt(bytes([0, 0, 2, 1, 1, 1, 0])) == 'tbt.odd_byte_skip'
    assert gate_tbt(bytes([0, 0, 2, 1, 1, 0, 1, 0, 0x80])) == 'tbt.pixel_high_bit'
    assert gate_tbt(bytes([0, 0, 2, 1, 0, 9])) == 'tbt.trailing_bytes'
    assert gate_tbt(bytes([0, 0, 2, 1])) == 'tbt.truncated' and gate_tbt(bytes([0, 0, 0, 1])) == 'tbt.empty_image'

    img = struct.pack('<hhHH', 2, 1, 0, 0)
    assert gate_img(img + bytes(4)) == 'img.ok_flag0'
    assert gate_img(struct.pack('<hhHH', 2, 1, 1, 0) + bytes(4)) == 'img.ok_flag1'
    assert gate_img(img + bytes(3)) == 'img.payload_size_mismatch'
    assert gate_img(img + bytes([0, 0, 0, 0x80])) == 'img.rgb555_high_bit'
    assert gate_img(struct.pack('<hhHH', 2, 1, 2, 0) + bytes(4)) == 'img.unknown_flag_2'
    assert gate_img(struct.pack('<hhHH', -2, 1, 0, 0)) == 'img.non_positive_dimensions'
    assert gate_png(b'\x89PNG\r\n\x1a\n' + bytes(4) + b'IHDR' + bytes(17)) == 'png.signature_ok'
    assert gate_png(b'\x89PNX' + bytes(40)) == 'png.bad_signature_89504e58'

    def wav(chunks, fix=True):
        out = bytearray(b'RIFF\0\0\0\0WAVE')
        for tag, payload, pad in chunks:
            out += tag + struct.pack('<I', len(payload)) + payload + (b'\0' if pad and len(payload) & 1 else b'')
        if fix:
            out[4:8] = struct.pack('<I', len(out) - 8)
        return bytes(out)

    pcm = struct.pack('<HHIIHH', 1, 1, 22050, 22050, 1, 8)
    assert gate_wav(wav([(b'fmt ', pcm, True), (b'data', b'\1\2', True)])) == 'wav.ok'
    assert gate_wav(wav([(b'fmt ', pcm, True), (b'data', b'\1\2\3', False)])) == 'wav.ok_terminal_unpadded_data'
    assert gate_wav(wav([(b'data', b'\1\2', True)])) == 'wav.data_precedes_fmt'
    assert gate_wav(wav([(b'fmt ', struct.pack('<HHIIHH', 2, 1, 22050, 22050, 1, 8), True)])) == 'wav.unsupported_encoding_2'
    assert gate_wav(wav([(b'fmt ', struct.pack('<HHIIHH', 1, 1, 22050, 22050, 3, 24), True)])) == 'wav.unsupported_pcm_1ch_24bit'
    assert gate_wav(wav([(b'fmt ', pcm, True)])) == 'wav.missing_data'
    assert gate_wav(wav([(b'fmt ', pcm, True)], fix=False)) == 'wav.riff_size_mismatch'
    assert gate_wav(b'ID3' + bytes(20)) == 'wav.not_riff'

    base = bytes(8192 * 4) + struct.pack('<H', 0) + bytes(4096 * 4) + struct.pack('<i', 0)
    assert gate_s32(base, False) == 's32.ok_layers1to4'
    assert gate_s32(base[:-1], False) == 's32.truncated_layer4'
    full = base + struct.pack('<i', 0) + struct.pack('<i', 0)
    portal = struct.pack('<H', 1) + bytes([2, 65, 66, 1, 2]) + struct.pack('<Hi', 4, -1)
    assert gate_s32(full + portal, False) == 's32.ok_layers1to7'
    assert gate_s32(full + portal + bytes([101]), True) == 's32.ok_layers1to7'
    assert gate_s32(full + portal + bytes([101]), False) == 's32.truncated_layer8'
    assert gate_s32(full + portal + struct.pack('<H', 0x8001) + bytes(10), False) == 's32.ok_layers1to8'
    assert gate_s32(full + portal + struct.pack('<H', 0) + b'x', False) == 's32.ok_layers1to8_unknown_trailing'
    assert gate_s32(base[:-4] + struct.pack('<i', -1), False) == 's32.negative_count_layer4'

    assert classify_member('text', 'abcdefghijklmnop.htm') == ('html', ['name_truncated_20'])
    assert classify_member('text', 'a.htm') == ('unknown', [])
    assert classify_member('text', 'list.spr') == ('list_spr_text', [])
    assert classify_member('sprite', '1234567890123456-.sp') == ('spr', ['name_truncated_20'])
    assert classify_member('sprite', '1.spr') == ('spr', []) and classify_member('tile', '1.xml') == ('xml_tile', [])
    assert classify_member('sprite', 'thumbs.db') == ('junk', []) and classify_member('sprite', 'a.lmp') == ('unknown', [])
    assert status_of('spr', 'spr.container_ok_palette') == 'supported_container_ok'
    assert status_of('spr', 'spr.truncated') == 'rejected_by_decoder_gate'
    assert status_of('html', 'text.ascii_head') == 'selected_members_only'
    index = struct.pack('<I', 1) + struct.pack('<I20sI', 7, b'A.spr', 3)
    assert parse_index(index, False) == [(7, b'A.spr', 3)]

    # 格式支援表引用的 Rust 模組必須存在；關卡照抄的常數要還在原始碼裡。
    modules = SRC / 'modern_lito'
    for name, (_, paths, _) in FORMATS.items():
        for path in paths:
            assert (modules / path).is_file(), '格式 %s 引用的模組不存在：%s' % (name, path)
    tile_source = (modules / 'tile.rs').read_text(encoding='utf-8')
    rust_types = {int(v) for v in re.findall(r'^\s+(\d+) => Ok\(Self::', tile_source, re.M)}
    assert rust_types == TIL_TYPES, 'tile.rs 支援的型別與本工具不同：%s' % sorted(rust_types ^ TIL_TYPES)
    sprite_source = (modules / 'sprite.rs').read_text(encoding='utf-8')
    for needle in ('max_blocks: 16_384', 'max_block_references: 65_536', '"prefix before first block"',
                   '(544, 384, -480, -384, u16::MAX, 0)'):
        assert needle in sprite_source, 'sprite.rs 已變動，請重新對照 gate_spr：' + needle
    assert len(re.findall(r'u16::MAX, 0\)|0x0f0f, 0xf1f1\)', sprite_source.split('fn empty_sentinel')[1].split('}')[0])) \
        == len(SPR_EMPTY_SENTINELS), 'sprite.rs 的空影格標記數量與本工具不同'
    print('self-test ok')


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    assert sys.byteorder == 'little'
    if args.self_test:
        self_test()
        return 0
    root = Path(os.environ.get('LITO_CLIENT_ROOT') or DEFAULT_ROOT)
    if not (root / 'Sprite.idx').is_file():
        raise SystemExit('找不到客戶端資源：%s（可用 LITO_CLIENT_ROOT 指定）' % root)
    produced = outputs(root)
    if args.check:
        drift = [str(p.relative_to(REPO)) for p in (OUT_JSON, OUT_MD, OUT_EXC)
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
