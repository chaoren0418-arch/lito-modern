"""Seal the new bounded byte consumer and preserve previous queue evidence."""
import datetime
import difflib
import hashlib
import json
from pathlib import Path

P = Path(__file__).resolve().parent.parent
G = P.parent
B = G.parent / "lito脫殼資源"
OUT = P / "target/modern-host-buffer-consumer-20261007"
TASK = "modern-host-buffer-consumer-20261007"
REVIEW = G.parent / f"監製工作流/{TASK}-scope-review.json"


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


before = {row["path"].replace("\\", "/"): row for row in read(OUT / "before-rust-pins.json")}
after = {str(f.relative_to(P)).replace("\\", "/"): pin(f) for f in (P / "src").rglob("*.rs")}
changed = sorted(name for name in before if after[name]["sha256"] != before[name]["sha256"])
new = sorted(set(after) - set(before))
assert changed == ["src/modern_lito/native_entry_transfer.rs", "src/modern_lito/native_startup_host.rs"], changed
assert new == ["src/modern_lito/native_host_buffers.rs", "src/modern_lito/native_host_buffers_tests.rs"], new
assert set(before) <= set(after)

scope = read(REVIEW)
adopted = [pin(REVIEW)]
for evidence in scope["evidence"]:
    if Path(evidence["path"]).name in ("native_entry_transfer.rs", "native_startup_host.rs"):
        original = pin(OUT / (Path(evidence["path"]).name + ".before"))
        assert (original["bytes"], original["sha256"]) == (evidence["bytes"], evidence["sha256"])
    else:
        adopted.append(check(evidence))
aggregate_path = B / "modern-contracts-20261007.json"
assert pin(aggregate_path)["sha256"] == "DC08927CDB282EE14CD80080AD7EDB579EF8560AD98C00E83BFD5877BB39964D"
aggregate = read(aggregate_path)
contract = next(c for c in aggregate["contracts"] if c["id"] == "saved_host_64AD74")
adopted.append(check(aggregate["source_PE"]))
assert contract["saved_host"]["copy_through_first_NUL"] is True
assert contract["saved_host"]["actual_initialized_bytes"] is None
assert contract["port_neighbor"]["width"] == 2

old_startup = (OUT / "native_startup_host.rs.before").read_text(encoding="utf-8-sig")
new_startup = (P / "src/modern_lito/native_startup_host.rs").read_text(encoding="utf-8-sig")
assert old_startup[old_startup.index("impl super::Runtime"):] == new_startup[new_startup.index("impl super::Runtime"):]
runtime = P / "src/modern_lito/native_city_runtime.rs"
assert runtime.read_bytes() == (OUT / "native_city_runtime.rs.before").read_bytes()
old_transfer = (OUT / "native_entry_transfer.rs.before").read_text(encoding="utf-8-sig")
new_transfer = (P / "src/modern_lito/native_entry_transfer.rs").read_text(encoding="utf-8-sig")
assert old_transfer[old_transfer.index("impl super::Runtime"):] == new_transfer[new_transfer.index("impl super::Runtime"):]

for name in ("build", "test-build", "test"):
    accounting = read(OUT / f"{name}-accounting.json")
    assert accounting["exit_code"] == 0 and accounting["attempts"] == 1
stdout = (OUT / "tests.stdout.log").read_text(encoding="utf-8-sig")
assert "test result: ok. 4 passed; 0 failed;" in stdout, stdout
assert stdout.count(" ... ok") == 4, stdout
artifacts = read(OUT / "artifacts.json")
assert artifacts["library_fresh"] is False
check(artifacts["library"])
check(artifacts["test"])
snapshot = read(OUT / "source-snapshot.json")
for row in snapshot["inputs"]:
    check(row["source"])
    check(row["snapshot"])
    assert row["source"]["sha256"] == row["snapshot"]["sha256"]

diff = ""
for name in changed:
    old = OUT / (Path(name).name + ".before")
    diff += "".join(difflib.unified_diff(old.read_text(encoding="utf-8-sig").splitlines(True),
        (P / name).read_text(encoding="utf-8-sig").splitlines(True), fromfile="before/" + name, tofile="after/" + name))
for name in new:
    diff += "".join(difflib.unified_diff([], (P / name).read_text(encoding="utf-8").splitlines(True), fromfile="/dev/null", tofile="after/" + name))
diff_path = OUT / "source.diff"
with diff_path.open("x", encoding="utf-8", newline="\n") as f:
    f.write(diff)
warnings = {}
for name in ("build", "test-build"):
    messages = [json.loads(line) for line in (OUT / f"{name}.jsonl").read_text(encoding="utf-8-sig").splitlines()]
    rows = [m for m in messages if m.get("reason") == "compiler-message" and m["message"]["level"] == "warning"]
    relevant = [m for m in rows if any(s["file_name"].replace("\\", "/").endswith(tuple(changed + new)) for s in m["message"]["spans"])]
    warnings[name] = {"total_warning_messages": len(rows), "changed_or_new_module_warning_messages": len(relevant)}
    assert not relevant

now = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
queue_path = G / "工作佇列.json"
queue_before_path = OUT / "queue.before.json"
with queue_before_path.open("xb") as f:
    f.write(queue_path.read_bytes())
queue = read(queue_path)
original_items = list(queue["items"])
index = read(G.parent / "LITO-監製索引.json")
assignment = next(t for t in index["dispatch_queue"]["items"] if t["id"] == TASK)
save(OUT / "assignment.json", {"latest_human_direction": index["latest_human_direction_20261007"], "task": assignment})
assert not any(t["id"] == TASK for t in queue["items"])
proposal = dict(assignment)
proposal.update(state="review", dependencies=[], upstream_dependency_ids=assignment["dependencies"],
    upstream_dependency_note="Cross-owner accepted inputs are the pinned scope gate; local queue does not rerun upstream tasks.",
    next_deliverable=str(G / f"{TASK}.json"),
    evidence=adopted + [after[n] for n in changed + new])
proposal["gates"] = assignment["gates"] + [
    {"id": "accepted_BIN_buffer_contract", "state": "accepted", "evidence": scope["evidence"][0]},
    {"id": "reviewed_original_consumer_gap", "state": "accepted", "evidence": adopted[0]}]
queue["items"].append(proposal)
assert queue["items"][:-1] == original_items
queue.update(updated_at=now, proposal_status="review", execution_state="waiting_review_after_offline_increment",
    ready_count=sum(t["state"] == "ready" for t in queue["items"]),
    current_task_id=TASK, current_handoff_path=str(G / f"{TASK}.json"),
    independent_work_selection="Scope-reviewed raw host/alias/port consumer is newly delivered for review. Existing blocked product gates remain intact; GUI absence does not prohibit qualified offline work.",
    new_product_work_completed=True, workflow_conversion_only=False)
queue["new_offline_increment"] = {"task_id": TASK, "CPU_passed": 4, "GUI_accepted": False, "overall_complete": False}
queue_path.write_text(json.dumps(queue, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

report = {
    "schema": "modern.host.buffer.consumer.offline.v1", "task_id": TASK, "updated_at": now,
    "state": "DELIVERED_SOURCE_SEMANTIC_OFFLINE_INCREMENT_REVIEW_REQUIRED", "overall_complete": False,
    "adopted_contracts": adopted, "assignment": pin(OUT / "assignment.json"),
    "source_changes": [after[n] for n in changed + new], "source_diff": pin(diff_path),
    "source_snapshot": pin(OUT / "source-snapshot.json"),
    "rust_source_accounting": {"before": len(before), "after": len(after), "new": len(new), "modified": len(changed), "old_files_byte_unchanged": len(before) - len(changed)},
    "semantics": {"host_capacity_bytes_including_NUL": 256, "alias_capacity_bytes_including_NUL": 28, "port_bits": 16,
        "copy_through_first_NUL": True, "copy_includes_NUL": True, "source_tail_ignored": True,
        "old_destination_tail_preserved": True, "uninitialized_distinct_from_empty": True,
        "raw_non_ASCII_unchanged": True, "byte_bound_is_Modern_safety_not_Native_clamp": True,
        "atomic_multi_field_validation": True, "port_signed_view_is_not_socket_authority": True},
    "actual_consumer_wiring": [
        {"path": str(P / changed[1]), "flow": "FixtureStartupHost::parse -> existing synthetic kind/canonical numeric-loopback gate ->HostBuffer::copy_payload; FixtureStartupHost::target reads HostBuffer::payload for same numeric address/port2000", "authority": "Explicit synthetic JSON only; Native050104CD return not asserted"},
        {"path": str(P / changed[0]), "flow": "Existing start_source_transfer_branch ->State::start ->AliasBuffer::copy_payload; State::alias now supplies cached C9 replay; Key.selected_name remains identical presentation payload", "authority": "Same existing source-scope/synthetic/epoch/navigation qualification; alias never supplies startup host"},
        {"path": str(P / changed[0]), "flow": "State::start with supplied target and State::return_two copy existing u16 target port into independent PortWord", "authority": "Records these explicit pure transfer steps; other fixture I/O/timer path port-global equivalence not claimed"}],
    "preserved_call_path_evidence": {"Runtime_configure_startup_gate_verbatim": True, "entry_transfer_Runtime_handlers_verbatim": True,
        "native_city_runtime_byte_unchanged": pin(runtime),
        "native_login_events_byte_unchanged": after["src/modern_lito/native_login_events.rs"],
        "native_entry_navigation_byte_unchanged": after["src/modern_lito/native_entry_navigation.rs"]},
    "build": {"target": "i686-pc-windows-msvc", "library": artifacts["library"], "recompiled": True, "accounting": pin(OUT / "build-accounting.json"), "warnings": warnings},
    "validation": {"passed": 4, "failed": 0, "new_CPU_checks_executions": 1, "old_PASS_replays": 0,
        "cases": ["Host256 and alias28 terminator bounds; raw bytes/source ownership/source tail/destination tail", "Independent fields; multi-field errors publish nothing; all65536 port bit/signed views", "Affected synthetic startup parse->buffer->target path with existing restrictions and refusals", "Affected transfer start->owned alias->replay accessor/return port/revision failure/Modern retirement data paths"],
        "stdout": pin(OUT / "tests.stdout.log"), "test_artifact": artifacts["test"],
        "compile_accounting": pin(OUT / "test-build-accounting.json"), "run_accounting": pin(OUT / "test-accounting.json")},
    "evidence_logs": [pin(OUT / n) for n in ("build.jsonl", "build.stderr.log", "test-build.jsonl", "test-build.stderr.log", "tests.stderr.log", "artifacts.json", "before-rust-pins.json")],
    "tools_and_documentation": [pin(P / n) for n in ("tools/build-modern-host-buffer-consumer-20261007.ps1", "tools/seal-modern-host-buffer-consumer-20261007.py", "docs/native-host-buffer-consumer.md")],
    "proposal_queue": {"before": pin(queue_before_path), "after": pin(queue_path), "existing_items_unchanged": len(original_items), "new_item_state": "review", "coverage": queue["coverage"]},
    "unknowns": contract["unknowns"] + ["Transfer byte_storage.host stays uninitialized; synthetic startup adapter does not publish Native saved-host authority", "Port projection only covers explicit start/return steps; unrelated current endpoint/timer I/O is not a global-byte trace", "Modern key retirement is not proof of Native alias/host/port clearing"],
    "boundary": {"Native_initializer_executed_proven": False, "runtime_host_owner_epoch_manufactured": False, "live_server_connect_qualified": False,
        "default_profile_or_peer_as_startup_authority": False, "private_next_layer_analysis": 0, "GUI": 0, "sockets": 0, "provider": 0, "Java_SQL": 0, "game_launches": 0, "READY_reads": 0, "claims": 0, "server_namespace_writes": 0,
        "retained_restrictions": "495520/050104CD/private42E550/42E6D0/42EA50; Native tools; T291/T27/refund/rebid/Store/auction/castle/full-WORLD"},
    "new_or_corrected_wire": False, "wire_notification_required": False, "handoff_to": ["supervisor"],
    "next_step": "Stop after this increment and await supervisor evidence review. No missing Native authority is replaced by a synthetic data copy.",
}
report_path = G / f"{TASK}.json"
save(report_path, report)
progress_path = G / "監製進度.json"
progress = read(progress_path)
if progress.get("workflow_handoff"):
    progress.setdefault("workflow_handoff_history", []).append({"replaced_at": now, "handoff": progress["workflow_handoff"]})
progress.update(updated_at=now, phase="OFFLINE_HOST_ALIAS_PORT_BUFFER_CONSUMER_CPU_VERIFIED_NATIVE_AUTHORITY_UNKNOWN",
    current_task="New rawhost256/alias28/port16 consumer integrated in synthetic startup and existing transfer alias/port data paths;4 newCPU checks passed once;Native initializer/runtime authority remains unknown.",
    workflow_task_id=TASK, workflow_task_state="review", workflow_execution_state="waiting_review_after_one_offline_increment", workflow_current_output=str(report_path), workflow_handoff=pin(report_path))
progress["modern_host_buffer_consumer_20261007"] = {"task_id": TASK, "state": "review", "handoff": pin(report_path), "queue": pin(queue_path), "new_CPU_passed": 4, "GUI": 0, "overall_complete": False}
progress_path.write_text(json.dumps(progress, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"report": pin(report_path), "new_tests": 4, "source_files": len(after), "queue_items": len(queue["items"]), "state": "review"}, ensure_ascii=False))
