# -*- coding: utf-8 -*-
"""Issue #307 靜態工具：函式層級的保守呼叫閉包＋副作用原語掃描（唯讀；不編譯、不執行專案程式）。

用法: python -I fnclosure.py <src/modern_lito> <mod:fn,mod:fn,...> [--list]

保守（over-approximation）規則：
  * 函式 F（位於模組 M）的候選被呼叫者 = 「F 本體出現過的識別字」中，
    名稱等於候選模組內任一 fn 名稱者；同名方法全部納入（不做型別解析）。
  * 候選模組 = M 自己 ＋ M 整個檔案（不含檔尾 #[cfg(test)]）以 super::X／modern_lito::X／
    use super::{X,...} 提到的所有同層模組。
  * 模組一旦被觸及，其 drop／fmt／eq／default／clone／serialize／deserialize／from／
    deref／hash／cmp／partial_cmp 等隱式呼叫的 trait 方法一併納入。
  * 本體以簡易 lexer 去除註解與字串後做大括號配對。
輸出每個可達函式命中的副作用原語行；沒有命中的函式只計數（--list 時列出）。
"""
import io, os, re, sys

SIDE = re.compile(
    r"std::fs\b|\bfs::|\bFile::|OpenOptions|std::net\b|TcpStream|TcpListener|UdpSocket|"
    r"std::process\b|Command::new|env::var|env::set_var|env::remove_var|temp_dir|current_dir|current_exe|"
    r"thread::spawn|thread::Builder|thread::scope|std::thread\b|\bwindows::|extern\s+\"system\"|\bunsafe\b|"
    r"SystemTime|Instant::now|create_dir|remove_file|remove_dir|std::io::stdin|read_to_end|read_exact|"
    r"waveOut|CreateWindow|RegisterClass|LoadLibrary|GetModuleHandle|OnceLock|LazyLock|static\s+mut\b|"
    r"mpsc::|Mutex|RwLock|Condvar|imports::|std::io::Write|\.write_all\(|\.flush\(|println!|eprintln!|dbg!"
)
IMPLICIT = {"drop", "fmt", "eq", "default", "clone", "serialize", "deserialize", "from", "deref",
            "deref_mut", "hash", "cmp", "partial_cmp", "try_from", "into", "next", "index", "add", "sub"}
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def blank_noncode(text):
    """把註解、字串、字元常值換成空白（保留換行與長度），供大括號配對與識別字掃描。"""
    out, i, n = list(text), 0, len(text)

    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == "r" and re.match(r'r#*"', text[i:i + 12]) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            m = re.match(r'r(#*)"', text[i:])
            close = '"' + m.group(1)
            j = text.find(close, i + len(m.group(0)))
            j = n if j < 0 else j + len(close)
            blank(i + 1, j)
            i = j
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(i + 1, min(j, n))
            i = j + 1
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", text[i:i + 12])
            if m:
                blank(i + 1, i + len(m.group(0)) - 1)
                i += len(m.group(0))
            else:
                i += 1  # lifetime
        else:
            i += 1
    return "".join(out)


def strip_tests(text):
    """只移除 `#[cfg(test)] mod name { ... }` 這種行內測試模組（大括號配對）。
    `#[cfg(test)] mod x;`（外部檔）與其他 cfg(test) 項目保留不動（保守：多掃不少掃）。"""
    code = blank_noncode(text)
    out, pos = [], 0
    for m in re.finditer(r"#\[cfg\(test\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([a-z]+\))?\s+)?mod\s+[A-Za-z_0-9]+\s*\{", code):
        if m.start() < pos:
            continue
        j, depth = m.end() - 1, 0
        while j < len(code):
            if code[j] == "{":
                depth += 1
            elif code[j] == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        out.append(text[pos:m.start()])
        out.append("\n" * text.count("\n", m.start(), j + 1))
        pos = j + 1
    out.append(text[pos:])
    return "".join(out)


def module_refs(code, modules):
    found = set()
    for m in re.finditer(r"(?:(?:super::)+|modern_lito::)([A-Za-z_][A-Za-z0-9_]*)", code):
        if m.group(1) in modules:
            found.add(m.group(1))
    for m in re.finditer(r"(?:(?:super::)+|modern_lito::)\{", code):
        i, depth, j = m.end(), 1, m.end()
        while j < len(code) and depth:
            depth += code[j] == "{"
            depth -= code[j] == "}"
            j += 1
        block, d, cur, items = code[i:j - 1], 0, "", []
        for ch in block:
            if ch == "{":
                d += 1
            elif ch == "}":
                d -= 1
            elif ch == "," and d == 0:
                items.append(cur)
                cur = ""
                continue
            if d == 0:
                cur += ch
        items.append(cur)
        for it in items:
            mm = IDENT.search(it)
            if mm and mm.group(0) in modules:
                found.add(mm.group(0))
    return found


class Mod:
    def __init__(self, name, path, modules):
        with io.open(path, encoding="utf-8", errors="replace") as fh:
            raw = strip_tests(fh.read())
        self.name, self.raw = name, raw
        self.code = blank_noncode(raw)
        self.refs = module_refs(self.code, modules) | {name}
        self.fns = {}  # name -> list of (start,end,line)
        for m in re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", self.code):
            i = m.end()
            depth_paren = 0
            # 找到本體的第一個 '{'（跳過簽章；遇到 ';' 表示只有宣告）
            while i < len(self.code):
                ch = self.code[i]
                if ch in "([":
                    depth_paren += 1
                elif ch in ")]":
                    depth_paren -= 1
                elif ch == ";" and depth_paren == 0:
                    i = -1  # 只有宣告（trait 方法／extern），沒有本體
                    break
                elif ch == "{":
                    break
                i += 1
            if i < 0 or i >= len(self.code):
                continue
            j, depth = i, 0
            while j < len(self.code):
                if self.code[j] == "{":
                    depth += 1
                elif self.code[j] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            line = self.code.count("\n", 0, m.start()) + 1
            self.fns.setdefault(m.group(1), []).append((m.start(), j + 1, line))


def main():
    src = sys.argv[1]
    roots = [tuple(r.split(":")) for r in sys.argv[2].split(",")]
    show = "--list" in sys.argv
    modules = {f[:-3] for f in os.listdir(src) if f.endswith(".rs")}
    cache = {}

    def mod(n):
        if n not in cache:
            cache[n] = Mod(n, os.path.join(src, n + ".rs"), modules)
        return cache[n]

    seen, stack, touched = set(), [], set()
    for m, f in roots:
        if m not in modules or f not in mod(m).fns:
            print("!! root not found: %s:%s" % (m, f))
            continue
        stack.append((m, f))

    def touch(m):
        if m in touched:
            return
        touched.add(m)
        for f in mod(m).fns:
            if f in IMPLICIT:
                stack.append((m, f))

    while stack:
        key = stack.pop()
        if key in seen:
            continue
        seen.add(key)
        m, f = key
        touch(m)
        M = mod(m)
        for (a, b, _line) in M.fns[f]:
            idents = set(IDENT.findall(M.code[a:b]))
            for cand in sorted(M.refs):
                C = mod(cand)
                for name in idents & set(C.fns):
                    if (cand, name) not in seen:
                        stack.append((cand, name))

    hits_total = 0
    by_mod = {}
    for (m, f) in sorted(seen):
        M = mod(m)
        for (a, b, line) in M.fns[f]:
            body = M.code[a:b]
            rows = []
            for k, ln in enumerate(body.splitlines()):
                if SIDE.search(ln):
                    rows.append((line + k, M.raw.splitlines()[line + k - 1].strip()[:160]))
            by_mod.setdefault(m, []).append((f, line, rows))
            hits_total += len(rows)
    print("# roots=%s" % sys.argv[2])
    print("# reachable: %d fn names in %d modules; side-effect lines=%d" % (len(seen), len(by_mod), hits_total))
    for m in sorted(by_mod):
        rows = by_mod[m]
        nh = sum(len(r[2]) for r in rows)
        print("%s\tfns=%d\thits=%d" % (m, len(rows), nh))
        for f, line, hits in rows:
            if hits:
                print("  fn %s @%d" % (f, line))
                for ln, txt in hits:
                    print("      %d: %s" % (ln, txt))
            elif show:
                print("  fn %s @%d" % (f, line))


if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    main()
