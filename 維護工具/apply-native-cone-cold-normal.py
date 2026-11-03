"""Apply the qualified rebuild-only cold/join overlay with source recovery archives."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';SRC=P/'src/modern_lito';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads')and not queue.get('source_and_corresponding_QA_frozen')
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(ROOT)
base=V/'native-class2-normal/validation-20261018-014519-231950'
inputs=read(base/'compiled-inputs.json');old_source=V/'native-class2-normal/application-20261018-014327-437770/applied-source-inputs.json';source=read(old_source)
assert len(source)==1034 and len(inputs)==1035 and sha(old_source)==queue['current_product_identity']['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h
checkpoint=V/'native-pending-join-departure/checkpoint-20261018-055148-335305';gate=read(checkpoint/'evidence.json')
assert gate['status']=='combined_cold_and_pending_join_candidate_scoped_gates_qualified_normal_unapplied'
assert gate['request_contracts']['cases_each']==3 and gate['request_contracts']['observed_actual_exit_code']==0 and gate['request_contracts']['recorded_original_identities_all_terminal']
assert gate['pending_join_cases_each']==14 and gate['existing34_C02_sessions']==8
for item in [gate['proposal'],gate['pending_join_contracts'],*gate['existing34_C02_scopes']]:assert sha(item['path'])==item['sha256']
proposal=Path(gate['proposal']['path']);p=read(proposal);assert p['files']==11 and p['source_manifest_sha256']==sha(old_source)
payload={}
for n,h in p['proposed_sources'].items():
    f=proposal.parent/'sources'/n;assert sha(f)==h;payload[(SRC/n).relative_to(ROOT).as_posix()]=f.read_bytes()
for n,h in p['changed_original_files'].items():assert sha(SRC/n)==h
qa=ROOT/'維護工具/candidates/native-cone-cold-normal';prepared=read(qa/'prepared.json')
assert sha(qa/'native_cone_cold.rs')==prepared['test_source_sha256']and sha(qa/'fixtures/native-cone-cold-current.json')==prepared['fixture_sha256']
payload[(P/'tests/native_cone_cold.rs').relative_to(ROOT).as_posix()]=(qa/'native_cone_cold.rs').read_bytes()
payload[(P/'tests/fixtures/native-cone-cold-current.json').relative_to(ROOT).as_posix()]=(qa/'fixtures/native-cone-cold-current.json').read_bytes()
join=Path(read(gate['pending_join_contracts']['path'])['proposal']);jp=read(join/'proposal.json')
assert sha(join/'probe-addon.rs')==jp['probe_addon_sha256']
test=(join/'probe-addon.rs').read_text('utf-8-sig').replace('use super::native_room::{EarlyRoster, Member};','use super::super::native_room::{EarlyRoster, Member};').replace('pub fn pending_join_departure_replay()','fn pending_join_departure_replay()',1)
test='use super::*;\n'+test+'\n#[test]\nfn observed_early_departure_and_fourteen_adversarial_join_contracts(){\n let result=pending_join_departure_replay().unwrap();\n assert_eq!(result["cases"].as_array().unwrap().len(),14);\n assert_eq!(result["new_network_sessions"],0);\n}\n'
payload[(SRC/'native_pending_join_departure_tests.rs').relative_to(ROOT).as_posix()]=test.encode('utf-8')
login=(SRC/'native_login.rs').relative_to(ROOT).as_posix();text=payload[login].decode('utf-8-sig');assert 'mod pending_join_departure_tests;'not in text
payload[login]=(text+'\n#[cfg(test)]\n#[path = "native_pending_join_departure_tests.rs"]\nmod pending_join_departure_tests;\n').encode('utf-8')
assert len(payload)==14
before_meta=read(checkpoint/'readback.json')['after_files'];assert len(before_meta)==8 and all(sha(ROOT/n)==h for n,h in before_meta.items())
out=V/'native-cone-cold-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
existing={**{n:ROOT/n for n in payload if(ROOT/n).exists()},**{n:ROOT/n for n in before_meta}};before={n:sha(f)for n,f in existing.items()}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,f in existing.items():z.write(f,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
after={n:hashlib.sha256(data).hexdigest().upper()for n,data in payload.items()}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,data in payload.items():z.writestr(n,data)
with zipfile.ZipFile(out/'proposal.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
state=dict(status='prepared_cold_join_normal_application',applied=False,before_source_manifest_sha256=sha(old_source),candidate_checkpoint=dict(path=str(checkpoint/'evidence.json'),sha256=sha(checkpoint/'evidence.json')),candidate_proposal=dict(path=str(proposal),sha256=sha(proposal)),before_files=before,after_files=after,new_files=[n for n in payload if not(ROOT/n).exists()],before_archive_sha256=sha(out/'before.zip'),proposal_archive_sha256=sha(out/'proposal.zip'),normal_validation_complete=False,GUI_verified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,data in payload.items():
    target=ROOT/n;assert target.resolve().is_relative_to(P)
    temp=out/Path(n).name;temp.write_bytes(data);os.replace(temp,target);assert sha(target)==after[n];inputs[n]=source[n]=after[n]
source=dict(sorted(source.items()));inputs=dict(sorted(inputs.items()));assert len(source)==1040 and len(inputs)==1041
save(out/'applied-source-inputs.json',source);save(out/'applied-compiled-inputs.json',inputs)
state.update(status='applied_cold_join_pending_normal_related_and_current_source_runtime_validation',applied=True,source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=len(source),compiled_input_files=len(inputs));save(out/'application.json',state)
save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='冰錐／入房順序合併候選已套入重建版正常來源，另加入三項請求／S23／目標身分測試與十四入房契約之私有測試；正常兩主、相關目標及當版C02／G06等資格尚待。舊86E6所有結果只保留原來源資格，不移記新版本。先沿唯一入口建置兩版、執行相關測試並保持來源與QA凍結。登入器／正式JAR只讀且產物獨立，GUI／實體音效、完整G06與G01–G12未完成；8／60秒、Release與G11不變。'
for n in before_meta:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);old=value['current_product_identity'];assert old['source_manifest_sha256']==state['before_source_manifest_sha256']
        value['retained_source_86E6_product_identity']=old;value['retained_source_86E6_goal_validation']=value.get('current_source_goal_validation')
        value['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,source_qualification_pending=True,Debug_normal_sha256=old['Debug_normal_sha256'],Release_normal_sha256=old['Release_normal_sha256'],existing_normal_source_manifest_sha256=state['before_source_manifest_sha256'],Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        value['current_native_cold_join_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True)
        value['current_source_goal_validation']={key:dict(state='cold_join_current_source_validation_pending',complete=False)for key in(value.get('current_source_goal_validation')or{})}
        for item in value.get('items',[]):
            if item['id'].startswith(('G01-','G03-')):item.update(state='cold_join_current_source_validation_pending',complete=False)
        value.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='cold-join-normal-build-and-related-validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'tracker-readback.json',dict(after_files={n:sha(ROOT/n)for n in before_meta},source_manifest_sha256=state['source_manifest_sha256']))
assert all(sha(ROOT/n)==h for n,h in read(out/'tracker-readback.json')['after_files'].items())
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],source_files=1040,changed_files=14,normal_validation_complete=False),ensure_ascii=False))
