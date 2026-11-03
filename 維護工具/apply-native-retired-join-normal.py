"""Apply only the qualified retired-peer ordering repair and its normal regression."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current';P=ROOT/'LiTo381-Modern';SRC=P/'src/modern_lito'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')
normal=V/'native-cone-cold-normal/validation-20261018-055501-340560';nc=read(normal/'cohort.json');no=read(normal/'completion-observation.json')
assert no['observed_terminal_exit_code']==0 and no['recorded_original_owned_processes_all_terminal']and sha(normal/'cohort.json')==no['cohort_sha256']
source=read(normal/'source-inputs.json');inputs=read(normal/'compiled-inputs.json');assert len(source)==1040 and len(inputs)==1041
assert sha(normal/'source-inputs.json')==queue['current_product_identity']['source_manifest_sha256']==nc['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h
qp=V/'native-retired-join-order/qualified-20261018-062835-125224/qualified.json';q=read(qp)
assert q['status']=='independently_qualified_unchanged_red_and_both_retired_peer_pending_join_contracts'
assert q['cases_each']==18 and q['new_cases_each']==4 and q['preserved_pending_join_cases_each']==14 and q['recorded_original_identities_all_terminal']
assert q['base_source_manifest_sha256']==nc['source_manifest_sha256']and sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip')as z:
    for n,h in q['raw_files'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
candidate=Path(q['proposal']);pj=read(candidate/'proposal.json');assert sha(candidate/'proposal.json')==q['proposal_sha256']
payload={}
for n,h in pj['sources'].items():
    assert sha(candidate/n)==h and sha(SRC/n)==pj['before_sources'][n];payload[(SRC/n).relative_to(ROOT).as_posix()]=(candidate/n).read_bytes()
fixed=Path(pj['fixed_probe']['path']);assert sha(fixed)==pj['fixed_probe']['sha256']
test=SRC/'native_pending_join_departure_tests.rs';old=test.read_text('utf-8-sig');assert 'fn retired_peer_pending_join_replay('not in old
addition=fixed.read_text('utf-8-sig').replace('pub fn pending_join_departure_replay()','fn retired_peer_pending_join_replay()',1).replace('use super::native_room::{self,Packet,Departure};','use super::super::native_room::{self,Packet,Departure};',1)
addition+='\n#[test]\nfn retired_peer_notifications_cannot_preempt_pending_join_or_erase_new_roster(){\n let result=retired_peer_pending_join_replay().unwrap();\n assert_eq!(result["cases"].as_array().unwrap().len(),4);\n assert_eq!(result["new_network_sessions"],0);\n}\n'
payload[test.relative_to(ROOT).as_posix()]=(old+'\n'+addition).encode('utf-8');assert len(payload)==3
previous=normal/'checkpoint-completed-20261018-062334-232560/readback.json';before_meta=read(previous)['after_files'];assert len(before_meta)==8 and all(sha(ROOT/n)==h for n,h in before_meta.items())
out=V/'native-retired-join-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
existing={**{n:ROOT/n for n in payload},**{n:ROOT/n for n in before_meta}};before={n:sha(f)for n,f in existing.items()}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,f in existing.items():z.write(f,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
after={n:hashlib.sha256(data).hexdigest().upper()for n,data in payload.items()}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,data in payload.items():z.writestr(n,data)
with zipfile.ZipFile(out/'proposal.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
state=dict(status='prepared_retired_join_normal_application',applied=False,before_source_manifest_sha256=nc['source_manifest_sha256'],candidate_qualification=dict(path=str(qp),sha256=sha(qp)),candidate_proposal=dict(path=str(candidate/'proposal.json'),sha256=sha(candidate/'proposal.json')),before_files=before,after_files=after,new_files=[],before_archive_sha256=sha(out/'before.zip'),proposal_archive_sha256=sha(out/'proposal.zip'),normal_validation_complete=False,GUI_verified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,data in payload.items():
    target=ROOT/n;assert target.resolve(strict=True).is_relative_to(P)
    temp=out/Path(n).name;temp.write_bytes(data);os.replace(temp,target);assert sha(target)==after[n];inputs[n]=source[n]=after[n]
source=dict(sorted(source.items()));inputs=dict(sorted(inputs.items()));assert len(source)==1040 and len(inputs)==1041
save(out/'applied-source-inputs.json',source);save(out/'applied-compiled-inputs.json',inputs)
state.update(status='applied_retired_join_pending_normal_related_and_current_source_runtime_validation',applied=True,source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=1040,compiled_input_files=1041);save(out/'application.json',state)
save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='已將已知舊房間peer通知優先一次性消耗之修補與四案正常回歸套入；只改兩個Runtime來源與一個私有測試檔，冰錐與其他來源未改。874B正常兩版各89過維持原版本資格；未改874B控制器已重現不同房拒絕與同房誤刪新暫存成員，候選兩版各18影子案例通過、9記錄身分退出。新來源正常兩主及相關檢查尚待，須沿唯一入口兩版建置與凍結驗證，再執行最終來源G06及當版實際流程。登入器／正式JAR只讀且產物獨立；GUI／實體輸入音效、完整G06與G01–G12未完成，8／60秒、Release與G11不變。'
for n in before_meta:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);old_identity=value['current_product_identity'];assert old_identity['source_manifest_sha256']==state['before_source_manifest_sha256']
        value['retained_source_874B_product_identity']=old_identity;value['retained_source_874B_goal_validation']=value.get('current_source_goal_validation')
        for key,item in value.items():
            if key.startswith('current_native')and isinstance(item,dict)and item.get('source_manifest_sha256')==state['before_source_manifest_sha256']:item.update(applies_to_current_source=False,retained_source_version_only=True)
        value['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,source_qualification_pending=True,Debug_normal_sha256=old_identity['Debug_normal_sha256'],Release_normal_sha256=old_identity['Release_normal_sha256'],existing_normal_source_manifest_sha256=state['before_source_manifest_sha256'],Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        value['current_native_retired_join_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True)
        value['current_source_goal_validation']={key:dict(state='retired_join_current_source_validation_pending',complete=False)for key in(value.get('current_source_goal_validation')or{})}
        for item in value.get('items',[]):
            if item['id'].startswith(('G01-','G03-')):item.update(state='retired_join_current_source_validation_pending',complete=False)
        value.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='retired-peer-order-normal-build-and-final-source-G06',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'tracker-readback.json',dict(after_files={n:sha(ROOT/n)for n in before_meta},source_manifest_sha256=state['source_manifest_sha256']))
assert all(sha(ROOT/n)==h for n,h in read(out/'tracker-readback.json')['after_files'].items())
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],changed_files=3,source_files=1040,normal_validation_complete=False),ensure_ascii=False))
