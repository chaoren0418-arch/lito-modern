"""Independent skill131 widget, wire and passive-authority comparison."""
from pathlib import Path
from PIL import Image
import json, hashlib, sys
p = Path(sys.argv[1]).resolve()
read = lambda f: json.loads(f.read_text(encoding="utf-8-sig"))
r = read(p / "receipt.json")
assert r["status"] == "verified_world_tree_native_GUI_defined_scenario"
assert r["server"]["actual_client_sessions"] == 4 and r["server"]["memberships_empty"]
assert r["server"]["classic_world_members_after_disconnect"] == 0
assert r["server"]["authority_mutations"] == 0 and not r["server"]["sql_access"]
assert r["inputs_unchanged"] and r["private_desktop"] and not r["physical_input_verified"]
runtime = Path(r["runtime"])
frames = [json.loads(s) for s in next((runtime / "captures").rglob("frames.jsonl")).read_text(encoding="utf-8-sig").splitlines()]
tx = [bytes.fromhex(f["plain"]) for f in frames if f["direction"] == "client_to_server" and f.get("plain") != "MASKED_C_LOGIN"]
rx = [bytes.fromhex(f["plain"]) for f in frames if f["direction"] == "server_to_client"]
disabled = r["scenario"] == "disabled"
assert sum(b == bytes([0x14, 16, 2]) for b in tx) == (4 if disabled else 2)
assert sum(b[0] == 0x5a for b in rx) == (0 if disabled else 2)
assert sum(b == bytes([8]) for b in tx) == (0 if disabled else 2)
assert sum(b == bytes([0x10, 0x14, 1, 0]) for b in rx) == (4 if disabled else 0)
assert sum(b == bytes([0x25, 7]) for b in rx) == (4 if disabled else 2)
reason = "classic-world-compatibility-disabled" if disabled else "World-Tree-classic-world-ready"
assert sum(reason in f.get("note", "") for f in frames) == (4 if disabled else 2)
states = [json.loads(s)["state"] for s in (p / "server-trace.jsonl").read_text(encoding="utf-8-sig").splitlines()]
checks = []
expected_moves = []
for role, account in [("a", "PeerFixture"), ("b", "PeerFixture2")]:
    before = r[f"before_{role}"]
    walked = next(w for w in r["walks"] if w["role"] == role)
    after = walked["after"]
    dx, dy, heading = (0, -1, 0) if disabled else (1, 0, 2) if role == "a" else (-1, 0, 6)
    assert [after["x"], after["y"]] == [walked["before"]["x"] + dx, walked["before"]["y"] + dy]
    expected_moves.append(bytes([0x0a]) + int(walked["before"]["x"]).to_bytes(2, "little") + int(walked["before"]["y"]).to_bytes(2, "little") + bytes([heading]))
    assert before["object_id"] == after["object_id"] and before["character"] == after["character"] and before["source"] == after["source"]
    assert any(s["test_account"] == account and s["object_id"] == after["object_id"] and [s["x"], s["y"]] == [after["x"], after["y"]] for s in states)
    selected = [s for s in r["steps"] if s["role"] == role and (s["step"].startswith("refused-") if disabled else s["step"] == "world-ready")]
    assert len(selected) == (2 if disabled else 1)
    if disabled:
        assert after["phase"] == "BATTLE" and after["room"] == before["room"] and after["mp"] == before["mp"]
        assert all("目前地圖無法使用此技能" in s["status"] for s in selected)
    else:
        assert after["phase"] == "WORLD" and after["classic_world_map"] == 4 and after["room"] == 0 and after["channel"] == -1
    image_stats = []
    for step in selected:
        image_path = Path(step["png"])
        assert hashlib.sha256(image_path.read_bytes()).hexdigest().upper() == step["png_sha256"]
        with Image.open(image_path) as im:
            pixels = im.convert("RGB")
            # Sample the middle of the actual 800x600 world, excluding HUD and letterbox.
            colors = [pixels.getpixel((x, y)) for x in range(80, 880, 80) for y in range(300, 1000, 80)]
            colored = sum(max(c) > 32 and max(c) - min(c) > 15 for c in colors)
            assert colored > 50
            image_stats.append({"step": step["step"], "colored_samples": colored, "samples": len(colors)})
    checks.append({"role": role, "phase": after["phase"], "MP": after["mp"], "walk": [after["x"], after["y"]], "images": image_stats})
actual_moves = [b for b in tx if b[0] == 0x0a]
assert sorted(actual_moves) == sorted(expected_moves), "extra or incorrect GUI movement requests"
out = {"status": "verified_skill131_GUI_defined_scenario", "scenario": r["scenario"], "compatibility_lab": not disabled, "deployment_configuration_verified": False, "checks": checks, "actual_sessions": 4, "exact_C_MOVE": 2, "extra_C_MOVE": 0, "physical_input_verified": False, "full_WORLD_qualified": False, "receipt_sha256": hashlib.sha256((p / "receipt.json").read_bytes()).hexdigest().upper()}
(p / "world-tree-gui-comparison.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"status": out["status"], "scenario": r["scenario"], "checks": checks}, ensure_ascii=False))
