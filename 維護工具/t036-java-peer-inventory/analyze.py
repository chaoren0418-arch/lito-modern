"""Static (read-only) closure analysis for T036-9 (#314): Java peer / TLS inventory.

Reads Rust sources under LiTo381-Modern/{src,tests}; never builds or runs anything.
Output: <repo>/scratch/issue-314/analysis.json (scratch/ is untracked)
"""
import re, sys, io, json, collections
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rows as rowsmod

ROOT = Path(__file__).resolve().parents[2]
CRATE = ROOT / "LiTo381-Modern"


def mask(src: str):
    """Return text of same length: comments blanked; string/char contents kept except braces/parens/quotes blanked-safe."""
    out = list(src)
    i, n = 0, len(src)
    strings = []  # (start, end, text)
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = " "
            i = j
        elif c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1; j += 2
                elif src.startswith("*/", j):
                    depth -= 1; j += 2
                else:
                    j += 1
            for k in range(i, j):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        elif c == '"' or (c in "rb" and re.match(r'(?:b?r#*"|b")', src[i:i + 12]) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_"))):
            m = re.match(r'(b?r)(#*)"', src[i:i + 40])
            if m:
                hashes = m.group(2)
                start = i + len(m.group(0))
                end = src.find('"' + hashes, start)
                end = n if end < 0 else end
                close = end + 1 + len(hashes)
            else:
                start = i + (2 if c == "b" else 1)
                j = start
                while j < n and src[j] != '"':
                    j += 2 if src[j] == "\\" else 1
                end = j
                close = j + 1
            strings.append((start, end, src[start:end]))
            for k in range(start, end):
                if out[k] in "{}()":
                    out[k] = "_"
            i = close
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", src[i:i + 12])
            if m:
                for k in range(i + 1, i + len(m.group(0)) - 1):
                    out[k] = "_"
                i += len(m.group(0))
            else:
                i += 1
        else:
            i += 1
    return "".join(out), strings


FN_RE = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")
CALL_RE = re.compile(r"(?<![A-Za-z0-9_!])([A-Za-z_][A-Za-z0-9_]*)\s*(?:::\s*<[^<>()]*>)?\s*\(")
PATH_CALL_RE = re.compile(r"((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)+)([A-Za-z_][A-Za-z0-9_]*)\s*(?:::\s*<[^<>()]*>)?\s*\(")
KEYWORDS = {"if", "while", "for", "match", "loop", "return", "fn", "Some", "Ok", "Err", "None", "assert", "matches", "let", "move", "in", "as", "unsafe", "impl", "where", "pub", "super", "crate", "self", "Self"}

MARKERS = {
    "spawn": re.compile(r"\bCommand::new\b"),
    "tcp_listen": re.compile(r"\bTcpListener::bind\b"),
    "tcp_connect": re.compile(r"\bTcpStream::connect(?:_timeout)?\b"),
    "tls_connect": re.compile(r"\bconnect_and_negotiate\b|\bModernTlsConnection\b|\bSessionController::new(?:_canonical)?\b|\bprobe_profile\b"),
    "fs_write": re.compile(r"\b(?:std::)?fs::write\b|\bFile::create\b|\bOpenOptions::new\b|\bcreate_dir(?:_all)?\b"),
    "fs_delete": re.compile(r"\bremove_file\b|\bremove_dir(?:_all)?\b|\bfs::rename\b"),
    "hwnd": re.compile(r"\bCreateWindowEx[AW]\b|\bshow_canonical_window\b|\bShowWindow\b|\bRegisterClass"),
    "job": re.compile(r"\bOwnedProcessJob\b"),
    "audio": re.compile(r"\bwaveOut|\bWinMM|\bPlaySound"),
}
ENV_RE = re.compile(r"\b(LITO_[A-Z0-9_]+|JAVA_HOME)\b")
CLASS_RE = re.compile(r"\blocal\.lito381\.server\.[A-Za-z0-9_]+")


class FileInfo:
    def __init__(self, path: Path):
        self.path = path
        self.rel = path.relative_to(ROOT).as_posix()
        self.src = path.read_text(encoding="utf-8", errors="replace")
        self.masked, self.strings = mask(self.src)
        self.line_starts = [0]
        for m in re.finditer("\n", self.src):
            self.line_starts.append(m.end())
        self.fns = []  # dict(name, line, start, end, body)
        self.by_name = collections.defaultdict(list)
        self.parents = []  # FileInfo that declare this as module
        self.includers = []
        self.included = []
        self.children = {}
        self.macros = {}
        self._parse()

    def line_of(self, pos):
        import bisect
        return bisect.bisect_right(self.line_starts, pos)

    def _match_brace(self, open_pos):
        depth, i, m, n = 0, open_pos, self.masked, len(self.masked)
        while i < n:
            ch = m[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return i + 1
            i += 1
        return n

    def _parse(self):
        m = self.masked
        impls = []  # (start, end, type)
        for im in re.finditer(r"\bimpl\b([^{;]*)\{", m):
            head = re.sub(r"<[^<>]*>", "", re.sub(r"<[^<>]*>", "", im.group(1)))
            head = head.split(" where ")[0]
            if " for " in head:
                head = head.split(" for ")[-1]
            ids = re.findall(r"[A-Za-z_][A-Za-z0-9_]*", head)
            if ids:
                impls.append((im.start(), self._match_brace(im.end() - 1), ids[-1]))
        self.impls = impls
        for f in FN_RE.finditer(m):
            # find body start: first '{' before a ';' at paren depth 0
            i, depth, n = f.end(), 0, len(m)
            body_open = -1
            while i < n:
                ch = m[i]
                if ch in "(<[":
                    depth += 1 if ch != "<" else 0
                elif ch in ")]":
                    depth -= 1
                elif ch == ";" and depth <= 0:
                    break
                elif ch == "{" and depth <= 0:
                    body_open = i
                    break
                i += 1
            if body_open < 0:
                continue
            end = self._match_brace(body_open)
            # attribute prefix (up to 600 chars back to previous '}' or ';')
            owner = None
            for a, b, ty in impls:
                if a < f.start() < b and (owner is None or a > owner[0]):
                    owner = (a, ty)
            d = dict(name=f.group(1), owner=owner[1] if owner else None, line=self.line_of(f.start()), start=f.start(), end=end,
                     body=self.src_masked_strings(f.start(), end))
            self.fns.append(d)
            self.by_name[d["name"]].append(d)
        for mm in re.finditer(r"macro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)\s*\{", m):
            end = self._match_brace(mm.end() - 1)
            self.macros[mm.group(1)] = self.src_masked_strings(mm.start(), end)

    def src_masked_strings(self, a, b):
        return self.masked[a:b]


def build():
    files = {}
    for base in ("src", "tests"):
        for p in sorted((CRATE / base).rglob("*.rs")):
            fi = FileInfo(p)
            files[fi.rel] = fi
    # module tree
    stems = collections.defaultdict(list)
    for fi in files.values():
        stem = fi.path.stem if fi.path.stem != "mod" else fi.path.parent.name
        stems[stem].append(fi)
    for fi in files.values():
        for mm in re.finditer(r'((?:#\[[^\]]*\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;', fi.masked):
            pa = re.search(r'#\[\s*path\s*=\s*"([^"]+)"\s*\]', mm.group(1))
            pathattr, name = (pa.group(1) if pa else None), mm.group(2)
            cands = []
            d = fi.path.parent
            if pathattr:
                cands.append((d / pathattr))
            else:
                own = d if fi.path.stem in ("mod", "lib", "main") or fi.path.parent.name == "tests" and False else d / fi.path.stem
                cands += [d / f"{name}.rs", d / name / "mod.rs", own / f"{name}.rs", own / name / "mod.rs"]
            for c in cands:
                try:
                    rel = c.resolve().relative_to(ROOT).as_posix()
                except Exception:
                    continue
                if rel in files:
                    fi.children[name] = files[rel]
                    files[rel].parents.append(fi)
                    break
    for fi in files.values():
        for mm in re.finditer(r'include!\s*_\s*"([^"]+\.rs)"', fi.masked.replace("(", "_")):
            try:
                rel = (fi.path.parent / mm.group(1)).resolve().relative_to(ROOT).as_posix()
            except Exception:
                continue
            if rel in files and files[rel] is not fi:
                files[rel].includers.append(fi)
                fi.included.append(files[rel])
    return files, stems


def visible(fi, files, memo={}):
    """Files whose fns are name-visible from fi: include!-siblings/includers, and mod parents
    (when fi reaches them through super::). Plain BFS; over-approximate by design."""
    if fi.rel in memo:
        return memo[fi.rel]
    res, work = [], [fi]
    while work:
        f = work.pop(0)
        if f in res:
            continue
        res.append(f)
        work += f.included + f.includers
        if re.search(r"\bsuper::", f.masked):
            work += [p for p in f.parents if p.path.name not in ("mod.rs", "lib.rs", "main.rs")]
    memo[fi.rel] = res
    return res


def closure(fi, start_bodies, files, stems):
    """Return (reached fns list, combined text)."""
    seen = set()
    texts = []
    work = []
    for b in start_bodies:
        texts.append((fi.rel, 0, "<test>", b))
        work.append((fi, b, None))
    while work:
        f, body, owner_ty = work.pop()
        vis = visible(f, files)
        targets = []
        for m in CALL_RE.finditer(body):
            nm = m.group(1)
            if nm in KEYWORDS:
                continue
            pre = body[max(0, m.start() - 80):m.start()].rstrip()
            kind, ty = "bare", None
            if pre.endswith("."):
                kind = "method"
            elif pre.endswith("::"):
                seg = re.search(r"([A-Za-z_][A-Za-z0-9_]*)\s*(?:::\s*<[^<>]*>\s*)?::$", pre)
                ty = seg.group(1) if seg else None
                kind = "typed" if ty and (ty[0].isupper()) else "path"
                if ty == "Self":
                    ty = owner_ty
            for v in vis:
                for d in v.by_name.get(nm, ()):
                    if kind == "bare" and d["owner"] is not None:
                        continue
                    if kind == "method" and d["owner"] is None:
                        continue
                    if kind == "typed" and ty is not None and d["owner"] != ty:
                        continue
                    if kind == "typed" and ty is None and d["owner"] is None:
                        continue
                    targets.append((v, d))
        for m in PATH_CALL_RE.finditer(body):
            segs = [s.strip() for s in m.group(1).split("::") if s.strip()]
            cands = []
            for s in segs:
                nxt = list(stems.get(s, ()))
                for c0 in cands + [f]:
                    if s in c0.children:
                        nxt.append(c0.children[s])
                for c1 in nxt:
                    if c1 not in cands:
                        cands.append(c1)
            for cand in cands:
                for d in cand.by_name.get(m.group(2), ()):
                    targets.append((cand, d))
        # macro invocations defined in visible files
        for m in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)!\s*[\(\[\{]", body):
            for v in vis:
                if m.group(1) in v.macros:
                    key = (v.rel, "macro", m.group(1))
                    if key not in seen:
                        seen.add(key)
                        texts.append((v.rel, 0, m.group(1) + "!", v.macros[m.group(1)]))
                        work.append((v, v.macros[m.group(1)], None))
        for v, d in targets:
            key = (v.rel, d["start"])
            if key in seen:
                continue
            seen.add(key)
            texts.append((v.rel, d["line"], d["name"], d["body"]))
            work.append((v, d["body"], d["owner"]))
    return texts


def analyze_text(texts):
    marks = collections.defaultdict(list)
    envs = collections.defaultdict(list)
    classes = set()
    modes = set()
    for rel, line, name, body in texts:
        for k, rx in MARKERS.items():
            if rx.search(body):
                marks[k].append(f"{rel}:{line}:{name}")
        for e in ENV_RE.findall(body):
            envs[e].append(f"{rel}:{line}:{name}")
        classes.update(CLASS_RE.findall(body))
    return marks, envs, classes


def main():
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    files, stems = build()
    rws = rowsmod.load()
    out = []
    miss = 0
    for r in rws:
        fi = files.get(r["file"])
        rec = dict(r)
        if fi is None:
            rec["error"] = "file-missing"; miss += 1; out.append(rec); continue
        short = r["name"].split("::")[-1]
        cands = [d for d in fi.by_name.get(short, ())]
        body = None
        attr = ""
        if cands:
            d = min(cands, key=lambda d: abs(d["line"] - r["line"]))
            body = d["body"]
            rec["fn_line"] = d["line"]
            pre = fi.src[max(0, d["start"] - 700):d["start"]]
            k = max(pre.rfind("}\n"), pre.rfind(";\n"))
            attr = pre[k + 1:] if k >= 0 else pre
        else:
            # macro-generated: use the invocation line(s)
            ls = fi.line_starts
            a = ls[r["line"] - 1] if 0 < r["line"] <= len(ls) else 0
            b = ls[min(len(ls) - 1, r["line"] + 12)]
            seg = fi.masked[a:b]
            mm = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)!\s*\(", seg)
            if mm:
                close = seg.find(");")
                body = seg[: close + 2 if close >= 0 else len(seg)]
                rec["macro"] = mm.group(1)
            else:
                rec["error"] = "fn-not-found"; miss += 1; out.append(rec); continue
        ig = re.search(r'#\[ignore(?:\s*=\s*"((?:[^"\\]|\\.)*)")?\s*\]', attr)
        rec["ignore_reason"] = (ig.group(1) or "") if ig else None
        rec["has_ignore_attr"] = bool(ig)
        texts = closure(fi, [body], files, stems)
        marks, envs, classes = analyze_text(texts)
        rec["marks"] = {k: sorted(set(v))[:6] for k, v in marks.items()}
        rec["envs"] = {k: sorted(set(v))[:4] for k, v in envs.items()}
        rec["classes"] = sorted(classes)
        # peer launches: call sites outside the generic launcher fns
        launches, user_classes, lits = [], set(), set()
        for rel, line, name, tb in texts:
            if "Command::new" in tb and name != "<test>" and not re.search(r'\.arg\(\s*"local\.lito381', tb):
                continue  # generic launcher (class passed as parameter)
            if name in ("peer_ready_budget", "sql_peer_ready_budget_includes_healing_without_changing_gameplay_deadlines"):
                continue
            user_classes.update(CLASS_RE.findall(tb))
            lits.update(re.findall(r'"([a-z0-9][a-z0-9\-]{1,40})"', tb))
            for mm in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)::(start[a-z_]*)\s*\(", tb):
                i, depth = mm.end(), 1
                while i < len(tb) and depth:
                    depth += tb[i] == "("
                    depth -= tb[i] == ")"
                    i += 1
                args = tb[mm.end():i - 1]
                strs = re.findall(r'"([^"]*)"', args)
                launches.append(dict(site=f"{rel}:{line}:{name}", ty=mm.group(1), fn=mm.group(2), strs=strs,
                                     dynamic=bool(re.search(r"\b(mode|class|peer_mode|kind)\b", re.sub(r'"[^"]*"', "", args)))))
        rec["launches"] = launches
        rec["user_classes"] = sorted(user_classes)
        rec["lits"] = sorted(lits)
        rec["closure_fns"] = len(texts)
        rec["closure_files"] = sorted({t[0] for t in texts})
        # string literal args that look like peer modes in the test's own + direct text
        rec["body_strings"] = sorted(set(re.findall(r'"([a-z0-9][a-z0-9\-]{2,40})"', body)))[:30]
        # file-level java markers (for false-negative review)
        vis = visible(fi, files)
        rec["file_has_java"] = any(re.search(r"LITO_MODERN_TEST_JAVA|LITO_MODERN_TLS_TEST_FIXTURES|LITO_T043_REFERENCE_JAR|JavaPeer|Command::new", v.masked) for v in vis)
        out.append(rec)
    (ROOT / "scratch" / "issue-314").mkdir(parents=True, exist_ok=True)
    (ROOT / "scratch" / "issue-314" / "analysis.json").write_text(json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")
    print("rows", len(out), "unresolved", miss)
    for r in out:
        if r.get("error"):
            print(" ERR", r["error"], r["target"], r["name"], r["file"], r["line"])


if __name__ == "__main__":
    main()
