#!/usr/bin/env python3
"""唯讀盤點客戶端 Sound/*.wav 是否符合 audio::parse_pcm_wav 的嚴格契約（T059／#340）。

只讀取 <client-root>/Sound 內的檔案；不開音效裝置、不播放、不寫客戶端目錄。
規則逐條對照 LiTo381-Modern/src/modern_lito/audio.rs 與
combat_audio::PreparedSound::from_wav（含播放長度上限）。

加 --list-spr 時，另外以獨立的正規表示式（不是 Rust 解析器）統計已解出的
list.spr 內影格音效標記（`[`、`<`、`^`）的形狀與引用的音效編號。

用法：
  python -I -X utf8 audit-shared-sound-wav.py <client-root> [--json <輸出檔>] [--list-spr <list.spr>]
"""
import collections
import hashlib
import json
import os
import re
import struct
import sys

MAX_WAVE_BYTES = 16 * 1024 * 1024
MAX_CHUNKS = 256
MAX_FRAMES = 16 * 1024 * 1024
MAX_SHARED_ID = 10_000  # PreparedSound: 1..10_000
MAX_PLAYBACK_MS = 60_000  # combat_audio::MAX_PLAYBACK_MILLIS
fmt_sizes = collections.Counter()  # 通過解析的 fmt 區塊長度（含之後因其他規則被拒者）


def parse(data):
    """回傳 (format dict, pcm_len, padding, chunk ids)；不合契約時丟 ValueError。"""
    if len(data) > MAX_WAVE_BYTES:
        raise ValueError("input bytes limit")
    if len(data) < 12:
        raise ValueError("truncated header")
    if data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise ValueError("unsupported container")
    if struct.unpack_from("<I", data, 4)[0] + 8 != len(data):
        raise ValueError("RIFF size must match the complete file")
    pos, fmt, pcm, chunks, padding = 12, None, None, [], "Canonical"
    while pos < len(data):
        if len(chunks) == MAX_CHUNKS:
            raise ValueError("chunk count limit")
        if pos + 8 > len(data):
            raise ValueError("truncated chunk header")
        cid = data[pos:pos + 4]
        size = struct.unpack_from("<I", data, pos + 4)[0]
        start, end = pos + 8, pos + 8 + size
        if end > len(data):
            raise ValueError("truncated chunk payload")
        if size & 1 and end == len(data) and cid == b"data":
            padding, nxt = "TerminalUnpaddedData", end
        else:
            if end + (size & 1) > len(data):
                raise ValueError("missing chunk pad")
            nxt = end + (size & 1)
        payload = data[start:end]
        if cid == b"fmt ":
            if fmt is not None:
                raise ValueError("duplicate fmt")
            if len(payload) < 16:
                raise ValueError("short fmt")
            tag, ch, rate, brate, align, bits = struct.unpack_from("<HHIIHH", payload, 0)
            if tag != 1:
                raise ValueError(f"unsupported encoding {tag:#06x}")
            if not (len(payload) == 16 or (len(payload) in (18, 40) and not any(payload[16:]))):
                raise ValueError(f"PCM fmt {len(payload)} bytes is not 16, or 18/40 with zero tail")
            fmt_sizes[len(payload)] += 1
            if ch not in (1, 2) or bits not in (8, 16):
                raise ValueError(f"unsupported PCM layout {ch}ch/{bits}bit")
            if not 1 <= rate <= 192_000:
                raise ValueError("sample rate out of range")
            if align != ch * (bits // 8):
                raise ValueError("block align mismatch")
            if brate != rate * align:
                raise ValueError("byte rate mismatch")
            fmt = {"channels": ch, "sample_rate": rate, "bits": bits, "block_align": align, "byte_rate": brate}
        elif cid == b"data":
            if pcm is not None:
                raise ValueError("duplicate data")
            if fmt is None:
                raise ValueError("data precedes fmt")
            pcm = len(payload)
        chunks.append(cid.decode("latin-1"))
        pos = nxt
    if fmt is None:
        raise ValueError("missing fmt")
    if pcm is None:
        raise ValueError("missing data")
    if pcm % fmt["block_align"]:
        raise ValueError("partial PCM frame")
    if pcm // fmt["block_align"] > MAX_FRAMES:
        raise ValueError("frame count limit")
    return fmt, pcm, padding, chunks


FRAME = re.compile(r"\d+[.,:]\d+:\d+((?:[!>]|[\[\]<^]\d+)+)")
EVENT = re.compile(r"([!>\[\]<^])(\d*)")


def audit_list_spr(path, sound_dir, rejected_ids):
    """影格音效標記統計。同一影格連續的 `[`／`<` 是一個音效的加權候選。"""
    with open(path, "rb") as handle:
        data = handle.read()
    shapes, examples = collections.Counter(), {}
    ids = {"[": set(), "<": set(), "^": set()}
    bank_sizes = {"[": collections.Counter(), "<": collections.Counter(), "^": collections.Counter()}
    mixed = misplaced = 0
    for number, line in enumerate(data.decode("latin-1").splitlines(), 1):
        for match in FRAME.finditer(line):
            events = EVENT.findall(match.group(1))
            cues = [(kind, int(value)) for kind, value in events if kind in "[<^"]
            if not cues:
                continue
            kinds = "".join(kind for kind, _ in events)
            shapes[kinds] += 1
            examples.setdefault(kinds, f"line {number}: {match.group(0)}")
            for kind, value in cues:
                ids[kind].add(value)
            if len({kind for kind, _ in cues}) > 1:
                mixed += 1
            else:
                bank_sizes[cues[0][0]][len(cues)] += 1
            if kinds.lstrip("[<^").strip("!>]") != "":
                misplaced += 1
    references = {}
    for kind, values in ids.items():
        nonzero = sorted(value for value in values if value)
        missing = [value for value in nonzero if not os.path.isfile(os.path.join(sound_dir, f"{value}.wav"))]
        references[kind] = {
            "distinct_ids": len(values),
            "has_zero_candidate": 0 in values,
            "ids_10000_or_more": [value for value in nonzero if value >= MAX_SHARED_ID],
            "missing_wav_count": len(missing),
            "missing_wav_first_40": missing[:40],
            "rejected_by_strict_parser": sorted(set(nonzero) & rejected_ids),
        }
    return {
        "path": path,
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest().upper(),
        "frames_with_audio_markers": sum(shapes.values()),
        "marker_shapes": {kinds: {"frames": count, "example": examples[kinds]} for kinds, count in shapes.most_common()},
        "candidates_per_frame": {kind: dict(sorted(sizes.items())) for kind, sizes in bank_sizes.items()},
        "frames_mixing_marker_kinds": mixed,
        "frames_with_audio_after_non_audio": misplaced,
        "references": references,
    }


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    sound_dir = os.path.join(argv[1], "Sound")
    out = argv[argv.index("--json") + 1] if "--json" in argv else None
    names = sorted(os.listdir(sound_dir))
    numeric = re.compile(r"^(\d+)\.wav$", re.IGNORECASE)
    formats, paddings, chunk_sets = collections.Counter(), collections.Counter(), collections.Counter()
    rejected, non_numeric, out_of_namespace, empty, too_long = [], [], [], [], []
    durations = []
    listing = hashlib.sha256()
    total_bytes = 0
    for name in names:
        path = os.path.join(sound_dir, name)
        if not os.path.isfile(path):
            continue
        match = numeric.match(name)
        if not name.lower().endswith(".wav"):
            non_numeric.append(name)
            continue
        if not match:
            non_numeric.append(name)
            continue
        sound_id = int(match.group(1))
        size = os.path.getsize(path)
        total_bytes += size
        listing.update(f"{name.lower()}:{size}\n".encode())
        if size > MAX_WAVE_BYTES:
            rejected.append({"file": name, "error": "exceeds 16 MiB"})
            continue
        with open(path, "rb") as handle:
            data = handle.read()
        try:
            fmt, pcm, padding, chunks = parse(data)
        except ValueError as error:
            rejected.append({"file": name, "bytes": size, "error": str(error)})
            continue
        if not 1 <= sound_id < MAX_SHARED_ID:
            out_of_namespace.append(name)
        if pcm == 0:
            empty.append(name)
            continue
        millis = -(-pcm * 1000 // fmt["byte_rate"])
        if millis > MAX_PLAYBACK_MS:
            too_long.append({"file": name, "duration_ms": millis})
        durations.append((millis, name))
        formats[f'{fmt["channels"]}ch/{fmt["sample_rate"]}Hz/{fmt["bits"]}bit'] += 1
        paddings[padding] += 1
        chunk_sets[",".join(chunks)] += 1
    durations.sort()
    report = {
        "sound_dir": sound_dir,
        "numeric_wav_files": len(durations) + len(rejected) + len(empty),
        "total_bytes": total_bytes,
        "name_size_listing_sha256": listing.hexdigest().upper(),
        "accepted_nonempty": len(durations),
        "rejected_by_strict_parser": rejected,
        "empty_pcm": empty,
        "outside_shared_id_namespace_1_9999": out_of_namespace,
        "non_numeric_or_non_wav_names": non_numeric,
        "longer_than_playback_bound_ms": {"bound_ms": MAX_PLAYBACK_MS, "files": too_long},
        "formats": dict(formats.most_common()),
        "fmt_chunk_bytes": {str(size): count for size, count in sorted(fmt_sizes.items())},
        "padding": dict(paddings),
        "chunk_layouts": dict(chunk_sets.most_common()),
        "duration_ms": {
            "min": durations[0] if durations else None,
            "median": durations[len(durations) // 2] if durations else None,
            "max": durations[-1] if durations else None,
            "longest_10": durations[-10:][::-1],
        },
    }
    if "--list-spr" in argv:
        rejected_ids = {int(numeric.match(row["file"]).group(1)) for row in rejected}
        report["list_spr"] = audit_list_spr(argv[argv.index("--list-spr") + 1], sound_dir, rejected_ids)
    text = json.dumps(report, ensure_ascii=False, indent=2)
    if out:
        with open(out, "w", encoding="utf-8") as handle:
            handle.write(text)
    summary = dict(report)
    for key in ("rejected_by_strict_parser", "non_numeric_or_non_wav_names", "outside_shared_id_namespace_1_9999", "empty_pcm"):
        summary[key] = {"count": len(report[key]), "first_20": report[key][:20]}
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
