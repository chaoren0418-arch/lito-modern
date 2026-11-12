"""Own-file (direct) markers of each Java peer class referenced by the inventory. Read-only.
Usage: python -I java_direct.py <out_dir> <server_test_java_dir>   (needs <out_dir>/java-peer-tls-inventory.tsv)
"""
import re, sys, io, json, collections, subprocess, csv
from pathlib import Path
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
OUT = Path(sys.argv[1]).resolve()
SREPO = Path(r"D:\lineage開發\3-lito開發")
REL = "lito伺服器/LiTo381-Server/src/test/java/local/lito381/server"
TEST = Path(sys.argv[2]).resolve()
SNAP = Path(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\relocation-baseline-20261003\fixture\java\test-classes\local\lito381\server")


def strip(src):
    src = re.sub(r"/\*.*?\*/", " ", src, flags=re.S)
    return re.sub(r"//[^\n]*", " ", src)


rows = list(csv.DictReader((OUT / "java-peer-tls-inventory.tsv").open(encoding="utf-8"), delimiter="\t"))
use = collections.defaultdict(collections.Counter)
for r in rows:
    for c in r["peer_classes"].split(","):
        if c != "-":
            use[c][r["batch"].split("-")[0]] += 1

chg = {}
o = subprocess.run(["git", "-C", str(SREPO), "-c", "core.quotepath=false", "diff", "--name-status", "HEAD", "origin/main", "--", REL],
                   capture_output=True, text=True, encoding="utf-8").stdout
for l in o.splitlines():
    st, *paths = l.split("\t")
    chg[Path(paths[-1]).stem] = st

binds = []
with (OUT / "java-peer-classes.tsv").open("w", encoding="utf-8", newline="\n") as f:
    f.write("\t".join(["class", "tests_by_batch", "transport", "listen", "tmpdir", "keytool_or_process", "mysql_fixture_ref", "go_gate_stdin", "jdbc", "getenv", "exit_codes", "origin_main_vs_HEAD", "in_20261003_snapshot", "lines"]) + "\n")
    for c in sorted(use):
        p = TEST / f"{c}.java"
        if not p.exists():
            f.write(f"{c}\t{dict(use[c])}\tMISSING\n"); continue
        raw = p.read_text(encoding="utf-8", errors="replace")
        s = strip(raw)
        tls = bool(re.search(r"ModernTlsTestMaterial|SSLServerSocket|serverContext", s))
        stdio = bool(re.search(r"System\.in\b", s)) and not tls
        listen = []
        for m in re.finditer(r"[^;{}]*(?:new ServerSocket\(|createServerSocket\(|new InetSocketAddress\()[^;]*", s):
            line = re.sub(r"\s+", " ", m.group(0)).strip()
            binds.append((c, line[:200]))
            if re.search(r"127\s*,\s*0\s*,\s*0\s*,\s*1|127\.0\.0\.1|getLoopbackAddress|loopback", line):
                listen.append("loopback:0" if re.search(r"\(\s*0\s*,|,\s*0\s*\)", line) else "loopback")
            else:
                listen.append("REVIEW")
        tmp = sorted(set(re.findall(r'createTemp(?:Directory|File)\(\s*(?:[a-zA-Z]+\s*,\s*)?"([^"]+)"', s)))
        proc = "ProcessBuilder" if re.search(r"\bProcessBuilder\b|getRuntime\(\)\.exec", s) else "-"
        mysql = sorted(set(re.findall(r"\b(ModernSettlementMysqlFixture|ModernQuestSqlFixture)\b", s)))
        go = "yes" if re.search(r"'G'\s*,\s*'O'", s) else "-"
        jdbc = "yes" if re.search(r"DriverManager|jdbc:|\.jdbc\(", s) else "-"
        env = sorted(set(re.findall(r'System\.getenv\(\s*"?([A-Za-z0-9_.]+)"?', s)))
        ex = sorted(set(re.findall(r"System\.exit\(\s*(\d+)", s)))
        f.write("\t".join([c, ",".join(f"{k}:{v}" for k, v in sorted(use[c].items())),
                           "TLS" if tls else ("stdio" if stdio else "other"),
                           ",".join(sorted(set(listen))) or "-", ",".join(tmp) or "-", proc, ",".join(mysql) or "-", go, jdbc,
                           ",".join(env) or "-", ",".join(ex) or "-", chg.get(c, "same"), "yes" if (SNAP / f"{c}.class").exists() else "NO", str(raw.count("\n") + 1)]) + "\n")
print("classes", len(use))
for b in binds:
    if not re.search(r"127\s*,\s*0\s*,\s*0\s*,\s*1|127\.0\.0\.1|getLoopbackAddress|loopback", b[1]):
        print("REVIEW-BIND", b)
print(open(OUT / "java-peer-classes.tsv", encoding="utf-8").read())
