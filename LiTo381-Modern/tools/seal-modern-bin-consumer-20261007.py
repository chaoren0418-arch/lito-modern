"""Seal one new offline increment; no build, test, network or GUI execution."""
import datetime
import difflib
import hashlib
import json
from pathlib import Path

P = Path(__file__).resolve().parent.parent
G = P.parent
B = G.parent / "lito脫殼資源"
OUT = P / "target/modern-bin-consumer-20261007"
TASK = "modern-bin-consumer-20261007"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def pin(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest().upper()}


def check(expected):
    actual = pin(Path(expected["path"]))
    assert (actual["bytes"], actual["sha256"]) == (expected["bytes"], expected["sha256"]), actual
    return actual


def save(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write("\n")


aggregate_path = B / "modern-contracts-20261007.json"
aggregate_pin = pin(aggregate_path)
assert aggregate_pin["sha256"] == "DC08927CDB282EE14CD80080AD7EDB579EF8560AD98C00E83BFD5877BB39964D"
aggregate = read(aggregate_path)
assets = next(c for c in aggregate["contracts"] if c["id"] == "native_background_assets")
adopted = [aggregate_pin, check(aggregate["source_PE"])]
adopted += [check(p) for p in assets["handoff_refs"]]
adopted += [check(r["rgba"]) for r in assets["resources"]]
assert [r["id"] for r in assets["resources"]] == [6314, 7482]
assert all(r["scene_binding"] is None and r["draw_position"] is None for r in assets["resources"])
assert assets["consumer"]["native_scene_binding_allowed"] is False

for name in ("build", "test-build", "test"):
    accounting = read(OUT / f"{name}-accounting.json")
    assert accounting["exit_code"] == 0 and accounting["attempts"] == 1
stdout = (OUT / "tests.stdout.log").read_text(encoding="utf-8-sig")
assert "test result: ok. 3 passed; 0 failed;" in stdout, stdout
assert stdout.count(" ... ok") == 3, stdout
artifacts = read(OUT / "artifacts.json")
assert artifacts["library_fresh"] is False
check(artifacts["library"])
check(artifacts["test"])
snapshot = read(OUT / "source-snapshot.json")
for row in snapshot["inputs"]:
    check(row["source"])
    check(row["snapshot"])
    assert row["source"]["sha256"] == row["snapshot"]["sha256"]

before = {row["path"].replace("\\", "/"): row for row in read(OUT / "before-rust-pins.json")}
after = {str(f.relative_to(P)).replace("\\", "/"): pin(f) for f in (P / "src").rglob("*.rs")}
changed = [name for name in before if after[name]["sha256"] != before[name]["sha256"]]
new = sorted(set(after) - set(before))
assert changed == ["src/modern_lito/native_mode_menu_skin.rs"], changed
assert new == ["src/modern_lito/native_background_assets.rs", "src/modern_lito/native_background_assets_tests.rs"], new
assert set(before) <= set(after)
diff = "".join(difflib.unified_diff(
    (OUT / "native_mode_menu_skin.before.rs").read_text(encoding="utf-8-sig").splitlines(True),
    (P / changed[0]).read_text(encoding="utf-8-sig").splitlines(True),
    fromfile="before/" + changed[0], tofile="after/" + changed[0]))
for name in new:
    diff += "".join(difflib.unified_diff([], (P / name).read_text(encoding="utf-8").splitlines(True),
                                        fromfile="/dev/null", tofile="after/" + name))
diff_path = OUT / "source.diff"
with diff_path.open("x", encoding="utf-8", newline="\n") as f:
    f.write(diff)

warnings = {}
for name in ("build", "test-build"):
    messages = [json.loads(line) for line in (OUT / f"{name}.jsonl").read_text(encoding="utf-8-sig").splitlines()]
    warning_rows = [m for m in messages if m.get("reason") == "compiler-message" and m["message"]["level"] == "warning"]
    new_warnings = [m for m in warning_rows if any(s["file_name"].replace("\\", "/") in new for s in m["message"]["spans"])]
    warnings[name] = {"total_warning_messages": len(warning_rows), "new_module_warning_messages": len(new_warnings)}
    assert not new_warnings

now = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
report = {
    "schema": "modern.bin.consumer.offline.v1", "task_id": TASK, "updated_at": now,
    "state": "DELIVERED_OFFLINE_INCREMENT_REVIEW_REQUIRED", "overall_complete": False,
    "increment": "Pinned Native background RGBA -> existing Img -> explicit source-byte selector -> caller-positioned CPU Canvas",
    "adopted_contracts": adopted,
    "source_evidence": {"PE_sha256": aggregate["source_PE"]["sha256"],
                        "selector_VA": "00496F05", "selector_RVA": "00096F05",
                        "factory_call_VA": "00496F28", "factory_call_RVA": "00096F28",
                        "factory_target_VA": "00495520", "factory_target_RVA": "00095520",
                        "flag_VA": "0064AFA0", "flag_RVA": "0024AFA0"},
    "consumer_audit": [
        {"id": "native_background_assets", "before": "No 6314/7482 or64AFA0 consumer in Native Rust; skin only loads7580 channel background", "after": "Integrated public native_mode_menu_skin::background API; three new CPU tests use actual delivered buffers"},
        {"id": "495520_call_boundary", "status": "Qualified factory boundary consumed as scope limit; no invented draw method or owner"},
        {"id": "S_F4_244", "status": "Already implemented in native_entry_transfer decode/address_from_word/consume_source_transfer_event; no old PASS rerun or duplicate implementation"},
        {"id": "saved_host_64AD74", "status": "Current owned transfer state audited; no new startup host or implicit initialized buffer invented; outside this one increment"}],
    "source_changes": [after[name] for name in changed + new],
    "consumer_call_chain": ["native_mode_menu_skin::background::load_selected(root, explicit Option<u8>)", "select: zero ->6314; nonzero ->7482; unknown ->None without file access", "BackgroundAsset::load ->existing adapter::read_pinned ->from_rgba ->existing Img", "BackgroundAsset::blit_preview(explicit caller origin) ->existing Canvas::blit_rgba"],
    "source_diff": pin(diff_path), "source_snapshot": pin(OUT / "source-snapshot.json"),
    "rust_source_accounting": {"before": len(before), "after": len(after), "new": len(new), "modified": len(changed), "old_files_byte_unchanged": len(before) - len(changed)},
    "build": {"target": "i686-pc-windows-msvc", "library": artifacts["library"], "library_recompiled": True, "accounting": pin(OUT / "build-accounting.json"), "warnings": warnings},
    "validation": {"new_CPU_checks": 3, "passed": 3, "failed": 0, "executions": 1,
                   "cases": ["256 source byte values; unknown input performs no I/O", "Actual6314/7482 identities, dimensions and every pixel of caller-defined clipped CPU composition", "Wrong identity, corrupt SHA, empty/short/long buffers, relative and parent-component paths rejected"],
                   "test_artifact": artifacts["test"], "stdout": pin(OUT / "tests.stdout.log"),
                   "test_compile_accounting": pin(OUT / "test-build-accounting.json"), "test_run_accounting": pin(OUT / "test-accounting.json")},
    "evidence_logs": [pin(OUT / name) for name in ("build.jsonl", "build.stderr.log", "test-build.jsonl", "test-build.stderr.log", "tests.stderr.log", "artifacts.json", "before-rust-pins.json")],
    "tools_and_documentation": [pin(P / name) for name in ("tools/build-modern-bin-consumer-20261007.ps1", "tools/seal-modern-bin-consumer-20261007.py", "docs/native-background-offline-consumer.md")],
    "boundary": {"offline_implemented": True, "runtime_scene_binding": False, "native_render_parity": False, "GUI_acceptance": False,
                 "flag_unknown_preserved": True, "coordinates_caller_preview_only": True, "default_route_unchanged": True,
                 "new_or_corrected_wire": False, "old_PASS_replays": 0, "Java_SQL": 0, "sockets": 0, "GUI": 0, "READY_reads": 0, "claims": 0,
                 "private_next_layer_analysis": 0, "server_namespace_writes": 0,
                 "retained_restrictions": "T291/T27/refund/rebid; auction/castle/full-WORLD/store dependencies; private backing42E550/42E6D0/42EA50; Native tool limits"},
    "next_BIN_request": {
        "id": "modern-background-scene-binding", "supplier": "BIN via supervisor; qualified existing evidence only",
        "consumer": "native_mode_menu_skin::background then explicit qualified Native scene consumer",
        "call_points": ["496F05 flag selection", "496F28 ->495520 factory result EAX/EDI", "75F6E8 owner+0C comparison5"],
        "required_fields": ["Source PE SHA and VA/RVA of each existing qualified owner publication/reset/destruction anchor", "Flag64AFA0 initialized value and lifecycle scope", "495520 result/backdrop owner link distinct from child descriptors", "Qualified backdrop x/y, crop/clip/transform and drawing mode with supporting existing artifact", "Source-established scene association or an explicit unknown"],
        "existing_evidence": [adopted[0], adopted[2]],
        "acceptance": "Evidence independently establishes the requested fields; then add targeted scene binding checks. Child x/y, count8, type5 or argument7 cannot substitute for backdrop origin or named scene.",
        "stop_condition": "Do not expand retained private next layers. Missing fields stay unknown; no GUI or user-presence prerequisite for independently qualified offline inputs.",
        "blocks_completed_increment": False},
    "handoff_to": ["supervisor"], "wire_notification_required": False,
}
report_path = G / f"{TASK}.json"
save(report_path, report)
progress_path = G / "監製進度.json"
progress = read(progress_path)
progress.update(updated_at=now, phase="BIN_BACKGROUND_RGBA_OFFLINE_CONSUMER_CPU_VERIFIED_LIVE_SCENE_UNKNOWN",
                current_task="Completed new pinned6314/7482 RGBA selector/Img/Canvas offline consumer;3 newCPU checks passed once; production scene/owner/draw binding remains unknown.",
                workflow_task_id=TASK, workflow_task_state="review", workflow_execution_state="waiting_review_after_one_offline_increment",
                workflow_current_output=str(report_path))
progress["modern_bin_consumer_20261007"] = {"task_id": TASK, "state": "review", "handoff": pin(report_path), "new_CPU_passed": 3, "GUI": 0, "overall_complete": False}
progress_path.write_text(json.dumps(progress, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"report": pin(report_path), "new_tests": 3, "source_files": len(after), "state": "review"}, ensure_ascii=False))
