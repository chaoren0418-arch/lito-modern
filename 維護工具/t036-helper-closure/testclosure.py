# -*- coding: utf-8 -*-
"""整合測試（tests/*.rs）的靜態 helper 閉合工具：函式層級呼叫閉包（over-approximation）＋副作用原語掃描。

來源：Issue #310（T036-5）。唯讀：只讀 .rs 原始碼文字，不編譯、不執行任何專案程式。
用法（在 repo 根目錄）:
  python -I 維護工具/static-test-closure.py LiTo381-Modern <test 檔 stem,逗號分隔>
         [--chains] [--only <test fn>] [--fns] [--tsv <輸出檔>]
  --chains 列出每個帶副作用原語的可達函式與一條呼叫鏈；--fns 列出全部可達函式；
  --tsv   每個測試一列：target、測試名、行號、可達函式數、原語統計、可達模組、可達模組內的頂層巨集呼叫。

輸出是「候選」，不是裁定：原語以正規表示式比對，`.bind(`／`symlink_metadata`／`OpenOptions`（唯讀開檔）
這類誤報要人工讀該行排除；反過來，工具無法知道「cancel 旗標為真時在碰檔案前就返回」這類動態條件。

解析規則（寧可多連、不可少連）：
  * 名稱依「檔案作用域」解析：同檔定義 → `use` 匯入（含別名、巢狀大括號）→ glob 匯入；都找不到視為 std／外部。
    `#[path = "x.rs"] mod name;` 的子模組以宣告名稱與檔名雙向對應，`super::` 指向宣告它的檔案。
  * `mod::f`／裸 `f`（不論有沒有括號，涵蓋函式當值傳遞）→ 對應模組的自由函式、static／const 初始化式。
  * `Type::m` → 該型別（含別名展開）的 impl 函式；`Self::m` → 目前 impl 型別；
    型別解析不到且不是 std 常見型別（泛型參數、`<X as T>::m` 等）→ 視為方法呼叫。
  * `.m(args)` → 所有「已觸及型別」與其實作的 crate trait 上名為 m、有 self 接收者、
    參數個數相符（數不準時不過濾）的函式，做到不動點。沒有括號的 `.field` 不算呼叫。
  * 型別觸及＝型別名稱出現在任一可達函式（含簽名）內；再對 struct／enum 欄位、別名、
    被點名 trait 的所有實作者做遞移閉包。型別名稱在該檔解析不到時，依名稱觸及全 crate 同名型別。
  * 隱含呼叫：已觸及型別上「外部 trait」的 impl 函式（Drop／Display／From／Iterator／Read／Write／serde…）
    一律視為可達；型別所在檔案的 serde `with=`／`default=` 函式也視為可達。
  * `name!` → crate 內同名 macro_rules! 本體。
  * `#[cfg(test)]` 項目與模組對整合測試不可見，排除。

已知限制（使用時要人工補）：
  * 頂層巨集呼叫（例如 `record!{...}`）產生的函式／impl 看不到；`--tsv` 最後一欄會列出可達模組內有哪些。
  * 以 `#[path]` 把 src 內 `#[cfg(test)]` 模組拉進測試 target 的情況（native_library_regression）不支援，要人工讀。
  * 檔尾的 `unresolved …` 清單是解析不到的名稱，多半是區域 closure／泛型參數；要逐項確認。
"""
import io, os, re, sys, collections

IDENT = r"[A-Za-z_][A-Za-z0-9_]*"

SINKS = [
    ("FS_WRITE", re.compile(r"fs::write\b|File::create|OpenOptions|create_dir|remove_file|remove_dir|fs::rename|fs::copy|set_permissions|\.set_len\(|hard_link|symlink|File::options")),
    ("FS_READ", re.compile(r"\bfs::|File::open|read_dir|\.metadata\(|\.exists\(|\.is_file\(|\.is_dir\(|canonicalize|read_link|std::fs\b|\bFile\b")),
    ("NET", re.compile(r"TcpStream|TcpListener|UdpSocket|std::net::Tcp|std::net::Udp|ToSocketAddrs|rustls|ClientConnection|\.connect\(|\.bind\(")),
    ("PROC", re.compile(r"Command::new|std::process\b|process::exit|process::abort|process::id|\bCommand\b")),
    ("ENV", re.compile(r"env::var|env::set_var|env::remove_var|env::vars|current_dir|current_exe|temp_dir|env::args|home_dir|std::env\b")),
    ("THREAD", re.compile(r"thread::spawn|thread::Builder|thread::scope|thread::sleep|std::thread\b|\bmpsc::|\.spawn\(")),
    ("WIN", re.compile(r"\bwindows::|\bunsafe\b|extern\s+\"")),
    ("TIME", re.compile(r"SystemTime|Instant::now")),
    ("STDIO", re.compile(r"stdin\(|stdout\(|stderr\(|println!|eprintln!|print!|eprint!|dbg!")),
]

STD_TYPES = set("""Vec String Box Arc Rc Some None Ok Err Option Result HashMap BTreeMap HashSet BTreeSet VecDeque
BinaryHeap Instant Duration SystemTime Path PathBuf File OpenOptions Mutex RwLock Cell RefCell Condvar Ordering Cow
PhantomData SocketAddr SocketAddrV4 SocketAddrV6 IpAddr Ipv4Addr Ipv6Addr TcpStream TcpListener UdpSocket Command
Stdio OnceLock LazyLock OnceCell AtomicBool AtomicU8 AtomicU16 AtomicU32 AtomicU64 AtomicUsize AtomicI32 AtomicI64
AtomicIsize Weak Pin NonNull NonZeroU8 NonZeroU16 NonZeroU32 NonZeroU64 NonZeroUsize Wrapping Reverse Range
RangeInclusive Bound Layout ManuallyDrop MaybeUninit Cursor BufReader BufWriter ErrorKind OsString OsStr CString
CStr Utf8Error Sha256 Value Number Map Sender Receiver SyncSender Builder JoinHandle Thread Barrier Shutdown
Entry Component Formatter Arguments Infallible Poll Waker Context Hasher DefaultHasher RandomState Iterator
Any TypeId Location Backtrace Char Decoder Encoder ZlibDecoder ZlibEncoder DeflateDecoder Compression GzDecoder
u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 bool char str""".split())

LOWER_ROOTS = ("self", "super", "crate", "modern_lito", "lito_modern")


def mask(src):
    """把註解、字串、字元常值的內容換成空白（保留長度與換行）。"""
    out = list(src)
    i, n = 0, len(src)

    def blank(a, b):
        for k in range(a, min(b, n)):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == '"' or (c in "rb" and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")) and re.match(r'(?:br|rb|r|b)#*"', src[i:i + 12])):
            m = re.match(r'(br|rb|r|b)?(#*)"', src[i:i + 12])
            prefix, hashes = m.group(1) or "", m.group(2)
            start = i + m.end()
            if "r" in prefix:
                end_tok = '"' + hashes
                j = src.find(end_tok, start)
                j = n if j < 0 else j
                blank(start, j)
                i = j + len(end_tok)
            else:
                j = start
                while j < n and src[j] != '"':
                    j += 2 if src[j] == "\\" else 1
                blank(start, j)
                i = j + 1
        elif c == "'":
            if i + 1 < n and src[i + 1] == "\\":
                j = src.find("'", i + 3)
                j = n if j < 0 else j
                blank(i + 1, j)
                i = j + 1
            elif i + 2 < n and src[i + 2] == "'":
                blank(i + 1, i + 2)
                i += 3
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def match_brace(text, i):
    depth, n = 0, len(text)
    while i < n:
        ch = text[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def item_end(text, i):
    n = len(text)
    depth = 0
    while i < n:
        ch = text[i]
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        elif ch == ";" and depth <= 0:
            return i + 1
        elif ch == "{":
            return match_brace(text, i)
        i += 1
    return n


CFG_TEST = re.compile(r"#\[cfg\((?:test|all\(test[^\]]*)\)\]")
ATTR = re.compile(r"\s*#\[[^\]]*\]")


def strip_cfg_test(masked, orig, excluded_mods):
    out = masked
    pos = 0
    while True:
        m = CFG_TEST.search(out, pos)
        if not m:
            break
        j = m.end()
        attrs_start = j
        while True:
            a = ATTR.match(out, j)
            if not a:
                break
            j = a.end()
        attr_text = orig[attrs_start:j]
        e = item_end(out, j)
        item = out[j:e]
        mm = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(%s)\s*;" % IDENT, item)
        if mm:
            p = re.search(r'path\s*=\s*"([^"]+)"', attr_text)
            excluded_mods.add(os.path.splitext(os.path.basename(p.group(1)))[0] if p else mm.group(1))
        out = out[:m.start()] + re.sub(r"[^\n]", " ", out[m.start():e]) + out[e:]
        pos = e
    return out


def expand_use(tree):
    """`a::b::{c, d::{e, f as g}, h::*}` → [(['a','b','c'],None), ...]"""
    tree = tree.strip()
    res = []

    def rec(prefix, s):
        s = s.strip()
        if not s:
            return
        # split top-level commas
        parts, depth, cur = [], 0, ""
        for ch in s:
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
            if ch == "," and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += ch
        parts.append(cur)
        for p in parts:
            p = p.strip()
            if not p:
                continue
            b = p.find("{")
            if b >= 0:
                head = [x for x in re.split(r"\s*::\s*", p[:b].strip()) if x]
                rec(prefix + head, p[b + 1:p.rfind("}")])
            else:
                alias = None
                m = re.match(r"(.*?)\s+as\s+(%s|_)$" % IDENT, p, flags=re.S)
                if m:
                    p, alias = m.group(1), m.group(2)
                segs = [x.strip() for x in re.split(r"\s*::\s*", p.strip()) if x.strip()]
                res.append((prefix + segs, alias))

    rec([], tree)
    return res


def split_top(text):
    """以最外層逗號切開；回傳 None 表示無法可靠計數。"""
    depth, angle, parts, cur = 0, 0, [], ""
    prev = ""
    for ch in text:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "<":
            angle += 1
        elif ch == ">" and prev != "-" and angle > 0:
            angle -= 1
        if ch == "," and depth == 0 and angle == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
        prev = ch
    if cur.strip():
        parts.append(cur)
    return parts


def match_paren(text, i):
    depth, n = 0, len(text)
    while i < n:
        ch = text[i]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def fn_signature(text):
    """`fn name<..>(params)` → (has_self, 非 self 參數個數)；失敗回傳 (None, None)。"""
    i = text.find("(")
    lt = text.find("<")
    if 0 <= lt < i:
        depth, k = 0, lt
        while k < len(text):
            if text[k] == "<":
                depth += 1
            elif text[k] == ">" and text[k - 1] != "-":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        i = text.find("(", k)
    if i < 0:
        return None, None
    e = match_paren(text, i)
    parts = split_top(text[i + 1:e - 1])
    has_self = bool(parts) and bool(re.match(r"\s*(?:&\s*(?:'%s\s*)?)?(?:mut\s+)?self\b" % IDENT, parts[0]))
    return has_self, len(parts) - (1 if has_self else 0)


def call_arity(text, pos):
    """pos 指向方法名稱之後；回傳引數個數、None（不確定）或 -1（不是呼叫）。"""
    m = re.match(r"\s*(::\s*<)?", text[pos:])
    k = pos + m.end()
    if m.group(1):
        depth = 1
        while k < len(text) and depth:
            if text[k] == "<":
                depth += 1
            elif text[k] == ">" and text[k - 1] != "-":
                depth -= 1
            k += 1
        k += len(text[k:]) - len(text[k:].lstrip())
    if k >= len(text) or text[k] != "(":
        return -1
    e = match_paren(text, k)
    args = text[k + 1:e - 1]
    if "<" in args or "|" in args:
        return None
    return len(split_top(args))


class Fn(object):
    __slots__ = ("mod", "name", "impl", "trait", "line", "text", "is_test", "kind", "impl_tids", "trait_tids", "has_self", "nparams")

    def __repr__(self):
        q = (self.impl + "::") if self.impl else ""
        return "%s::%s%s@%d" % (self.mod, q, self.name, self.line)


def head_type(s):
    """impl 標頭片段 → 最外層型別的最後一段識別字。"""
    s = s.strip()
    s = re.sub(r"^(?:&\s*(?:'%s\s*)?(?:mut\s+)?|dyn\s+|\*\s*(?:const|mut)\s+)+" % IDENT, "", s)
    depth, cut = 0, len(s)
    for k, ch in enumerate(s):
        if ch == "<":
            if depth == 0:
                cut = k
                break
            depth += 1
    ids = re.findall(IDENT, s[:cut])
    return ids[-1] if ids else None


class File(object):
    pass


def parse_file(path, mod, excluded_mods, is_testfile=False):
    with io.open(path, encoding="utf-8", errors="replace") as fh:
        orig = fh.read()
    masked = mask(orig)
    if not is_testfile:
        masked = strip_cfg_test(masked, orig, excluded_mods)
    F = File()
    F.mod, F.path, F.masked = mod, path, masked
    F.fns, F.types, F.aliases, F.traits = [], {}, set(), set()
    F.imports, F.globs, F.mod_alias = collections.defaultdict(set), set(), {}
    blocks = []
    for m in re.finditer(r"\b(impl|trait)\b([^{;]*)\{", masked):
        head = m.group(2)
        end = match_brace(masked, m.end() - 1)
        if m.group(1) == "trait":
            t = re.match(r"\s*(%s)" % IDENT, head)
            if t:
                blocks.append((m.start(), end, t.group(1), None, True))
                F.types.setdefault(t.group(1), "")
                F.traits.add(t.group(1))
            continue
        h = re.sub(r"^\s*<(?:[^<>]|<(?:[^<>]|<[^<>]*>)*>)*>", "", head)
        h = re.split(r"\bwhere\b", h)[0]
        trait = None
        if re.search(r"\bfor\b", h):
            tr, ty = re.split(r"\bfor\b", h, 1)
            trait = head_type(tr.replace("!", ""))
            tyname = head_type(ty)
        else:
            tyname = head_type(h)
        if tyname:
            blocks.append((m.start(), end, tyname, trait, False))
    F.blocks = blocks
    for m in re.finditer(r"\b(struct|enum|union|type)\s+(%s)" % IDENT, masked):
        e = item_end(masked, m.end())
        F.types[m.group(2)] = F.types.get(m.group(2), "") + " " + masked[m.end():e]
        if m.group(1) == "type":
            F.aliases.add(m.group(2))

    def enclosing(pos):
        best = None
        for b in blocks:
            if b[0] < pos < b[1] and (best is None or b[0] > best[0]):
                best = b
        return best

    def add(name, start, end, kind):
        f = Fn()
        f.mod, f.name, f.kind = mod, name, kind
        f.line = masked.count("\n", 0, start) + 1
        f.text = masked[start:end]
        f.impl = f.trait = None
        f.is_test = False
        f.has_self, f.nparams = None, None
        if kind == "fn":
            f.has_self, f.nparams = fn_signature(f.text)
            b = enclosing(start)
            if b:
                f.impl, f.trait = b[2], (b[2] if b[4] else b[3])
            pre = masked[max(0, start - 300):start]
            f.is_test = bool(re.search(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?$", pre))
        F.fns.append(f)

    for m in re.finditer(r"\bfn\s+(%s)" % IDENT, masked):
        add(m.group(1), m.start(), item_end(masked, m.end()), "fn")
    for m in re.finditer(r"\bmacro_rules!\s*(%s)\s*[\{\(]" % IDENT, masked):
        add(m.group(1) + "!", m.start(), item_end(masked, m.end() - 1) if masked[m.end() - 1] == "{" else item_end(masked, m.end()), "macro")
    for m in re.finditer(r"\b(?:static|const)\s+(?:mut\s+)?(%s)\s*:" % IDENT, masked):
        add(m.group(1), m.start(), item_end(masked, m.end()), "static")
    for m in re.finditer(r"\buse\s+([^;]+);", masked):
        for segs, alias in expand_use(m.group(1)):
            if not segs:
                continue
            leaf = segs[-1]
            prev = segs[-2] if len(segs) > 1 else None
            if leaf == "*":
                if prev:
                    F.globs.add(prev)
                continue
            if leaf == "self":
                if prev:
                    F.mod_alias[alias or prev] = prev
                continue
            name = alias or leaf
            if name == "_":
                name = leaf
            F.mod_alias.setdefault(name, leaf)
            F.imports[name].add((prev, leaf))
    F.decl_mods = []
    for m in re.finditer(r'(?:#\[path\s*=\s*"([^"]*)"\]\s*)?(?:pub(?:\([^)]*\))?\s+)?\bmod\s+(%s)\s*;' % IDENT, masked):
        stem = m.group(2)
        if m.group(1) is not None:
            stem = os.path.splitext(os.path.basename(orig[m.start(1):m.end(1)]))[0]
        F.decl_mods.append((m.group(2), stem))
    F.serde_fns = set(re.findall(r'(?:with|default|skip_serializing_if)\s*=\s*"(?:[A-Za-z_0-9:]*::)?(%s)"' % IDENT, orig))
    F.top_macros = sorted(set(re.findall(r"(?m)^(%s)!\s*[\(\{\[]" % IDENT, masked)) - {"macro_rules", "thread_local"})
    F.std_bare = set()
    for m in re.finditer(r"\buse\s+std::(fs|env|thread|process|net)::(\{[^}]*\}|%s)" % IDENT, masked):
        for nm in re.findall(IDENT, m.group(2)):
            if nm[0].islower() and nm != "self":
                F.std_bare.add((m.group(1), nm))
    return F


class Graph(object):
    def __init__(self, root, stems):
        src = os.path.join(root, "src")
        excluded = set()
        self.files = {}
        paths = []
        for dp, _dn, fnames in os.walk(src):
            if os.path.basename(dp) == "bin":
                continue
            for fn in fnames:
                if fn.endswith(".rs") and fn != "main.rs":
                    paths.append(os.path.join(dp, fn))
        for p in paths:
            stem = os.path.splitext(os.path.basename(p))[0]
            mod = "modern_lito" if stem == "mod" else stem
            self.files[mod] = parse_file(p, mod, excluded)
        self.excluded = sorted(excluded)
        for x in excluded:
            self.files.pop(x, None)
        self.lib_mods = set(self.files)
        self.decl_alias = collections.defaultdict(list)
        self.cache = {}
        self.parent = {}
        for mod, F in list(self.files.items()):
            for (nm, stem) in F.decl_mods:
                if stem in self.files and stem != mod:
                    if stem != nm:
                        self.decl_alias[nm].append(stem)
                    if mod != "modern_lito":
                        self.parent[stem] = mod
        for s in stems:
            self.files["T:" + s] = parse_file(os.path.join(root, "tests", s + ".rs"), "T:" + s, set(), True)
        # 索引
        self.free = collections.defaultdict(list)      # (mod,name) → fn／static
        self.macros = collections.defaultdict(list)
        self.type_by_name = collections.defaultdict(set)
        self.tdef = {}
        self.is_trait = set()
        self.is_alias = set()
        for mod, F in self.files.items():
            for t, body in F.types.items():
                self.tdef[(mod, t)] = body
                if not mod.startswith("T:"):
                    self.type_by_name[t].add((mod, t))
                if t in F.traits:
                    self.is_trait.add((mod, t))
                if t in F.aliases:
                    self.is_alias.add((mod, t))
            for f in F.fns:
                if f.kind == "macro":
                    self.macros[f.name].append(f)
                elif f.kind == "static" or not f.impl:
                    self.free[(mod, f.name)].append(f)
        # impl 函式索引（型別依檔案作用域解析）
        self.meth = collections.defaultdict(list)          # (tid,name) → fns
        self.ext_impl = collections.defaultdict(list)      # tid → 外部 trait 的 impl 函式
        self.traits_of = collections.defaultdict(set)      # tid → crate trait tids
        self.impls_of = collections.defaultdict(set)       # trait tid → 實作者 tids
        self.ext_types_fns = []                            # impl 於外部型別上的函式（保守：永遠可達候選）
        for mod, F in self.files.items():
            for f in F.fns:
                if f.kind != "fn" or not f.impl:
                    continue
                tids = self.rtype(F, f.impl, None)
                ttids = self.rtype(F, f.trait, None) if f.trait else set()
                ttids = {t for t in ttids if t in self.is_trait}
                f.impl_tids, f.trait_tids = tids, ttids
                if not tids and f.impl in self.type_by_name:
                    tids = set(self.type_by_name[f.impl])
                    f.impl_tids = tids
                if not tids:
                    tids = {("<ext>", f.impl)}
                    self.ext_types_fns.append(f)
                for tid in tids:
                    self.meth[(tid, f.name)].append(f)
                    if f.trait and not ttids and tid not in self.is_trait:
                        self.ext_impl[tid].append(f)
                    for tt in ttids:
                        if tt != tid:
                            self.traits_of[tid].add(tt)
                            self.impls_of[tt].add(tid)
        self.contain = {}
        self.unresolved = collections.Counter()
        self.unresolved_fn = collections.Counter()
        self.unres_by_fn = collections.defaultdict(set)
        self.free_names = {k[1] for k in self.free if not k[0].startswith("T:")}

    # ---- 名稱解析 ----
    def qmods(self, mod, q):
        """限定詞 q 在模組檔 mod 的語境下指向哪些模組檔。"""
        F = self.files.get(mod)
        if F is None or q is None:
            return []
        if q == "self":
            return [mod]
        if q == "super":
            return [mod, self.parent.get(mod, "modern_lito")]
        if q in ("crate", "modern_lito", "lito_modern"):
            return ["modern_lito"]
        out = []
        t = F.mod_alias.get(q)
        for c in (t, q):
            if c is None:
                continue
            if c in self.lib_mods:
                out.append(c)
            out.extend(self.decl_alias.get(c, ()))
        if not out and q[0].islower() and q not in ("std", "core", "alloc") and self.inline_mod(F, q):
            out.append(mod)
        return list(dict.fromkeys(out))

    def rtype_in(self, mod, name, depth=0):
        F = self.files.get(mod)
        if F is None or depth > 5:
            return set()
        if name in F.types:
            return {(mod, name)}
        out = set()
        for (prev, leaf) in F.imports.get(name, ()):
            for m in self.qmods(mod, prev):
                if m != mod or leaf != name:
                    out |= self.rtype_in(m, leaf, depth + 1)
        if not out:
            for g in F.globs:
                for m in self.qmods(mod, g):
                    if m != mod:
                        out |= self.rtype_in(m, name, depth + 1)
        return out

    def rtype(self, F, name, q, depth=0):
        if name is None:
            return set()
        if q is None:
            return self.rtype_in(F.mod, name)
        out = set()
        for m in self.qmods(F.mod, q):
            out |= self.rtype_in(m, name)
        if not out and q in ("super", "crate", "modern_lito", "lito_modern"):
            out = set(self.type_by_name.get(name, ()))
        return out

    def rfree_in(self, mod, name, depth=0):
        F = self.files.get(mod)
        if F is None or depth > 5:
            return []
        if (mod, name) in self.free:
            return list(self.free[(mod, name)])
        out = []
        for (prev, leaf) in F.imports.get(name, ()):
            for m in self.qmods(mod, prev):
                if m != mod or leaf != name:
                    out += self.rfree_in(m, leaf, depth + 1)
        if not out:
            for g in F.globs:
                for m in self.qmods(mod, g):
                    if m != mod:
                        out += self.rfree_in(m, name, depth + 1)
        return out

    def rfree(self, F, name, q):
        if q is None:
            return self.rfree_in(F.mod, name)
        out = []
        for m in self.qmods(F.mod, q):
            out += self.rfree_in(m, name)
        return out

    def inline_mod(self, F, q):
        key = ("inl", F.mod, q)
        if key not in self.cache:
            self.cache[key] = bool(re.search(r"\bmod\s+%s\s*\{" % re.escape(q), F.masked))
        return self.cache[key]

    def alias_expand(self, tids):
        out, stack = set(), list(tids)
        while stack:
            t = stack.pop()
            if t in out:
                continue
            out.add(t)
            if t in self.is_alias:
                stack.extend(self.contains(t))
        return out

    def contains(self, tid):
        if tid not in self.contain:
            F = self.files.get(tid[0])
            res = set()
            if F is not None:
                body = self.tdef.get(tid, "")
                for m in TOK.finditer(body):
                    quals, name = m.group(2), m.group(3)
                    qs = re.findall(IDENT, re.sub(r"<[^<>]*>", "", quals))
                    res |= self.rtype(F, name, qs[-1] if qs else None)
                    for k, qq in enumerate(qs):
                        res |= self.rtype(F, qq, qs[k - 1] if k else None)
            self.contain[tid] = res
        return self.contain[tid]

    # ---- 每個函式的邊 ----
    def analyze(self, f):
        key = id(f)
        if key in self.cache:
            return self.cache[key]
        F = self.files[f.mod]
        direct, deferred, touched = [], set(), set()
        for m in TOK.finditer(f.text):
            dot, quals, name, bang = m.group(1), m.group(2), m.group(3), m.group(4)
            qs = re.findall(IDENT, re.sub(r"<[^<>]*>", "", quals))
            q = qs[-1] if qs else None
            if bang:
                direct.extend(self.macros.get(name + "!", []))
                continue
            if not dot and not qs and m.start(3) >= 2 and re.search(r">\s*::\s*$", f.text[max(0, m.start(3) - 12):m.start(3)]):
                deferred.add((name, None, False))  # `<X as Trait>::name`
                continue
            if dot:
                if not qs:
                    ar = call_arity(f.text, m.end(3))
                    if ar != -1:
                        deferred.add((name, ar, True))
                continue
            for k, qq in enumerate(qs):
                touched |= self.rtype(F, qq, qs[k - 1] if k else None)
            tids = self.rtype(F, name, q) if (q is None or q[0].islower()) else set()
            touched |= tids
            if q is None or q[0].islower():
                fr = self.rfree(F, name, q)
                direct.extend(fr)
                if not tids and not fr and q is None and name not in STD_TYPES:
                    if name in self.type_by_name and name[0].isupper():
                        self.unresolved[(f.mod, name)] += 1
                        self.unres_by_fn[key].add(name)
                        touched |= self.type_by_name[name]  # 解析不到：依名稱全部觸及
                    elif name in self.free_names and name[0].islower() and f.text[m.end(3):m.end(3) + 1] == "(" and name != f.name:
                        self.unresolved_fn[(f.mod, name)] += 1
                        self.unres_by_fn[key].add(name + "()")
                continue
            # Type::name
            if q == "Self":
                qt = set(getattr(f, "impl_tids", set()) or ())
                if not qt and f.impl:
                    qt = {("<ext>", f.impl)}
            else:
                qt = self.rtype(F, q, qs[-2] if len(qs) > 1 else None)
            if qt:
                found = False
                for t in self.alias_expand(qt):
                    for g in self.meth.get((t, name), []):
                        direct.append(g)
                        found = True
                    if t in self.is_trait:
                        deferred.add((name, None, False))
                    for tt in self.traits_of.get(t, ()):
                        direct.extend(self.meth.get((tt, name), []))
                if not found:
                    pass
            elif q not in STD_TYPES or q in self.type_by_name:
                deferred.add((name, None, False))
                if q in self.type_by_name:
                    self.unresolved[(f.mod, q)] += 1
        self.cache[key] = (direct, deferred, touched)
        return self.cache[key]

    def sink_hits(self, f):
        key = ("s", id(f))
        if key not in self.cache:
            hits = []
            extra = self.files[f.mod].std_bare
            for k, line in enumerate(f.text.split("\n")):
                cats = [c for c, rx in SINKS if rx.search(line)]
                for (stdmod, nm) in extra:
                    if re.search(r"(?<![\w:.])%s\s*\(" % re.escape(nm), line):
                        cats.append({"fs": "FS_READ", "env": "ENV", "thread": "THREAD", "process": "PROC", "net": "NET"}[stdmod] + "*")
                if cats:
                    hits.append((f.line + k, cats))
            self.cache[key] = hits
        return self.cache[key]

    def closure(self, rootfn):
        seen = {id(rootfn): rootfn}
        parent = {id(rootfn): None}
        touched, deferred = set(), set()
        queue = collections.deque([rootfn])
        is_t = rootfn.mod

        def push(g, via):
            if g.mod.startswith("T:") and g.mod != is_t:
                return
            if id(g) not in seen:
                seen[id(g)] = g
                parent[id(g)] = via
                queue.append(g)

        def cands(o, d):
            name, ar, dot = d
            for g in self.meth.get((o, name), []):
                if dot and g.has_self is False:
                    continue
                if dot and ar is not None and g.nparams is not None and g.has_self and g.nparams != ar:
                    continue
                yield g

        tparent = {}
        while queue:
            f = queue.popleft()
            direct, dfr, tch = self.analyze(f)
            for g in direct:
                push(g, f)
            new_def = dfr - deferred
            deferred |= new_def
            stack = [t for t in tch if t not in touched]
            new_types = []
            while stack:
                t = stack.pop()
                if t in touched:
                    continue
                touched.add(t)
                tparent[t] = f
                new_types.append(t)
                stack.extend(self.contains(t))
                if t in self.is_trait:
                    stack.extend(self.impls_of.get(t, ()))
            for t in new_types:
                TF = self.files.get(t[0])
                if TF is not None:
                    for nm in TF.serde_fns:
                        for g in self.free.get((t[0], nm), []):
                            push(g, f)
                for g in self.ext_impl.get(t, []):
                    push(g, f)
                owners = [t] + list(self.traits_of.get(t, ()))
                for o in owners:
                    for d in deferred:
                        for g in cands(o, d):
                            push(g, f)
            if new_def:
                for t in list(touched):
                    for o in [t] + list(self.traits_of.get(t, ())):
                        for d in new_def:
                            for g in cands(o, d):
                                push(g, f)
                for d in new_def:
                    for g in self.ext_types_fns:
                        if g.name == d[0]:
                            push(g, f)
        return seen, parent, touched


TOK = re.compile(r"(\.\s*)?((?:%s\s*::\s*(?:<[^<>()]*>\s*::\s*)?)*)(%s)(\s*!(?!=))?" % (IDENT, IDENT))


def main():
    root = sys.argv[1]
    stems = sys.argv[2].split(",")
    chains = "--chains" in sys.argv
    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    listfns = "--fns" in sys.argv
    tsv = io.open(sys.argv[sys.argv.index("--tsv") + 1], "w", encoding="utf-8", newline="\n") if "--tsv" in sys.argv else None
    G = Graph(root, stems)
    print("# excluded cfg(test) modules: %d; lib modules: %d; path-declared submodules: %d" % (len(G.excluded), len(G.lib_mods), len(G.parent)))
    for g in G.ext_types_fns:
        if not g.mod.startswith("T:") and G.sink_hits(g):
            print("# impl-on-external-type fn with sink tokens: %r %s" % (g, sorted({c for _l, cc in G.sink_hits(g) for c in cc})))
    for s in stems:
        mod = "T:" + s
        for t in [f for f in G.files[mod].fns if f.is_test]:
            if only and t.name != only:
                continue
            seen, parent, touched = G.closure(t)
            cats_all = collections.Counter()
            rows = []
            for g in seen.values():
                h = G.sink_hits(g)
                if not h:
                    continue
                cs = sorted({c for _l, cc in h for c in cc})
                for c in cs:
                    cats_all[c] += 1
                chain, p = [], parent[id(g)]
                while p is not None:
                    chain.append(repr(p))
                    p = parent[id(p)]
                rows.append((g, cs, h, chain))
            mods = sorted({g.mod for g in seen.values() if not g.mod.startswith("T:")})
            summary = ", ".join("%s=%d" % kv for kv in sorted(cats_all.items())) or "NONE"
            print("## %s::%s  reach=%d fns/%d mods  types=%d  sinks: %s" % (s, t.name, len(seen), len(mods), len(touched), summary))
            tm = sorted({"%s:%s!" % (mm, x) for mm in set(mods) | {tt[0] for tt in touched} if mm in G.files for x in G.files[mm].top_macros})
            if tm:
                print("   top-level macro invocations in reached modules: " + ", ".join(tm))
            un = sorted({"%s:%s" % (g.mod, n) for g in seen.values() for n in G.unres_by_fn.get(id(g), ())})
            if un:
                print("   unresolved names inside this closure: " + ", ".join(un))
            if tsv:
                tsv.write("%s\t%s\t%d\t%d\t%s\t%s\t%s\n" % (s, t.name, t.line, len(seen), summary, ",".join(mods), ",".join(tm)))
            if chains:
                for g, cs, h, chain in sorted(rows, key=lambda r: repr(r[0])):
                    print("   %-28s %s lines=%s" % ("/".join(cs), repr(g), ",".join(str(l) for l, _ in h[:8])))
                    print("        via " + " < ".join(chain[:7]))
            if listfns:
                for g in sorted(seen.values(), key=repr):
                    print("   . " + repr(g))
    if tsv:
        tsv.close()
    print("# unresolved type-like tokens (file, name, count):")
    for (k, v) in sorted(G.unresolved.items()):
        print("#   %s %s %d" % (k[0], k[1], v))
    print("# unresolved bare calls matching a crate free fn name (file, name, count):")
    for (k, v) in sorted(G.unresolved_fn.items()):
        print("#   %s %s %d" % (k[0], k[1], v))


if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    main()
