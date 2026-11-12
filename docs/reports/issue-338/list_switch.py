# Read-only inventory of list.spr attribute 100 (native 43E377: count, then count*(x0 y0 x1 y1)).
# usage: python -I list_switch.py <list.spr> <out.json>
import sys, re, json, hashlib

raw = open(sys.argv[1], "rb").read()
text = raw.decode("latin-1")
sprite = None
rows = []
header = re.compile(r"^#(\d+)\s+(\d+)(?:=(\d+))?\s*(.*)$")
attr = re.compile(r"\b100\.([^()\s]*)\(([^)]*)\)")
for line_no, line in enumerate(text.splitlines(), 1):
    m = header.match(line)
    if m:
        sprite = {"id": int(m.group(1)), "frames": int(m.group(2)), "file": m.group(3), "label": m.group(4).strip()[:40], "line": line_no}
    for a in attr.finditer(line):
        nums = [int(v) for v in re.findall(r"-?\d+", a.group(2))]
        ok = bool(nums) and len(nums) == 1 + 4 * nums[0]
        rows.append({"sprite": sprite and sprite["id"], "label": sprite and sprite["label"], "line": line_no,
                     "name": a.group(1), "body": a.group(2).strip(), "count": nums[0] if nums else None,
                     "rects": [nums[1 + 4 * i: 5 + 4 * i] for i in range(nums[0])] if ok else None, "well_formed": ok})
out = {"list_spr": {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest().upper()},
       "attribute_100_rows": len(rows), "malformed": sum(not r["well_formed"] for r in rows), "rows": rows}
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(out, ensure_ascii=True, indent=1))
print(len(rows), "rows; malformed", out["malformed"])
for r in rows[:25]:
    print(r["sprite"], r["label"].encode("ascii", "replace").decode(), r["name"], r["body"])
odd = [r for r in rows if r["rects"] and any(v % 2 for rc in r["rects"] for v in (rc[0], rc[2]))]
print("rows with odd x offsets", len(odd))
