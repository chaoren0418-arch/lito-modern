# Read-only inventory of list.spr body-linked layers: 101 (shadow), 105
# (attachments) and the 104 draw-flag word of every body, shadow and attachment
# sprite, plus rows that combine 100 (switch) with a linked layer.
# usage: python -I list_layers.py <list.spr> <out.json>
import sys, re, json, hashlib, collections

raw = open(sys.argv[1], "rb").read()
text = raw.decode("latin-1")
header = re.compile(r"^#(\d+)\s+(\d+)(?:=(\d+))?\s*(.*)$")
attr = re.compile(r"\b(\d+)\.([^()\s]*)\(([^)]*)\)")
sprites = {}
current = None
for line_no, line in enumerate(text.splitlines(), 1):
    m = header.match(line)
    if m:
        current = {"id": int(m.group(1)), "line": line_no, "label": m.group(4).strip()[:32], "attrs": collections.defaultdict(list)}
        # A later duplicate header replaces the earlier row, as a table load would.
        sprites[current["id"]] = current
    if current is None:
        continue
    for a in attr.finditer(line):
        current["attrs"][int(a.group(1))].append(a.group(3).strip())


def numbers(body):
    return [int(v) for v in re.findall(r"-?\d+", body)]


def flags(sprite_id):
    row = sprites.get(sprite_id)
    if row is None:
        return None
    values = row["attrs"].get(104)
    if not values:
        return 0
    n = numbers(values[-1])
    return n[0] if len(n) == 1 else "malformed"


body_flags = collections.Counter()
shadow_flags = collections.Counter()
attachment_flags = collections.Counter()
attachment_counts = collections.Counter()
malformed = []
examples = collections.defaultdict(list)
switch_with_layers = []
bodies_with_attachments = 0
bodies_with_shadow = 0
for sid, row in sorted(sprites.items()):
    a = row["attrs"]
    if 104 in a:
        body_flags[str(flags(sid))] += 1
    shadow = None
    if 101 in a:
        n = numbers(a[101][-1])
        if len(n) != 1:
            malformed.append({"sprite": sid, "attr": 101, "body": a[101][-1]})
        else:
            shadow = n[0]
            bodies_with_shadow += 1
            f = str(flags(shadow))
            shadow_flags[f] += 1
            if len(examples["shadow:" + f]) < 6:
                examples["shadow:" + f].append([sid, shadow])
    attachments = []
    if 105 in a:
        n = numbers(a[105][-1])
        if not n or len(n) != n[0] + 1:
            malformed.append({"sprite": sid, "attr": 105, "body": a[105][-1]})
        else:
            attachments = n[1:]
            bodies_with_attachments += 1
            attachment_counts[str(n[0])] += 1
            for t in attachments:
                f = str(flags(t))
                attachment_flags[f] += 1
                if len(examples["attachment:" + f]) < 6:
                    examples["attachment:" + f].append([sid, t])
    if 100 in a and (shadow is not None or attachments):
        switch_with_layers.append({"sprite": sid, "label": row["label"], "shadow": shadow,
                                   "shadow_flags": None if shadow is None else flags(shadow),
                                   "attachments": attachments, "attachment_flags": [flags(t) for t in attachments],
                                   "switch": a[100][-1]})
# Which body rows with a linked layer the layer plan accepts, before and after
# #339. Body word: 0 or 1 (unchanged; the cube exception for 128 is ignored
# here). Before: attachment word 0 or 8 and shadow sprite word 0. After:
# attachment word 0, 1, 8 or 128; the shadow sprite word is not read (4E0233).
plan = collections.Counter()
newly = []
for sid, row in sorted(sprites.items()):
    a = row["attrs"]
    if 101 not in a and 105 not in a:
        continue
    n101 = numbers(a[101][-1]) if 101 in a else []
    n105 = numbers(a[105][-1]) if 105 in a else []
    shadow_word = flags(n101[0]) if len(n101) == 1 else None
    words = [flags(t) for t in n105[1:]] if n105 and len(n105) == n105[0] + 1 else []
    if (101 in a and shadow_word is None) or any(w is None for w in words):
        plan["layer_sprite_absent"] += 1
        continue
    body_ok = flags(sid) in (0, 1)
    before = body_ok and all(w in (0, 8) for w in words) and (101 not in a or shadow_word == 0)
    after = body_ok and all(w in (0, 1, 8, 128) for w in words)
    plan["rows_with_layers"] += 1
    plan["accepted_before"] += before
    plan["accepted_after"] += after
    if after and not before:
        newly.append(sid)
    if not after:
        plan["refused_body_word" if not body_ok else "refused_attachment_word"] += 1
out = {
    "layer_plan_acceptance": dict(sorted(plan.items())),
    "newly_accepted_bodies": newly,
    "list_spr": {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest().upper()},
    "sprites": len(sprites),
    "bodies_with_104": sum(body_flags.values()),
    "body_104_values": dict(sorted(body_flags.items())),
    "bodies_with_shadow": bodies_with_shadow,
    "shadow_sprite_104_values": dict(sorted(shadow_flags.items())),
    "bodies_with_attachments": bodies_with_attachments,
    "attachment_count_histogram": dict(sorted(attachment_counts.items())),
    "attachment_sprite_104_values": dict(sorted(attachment_flags.items())),
    "examples_body_then_layer": dict(sorted(examples.items())),
    "malformed": malformed,
    "switch_rows_with_linked_layers": switch_with_layers,
}
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(out, ensure_ascii=True, indent=1))
for k, v in out.items():
    if k not in ("examples_body_then_layer", "switch_rows_with_linked_layers"):
        print(k, v)
print("examples", json.dumps(out["examples_body_then_layer"]))
print("switch rows with linked layers", len(switch_with_layers))
for r in switch_with_layers[:40]:
    print(r)
