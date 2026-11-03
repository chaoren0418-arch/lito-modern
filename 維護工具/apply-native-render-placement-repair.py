"""Apply the one-file raster metadata repair after original failure preservation."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
C=R/'維護工具/candidates/native-render-placement-20261019-200226-719223'
OLD='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');assert w['current_product_identity']['source_manifest_sha256']==OLD
assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
assert w['current_full_default_library_verified']
normal=w['current_native_rental_service_entry_return_normal_qualification'];lib=w['current_native_catalog_library'];res=w['current_native_library_resource_optin'];failed=w['current_native_library_shared_resource_optin']
assert normal['qualification_granted']and res['qualified']and failed['original_failure_preserved']and not failed['qualified']
assert normal['source_manifest_sha256']==lib['source_manifest_sha256']==res['source_manifest_sha256']==failed['source_manifest_sha256']==OLD
assert sha(normal['qualification'])==normal['qualification_sha256']and sha(res['qualification'])==res['qualification_sha256']
assert sha(lib['completion'])==lib['completion_sha256']and sha(failed['raw_archive'])==failed['raw_archive_sha256']
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(R)
base=Path(lib['run']);pins=read(base/'source-inputs.json');inputs=read(base/'compiled-inputs.json')
assert sha(base/'source-inputs.json')==OLD and len(pins)==1048 and len(inputs)==1049
assert all(sha(R/n)==h for n,h in inputs.items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
c=read(C/'candidate.json');assert c['base_source_manifest_sha256']==OLD and not c['compiled']and not c['applied']
assert c['failed_evidence']==failed['analysis']and c['failure_archive_sha256']==failed['raw_archive_sha256']
assert len(c['changed_files'])==1
payloads={}
for n in c['changed_files']:
    assert sha(R/n)==c['before_files'][n]and sha(C/Path(n).name)==c['candidate_files'][n]
    payloads[n]=(C/Path(n).name).read_bytes()
newpins=dict(pins);newinputs=dict(inputs)
for n,b in payloads.items():newpins[n]=newinputs[n]=hashlib.sha256(b).hexdigest().upper()
source_bytes=data(newpins);source=hashlib.sha256(source_bytes).hexdigest().upper();assert source!=OLD
out=V/'native-render-placement-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='applied_native_render_placement_pending_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=source,
    changed_files=list(payloads),created_files=[],after_files=c['candidate_files'],source_files=1048,compiled_input_files=1049,
    candidate=str(C/'candidate.json'),candidate_sha256=sha(C/'candidate.json'),normal_validation_complete=False,
    normal_product_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,
    formal_JAR_modified=False,clocks_changed=False,normal_GUI_verified=False,whole_goal_complete=False,change_scope=c['change_scope'])
work='來源'+source[:8]+'套用一檔原生繪製回傳排序／可見性標號修補，保留原光柵插入順序並新增獨立重現案。先正常兩版19冒煙與完整library Debug／Release，再原資源及正常GUI；BC2C的library兩版1969過、native77各10過、額外資源Debug65過7敗保留原來源。六項舊資源／Text參考條件仍待處理，不放寬斷言、不交人工。'
staged=dict(payloads)
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);identity=v['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        v['qualification_before_render_placement_repair']=dict(source_manifest_sha256=OLD,normal=normal,library=lib,resource=res,resource_failure=failed,application=v['current_native_rental_service_entry_return_application'])
        identity.update(source_manifest_sha256=source,source_files=1048,compiled_input_files=1049,source_qualification_pending=True,
            Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        v.update(current_full_default_library_verified=False,current_native_render_placement_application=record,
            working_tree_matches_current_normal_qualification=False,working_tree_modified_since_current_normal_qualification=True,
            next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        for info in v['current_source_goal_validation'].values():info.update(complete=False,current_source_manifest_sha256=source,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in v.get('items',[]):item.update(complete=False,current_source_manifest_sha256=source,current_evidence_applies_to_current_source=False)
        v['current_focus']=dict(item='native-render-placement-validation',work=work,applies_to_current_source=True,goal_complete=False)
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
