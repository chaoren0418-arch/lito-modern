"""Parse docs/reports/issue-54.md classification rows + whitelists. Read-only."""
import re, sys, io, collections
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
R54 = ROOT / "docs" / "reports" / "issue-54.md"


def load():
    text = R54.read_text(encoding="utf-8")
    lines = text.split("\n")
    rows = []
    for i, l in enumerate(lines, 1):
        if not l.startswith("| ") or "| `" not in l:
            continue
        f = [x.strip() for x in l.strip().strip("|").split(" | ")]
        if len(f) < 7 or f[0] in ("Target",):
            continue
        src = f[5].strip("`")
        m = re.match(r"(.+):(\d+)$", src)
        rows.append(dict(target=f[0], name=f[1].strip("`"), ignored=f[2], features=f[3],
                         cls=f[4], file=m.group(1) if m else src, line=int(m.group(2)) if m else 0,
                         reason=" | ".join(f[6:]), r54line=i))
    # whitelists
    wl_lib, wl_tests = set(), set()
    blocks = re.findall(r"```text\n(.*?)```", text, re.S)
    for b in blocks:
        for l in b.strip().split("\n"):
            p = l.split("\t")
            if len(p) == 1 and p[0].startswith("modern_lito::"):
                wl_lib.add(p[0])
            elif len(p) == 4:
                wl_tests.add((p[0], p[1]))
    for r in rows:
        r["white"] = (r["name"] in wl_lib) if r["target"] == "lib" else ((r["target"], r["name"]) in wl_tests)
    return rows


if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    rows = load()
    print(len(rows), sum(r["white"] for r in rows))
    c = collections.Counter((r["target"] == "lib", r["cls"], r["white"]) for r in rows)
    for k, v in sorted(c.items(), key=str):
        print(k, v)
    norm = collections.Counter()
    for r in rows:
        if r["white"]:
            continue
        t = re.sub(r"LiTo381-Modern/\S+", "<p>", r["reason"])
        t = re.sub(r"`[^`]+`", "<c>", t)
        norm[(r["target"] == "lib", r["cls"], t[:150])] += 1
    for k, v in norm.most_common(70):
        print(v, k)
