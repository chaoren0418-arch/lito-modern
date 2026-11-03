"""Register static preparation while preserving the live full-library reservation."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
SOURCE='4018AE86474EA2977F0083034E81A86D4F77F1C51EDF828B2456B369876DA24D'
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==SOURCE
assert w['active_workload_sessions']=={'full_library':45118}and w['source_and_corresponding_QA_frozen']
library=Path(w['active_workloads']['full_library']['run']);assert read(library/'cohort.json')['source_manifest_sha256']==SOURCE
for n,h in read(library/'compiled-inputs.json').items():assert sha(R/n)==h
prepared=V/'native-installed-resource-profile-nondefault-character-fixture/prepared-20261019-205404-594023'
fixture=read(prepared/'prepared.json');assert fixture['source_manifest_sha256']==SOURCE and not fixture['compiler_or_runtime_started']
for n,h in fixture['fixture_sources'].items():assert sha(prepared/n)==h
contract=V/'native-nondefault-stat-contract/20261019-202516-758089/readback.json';static=read(contract)
assert not static['JVM_started']and not static['stat_continuity_verified']and static['network_sessions']==0
candidate=R/'維護工具/candidates/native-current-nondefault-passive-20261019-210334-920608';probe=read(candidate/'preparation.json')
assert probe['base_source_manifest_sha256']==SOURCE and not probe['compiled']and not probe['runtime_started']
assert sha(candidate/'probe.rs')==probe['probe_sha256']
assert probe['passive_snapshot_only']and not probe['handler_or_constructor_or_clocks_changed']
for n,h in probe['fixture_sources'].items():assert sha(candidate/n)==h
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'native-nondefault-preparation-checkpoint'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
records=dict(current_native_nondefault_initial_fixture_preparation=dict(source_manifest_sha256=SOURCE,prepared=str(prepared),prepared_sha256=sha(prepared/'prepared.json'),initial_fields_per_character=14,compiled=False,runtime_started=False,stat_continuity_verified=False),
 current_native_nondefault_stat_contract=dict(formal_jar_sha256=static['formal_jar_sha256'],readback=str(contract),readback_sha256=sha(contract),methods=9,static_only=True,stat_continuity_verified=False),
 current_native_nondefault_same_endpoint_probe_preparation=dict(source_manifest_sha256=SOURCE,candidate=str(candidate/'preparation.json'),candidate_sha256=sha(candidate/'preparation.json'),probe_sha256=probe['probe_sha256'],initial_server_fields=14,wire_projection_fields=9,original_five_wire_claimed=False,passive_server_14_field_trace_prepared=True,compiled=False,runtime_started=False,stat_continuity_verified=False))
evidence=dict(status='static_preparation_registered_original_library45118_unchanged',source_manifest_sha256=SOURCE,active_workload_sessions=w['active_workload_sessions'],records=records,additional_compiler_or_runtime_started=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(evidence));(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());staged={}
for n in before:
 p=R/n
 if p.suffix=='.json':
  v=read(p);assert v['active_workload_sessions']=={'full_library':45118};v.update(records)
  v.update(last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  staged[n]=data(v)
 else:staged[n]=p.read_bytes()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),active_session=45118,prepared_only=True),ensure_ascii=False))
