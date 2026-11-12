# -*- coding: utf-8 -*-
"""Issue #307 靜態工具：檢查一組模組 S 是否對模組引用封閉（唯讀）。

用法: python -I setcheck.py <src/modern_lito> <S 以逗號分隔>
輸出: S 內每個模組自身的副作用分類，以及它引用到 S 以外模組的具體路徑（需人工逐條確認）。
若「S 以外的引用」全部是純資料型別／常數／已另行逐函式確認的純函式，
則 S 內函式能觸及的副作用原語只會是 S 內各模組自身列出的分類。
"""
import io, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnclosure import Mod  # noqa: E402
from modcats import CATS  # noqa: E402


def main():
    src, S = sys.argv[1], sys.argv[2].split(",")
    modules = {f[:-3] for f in os.listdir(src) if f.endswith(".rs")}
    Sset = set(S)
    for n in S:
        if n not in modules:
            print("!! no module %s" % n)
            continue
        M = Mod(n, os.path.join(src, n + ".rs"), modules)
        own = {}
        for ln, line in enumerate(M.code.splitlines(), 1):
            for name, rx in CATS:
                if rx.search(line):
                    own.setdefault(name, []).append(ln)
        print("%s  own=[%s]" % (n, ", ".join("%s(%d)" % (k, len(v)) for k, v in own.items()) or "-"))
        for d in sorted(M.refs - Sset - {n}):
            found = set(re.findall(r"\b%s::[A-Za-z_0-9:]+" % re.escape(d), M.code))
            for m in re.finditer(r"\b%s::\{" % re.escape(d), M.code):
                i, depth, j = m.end(), 1, m.end()
                while j < len(M.code) and depth:
                    depth += M.code[j] == "{"
                    depth -= M.code[j] == "}"
                    j += 1
                found.add("%s::{%s}" % (d, re.sub(r"\s+", "", M.code[i:j - 1])))
            print("    OUT -> %s : %s" % (d, "; ".join(sorted(found))[:500] or "(module named only)"))


if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    main()
