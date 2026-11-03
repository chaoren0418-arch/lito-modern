"""Verify the actual C02 compatibility-disabled branch without production SQL."""
from pathlib import Path
import json, sys, hashlib

p = Path(sys.argv[1]).resolve()
read = lambda f: json.loads(f.read_text(encoding="utf-8-sig"))
r = read(p / "receipt.json")
assert r["status"] == "verified_formal_config_world_tree_refusal"
assert r["test_exit_code"] == r["server_exit_code"] == 0
assert r["server"]["actual_client_sessions"] == 4 and r["server"]["memberships_empty"]
assert r["server"]["authority_mutations"] == 0 and not r["server"]["sql_access"]
runtime = Path(r["runtime"])
assert "classic.world.compatibility.enabled=false" in (runtime / "server.properties").read_text(encoding="utf-8-sig")
frames = [json.loads(s) for s in next((runtime / "captures").rglob("frames.jsonl")).read_text(encoding="utf-8-sig").splitlines()]
client = [bytes.fromhex(f["plain"]) for f in frames if f["direction"] == "client_to_server" and f.get("plain") != "MASKED_C_LOGIN"]
server = [bytes.fromhex(f["plain"]) for f in frames if f["direction"] == "server_to_client"]
assert sum(b == bytes([0x14, 16, 2]) for b in client) == 4
assert sum(b == bytes([0x10, 0x14, 0x01, 0]) for b in server) == 4
assert not any(b[0] == 0x5a for b in server) and not any(b == bytes([8]) for b in client)
assert not any(b[0] == 0x37 and b[-2:] == bytes([169, 0]) for b in server)
assert sum(b == bytes([0x25, 7]) for b in server) == 4
# Keep the actual paralysis opcode/layout independently visible before asserting it.
paralysis = [f for f in frames if "classic-world-compatibility-disabled" in f.get("note", "")]
assert len(paralysis) == 4
states = [json.loads(s)["state"] for s in (p / "server-trace.jsonl").read_text(encoding="utf-8-sig").splitlines()]
checks = []
for role, account, oid in [("a", "PeerFixture", 1), ("b", "PeerFixture2", 3)]:
    v = read(p / f"{role}-world-entry.json")
    before, after, walked = (v[x] for x in ["before", "after_refused", "after_walk"])
    assert after["skill_feedback"] == {"Rejected": 276}
    assert before["phase"] == after["phase"] == walked["phase"] == "battle_pending"
    assert before["world"]["generation"] == after["world"]["generation"] == walked["world"]["generation"]
    assert before["world"]["map_id"] == after["world"]["map_id"]
    assert before["world"]["mp"] == after["world"]["mp"] == walked["world"]["mp"]
    assert after["world"]["completed_portals"] == 0 and after["world"]["portal"] is None
    assert before["inventory"]["items"] == after["inventory"]["items"] == walked["inventory"]["items"]
    assert before["joined_room"]["id"] == after["joined_room"]["id"] == walked["joined_room"]["id"]
    assert any(s["test_account"] == account and s["object_id"] == oid and s["phase"] == "BATTLE" and [s["x"], s["y"]] == v["walk_target"] for s in states)
    assert v["two_explicit_requests_elapsed_ms"] < 2000, "client retained its two-second request lock"
    checks.append({"role": role, "MP": after["world"]["mp"], "walk_target": v["walk_target"], "two_explicit_requests_elapsed_ms": v["two_explicit_requests_elapsed_ms"]})
out = {"status": "verified_C02_compatibility_disabled_refusal_and_explicit_retry", "scope": "memory fixtures with false compatibility flag; same C02 branch as non-memory default; live deployment settings not read or changed", "checks": checks, "C14": 4, "message276": 4, "teleport_unlock7_notes": 4, "paralysis_packets": paralysis, "S5A": 0, "C08": 0, "actual_sessions": 4, "SQL_access": False, "receipt_sha256": hashlib.sha256((p / "receipt.json").read_bytes()).hexdigest().upper()}
(p / "world-tree-refusal-comparison.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"status": out["status"], "checks": checks}, ensure_ascii=False))
