#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T036（#309）跨函式 helper 閉合審查：整合測試的靜態呼叫閉包與副作用原語盤點。

只讀 `LiTo381-Modern/src` 與 `LiTo381-Modern/tests` 的 .rs 原始碼；不編譯、不執行任何測試。
輸出寫到 --out 指定的 JSON（由呼叫者決定路徑，請放 scratch/ 或 docs/reports/issue-N/）。

方法（保守的過度近似，不是編譯器等級的呼叫圖）：
  1. 自寫 lexer＋item parser，索引所有 fn／impl／trait／struct／enum／type／const／static／macro_rules。
  2. 從每個 #[test] fn 出發做 worklist：
     - `Q::name`：Q 是 crate 內型別 → 該型別 impl 裡的 name；Q 是模組 → 該模組的自由項目；
     - `.name(`：所有「已觸及型別」的 inherent 方法 name（固定點）；
     - 裸識別字：同檔項目 → use 匯入 → glob 匯入；
     - `name!`：crate 內 macro_rules；
     - 觸及型別時，納入其定義（欄位型別遞移）與它的所有 trait impl（含 Drop／Default／From…）
       以及那些 trait 的預設方法。
  3. 在閉包內的每個項目找副作用原語（fs 讀／寫、net、process、env、thread、time、Win32／unsafe／FFI、
     全域可變狀態），並記下抵達該項目的見證路徑。
限制見 docs/reports/issue-309.md「方法的限制」。
"""
import argparse
import bisect
import collections
import io
import json
import os
import re
import sys

TOKEN_RE = re.compile(
    r"""
    (?P<ws>\s+)
  | (?P<lc>//[^\n]*)
  | (?P<bc>/\*.*?\*/)
  | (?P<rs>b?r(?P<h>\#*)".*?"(?P=h))
  | (?P<s>b?"(?:[^"\\]|\\.)*")
  | (?P<ch>b?'(?:[^'\\\n]|\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|.))')
  | (?P<lt>'[A-Za-z_][A-Za-z0-9_]*)
  | (?P<id>(?:r\#)?[A-Za-z_][A-Za-z0-9_]*)
  | (?P<num>[0-9][A-Za-z0-9_]*(?:\.[0-9][A-Za-z0-9_]*)?)
  | (?P<p>::|->|=>|[{}()\[\]<>;,.\#!=&|+\-*/%^~?@:$])
  | (?P<other>.)
    """,
    re.X | re.S,
)

KEYWORDS = set(
    """as break const continue crate else enum extern false fn for if impl in let loop match mod move mut
    pub ref return self Self static struct super trait true type unsafe use where while async await dyn
    union macro_rules""".split()
)


class Tok:
    __slots__ = ("k", "v", "line")

    def __init__(self, k, v, line):
        self.k = k
        self.v = v
        self.line = line


def lex(text):
    starts = [0]
    for m in re.finditer("\n", text):
        starts.append(m.end())
    toks = []
    for m in TOKEN_RE.finditer(text):
        k = m.lastgroup
        if k == "h":
            k = "rs"
        if k in ("ws", "lc", "bc", "other"):
            continue
        v = m.group(k)
        line = bisect.bisect_right(starts, m.start())
        if k in ("rs", "s"):
            toks.append(Tok("s", v, line))
        elif k == "ch":
            toks.append(Tok("c", v, line))
        elif k == "lt":
            toks.append(Tok("l", v, line))
        elif k == "id":
            if v.startswith("r#"):
                v = v[2:]
            toks.append(Tok("i", v, line))
        elif k == "num":
            toks.append(Tok("n", v, line))
        else:
            toks.append(Tok("p", v, line))
    return toks


OPEN = {"{": "}", "(": ")", "[": "]"}
CLOSE = {"}", ")", "]"}


def match_close(toks, i):
    """toks[i] is an opening bracket; return index of its closing bracket."""
    depth = 0
    n = len(toks)
    while i < n:
        t = toks[i]
        if t.k == "p":
            if t.v in OPEN:
                depth += 1
            elif t.v in CLOSE:
                depth -= 1
                if depth == 0:
                    return i
        i += 1
    return n - 1


class Item:
    __slots__ = (
        "id", "kind", "name", "file", "modnames", "inline", "impl_type", "trait", "has_self",
        "lo", "hi", "line", "attrs", "is_test", "ignored", "generic_impl", "sig_hi", "testonly", "impl_qual",
    )

    def __init__(self, **kw):
        for k in self.__slots__:
            setattr(self, k, kw.get(k))

    def label(self):
        own = ""
        if self.impl_type or self.trait:
            if self.impl_type and self.trait:
                own = "<%s as %s>::" % (self.impl_type, self.trait)
            else:
                own = "%s::" % (self.impl_type or self.trait)
        return "%s:%d %s%s" % (self.file, self.line, own, self.name)


class FileInfo:
    def __init__(self, rel, toks):
        self.rel = rel
        self.toks = toks
        self.items = []
        self.mod_decls = []  # (name, path_attr or None, inline_path tuple)
        self.imports = collections.defaultdict(set)  # alias -> {(qual, leaf)}
        self.globs = set()  # qual module/type names
        self.modnames = set()
        self.win_imports = set()
        self.test_ranges = []   # token ranges under #[cfg(test)]
        self.parents = set()    # files declaring this file as a module
        self.children = {}      # (name, inline) -> child file

    def in_test_range(self, i):
        return any(a <= i < b for a, b in self.test_ranges)


class Index:
    def __init__(self):
        self.files = {}
        self.items = []

    def add(self, **kw):
        it = Item(id=len(self.items), **kw)
        self.items.append(it)
        self.files[it.file].items.append(it)
        return it


def parse_use(toks, i, fi):
    """toks[i] == 'use'. Record imports; return index after ';'."""
    j = i + 1
    n = len(toks)
    end = j
    depth = 0
    while end < n:
        v = toks[end].v
        if toks[end].k == "p":
            if v == "{":
                depth += 1
            elif v == "}":
                depth -= 1
            elif v == ";" and depth == 0:
                break
        end += 1
    seq = toks[j:end]
    if fi.in_test_range(i):
        return end + 1
    is_win = any(t.k == "i" and t.v == "windows" for t in seq[:3])

    def tree(k, prefix):
        # parse one use-tree starting at seq[k]; return next index
        path = list(prefix)
        while k < len(seq):
            t = seq[k]
            if t.k == "i":
                path.append(t.v)
                k += 1
                if k < len(seq) and seq[k].v == "::":
                    k += 1
                    continue
                alias = path[-1]
                if k < len(seq) and seq[k].k == "i" and seq[k].v == "as":
                    alias = seq[k + 1].v
                    k += 2
                leaf = path[-1]
                qual = path[-2] if len(path) > 1 else None
                if leaf == "self":
                    leaf = qual
                    qual = path[-3] if len(path) > 2 else None
                    if alias == "self":
                        alias = leaf
                if leaf is not None:
                    fi.imports[alias].add((qual, leaf))
                    if is_win:
                        fi.win_imports.add(alias)
                return k
            if t.v == "*":
                if path:
                    fi.globs.add(path[-1])
                    if is_win:
                        fi.win_imports.add("*")
                return k + 1
            if t.v == "{":
                k += 1
                while k < len(seq) and seq[k].v != "}":
                    if seq[k].v == ",":
                        k += 1
                        continue
                    k = tree(k, path)
                return k + 1
            if t.v == "::":
                k += 1
                continue
            k += 1
        return k

    k = 0
    while k < len(seq):
        if seq[k].v == ",":
            k += 1
            continue
        nk = tree(k, [])
        k = nk if nk > k else k + 1
    return end + 1


def attr_text(toks, lo, hi):
    return " ".join(t.v for t in toks[lo:hi])


def is_cfg_test(attr_texts):
    for x in attr_texts:
        # cfg(test) / cfg(all(test, …)) only; cfg(any(test, feature = "…")) can be compiled for
        # integration targets, so it stays in the universe (conservative).
        if x.startswith("cfg (") and re.search(r"\btest\b", x) and "not ( test" not in x and "any (" not in x:
            return True
    return False


def parse_impl_header(toks, i, body):
    """toks[i]=='impl', toks[body]=='{'. Return (trait, type, generic_impl)."""
    hdr = toks[i + 1:body]
    k = 0
    generics = set()
    if hdr and hdr[0].v == "<":
        depth = 0
        prev = None
        while k < len(hdr):
            v = hdr[k].v
            if v == "<":
                depth += 1
            elif v == ">":
                depth -= 1
                if depth == 0:
                    k += 1
                    break
            elif hdr[k].k == "i" and depth == 1 and prev in ("<", ",") and v != "const":
                generics.add(v)
            elif hdr[k].k == "i" and depth == 1 and prev == "const":
                generics.add(v)
            prev = v
            k += 1
    rest = hdr[k:]
    depth = 0
    for_at = None
    where_at = len(rest)
    for q, t in enumerate(rest):
        if t.v == "<":
            depth += 1
        elif t.v == ">":
            depth -= 1
        elif t.k == "i" and depth == 0:
            if t.v == "for" and for_at is None:
                for_at = q
            elif t.v == "where":
                where_at = q
                break
    rest = rest[:where_at]

    def last_path_ident(seq):
        name = None
        qual = None
        depth = 0
        for t in seq:
            if t.v == "<":
                depth += 1
                if depth == 1 and name is not None:
                    break
            elif t.v == ">":
                depth -= 1
            elif t.v in ("(", "["):
                return (name or "*"), qual
            elif t.k == "i" and depth == 0 and t.v not in ("dyn", "mut", "const", "unsafe"):
                qual = name
                name = t.v
        return name, qual

    if for_at is not None:
        trait, _ = last_path_ident(rest[:for_at])
        typ, qual = last_path_ident(rest[for_at + 1:])
    else:
        trait = None
        typ, qual = last_path_ident(rest)
    generic_impl = typ in generics or typ == "*" or typ is None
    return trait, (typ or "*"), generic_impl, qual


def parse_items(idx, fi, lo, hi, inline, impl_type=None, trait=None, generic_impl=False, impl_qual=None):
    toks = fi.toks
    i = lo
    attrs = []
    while i < hi:
        t = toks[i]
        v = t.v
        if t.k == "p" and v == "#":
            j = i + 1
            if j < hi and toks[j].v == "!":
                j += 1
            if j < hi and toks[j].v == "[":
                e = match_close(toks, j)
                attrs.append((j + 1, e))
                i = e + 1
                continue
            i += 1
            continue
        if t.k != "i":
            if t.k == "p" and v in OPEN:
                i = match_close(toks, i) + 1
            else:
                i += 1
            if t.k == "p" and v == ";":
                attrs = []
            continue
        if v == "pub":
            i += 1
            if i < hi and toks[i].v == "(":
                i = match_close(toks, i) + 1
            continue
        if v in ("unsafe", "async", "default"):
            i += 1
            continue
        if v == "extern":
            i += 1
            if i < hi and toks[i].k == "s":
                i += 1
            if i < hi and toks[i].v == "{":
                e = match_close(toks, i)
                idx.add(kind="extern", name="extern-block", file=fi.rel, modnames=None, inline=inline,
                        lo=i, hi=e + 1, line=t.line, attrs=[attr_text(toks, a, b) for a, b in attrs])
                # foreign fn declarations
                parse_items(idx, fi, i + 1, e, inline)
                attrs = []
                i = e + 1
            continue
        if v == "use":
            if is_cfg_test([attr_text(toks, a, b) for a, b in attrs]):
                while i < hi and toks[i].v != ";":
                    i += 1
                i += 1
            else:
                i = parse_use(toks, i, fi)
            attrs = []
            continue
        if v == "mod" and i + 1 < hi and toks[i + 1].k == "i":
            name = toks[i + 1].v
            a_txt = [attr_text(toks, a, b) for a, b in attrs]
            if i + 2 < hi and toks[i + 2].v == "{":
                e = match_close(toks, i + 2)
                if is_cfg_test(a_txt):
                    fi.test_ranges.append((i, e + 1))
                parse_items(idx, fi, i + 3, e, inline + (name,))
                i = e + 1
            else:
                p = None
                for a, b in attrs:
                    if toks[a].v == "path" and toks[a + 1].v == "=":
                        p = toks[a + 2].v
                        p = p[p.index('"') + 1:p.rindex('"')]
                fi.mod_decls.append((name, p, inline, is_cfg_test(a_txt) or fi.in_test_range(i)))
                i += 3
            attrs = []
            continue
        if v == "fn" and i + 1 < hi and toks[i + 1].k == "i":
            name = toks[i + 1].v
            j = i + 2
            depth = 0
            body = None
            has_self = False
            first_paren_done = False
            while j < hi:
                tv = toks[j].v
                if toks[j].k == "p":
                    if tv in ("(", "["):
                        if tv == "(" and depth == 0 and not first_paren_done:
                            e = match_close(toks, j)
                            has_self = any(x.k == "i" and x.v == "self" for x in toks[j:e])
                            first_paren_done = True
                        depth += 1
                    elif tv in (")", "]"):
                        depth -= 1
                    elif tv == "{" and depth == 0:
                        body = j
                        break
                    elif tv == ";" and depth == 0:
                        break
                j += 1
            if body is not None:
                e = match_close(toks, body)
                end = e + 1
            else:
                end = j + 1
            a_txt = [attr_text(toks, a, b) for a, b in attrs]
            is_test = any(x == "test" or x.endswith(":: test") for x in a_txt)
            ignored = any(x == "ignore" or x.startswith("ignore ") for x in a_txt)
            idx.add(kind="fn", name=name, file=fi.rel, modnames=None, inline=inline, impl_type=impl_type,
                    trait=trait, has_self=has_self, lo=i, hi=end, line=t.line, attrs=a_txt,
                    is_test=is_test, ignored=ignored, generic_impl=generic_impl,
                    sig_hi=body if body is not None else end, impl_qual=impl_qual)
            attrs = []
            i = end
            continue
        if v == "impl":
            j = i + 1
            while j < hi and toks[j].v != "{":
                if toks[j].v == ";":
                    break
                j += 1
            if j >= hi or toks[j].v != "{":
                i = j + 1
                attrs = []
                continue
            e = match_close(toks, j)
            if is_cfg_test([attr_text(toks, a, b) for a, b in attrs]):
                fi.test_ranges.append((i, e + 1))
            tr, ty, gen, iq = parse_impl_header(toks, i, j)
            idx.add(kind="implhdr", name=ty, file=fi.rel, modnames=None, inline=inline, impl_type=ty,
                    trait=tr, lo=i, hi=j, line=t.line, attrs=[], generic_impl=gen, impl_qual=iq)
            parse_items(idx, fi, j + 1, e, inline, impl_type=ty, trait=tr, generic_impl=gen, impl_qual=iq)
            attrs = []
            i = e + 1
            continue
        if v == "trait" and i + 1 < hi and toks[i + 1].k == "i":
            name = toks[i + 1].v
            j = i + 2
            while j < hi and toks[j].v not in ("{", ";"):
                j += 1
            if j < hi and toks[j].v == "{":
                e = match_close(toks, j)
                idx.add(kind="trait", name=name, file=fi.rel, modnames=None, inline=inline, lo=i, hi=j,
                        line=t.line, attrs=[])
                parse_items(idx, fi, j + 1, e, inline, impl_type=None, trait=name)
                i = e + 1
            else:
                i = j + 1
            attrs = []
            continue
        if v in ("struct", "enum", "union") and i + 1 < hi and toks[i + 1].k == "i":
            name = toks[i + 1].v
            j = i + 2
            depth = 0
            end = hi
            while j < hi:
                tv = toks[j].v
                if toks[j].k == "p":
                    if tv in ("(", "["):
                        depth += 1
                    elif tv in (")", "]"):
                        depth -= 1
                    elif tv == "{" and depth == 0:
                        end = match_close(toks, j) + 1
                        break
                    elif tv == ";" and depth == 0:
                        end = j + 1
                        break
                j += 1
            idx.add(kind="type", name=name, file=fi.rel, modnames=None, inline=inline, lo=i, hi=end,
                    line=t.line, attrs=[attr_text(toks, a, b) for a, b in attrs])
            attrs = []
            i = end
            continue
        if v in ("type", "const", "static") and i + 1 < hi and toks[i + 1].k == "i":
            k2 = i + 1
            if toks[k2].v == "mut":
                k2 += 1
            if v == "const" and toks[k2].v in ("fn", "unsafe", "extern", "async"):
                i += 1
                continue
            name = toks[k2].v
            j = k2 + 1
            depth = 0
            while j < hi:
                tv = toks[j].v
                if toks[j].k == "p":
                    if tv in OPEN:
                        depth += 1
                    elif tv in CLOSE:
                        depth -= 1
                    elif tv == ";" and depth == 0:
                        break
                j += 1
            kind = "type" if v == "type" else v
            if impl_type is None and trait is None:
                idx.add(kind=kind, name=name, file=fi.rel, modnames=None, inline=inline, lo=i, hi=j + 1,
                        line=t.line, attrs=[attr_text(toks, a, b) for a, b in attrs])
            else:
                idx.add(kind="assoc", name=name, file=fi.rel, modnames=None, inline=inline,
                        impl_type=impl_type, trait=trait, lo=i, hi=j + 1, line=t.line, attrs=[],
                        generic_impl=generic_impl, impl_qual=impl_qual)
            attrs = []
            i = j + 1
            continue
        if v == "macro_rules" and i + 2 < hi and toks[i + 1].v == "!":
            name = toks[i + 2].v
            j = i + 3
            e = match_close(toks, j)
            idx.add(kind="macro", name=name, file=fi.rel, modnames=None, inline=inline, lo=i, hi=e + 1,
                    line=t.line, attrs=[])
            attrs = []
            i = e + 1
            continue
        if i + 1 < hi and toks[i + 1].v == "!" and i + 2 < hi and toks[i + 2].v in OPEN:
            # item-level macro invocation (thread_local!, test generators…)
            e = match_close(toks, i + 2)
            inner = toks[i + 2:e]
            statics = [inner[q + 1].v if inner[q + 1].v != "mut" else inner[q + 2].v
                       for q in range(len(inner) - 2) if inner[q].k == "i" and inner[q].v == "static"]
            if statics:
                for nm in statics:
                    idx.add(kind="static", name=nm, file=fi.rel, modnames=None, inline=inline, lo=i,
                            hi=e + 1, line=t.line, attrs=["macro:" + v])
            else:
                idx.add(kind="macrocall", name=v, file=fi.rel, modnames=None, inline=inline, lo=i,
                        hi=e + 1, line=t.line, attrs=[attr_text(toks, a, b) for a, b in attrs])
            attrs = []
            i = e + 1
            if i < hi and toks[i].v == ";":
                i += 1
            continue
        i += 1


# ---------------------------------------------------------------- primitives
FS_WRITE = {"write", "create", "create_new", "create_dir", "create_dir_all", "remove_file", "remove_dir",
            "remove_dir_all", "rename", "copy", "hard_link", "set_permissions", "symlink_file",
            "symlink_dir", "set_len", "set_modified", "soft_link"}
FS_HEADS = {"fs", "File", "OpenOptions", "DirBuilder"}
OPEN_WRITE_FLAGS = {"write", "append", "create", "create_new", "truncate"}
NET_SEGS = {"net", "TcpStream", "TcpListener", "UdpSocket", "rustls", "ClientConnection", "webpki_roots",
            "ToSocketAddrs", "StreamOwned", "ClientConfig"}
PROC_SEGS = {"process", "Command", "Stdio", "Child"}
PATH_PROBE_METHODS = {"exists", "is_file", "is_dir", "metadata", "symlink_metadata", "read_dir",
                      "canonicalize", "try_exists", "read_link"}
WIN_HINT = re.compile(r"^(?:[A-Z][A-Za-z0-9]+(?:W|A|Ex|ExW|ExA)?)$")


def path_chain(toks, i):
    """Return list of idents of the `a::b::c` chain ending at toks[i]."""
    chain = [toks[i].v]
    j = i
    while j >= 2 and toks[j - 1].v == "::" and toks[j - 2].k == "i":
        chain.append(toks[j - 2].v)
        j -= 2
    chain.reverse()
    return chain


def scan_primitives(fi, it):
    toks = fi.toks
    out = []  # (kind, detail, line)
    lo, hi = it.lo, it.hi
    for i in range(lo, hi):
        t = toks[i]
        if t.k == "s":
            s = t.v
            if re.search(r"[A-Za-z]:\\\\|[A-Za-z]:\\|[A-Za-z]:/|\\\\|/|LITO_|\.(?:json|png|spr|pak|idx|til|s32|seg|xml|txt|bin|jar|class|tbl|img|wav|log|tsv|bmp)\b", s):
                if len(s) < 200:
                    out.append(("literal", s, t.line))
            continue
        if t.k != "i":
            continue
        v = t.v
        nxt = toks[i + 1].v if i + 1 < hi else ""
        prv = toks[i - 1].v if i > lo else ""
        if v == "unsafe":
            out.append(("unsafe", "unsafe", t.line))
            continue
        if v == "static" and nxt == "mut":
            out.append(("global", "static mut", t.line))
            continue
        if nxt == "::":
            continue  # not the chain tail
        if prv == "::":
            chain = path_chain(toks, i)
            head = set(chain[:-1])
            for seg in chain[:-1]:
                for q, leaf in fi.imports.get(seg, ()):
                    head.add(leaf)
                    if q:
                        head.add(q)
            last = chain[-1]
            txt = "::".join(chain)
            if head & FS_HEADS and "windows" not in head:
                if "fs" in head and len(chain) >= 2 and chain[-2] == "fs" and last in ("File", "OpenOptions"):
                    continue
                if "OpenOptions" in head or last == "options":
                    writes = [toks[q].v for q in range(lo + 1, hi - 2)
                              if toks[q - 1].v == "." and toks[q].v in OPEN_WRITE_FLAGS
                              and toks[q + 1].v == "(" and toks[q + 2].v != "false"]
                    if writes:
                        out.append(("fs-write", txt + " +" + "/".join(sorted(set(writes))), t.line))
                    else:
                        out.append(("fs-read", txt + " (no write/append/create/truncate flag in this fn)", t.line))
                else:
                    out.append(("fs-write" if last in FS_WRITE else "fs-read", txt, t.line))
            elif (head | {last}) & NET_SEGS:
                out.append(("net", txt, t.line))
            elif (head | {last}) & PROC_SEGS and "windows" not in head:
                out.append(("process", txt, t.line))
            elif "env" in head:
                out.append(("env", txt, t.line))
            elif "thread" in head:
                out.append(("thread", txt, t.line))
            elif last == "now" and head & {"Instant", "SystemTime"}:
                out.append(("time", txt, t.line))
            elif "windows" in head or "Win32" in head:
                out.append(("win32", txt, t.line))
            elif head & {"stdin", "stdout", "stderr"} or (("io" in head) and last in ("stdin", "stdout", "stderr")):
                out.append(("stdio", txt, t.line))
            elif last in ("set_hook", "catch_unwind", "take_hook"):
                out.append(("panic-hook", txt, t.line))
            continue
        if prv == ".":
            if v in PATH_PROBE_METHODS and nxt == "(":
                out.append(("fs-probe", "." + v + "()", t.line))
            continue
        # bare identifier
        if nxt == "!":
            if v in ("include_bytes", "include_str", "include"):
                arg = toks[i + 3].v if i + 3 < hi else ""
                out.append(("include", v + "!" + arg, t.line))
            elif v in ("env", "option_env"):
                arg = toks[i + 3].v if i + 3 < hi else ""
                out.append(("env-compile", v + "!(" + arg + ")", t.line))
            continue
        std_q = {q for q, leaf in fi.imports.get(v, ()) if q in ("fs", "net", "process", "env", "thread")}
        if std_q and nxt == "(":
            q0 = sorted(std_q)[0]
            leafs = sorted(leaf for q, leaf in fi.imports[v] if q == q0)
            kind = {"fs": "fs-write" if leafs[0] in FS_WRITE else "fs-read", "net": "net", "process": "process",
                    "env": "env", "thread": "thread"}[q0]
            out.append((kind, q0 + "::" + leafs[0] + " (imported as " + v + ")", t.line))
        elif v in ("OpenOptions",) and nxt != "::":
            out.append(("fs-write", v, t.line))
        elif v in ("TcpStream", "TcpListener", "UdpSocket"):
            out.append(("net", v, t.line))
        elif v in ("Command",) and nxt == "::":
            out.append(("process", v, t.line))
        elif (v in fi.win_imports or ("*" in fi.win_imports and WIN_HINT.match(v) and nxt == "(" and len(v) > 6
                                        and not v.islower())) and nxt == "(":
            out.append(("win32", v, t.line))
    return out


# ---------------------------------------------------------------- build
def build(root):
    idx = Index()
    base = os.path.join(root, "LiTo381-Modern")
    for sub in ("src", "tests"):
        for d, _, fs in os.walk(os.path.join(base, sub)):
            for f in sorted(fs):
                if not f.endswith(".rs"):
                    continue
                p = os.path.join(d, f)
                rel = os.path.relpath(p, root).replace("\\", "/")
                with io.open(p, encoding="utf-8-sig", errors="replace") as fh:
                    text = fh.read()
                fi = FileInfo(rel, lex(text))
                idx.files[rel] = fi
                parse_items(idx, fi, 0, len(fi.toks), ())
    # also record `use` inside bodies (conservative merge into the file scope)
    for fi in idx.files.values():
        toks = fi.toks
        for it in fi.items:
            if it.kind != "fn":
                continue
            for i in range(it.lo, it.hi):
                if toks[i].k == "i" and toks[i].v == "use" and toks[i - 1].v in ("{", ";", "}"):
                    parse_use(toks, i, fi)
    # module tree
    for rel, fi in idx.files.items():
        stem = os.path.basename(rel)[:-3]
        d = os.path.dirname(rel)
        for name, p, inline, tonly in fi.mod_decls:
            if p is not None:
                cands = [os.path.normpath(os.path.join(d, p)).replace("\\", "/")]
            else:
                cands = [d + "/" + name + ".rs", d + "/" + name + "/mod.rs",
                         d + "/" + stem + "/" + name + ".rs", d + "/" + stem + "/" + name + "/mod.rs"]
            for c in cands:
                if c in idx.files:
                    idx.files[c].modnames.add(name)
                    idx.files[c].parents.add(rel)
                    fi.children[(name, inline)] = (c, tonly)
                    break
    for rel, fi in idx.files.items():
        stem = os.path.basename(rel)[:-3]
        if stem == "mod":
            stem = os.path.basename(os.path.dirname(rel))
        fi.modnames.add(stem)
        for it in fi.items:
            it.modnames = {it.inline[-1]} if it.inline else fi.modnames
            it.testonly = fi.in_test_range(it.lo) or is_cfg_test(it.attrs or [])
    # non-test files of the library crate
    lib_root = "LiTo381-Modern/src/lib.rs"
    idx.lib_files = set()
    q = [lib_root] if lib_root in idx.files else []
    while q:
        cur = q.pop()
        if cur in idx.lib_files:
            continue
        idx.lib_files.add(cur)
        for (name, inline), (c, tonly) in idx.files[cur].children.items():
            if not tonly:
                q.append(c)
    return idx


class Resolver:
    def __init__(self, idx):
        self.idx = idx
        self.free = collections.defaultdict(list)      # name -> items (fn/const/static/macro)
        self.types = collections.defaultdict(list)     # name -> type/trait items
        self.impl_fns = collections.defaultdict(list)  # (file-agnostic type name) -> fn items
        self.impl_hdrs = collections.defaultdict(list)
        self.trait_default = collections.defaultdict(list)  # trait name -> default fns
        self.generic_fns = []
        self.allmods = set()
        for fi in idx.files.values():
            self.allmods |= fi.modnames
        for it in idx.items:
            if it.inline:
                self.allmods |= set(it.inline)
            if it.kind in ("type", "trait"):
                self.types[it.name].append(it)
            if it.kind in ("fn", "const", "static", "macro") and it.impl_type is None and it.trait is None:
                self.free[it.name].append(it)
            if it.kind in ("fn", "assoc") and it.impl_type is not None:
                if it.generic_impl:
                    self.generic_fns.append(it)
                else:
                    self.impl_fns[it.impl_type].append(it)
            if it.kind == "implhdr":
                self.impl_hdrs[it.impl_type].append(it)
            if it.kind == "fn" and it.impl_type is None and it.trait is not None:
                self.trait_default[it.trait].append(it)
        self._memo = {}
        self._busy = set()
        self.diag = collections.Counter()
        self._mod_files = collections.defaultdict(list)
        for fi in idx.files.values():
            for n in fi.modnames:
                self._mod_files[n].append(fi)
        self.trait_impl_fns = collections.defaultdict(list)  # (trait, fn name) -> impl fns
        for it in idx.items:
            if it.kind == "fn" and it.impl_type is not None and it.trait is not None:
                self.trait_impl_fns[(it.trait, it.name)].append(it)

    # --- name resolution helpers
    def in_module(self, cands, q):
        return [c for c in cands if q in c.modnames]

    def scoped(self, table, name, fi, qual=None, depth=0):
        """Resolve `name` (optionally `qual::name`) as seen from file `fi`.

        Returns a list of items; None when an unqualified name is not an item visible from this
        file (local variable, std/external item, generic parameter)."""
        key = (id(table), name, fi.rel, qual)
        if key in self._memo:
            return self._memo[key]
        if depth > 6 or key in self._busy:
            return None if qual is None else []
        self._busy.add(key)
        try:
            r = self._scoped(table, name, fi, qual, depth)
        finally:
            self._busy.discard(key)
        if depth == 0:
            self._memo[key] = r
        return r

    def _scoped(self, table, name, fi, qual, depth):
        files = self.idx.files
        cands = table.get(name) or []
        if qual is not None:
            if qual == "self":
                same = [c for c in cands if c.file == fi.rel]
                return same or (self.scoped(table, name, fi, None, depth + 1) or [])
            if qual == "super":
                out = [c for c in cands if c.file == fi.rel or c.file in fi.parents]
                if not out:
                    for pf in fi.parents:
                        out += self.scoped(table, name, files[pf], None, depth + 1) or []
                    out += self.scoped(table, name, fi, None, depth + 1) or []
                return out
            if qual in ("crate", "lito_modern"):
                root = files.get("LiTo381-Modern/src/lib.rs")
                return (self.scoped(table, name, root, None, depth + 1) or []) if root else []
            reals = {qual}
            if qual in fi.imports:
                reals |= {leaf for _, leaf in fi.imports[qual]}
            out = [c for c in cands if c.modnames & reals]
            if out:
                return out
            if not (reals & self.allmods):
                return []   # external path (std::, serde_json::, …)
            for f2 in self.mod_files(reals):
                out += self.scoped(table, name, f2, None, depth + 1) or []
            if not out and cands:
                self.diag[("qualified-not-found", "%s::%s" % (qual, name))] += 1
            return out
        if cands:
            same = [c for c in cands if c.file == fi.rel]
            if same:
                return same
        out = []
        if name in fi.imports:
            for q, leaf in fi.imports[name]:
                if q is None:
                    continue
                out += self.scoped(table, leaf, fi, q, depth + 1) or []
            return out
        for g in fi.globs:
            if g == "super":
                out += [c for c in cands if c.file in fi.parents]
                for pf in fi.parents:
                    out += self.scoped(table, name, files[pf], None, depth + 1) or []
            elif g in ("crate", "lito_modern"):
                root = files.get("LiTo381-Modern/src/lib.rs")
                if root:
                    out += self.scoped(table, name, root, None, depth + 1) or []
            elif g in self.allmods:
                out += [c for c in cands if g in c.modnames]
                for f2 in self.mod_files({g}):
                    if f2 is not fi:
                        out += self.scoped(table, name, f2, None, depth + 1) or []
        if out:
            seen = set()
            uniq = []
            for c in out:
                if c.id not in seen:
                    seen.add(c.id)
                    uniq.append(c)
            return uniq
        return None

    def mod_files(self, names):
        out = []
        for n in names:
            out += self._mod_files.get(n, [])
        return out


def item_refs(fi, it):
    """Deduplicated references made by one item: tuples whose first field is the kind."""
    toks = fi.toks
    lo, hi = it.lo, it.hi
    refs = set()
    for i in range(lo, hi):
        t = toks[i]
        if t.k != "i":
            continue
        v = t.v
        if v in KEYWORDS and v != "Self":
            continue
        nxt = toks[i + 1].v if i + 1 < hi else ""
        prv = toks[i - 1].v if i > lo else ""
        if prv == ".":
            if nxt == "(" or nxt == "::":
                refs.add(("m", v))
            continue
        if prv == "::":
            q = toks[i - 2] if i - 2 >= lo else None
            if q is None or q.k != "i":
                refs.add(("g", v))
                continue
            Q = q.v
            if Q == "Self":
                Q = it.impl_type or it.trait or "Self"
            qq = toks[i - 4].v if (i - 4 >= lo and toks[i - 3].v == "::" and toks[i - 4].k == "i") else None
            refs.add(("q", Q, qq, v))
            continue
        if v == "Self":
            v = it.impl_type or it.trait or "Self"
        if nxt == "!":
            refs.add(("mac", v))
        elif nxt == "::":
            refs.add(("head", v))
        elif nxt == "(":
            refs.add(("call", v))
        elif nxt == ":" or (nxt == "=" and (toks[i + 2].v if i + 2 < hi else "") != "="):
            refs.add(("type", v))   # field / binding name: only a type could be meant
        else:
            refs.add(("bare", v))
    return sorted(refs, key=lambda r: tuple(x or "" for x in r))


def analyze(idx, res, test, prim_cache, ref_cache, target_files_set):
    files = idx.files
    reached = {}       # item id -> parent item id
    order = []
    touched = set()    # type item ids
    methods = set()
    work = collections.deque()

    def allowed(it):
        if target_files_set is None:
            # lib unit test: the whole library crate, including #[cfg(test)] items
            return it.file.startswith(LIB_SRC) and "/src/bin/" not in it.file and not it.file.endswith("/main.rs")
        if it.file in target_files_set:
            return True
        return it.file in idx.lib_files and not it.testonly

    def reach(it, parent):
        if it.id in reached or not allowed(it):
            return
        reached[it.id] = parent
        order.append(it)
        work.append(it)

    def types_of(name, fi, qual=None):
        r = res.scoped(res.types, name, fi, qual)
        return [x for x in (r or ()) if allowed(x)]

    def owner_matches(f, ty):
        if f.impl_type != ty.name:
            return False
        return ty in types_of(ty.name, files[f.file], f.impl_qual)

    def call_method_on(ty, m, parent):
        for f in res.impl_fns.get(ty.name, ()):
            if f.name == m and owner_matches(f, ty):
                reach(f, parent)

    def touch_type(tits, parent):
        for ty in tits:
            if ty.id in touched:
                continue
            touched.add(ty.id)
            reach(ty, parent)
            for h in res.impl_hdrs.get(ty.name, ()):
                if h.trait is None:
                    continue
                hf = files[h.file]
                if ty not in types_of(ty.name, hf, h.impl_qual):
                    continue
                reach(h, ty.id)
                for f in res.impl_fns.get(ty.name, ()):
                    if f.file == h.file and f.trait == h.trait and f.inline == h.inline:
                        reach(f, ty.id)
                for d in res.trait_default.get(h.trait, ()):
                    reach(d, ty.id)
                touch_type([x for x in types_of(h.trait, hf) if x.kind == "trait"], ty.id)
            if ty.kind == "trait":
                for d in res.trait_default.get(ty.name, ()):
                    reach(d, ty.id)
            for m in list(methods):
                call_method_on(ty, m, parent)

    def generic(name, parent):
        for g in res.generic_fns:
            if g.name == name:
                reach(g, parent)

    reach(test, None)
    while work:
        it = work.popleft()
        fi = files[it.file]
        if it.id not in ref_cache:
            ref_cache[it.id] = item_refs(fi, it)
        for ref in ref_cache[it.id]:
            kind = ref[0]
            if kind == "m":
                v = ref[1]
                if v not in methods:
                    methods.add(v)
                    for tid in list(touched):
                        call_method_on(idx.items[tid], v, it.id)
                    generic(v, it.id)
            elif kind == "g":
                generic(ref[1], it.id)
            elif kind == "q":
                _, Q, qq, v = ref
                qt = types_of(Q, fi, qq)
                if qt:
                    touch_type(qt, it.id)
                    for ty in qt:
                        call_method_on(ty, v, it.id)
                        if ty.kind == "trait":
                            for d in res.trait_default.get(ty.name, ()):
                                if d.name == v:
                                    reach(d, it.id)
                            for f in res.trait_impl_fns.get((ty.name, v), ()):
                                reach(f, it.id)
                    continue
                hit = False
                r = types_of(v, fi, Q)
                if r:
                    touch_type(r, it.id)
                    hit = True
                r = res.scoped(res.free, v, fi, Q)
                if r:
                    for f in r:
                        reach(f, it.id)
                    hit = True
                if not hit and Q[:1].isupper():
                    generic(v, it.id)
            elif kind == "mac":
                v = ref[1]
                r = res.scoped(res.free, v, fi)
                if r is None:
                    r = res.free.get(v, [])   # #[macro_export] / textual scope: by name
                for f in r:
                    if f.kind == "macro":
                        reach(f, it.id)
            elif kind in ("head", "type"):
                touch_type(types_of(ref[1], fi), it.id)
            else:
                v = ref[1]
                touch_type(types_of(v, fi), it.id)
                if kind == "call":
                    r = res.scoped(res.free, v, fi)
                else:
                    # value position (fn pointer, const, static): same file or named import only
                    r = [c for c in res.free.get(v, ()) if c.file == fi.rel]
                    if not r and v in fi.imports:
                        r = res.scoped(res.free, v, fi)
                for f in r or ():
                    if f.kind != "macro":
                        reach(f, it.id)

    hits = []
    for it in order:
        if it.id not in prim_cache:
            prim_cache[it.id] = scan_primitives(files[it.file], it)
        for kind, detail, line in prim_cache[it.id]:
            hits.append((kind, detail, it.id, line))

    def witness(item_id):
        chain = []
        cur = item_id
        seen = set()
        while cur is not None and cur not in seen:
            seen.add(cur)
            chain.append(idx.items[cur].label())
            cur = reached[cur]
        return list(reversed(chain))

    return order, hits, witness


def test_full_name(idx, target_file, it):
    fi = idx.files[it.file]
    prefix = list(it.inline)
    if it.file != target_file:
        # included via `mod x;` from the target file (one level is enough for this repo)
        tfi = idx.files[target_file]
        d = os.path.dirname(target_file)
        for name, p, inline, _t in tfi.mod_decls:
            child = os.path.normpath(os.path.join(d, p)).replace("\\", "/") if p else d + "/" + name + ".rs"
            if child == it.file:
                prefix = list(inline) + [name] + prefix
                break
    return "::".join(prefix + [it.name])


def target_files(idx, target):
    root = "LiTo381-Modern/tests/%s.rs" % target
    if root not in idx.files:
        return None, []
    out = [root]
    seen = {root}
    q = [root]
    while q:
        cur = q.pop()
        fi = idx.files[cur]
        d = os.path.dirname(cur)
        stem = os.path.basename(cur)[:-3]
        for name, p, inline, _t in fi.mod_decls:
            if p:
                cands = [os.path.normpath(os.path.join(d, p)).replace("\\", "/")]
            else:
                cands = [d + "/" + name + ".rs", d + "/" + name + "/mod.rs", d + "/" + stem + "/" + name + ".rs"]
            for c in cands:
                if c in idx.files and c not in seen:
                    seen.add(c)
                    out.append(c)
                    q.append(c)
                    break
    return root, out


LIB_SRC = "LiTo381-Modern/src/"


def test_record(idx, it, order, hits, witness):
    by_kind = collections.defaultdict(list)
    seen = set()
    for kind, detail, item_id, line in hits:
        key = (kind, detail, item_id)
        if key in seen:
            continue
        seen.add(key)
        by_kind[kind].append({
            "detail": detail,
            "site": "%s:%d" % (idx.items[item_id].file, line),
            "in": idx.items[item_id].label(),
            "path": witness(item_id),
        })
    return {
        "source": "%s:%d" % (it.file, it.line),
        "ignored": bool(it.ignored),
        "attrs": it.attrs,
        "reached_items": len(order),
        "reached_files": sorted({o.file for o in order}),
        "primitives": by_kind,
    }


# verdict -> (severity, class letter, flag)
VERDICTS = {
    "falsepos": (0, "P", ""), "pure": (0, "P", ""), "time": (0, "P", "t"), "cp950": (0, "P", "c"),
    "thread": (1, "P", "T"), "gdi": (2, "G", "g"), "ro-root": (3, "R", ""), "loopback": (4, "L", ""),
    "write-owned": (5, "W", ""), "net-unreviewed": (8, "N", ""), "unknown": (9, "U", ""),
}
SKIP_KINDS = {"literal", "include", "env-compile"}


def summarize(result, scope_path, adj_path, out_dir):
    """Join the closure with manual site verdicts; unmatched sites fail closed (class U)."""
    with io.open(adj_path, encoding="utf-8") as fh:
        adj = json.load(fh)
    rules, overrides = adj["rules"], adj.get("overrides", [])

    def site_key(label):
        # "path/file.rs:123 Type::name" -> ("path/file.rs", "Type::name")
        loc, _, fn = label.partition(" ")
        return loc.rsplit(":", 1)[0], fn

    def verdict(target, test, kind, label):
        f, fn = site_key(label)
        for o in overrides:
            if o["target"] == target and o["test"] == test and f.endswith(o["file"]) and o.get("fn", "") in fn \
                    and (o.get("kinds", "*") == "*" or kind in o["kinds"]):
                return o["verdict"], o.get("note", "")
        if kind == "time":
            return "time", ""
        for r in rules:
            if f.endswith(r["file"]) and r.get("fn", "") in fn and (r.get("kinds", "*") == "*" or kind in r["kinds"]):
                return r["verdict"], r.get("note", "")
        return "unknown", ""

    scope = []
    with io.open(scope_path, encoding="utf-8-sig") as fh:
        for line in fh:
            line = line.rstrip("\r\n")
            if line and not line.startswith("#"):
                scope.append(line.split("\t"))
    os.makedirs(out_dir, exist_ok=True)
    sites = {}
    rows = []
    for cols in scope:
        target, name = cols[0], cols[1]
        tests = result["targets"].get(target, {}).get("tests", {})
        t = tests.get(name)
        if t is None:
            c = [n for n in tests if name.endswith("::" + n)]
            t = tests[c[0]] if len(c) == 1 else None
        if t is None or "primitives" not in t:
            rows.append([target, name, "?", "?", "", "0", "", "not found at HEAD", ""])
            continue
        worst = (0, "P")
        flags = set()
        reasons = collections.Counter()
        for kind, hs in t["primitives"].items():
            if kind in SKIP_KINDS:
                continue
            for h in hs:
                v, note = verdict(target, name, kind, h["in"])
                sev, letter, flag = VERDICTS[v]
                if flag:
                    flags.add(flag)
                if sev > worst[0]:
                    worst = (sev, letter)
                f, fn = site_key(h["in"])
                key = (kind, f, fn, v)
                e = sites.setdefault(key, {"tests": set(), "details": set(), "note": note})
                e["tests"].add((target, name))
                e["details"].add(h["detail"])
                if sev >= 1:
                    reasons[v] += 1
        rows.append([target, name, "yes" if t["ignored"] else "no", worst[1], "".join(sorted(flags)),
                     str(t["reached_items"]), str(len(t["reached_files"])),
                     ",".join("%s=%d" % kv for kv in sorted(reasons.items())), t["source"]])
    with io.open(os.path.join(out_dir, "closure-summary.tsv"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# target\ttest\tignored\tclass\tflags\treached_items\treached_files\tnon-pure site verdicts\tsource\n")
        for r in rows:
            fh.write("\t".join(r) + "\n")
    with io.open(os.path.join(out_dir, "sites.tsv"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# kind\tfile\tfn\tverdict\ttests\tdetails\tnote\n")
        for (kind, f, fn, v), e in sorted(sites.items()):
            fh.write("\t".join([kind, f, fn, v, str(len(e["tests"])), " | ".join(sorted(e["details"])), e["note"]]) + "\n")
    # same four columns as 維護工具/test-whitelist/tests.tsv (target, test, ignored, features)
    for fname, classes in (("approved-candidates.tsv", "PLW"), ("resource-root-readonly.tsv", "RG"),
                           ("still-excluded.tsv", "NU?")):
        with io.open(os.path.join(out_dir, fname), "w", encoding="utf-8", newline="\n") as fh:
            for r in rows:
                if r[3] in classes:
                    fh.write("\t".join([r[0], r[1], r[2], "-"]) + "\n")
    print("classes", sorted(collections.Counter(r[3] for r in rows).items()))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--targets", required=True, help="comma separated integration test targets")
    ap.add_argument("--out", required=True)
    ap.add_argument("--lib-tests", help="comma separated full names of lib unit tests (modern_lito::module::test)")
    ap.add_argument("--scope", help="TSV: target<TAB>test name (rows to classify)")
    ap.add_argument("--adjudication", help="JSON with manual per-site verdicts")
    ap.add_argument("--summary-dir", help="directory for closure-summary.tsv / sites.tsv / approved-candidates.tsv")
    args = ap.parse_args()
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
    idx = build(args.root)
    res = Resolver(idx)
    prim_cache = {}
    ref_cache = {}
    result = {"targets": {}, "stats": {"files": len(idx.files), "items": len(idx.items),
                                        "lib_nontest_files": len(idx.lib_files)}}
    for target in [t for t in args.targets.split(",") if t]:
        root, files = target_files(idx, target)
        if root is None:
            result["targets"][target] = {"error": "target file not found"}
            continue
        tests = []
        macrocalls = []
        for f in files:
            for it in idx.files[f].items:
                if it.kind == "fn" and it.is_test:
                    tests.append(it)
                elif it.kind == "macrocall" and f.startswith("LiTo381-Modern/tests/"):
                    macrocalls.append("%s:%d %s!" % (f, it.line, it.name))
        tinfo = {"files": files, "tests": {}, "item_macro_calls": macrocalls}
        for it in tests:
            order, hits, witness = analyze(idx, res, it, prim_cache, ref_cache, set(files))
            name = test_full_name(idx, root, it)
            tinfo["tests"][name] = test_record(idx, it, order, hits, witness)
        result["targets"][target] = tinfo
        print(target, len(tests), "tests")
    if args.lib_tests:
        libinfo = {"files": [], "tests": {}, "item_macro_calls": []}
        for full in [x for x in args.lib_tests.split(",") if x]:
            parts = full.split("::")
            cands = [it for it in idx.items if it.kind == "fn" and it.is_test and it.name == parts[-1]
                     and it.file.startswith(LIB_SRC)
                     and (len(parts) < 2 or parts[-2] in it.modnames or parts[-2] in idx.files[it.file].modnames)]
            if len(cands) != 1:
                libinfo["tests"][full] = {"error": "%d candidates" % len(cands)}
                continue
            order, hits, witness = analyze(idx, res, cands[0], prim_cache, ref_cache, None)
            libinfo["tests"][full] = test_record(idx, cands[0], order, hits, witness)
        result["targets"]["lib"] = libinfo
        print("lib", len(libinfo["tests"]), "tests")
    with io.open(args.out, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(result, fh, ensure_ascii=False, indent=1)
    if args.scope and args.adjudication and args.summary_dir:
        summarize(result, args.scope, args.adjudication, args.summary_dir)
    print("items", len(idx.items), "files", len(idx.files))
    for k, v in res.diag.most_common(40):
        print("diag", k, v)


if __name__ == "__main__":
    main()
