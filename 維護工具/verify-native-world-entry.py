"""Compare client WORLD snapshots, passive formal authority, and original wire capture."""
from pathlib import Path
import argparse, hashlib, json
from PIL import Image

def read(p):
    return json.loads(p.read_text(encoding="utf-8-sig"))

def verify(case):
    r = read(case / "receipt.json")
    assert r["status"] == "verified_roomless_world_entry"
    assert r["test_exit_code"] == r["server_exit_code"] == 0
    assert r["server"]["actual_client_sessions"] == 4 and r["server"]["memberships_empty"]
    assert r["server"]["authority_mutations"] == 0 and not r["server"]["sql_access"]
    assert r["server"]["classic_world_members_after_disconnect"] == 0
    runtime = Path(r["runtime"])
    capture = list((runtime / "captures").rglob("frames.jsonl"))
    assert len(capture) == 1
    frames = [json.loads(s) for s in capture[0].read_text(encoding="utf-8-sig").splitlines()]
    client = [bytes.fromhex(f["plain"]) for f in frames if f["direction"] == "client_to_server" and "plain" in f and f["plain"] != "MASKED_C_LOGIN"]
    assert sum(b == bytes([0x14, 16, 2]) for b in client) == 2
    assert sum(b == bytes([8]) for b in client) == 2
    server = [bytes.fromhex(f["plain"]) for f in frames if f["direction"] == "server_to_client" and "plain" in f and f["plain"] != "MASKED_C_LOGIN"]
    assert sum(b == bytes.fromhex("5a04001b81517e") for b in server) == 2
    states = [json.loads(s)["state"] for s in (case / "server-trace.jsonl").read_text(encoding="utf-8-sig").splitlines()]
    roles = []
    for role, account, oid in [("a", "PeerFixture", 1), ("b", "PeerFixture2", 3)]:
        v = read(case / f"{role}-world-entry.json")
        before, after, walked = (v[n] for n in ["before", "after", "after_walk"])
        assert before["phase"] == "battle_pending" and after["phase"] == walked["phase"] == "world_ready"
        assert after["joined_room"] is None and after["channel"] is None and not after["rooms"]
        assert after["world"]["generation"] == before["world"]["generation"] + 1
        assert after["world"]["map_id"] == 4 and after["world"]["completed_portals"] == 1
        assert after["inventory"]["owner"] == before["inventory"]["owner"]
        assert after["inventory"]["items"] == before["inventory"]["items"] == walked["inventory"]["items"]
        assert before["selected_character"] == after["selected_character"] == walked["selected_character"]
        own = walked["world"]["actors"][str(oid)]
        assert [own["x"], own["y"]] == v["walk_target"]
        assert walked["world"]["local_corrections"] == after["world"]["local_corrections"]
        matches = [s for s in states if s["test_account"] == account and s["phase"] == "WORLD" and [s["x"], s["y"]] == v["walk_target"]]
        assert matches, "no passive server confirmation of WORLD step"
        for state in matches:
            assert state["object_id"] == oid and state["room"] == 0 and state["channel"] == -1 and state["classic_world_map"] == 4
        image_path = case / f"{role}-world-entry.png"
        with Image.open(image_path) as im:
            assert im.size == (800, 600)
            rgba = im.convert("RGBA").tobytes()
            assert hashlib.sha256(rgba).hexdigest() == v["render"]["rgba_sha256"].lower()
            assert sum(1 for pixel in im.convert("RGB").getdata() if max(pixel) > 32 and max(pixel) - min(pixel) > 15) > 10000
        roles.append({"role": role, "object_id": oid, "walk_target": v["walk_target"], "inventory_count": len(after["inventory"]["items"]), "generation": after["world"]["generation"]})
    return {"status": "verified_roomless_world_entry", "roles": roles, "wire": {"skill131": 2, "S5A": 2, "C08": 2}, "actual_sessions": 4, "physical_input": False, "GUI_verified": False, "receipt_sha256": hashlib.sha256((case / "receipt.json").read_bytes()).hexdigest().upper()}

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("case", type=Path)
    args = p.parse_args()
    result = verify(args.case)
    output = args.case / "world-entry-comparison.json"
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(output)
