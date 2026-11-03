"""Independent raw packet/authority check; no product qualification implied."""
from pathlib import Path
import argparse, hashlib, json

def read(p):
    return json.loads(p.read_text(encoding="utf-8-sig"))

def verify(case):
    receipt = read(case / "receipt.json")
    assert receipt["status"] == "verified_raw_world_entry_contract"
    assert receipt["test_exit_code"] == receipt["server_exit_code"] == 0
    assert receipt["server"]["actual_client_sessions"] == 4
    assert receipt["server"]["authority_mutations"] == 0
    assert receipt["server"]["memberships_empty"] is True
    states = [json.loads(s)["state"] for s in (case / "server-trace.jsonl").read_text(encoding="utf-8-sig").splitlines()]
    results = []
    for role, account, oid in [("a", "PeerFixture", 1), ("b", "PeerFixture2", 3)]:
        report = read(case / f"{role}-world-entry.json")
        packets = [bytes.fromhex(r["body"]) for r in report["trace"]]
        portals = [i for i, b in enumerate(packets) if b[0] == 0x5a]
        assert len(portals) == 1
        n = portals[0]
        assert packets[n] == bytes.fromhex("5a04001b81517e")
        after = packets[n + 1:]
        departures = [i for i, b in enumerate(after) if b[0] == 0xd0]
        statuses = [i for i, b in enumerate(after) if b[0] == 0x0c and int.from_bytes(b[1:5], "little") == oid]
        assert len(departures) == 1 and len(statuses) == 1 and departures[0] < statuses[0]
        peer_name = after[departures[0]][5:].split(b"\x00", 1)[0].decode("cp950")
        assert peer_name != report["world"]["actors"][str(oid)]["name"], "S_D0 is a peer departure, not self"
        assert not any(b[0] == 0x28 for b in after), "WORLD contract changed: new map header"
        world = report["world"]
        assert world["map_id"] == 4 and world["completed_portals"] == 1
        own = world["actors"][str(oid)]
        assert [own["x"], own["y"]] == [33051, 32337]
        observed = [s for s in states if s["test_account"] == account and s["phase"] == "WORLD"]
        assert observed
        for state in observed:
            assert state["object_id"] == oid and state["room"] == 0 and state["channel"] == -1
            assert state["classic_world_map"] == 4 and [state["x"], state["y"]] == [own["x"], own["y"]]
            assert state["character"] in state["available_characters"]
        directory_names = [b[1:].split(b"\x00", 1)[0].decode("cp950") for b in packets if b[0] == 0xcb]
        assert own["name"] in directory_names, "WORLD alias must equal exact received character directory"
        results.append({"role": role, "object_id": oid, "world_states": len(observed), "peer_departure_before_status": True})
    return {"status": "verified_raw_world_entry_contract", "scope": "raw request reachability; original default memory books did not grant skill131", "production_login_qualified": False, "roles": results, "receipt_sha256": hashlib.sha256((case / "receipt.json").read_bytes()).hexdigest().upper()}

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("case", type=Path)
    args = parser.parse_args()
    result = verify(args.case)
    output = args.case / "world-entry-contract-comparison.json"
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(output)
