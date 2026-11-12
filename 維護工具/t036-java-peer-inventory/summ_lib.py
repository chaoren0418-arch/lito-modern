"""Shared helpers: load analysis.json and map each row to its #54 reason bucket."""
import json
from pathlib import Path

WORK = Path(__file__).resolve().parents[2] / "scratch" / "issue-314"
A = json.loads((WORK / "analysis.json").read_text(encoding="utf-8"))
JENV = {"LITO_MODERN_TEST_JAVA", "LITO_MODERN_TEST_JAVA_CLASSPATH", "LITO_MODERN_TLS_TEST_FIXTURES",
        "LITO_MODERN_TEST_SERVER_ROOT", "LITO_T043_REFERENCE_JAR", "JAVA_HOME"}


def r54cat(r):
    """Bucket of the #54 reason text; these four buckets are the 169 rows owned by T036-9."""
    t = r["reason"]
    if "Java peer／產生 fixture" in t: return "J125"
    if "未設定 LITO_MODERN_TLS_TEST_FIXTURES" in t: return "T38"
    if "未設定 LITO_MODERN_TEST_SERVER_ROOT" in t: return "S3"
    if "外部 Java peer 尚未核對" in t: return "X3"
    return "-"


def static(r):
    if r.get("error"): return "ERR"
    m, e = r["marks"], r["envs"]
    s = []
    if "spawn" in m: s.append("spawn")
    if JENV & set(e): s.append("jenv")
    if "tls_connect" in m: s.append("tls")
    if "job" in m: s.append("job")
    return "+".join(s) or "none"
