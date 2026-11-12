# -*- coding: utf-8 -*-
"""Issue #307 靜態工具：列出指定函式本體的摘要（唯讀）。

用法: python -I showfn.py <src/modern_lito> <mod:fn,...> [--body N]
每個函式輸出：位置、行數、命中的副作用原語行、本體中的跨模組引用（super::X::y / X::y）、
以及（--body N）本體前 N 行。供人工逐一閉合呼叫鏈使用。
"""
import io, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnclosure import Mod, SIDE  # noqa: E402


def main():
    src = sys.argv[1]
    want = [tuple(r.split(":")) for r in sys.argv[2].split(",")]
    nbody = int(sys.argv[sys.argv.index("--body") + 1]) if "--body" in sys.argv else 0
    modules = {f[:-3] for f in os.listdir(src) if f.endswith(".rs")}
    cache = {}
    for m, f in want:
        if m not in modules:
            print("!! no module %s" % m)
            continue
        M = cache.setdefault(m, Mod(m, os.path.join(src, m + ".rs"), modules))
        names = sorted(M.fns) if f == "*" else [f]
        for name in names:
            if name not in M.fns:
                print("!! %s:%s not found (non-test part)" % (m, name))
                continue
            for (a, b, line) in M.fns[name]:
                body = M.code[a:b]
                raw_lines = M.raw.splitlines()
                n = body.count("\n") + 1
                hits = [(line + k, raw_lines[line + k - 1].strip()[:170])
                        for k, ln in enumerate(body.splitlines()) if SIDE.search(ln)]
                refs = sorted(set(re.findall(r"(?:super::)?((?:[a-z_][a-z0-9_]*::)+[A-Za-z_][A-Za-z0-9_]*)", body)))
                refs = [r for r in refs if r.split("::")[0] in modules or r.startswith(("std::", "fs::", "windows::"))]
                print("== %s:%s @%d (%d lines) side=%d" % (m, name, line, n, len(hits)))
                for ln, txt in hits:
                    print("   SIDE %d: %s" % (ln, txt))
                if refs:
                    print("   refs: " + ", ".join(refs))
                for k in range(min(nbody, n)):
                    print("   | " + raw_lines[line + k - 1][:230])


if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    main()
