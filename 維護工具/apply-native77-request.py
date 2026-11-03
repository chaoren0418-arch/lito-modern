"""Apply the reviewed C14 branch and preserve five contract tests in the real target."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern';SRC=PROJECT/'src/modern_lito';PREP=ROOT/'維護工具/candidates/native77-request'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
candidate=queue['current_native77_request_candidate'];assert candidate['qualified'] and not candidate['applied'];qp=Path(candidate['qualification']);assert sha(qp)==candidate['qualification_sha256'];q=read(qp)
assert q['passed']==5 and q['failed']==q['ignored']==0 and q['source_manifest_sha256']==queue['current_product_identity']['source_manifest_sha256']
for n,h in q['candidate_pins'].items():assert sha(n)==h
normal=queue['current_native77_assets_revalidation'];assert normal['qualified'] and normal['current_assets_qualified_for_this_scope'] and normal['source_manifest_sha256']==q['source_manifest_sha256']
assert sha(normal['qualification'])==normal['qualification_sha256'];nq=read(normal['qualification']);old=Path(nq['original_cohort']);c=read(old/'cohort.json');inputs=read(old/'compiled-inputs.json');source=read(old/'source-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for p,v in c['profiles'].items():assert sha(v['normal']['path'])==nq['profiles'][p]['normal_sha256']
assets=read(normal['current_archive_inputs']);assert sha(normal['current_archive_inputs'])==normal['current_archive_inputs_sha256']
for n,v in (assets['archives']|assets['sound_metadata']).items():
    f=Path(n);assert (f.stat().st_size,f.stat().st_mtime_ns)==(v['bytes'],v['mtime_ns'])
p=read(PREP/'preparation.json')
for name,h in p['before'].items():assert sha(SRC/name)==h
test=PROJECT/'tests/native_attack_skills.rs';probe=(PREP/'request_probe.rs').read_text(encoding='utf-8-sig');start=probe.index('use modern_lito::{')
body=probe[start:].replace('use modern_lito::{','use lito_modern::modern_lito::{',1).replace('__WORLD_FIXTURE__','fixtures/native-world-current.json')
assert '#[test]' in body and body.count('#[test]')==5 and '__' not in body
original_test=test.read_text(encoding='utf-8-sig');assert 'mod disintegrate_request_contract' not in original_test
payload={(SRC/name).relative_to(ROOT).as_posix():(PREP/name).read_bytes() for name in ['native_skills.rs','native_skill_selection.rs']}
test_name=test.relative_to(ROOT).as_posix();payload[test_name]=(original_test+'\n\n// Current C02 skill77 request/selection contract; server retains MP/lawful authority.\nmod disintegrate_request_contract {\n'+body+'\n}\n').encode('utf-8')
metadata=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
 '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
app=ROOT/'驗證紀錄/current/native77-request-integration'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));app.mkdir(parents=True)
before={name:sha(ROOT/name) for name in [*payload,*metadata]}
with zipfile.ZipFile(app/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(app/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items())
after={name:hashlib.sha256(b).hexdigest().upper() for name,b in payload.items()}
with zipfile.ZipFile(app/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,b in payload.items():z.writestr(name,b)
with zipfile.ZipFile(app/'proposal.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in after.items())
state=dict(status='prepared_native77_request_application',before_source_manifest_sha256=q['source_manifest_sha256'],candidate_qualification=str(qp),candidate_qualification_sha256=sha(qp),previous_normal_qualification=normal,
    before={name:before[name] for name in payload},after=after,before_archive_sha256=sha(app/'before.zip'),proposal_archive_sha256=sha(app/'proposal.zip'),source_files=1030,compiled_input_files=1031,
    applied=False,normal_validation_complete=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(app/'application.json',state);(app/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for name,h in inputs.items():assert sha(ROOT/name)==h
for name,b in payload.items():
    tmp=app/Path(name).name;tmp.write_bytes(b);os.replace(tmp,ROOT/name);assert sha(ROOT/name)==after[name];source[name]=after[name];inputs[name]=after[name]
assert len(source)==1030 and len(inputs)==1031
save(app/'applied-source-inputs.json',source);save(app/'applied-compiled-inputs.json',inputs)
state.update(status='applied_native77_request_pending_normal_and_live_validation',applied=True,source_manifest_sha256=sha(app/'applied-source-inputs.json'));save(app/'application.json',state)
save(app.parent/'active-application.json',dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='77之C14物件／現行座標請求、十格範圍、5000ms重用及技能選單文字已套入重建版，5個已隔離驗證之契約測試已加入既有native_attack_skills測試目標；MP／善惡值交由伺服器決定，客戶端不扣改權威。新來源正常兩版／相關測試及C02實際施放尚待。舊C831F9BC兩主與新素材補驗只保留原版限定資格，不移記；C02三組連線前資料預檢可按相同harness／JAR／B64重用，尚無新server／sessions／施法。先建置當版正常程式、驗相關請求與選擇，再接實際C14／ack／冷卻／一般與暴擊。其他技能、完整G06／同端點與G01–G12、GUI實體輸入音效／G11G12仍待；登入器及素材只讀、產物獨立，原時限／預算不變。'
for name in metadata[:5]:
    path=ROOT/name;j=read(path);previous=j['current_product_identity'];assert previous['source_manifest_sha256']==q['source_manifest_sha256'];j['retained_sourceC831_product_identity']=previous
    j['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,source_qualification_pending=True,
        Debug_normal_sha256=previous['Debug_normal_sha256'],Release_normal_sha256=previous['Release_normal_sha256'],existing_normal_source_manifest_sha256=q['source_manifest_sha256'],Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,
        qualification_scope='native77_request_current_source_pending',GUI_verified=False,whole_goal_complete=False)
    for key,value in j.items():
        if key.startswith('current_native77') and isinstance(value,dict) and value.get('source_manifest_sha256')==q['source_manifest_sha256']:value.update(applies_to_current_source=False,retained_source_version_only=True)
    j['current_native77_request_candidate'].update(applied=True,application=str(app),applied_source_manifest_sha256=state['source_manifest_sha256'])
    j['current_native77_request_application']=dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],applied=True,normal_validation_pending=True)
    j['retained_sourceC831_goal_validation']=j.get('current_source_goal_validation');j['current_source_goal_validation']={key:dict(state='native77_request_current_source_validation_pending',complete=False) for key in (j.get('current_source_goal_validation') or {})}
    j.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(app/'application.json'),goal_complete=False,whole_goal_complete=False)
    if name==metadata[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-request-normal-and-live-validation',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for name in metadata[5:]:
    path=ROOT/name;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(app/'tracker-readback.json',dict(after_files={n:sha(ROOT/n) for n in metadata},production_sources=after,source_manifest_sha256=state['source_manifest_sha256'],normal_validation_complete=False))
print(json.dumps(dict(application=str(app),source_manifest_sha256=state['source_manifest_sha256'],changed_sources=3,added_tests=5,normal_validation_complete=False),ensure_ascii=False))
