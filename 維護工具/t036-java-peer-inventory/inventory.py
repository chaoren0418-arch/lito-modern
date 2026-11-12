"""T036-9 (#314) inventory generator. Read-only static analysis; writes TSVs + stats.

Usage: python -I inventory.py <out_dir> <server_test_java_dir>
Requires scratch/issue-314/analysis.json (run analyze.py first). <server_test_java_dir> is the
read-only directory holding the Modern*TestPeer.java sources (.../local/lito381/server).
"""
import re, sys, io, json, collections
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import analyze as AZ
import summ_lib as L

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=True)
A = L.A

# ---- mode lists parsed from world_integration_tests.rs::JavaPeer::start_mode_lifetime
W = (AZ.CRATE / "src/modern_lito/world_integration_tests.rs").read_text(encoding="utf-8")
body = W[W.index("fn start_mode_lifetime"):W.index("fn expect_successful_exit")]
blocks = re.findall(r"matches!\(\s*mode,(.*?)\)", body, re.S)
SERVERROOT_MODES = set(re.findall(r'"([^"]+)"', blocks[0]))
JOB_MODES = set(re.findall(r'"([^"]+)"', blocks[1]))
LONG_MODES = set(re.findall(r'"([^"]+)"', blocks[2]))
assert len(blocks) == 3 and "quest-sql" in JOB_MODES and "normal" not in JOB_MODES
T = (AZ.CRATE / "src/modern_lito/transport.rs").read_text(encoding="utf-8")
assert 'class == "local.lito381.server.ModernVipMainSqlTestPeer"' in T

# Rust puts these under the kill-on-close job + GO gate, but the Java peer source opens no MySQL for them
# (ModernCombatTestPeer.java: sql = records ? open : null; canonical-equipment => records == false).
NO_SQL_JOB_MODES = {"canonical-equipment"}
# Java classes whose own source file reads LITO_SHARED_RESOURCE_ROOT.
SHARED_JAVA = {p.stem for p in Path(sys.argv[2]).resolve().glob("*.java")
               if "LITO_SHARED_RESOURCE_ROOT" in p.read_text(encoding="utf-8", errors="replace")}
assert SHARED_JAVA, "server test source directory has no peer reading LITO_SHARED_RESOURCE_ROOT"


def family(r):
    sp = " ".join(r["marks"].get("spawn", []))
    fams = []
    if "world_integration_tests.rs" in sp: fams.append("WORLD")
    if "transport.rs" in sp: fams.append("TLS")
    if "protocol.rs" in sp: fams.append("STDIO")
    if re.search(r"(battle_payload_interop_tests|equipment_history_wire_interop_tests|npc_\w+_tests)\.rs", sp): fams.append("WIRE")
    if not fams:
        f = r["file"]
        if "transport" in f or "profile_probe_tls" in f: fams.append("TLS?")
        elif "LITO_MODERN_TEST_SERVER_ROOT" in r["envs"]: fams.append("FILEFIX")
        else: fams.append("?")
    return "+".join(fams)


def classify(r):
    fam = family(r)
    classes = [c.split(".")[-1] for c in r.get("user_classes", [])]
    lits = set(r.get("lits", []))
    job_modes = sorted(lits & JOB_MODES) if "WORLD" in fam else []
    sql_modes = [m for m in job_modes if m not in NO_SQL_JOB_MODES]
    root_modes = sorted(lits & SERVERROOT_MODES) if "WORLD" in fam else []
    vip_sql = "ModernVipMainSqlTestPeer" in classes
    ign = r.get("ignore_reason") or ""
    hwnd = r["cls"] == "GUI" or bool(re.search(r"INTERACTIVE|OPERATOR", ign))
    hwnd_branch = bool(r["marks"].get("hwnd")) and not hwnd
    needs = []
    e = set(r["envs"])
    if fam != "FILEFIX": needs.append("JAVA+CP")
    if "LITO_MODERN_TLS_TEST_FIXTURES" in e or "TLS" in fam or "WORLD" in fam: needs.append("TLSFIX")
    if fam == "FILEFIX" or root_modes or vip_sql: needs.append("SERVER_ROOT")
    # Rust-side need counts only when the env read sits in the test's own file (cross-file name
    # resolution is over-approximate); world-family helpers always read it; Java side by own-file getenv.
    own_sites = [s for s in r["envs"].get("LITO_SHARED_RESOURCE_ROOT", []) if s.startswith(r["file"] + ":")]
    far_sites = [s for s in r["envs"].get("LITO_SHARED_RESOURCE_ROOT", []) if not s.startswith(r["file"] + ":")]
    shared = bool(own_sites) or "WORLD" in fam or any(c in SHARED_JAVA for c in classes) or (bool(far_sites) and ("WIRE" in fam or "STDIO" in fam))
    shared_far = bool(far_sites) and not shared
    if shared: needs.append("SHARED")
    if sql_modes or vip_sql: needs.append("MYSQL")
    other = sorted(x for x in e if x not in L.JENV and x not in ("LITO_SHARED_RESOURCE_ROOT", "LITO_MODERN_PROBE_ACCOUNT", "LITO_MODERN_PROBE_PASSWORD"))
    fs = [k for k in ("fs_delete", "fs_write") if r["marks"].get(k)]
    return dict(family=fam, classes=classes, job_modes=job_modes, hwnd_branch=hwnd_branch, sql_modes=sql_modes, root_modes=root_modes, vip_sql=vip_sql, hwnd=hwnd,
                needs=needs, other_env=other, fs=fs, shared=shared, shared_far=shared_far, far_sites=far_sites, shared_ev="+".join(x for x, y in (("rust-own", own_sites), ("rust-cross-file", far_sites), ("java-getenv", [c for c in classes if c in SHARED_JAVA]), ("world-helper", "WORLD" in fam)) if y) or "-")


def scope(r):
    c = L.r54cat(r)
    if c != "-": return "本張:" + c
    if r["cls"] == "DB": return "他張:DB"
    if r["cls"] == "GUI": return "排除:GUI"
    if "LITO_SHARED_RESOURCE_ROOT" in r["reason"]: return "他張:共享資源根"
    return "他張:雜項環境變數"


def batch(r, k):
    sc = scope(r)
    if sc == "排除:GUI" or k["hwnd"]: return "X-GUI"
    if any(x.startswith("LITO_MODERN_REAL_DB") for x in k["other_env"]): return "X-REAL_DB"
    if k["sql_modes"] or k["vip_sql"] or sc == "他張:DB": return "D-自有隔離DB"
    if k["family"] == "FILEFIX": return "A0-檔案fixture"
    if k["family"] in ("WIRE", "STDIO") and not k["shared"]: return "A1-stdio"
    if k["family"] in ("WIRE", "STDIO"): return "B1-stdio+共享資源"
    if k["job_modes"]: return "B3-TLS+共享資源+job閘"
    if not k["shared"]: return "A2-TLS"
    return "B2-TLS+共享資源"


sel = []
for r in A:
    if r["white"] or r.get("error") or r["target"] != "lib": continue
    st = L.static(r)
    if L.r54cat(r) == "-" and "spawn" not in st and "jenv" not in st: continue
    k = classify(r)
    sel.append((r, k, scope(r), batch(r, k)))

cols = ["batch", "scope54", "family", "test", "source", "peer_classes", "sql_modes(owned mysqld)", "needs", "shared_evidence", "other_env", "rust_fs_marks", "hwnd", "ignore_reason", "r54_class"]
with (OUT / "java-peer-tls-inventory.tsv").open("w", encoding="utf-8", newline="\n") as f:
    f.write("\t".join(cols) + "\n")
    for r, k, sc, b in sorted(sel, key=lambda x: (x[3], x[2], x[0]["name"])):
        f.write("\t".join([b, sc, k["family"], r["name"], f'{r["file"]}:{r.get("fn_line", r["line"])}', ",".join(k["classes"]) or "-",
                           ",".join(k["sql_modes"]) + ("vip-main-sql" if k["vip_sql"] else "") + ("".join(" [job-only:" + m + "]" for m in k["job_modes"] if m in NO_SQL_JOB_MODES)) or "-", "+".join(k["needs"]) or "-", k["shared_ev"], ",".join(k["other_env"]) or "-",
                           ",".join(k["fs"]) or "-", "yes" if k["hwnd"] else ("branch-off" if k["hwnd_branch"] else "no"), (r.get("ignore_reason") or "").replace("\t", " "), r["cls"]]) + "\n")

print("selected", len(sel))
c = collections.Counter((sc, b) for _, _, sc, b in sel)
for k_, v in sorted(c.items()): print(v, *k_, sep="\t")
print("--- batch x family (本張 only)")
c = collections.Counter((b, k["family"]) for r, k, sc, b in sel if sc.startswith("本張"))
for k_, v in sorted(c.items()): print(v, *k_, sep="\t")
print("--- batch totals 本張 / 他張:共享 / 雜項")
for pre in ("本張", "他張:共享資源根", "他張:雜項環境變數", "他張:DB", "排除:GUI"):
    c = collections.Counter(b for r, k, sc, b in sel if sc.startswith(pre))
    print(pre, dict(sorted(c.items())))
print("--- classes per batch (本張+共享+雜項)")
pc = collections.defaultdict(collections.Counter)
for r, k, sc, b in sel:
    if sc in ("他張:DB",): continue
    for cl in k["classes"]: pc[b][cl] += 1
for b in sorted(pc): print(b, dict(sorted(pc[b].items())))
print("--- D rows in 本張")
for r, k, sc, b in sel:
    if sc.startswith("本張") and b.startswith(("D", "X")):
        print(b, sc, k["family"], r["name"].replace("modern_lito::", ""), k["classes"], k["sql_modes"], k["vip_sql"], "|", r.get("ignore_reason"))
print("--- other_env / fs in 本張+共享 non-D")
for r, k, sc, b in sel:
    if not b.startswith(("D", "X")) and (k["other_env"] or k["fs"]):
        print(b, r["name"].replace("modern_lito::", ""), k["other_env"], k["fs"], {m: v[:2] for m, v in r["marks"].items() if m.startswith("fs")})
print("--- family ? rows")
for r, k, sc, b in sel:
    if "?" in k["family"]: print(b, sc, k["family"], r["name"], r["file"])

# ---- drift: ignored #[test] fns in current src not present in #54 lib rows
files, stems = AZ.build()
known = {(r["file"], r["name"].split("::")[-1]) for r in A if r["target"] == "lib"}
known_all = {(r["file"], r["name"].split("::")[-1]) for r in A}
new = []
tot_ign = 0
for fi in files.values():
    if "/src/" not in fi.rel: continue
    for d in fi.fns:
        pre = fi.src[max(0, d["start"] - 700):d["start"]]
        kk = max(pre.rfind("}\n"), pre.rfind(";\n"), pre.rfind("{\n"))
        attr = AZ.mask(pre[kk + 1:] if kk >= 0 else pre)[0]
        if "#[test]" in attr.replace(" ", "") and re.search(r"#\[\s*ignore", attr):
            tot_ign += 1
            if (fi.rel, d["name"]) not in known:
                texts = AZ.closure(fi, [d["body"]], files, stems)
                marks, envs, classes = AZ.analyze_text(texts)
                new.append((fi.rel, d["line"], d["name"], "spawn" in marks, sorted(set(envs) & L.JENV), sorted(c.split(".")[-1] for c in classes)[:4]))
gone = [(r["file"], r["name"]) for r in A if r["target"] == "lib" and not r.get("error") and not r.get("macro") and not r.get("has_ignore_attr")]
print("--- drift: ignored #[test] fns in current src:", tot_ign, "| #54 lib rows:", sum(r["target"] == "lib" for r in A), "| macro rows:", sum(bool(r.get("macro")) for r in A))
print("new ignored not in #54:", len(new))
for n in new: print("  NEW", *n)
print("#54 lib rows whose fn no longer has #[ignore]:", len(gone))
for g in gone[:20]: print("  UNIGNORED", *g)
(L.WORK / "inventory_stats.json").write_text(json.dumps(dict(job_modes=sorted(JOB_MODES), serverroot_modes=sorted(SERVERROOT_MODES), long_modes=sorted(LONG_MODES)), ensure_ascii=False, indent=1), encoding="utf-8")

print("--- A-batch rows whose closure reaches a shared-root read only through another file (review)")
for r, k, sc, b in sel:
    if b.startswith("A") and (k["shared_far"] or re.search(r"shared|S32|SPR|\bmap\b|resource|pixels|render", r.get("ignore_reason") or "", re.I)):
        print(b, sc, r["name"].replace("modern_lito::", ""), "| far:", k["far_sites"][:2], "| ign:", r.get("ignore_reason"))