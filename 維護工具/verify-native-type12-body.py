"""Verify the actual type12 body is submitted, retaining unsupported attr1 diagnostics."""
from pathlib import Path
import json, sys, hashlib
p = Path(sys.argv[1]).resolve()
read = lambda f: json.loads(f.read_text(encoding="utf-8-sig"))
source = read(Path(__file__).resolve().parent.parent / "LiTo381-Modern/tests/fixtures/native-type12-869.json")
checks = []
for role in ["a", "b"]:
    v = read(p / f"{role}-world-entry.json")
    actor = next(a for a in v["render"]["actors"] if a["id"] == 1677722876)
    assert actor["rendered"] is True and actor["logical_graphics"] == 869
    assert actor["sprite_member"] == "869-0.spr" and actor["sprite_subframe"] == 0
    assert actor["phase"] == "static_prop" and actor["source_frame"] == 0
    assert actor["body_source"]["archive"] == "Sprite"
    assert actor["body_source"]["bytes"] == 1569 and actor["body_source"]["sha256"] == source["sprite"]["sha256"]
    assert actor["layer_error"] == "body flags outside neutral layer presentation"
    assert actor["native_dynamic_draw_flags_complete"] is False
    assert actor["source_frame_events"] == [] and actor["entered_events"] == []
    checks.append({"role": role, "member": actor["sprite_member"], "source_sha256": actor["body_source"]["sha256"], "layer_diagnostic_preserved": True, "scene_PNG_sha256": hashlib.sha256((p / f"{role}-world-entry.png").read_bytes()).hexdigest().upper()})
out = {"status": "verified_original_type12_single_image_body_submission", "checks": checks, "scope": "enabled classic compatibility laboratory; original source body, no invented animation", "attr1_semantics_verified": False, "full_native_draw_flags_verified": False, "whole_product_verified": False}
(p / "type12-body-comparison.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"status": out["status"], "checks": checks}, ensure_ascii=False))
