"""Apply the one-file raster metadata repair after original failure preservation."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
C=Path('D:\\lineage開發\\3-lito開發\\lito重建版\\維護工具\\candidates\\native-shared-world-identity-fixture-20261019-212326-812135')
OLD='4018AE86474EA2977F0083034E81A86D4F77F1C51EDF828B2456B369876DA24D'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==OLD
assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
normal=w['current_native_installed_resource_profile_normal_qualification'];failed=read(Path('D:\\lineage開發\\3-lito開發\\lito重建版\\驗證紀錄\\current\\native-installed-resource-profile-shared-resource-optin\\20261019-211852-446404\\failure-analysis-20261019-212325-419169\\evidence.json'))
assert normal['source_manifest_sha256']==failed['source_manifest_sha256']==OLD
assert normal['qualification_granted']and failed['observed_exit_code']==1 and failed['recorded_original_owned_processes_all_terminal']
assert sha(normal['qualification'])==normal['qualification_sha256']and sha(failed['raw_archive'])==failed['raw_archive_sha256']
qualification=read(normal['qualification']);assert qualification['observed_terminal_exit_code']==0 and qualification['recorded_owned_processes_all_terminal']and qualification['passed_per_profile']==19
base=Path(normal['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json')
assert sha(base/'source-inputs.json')==OLD and len(pins)==1048 and len(inputs)==1049
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(R)
assert all(sha(R/n)==h for n,h in inputs.items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
c=read(C/'candidate.json');assert c['base_source_manifest_sha256']==OLD and not c['compiled']and not c['applied']
assert c['failure_archive_sha256']==failed['raw_archive_sha256']and c['test_only_fixture_repair']and c['production_prefix_unchanged']
assert len(c['changed_files'])==1
payloads={}
for n in c['changed_files']:
    assert sha(R/n)==c['before_files'][n]and sha(C/Path(n).name)==c['candidate_files'][n]
    payloads[n]=(C/Path(n).name).read_bytes()
newpins=dict(pins);newinputs=dict(inputs)
for n,b in payloads.items():newpins[n]=newinputs[n]=hashlib.sha256(b).hexdigest().upper()
source_bytes=data(newpins);source=hashlib.sha256(source_bytes).hexdigest().upper();assert source!=OLD
out=V/'native-installed-resource-profile-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='applied_render_test_fixture_repair_pending_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=source,
    changed_files=list(payloads),created_files=[],after_files=c['candidate_files'],source_files=1048,compiled_input_files=1049,
    candidate=str(C/'candidate.json'),candidate_sha256=sha(C/'candidate.json'),normal_validation_complete=False,
    normal_product_source_modified=False,test_only_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,
    formal_JAR_modified=False,clocks_changed=False,normal_GUI_verified=False,whole_goal_complete=False,change_scope=c['change_scope'])
work='來源'+source[:8]+'只修world_scene測試fixture：依ID1001／1002核對原subframe與owner像素oracle，另驗深度順序1002／1001，原像素／遮罩雜湊與移動斷言保留。4018AE86額外資源Debug70過1敗／Release未啟動已封存；先定位原失敗，再正常兩版、完整library與原資源兩版，接GUI全流程，不交人工。'
staged=dict(payloads)
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);identity=v['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        v['qualification_before_shared_world_identity_fixture_repair']=dict(source_manifest_sha256=OLD,normal=normal,library=v['current_native_catalog_library'],resource=v['current_native_library_resource_optin'],shared_failure=failed,application=v['current_native_installed_resource_profile_application']); v['current_native_library_shared_resource_failure']=failed
        identity.update(source_manifest_sha256=source,source_files=1048,compiled_input_files=1049,source_qualification_pending=True,
            Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        v.update(current_full_default_library_verified=False,current_native_installed_resource_profile_application=record,
            working_tree_matches_current_normal_qualification=False,working_tree_modified_since_current_normal_qualification=True,
            next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        for info in v['current_source_goal_validation'].values():info.update(complete=False,current_source_manifest_sha256=source,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in v.get('items',[]):item.update(complete=False,current_source_manifest_sha256=source,current_evidence_applies_to_current_source=False)
        v['current_focus']=dict(item='native-render-test-fixture-validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=data(v)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in [*before,*payloads]:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in {**before,**c['before_files']}.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,b in payloads.items():z.writestr(n,b)
for n,b in [('before-source-inputs.json',(base/'source-inputs.json').read_bytes()),('before-compiled-inputs.json',(base/'compiled-inputs.json').read_bytes()),('applied-source-inputs.json',source_bytes),('applied-compiled-inputs.json',data(newinputs))]:(out/n).write_bytes(b)
(out/'application.py').write_bytes(Path(__file__).read_bytes())
assert len(staged)==9 and all(sha(R/n)==h for n,h in before.items())and all(sha(R/n)==h for n,h in inputs.items())
for n,b in staged.items():(R/n).write_bytes(b)
assert all(sha(R/n)==h for n,h in newinputs.items())
record.update(before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files={**before,**c['before_files']})
(out/'application.json').write_bytes(data(record));(out/'evidence.json').write_bytes(data(record))
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},source_after_files=record['after_files'],before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(application=str(out),source_manifest_sha256=source,changed_files=list(payloads),compiled=False),ensure_ascii=False))
