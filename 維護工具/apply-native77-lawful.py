"""Apply only the captured C02 rejection352 recognition and its regression test."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
old=ROOT/'驗證紀錄/current/native77-request-integration/validation-20261017-193125-656714';c=read(old/'cohort.json');source=read(old/'source-inputs.json');inputs=read(old/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert c['status']=='executed_native77_request_normals_related'
for p in ['Debug','Release']:assert read(next(old.glob(p+'-qualified-*/qualified.json')))['passed']==30
failure=ROOT/'驗證紀錄/current/native77-live/20261017-195153-538660';failed=read(failure/'receipt.json');assert failed['all_started_processes_terminal'] and failed['status']=='failed_preserved'
red=ROOT/'驗證紀錄/current/native77-lawful/red-20261017-195354-725276/red.json';rr=read(red);assert rr['actual']=='NoResponse' and rr['expected']=='Rejected(352)' and rr['test_exit_code']==101 and rr['source_manifest_sha256']==c['source_manifest_sha256']
test=ROOT/'維護工具/candidates/native77-lawful/test.rs';assert sha(test)==rr['test_sha256']
world='LiTo381-Modern/src/modern_lito/native_world.rs';tests='LiTo381-Modern/tests/native_attack_skills.rs';w=(ROOT/world).read_text(encoding='utf-8');before='if matches!(code, 276 | 278 | 280 | 285 | 299 | 316) {';assert w.count(before)==1
payload={world:w.replace(before,'if matches!(code, 276 | 278 | 280 | 285 | 299 | 316 | 352) {').encode('utf-8'),tests:((ROOT/tests).read_text(encoding='utf-8')+'\n// Captured current C02 lawful refusal; no local permission or resource prediction.\n'+test.read_text(encoding='utf-8')).encode('utf-8')}
metadata=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
app=ROOT/'驗證紀錄/current/native77-lawful'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));app.mkdir();pins={n:sha(ROOT/n) for n in [*payload,*metadata]}
with zipfile.ZipFile(app/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(ROOT/n,n)
with zipfile.ZipFile(app/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
after={n:hashlib.sha256(v).hexdigest().upper() for n,v in payload.items()}
with zipfile.ZipFile(app/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,v in payload.items():z.writestr(n,v)
with zipfile.ZipFile(app/'proposal.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
(app/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,v in payload.items():(ROOT/n).write_bytes(v);assert sha(ROOT/n)==after[n];source[n]=after[n];inputs[n]=after[n]
save(app/'applied-source-inputs.json',source);save(app/'applied-compiled-inputs.json',inputs)
qualifications={}
for name in ['20261017-194524-997219','20261017-195042-051473']:
    p=sorted((ROOT/'驗證紀錄/current/native77-live'/name).glob('qualified-*/qualified.json'))[-1];q=read(p);qualifications[q['profile']]=dict(path=str(p),sha256=sha(p),actual_requests=q['actual_requests'],actual_accepted_casts=q['actual_accepted_casts'],graphics_observed=q['graphics_observed'])
state=dict(status='applied_native77_lawful_pending_normal_and_live_validation',applied=True,before_source_manifest_sha256=c['source_manifest_sha256'],source_manifest_sha256=sha(app/'applied-source-inputs.json'),source_files=1030,compiled_input_files=1031,before={n:pins[n] for n in payload},after=after,red_test=str(red),red_test_sha256=sha(red),observed_failure=str(failure),previous_normal_qualification=str(old),retained_BA_live_cases=qualifications,normal_validation_complete=False,whole_goal_complete=False)
save(app/'application.json',state);save(app.parent/'active-application.json',dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='實際C02正義值499送352含$967參數，BA546594誤報NoResponse；原失敗及red test已封存。現僅加352至原生拒絕辨識與一個錄包回歸測試，等待當版正常兩主／各31直接相關測試及實際正義值拒絕補驗，來源／QA凍結。BA正常兩版各30、Debug一般11746／自然暴擊11748兩次及MP不足1請求0成功按原版保留，不移記整版；其他技能、完整G06／同端點、G01–G12、GUI實體輸入音效／G11G12仍待。登入器及素材唯讀，產物獨立，原時限預算不變。'
for n in metadata[:5]:
    p=ROOT/n;j=read(p);previous=j['current_product_identity'];assert previous['source_manifest_sha256']==c['source_manifest_sha256'];j['retained_sourceBA_product_identity']=previous
    j['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,source_qualification_pending=True,existing_normal_source_manifest_sha256=c['source_manifest_sha256'],Debug_normal_sha256=previous['Debug_normal_sha256'],Release_normal_sha256=previous['Release_normal_sha256'],Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,qualification_scope='native77_lawful352_pending',GUI_verified=False,whole_goal_complete=False)
    for key,value in j.items():
        if key.startswith('current_native77') and isinstance(value,dict) and value.get('source_manifest_sha256')==c['source_manifest_sha256']:value.update(applies_to_current_source=False,retained_source_version_only=True)
    j['retained_sourceBA_live_cases']=qualifications;j['current_native77_lawful_application']=dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],applied=True,normal_validation_pending=True)
    j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(app/'application.json'),goal_complete=False,whole_goal_complete=False)
    if n==metadata[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-lawful352-normal-and-live',work=work,goal_complete=False)
    save(p,j)
for n in metadata[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8')
save(app/'tracker-readback.json',dict(after={n:sha(ROOT/n) for n in metadata},source_manifest_sha256=state['source_manifest_sha256']));print(json.dumps(dict(application=str(app),source_manifest_sha256=state['source_manifest_sha256'],changed_files=2,added_regressions=1),ensure_ascii=False))
