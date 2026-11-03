"""Publish independently read current C02 diagnostic cases, retaining incomplete gates."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('qualifications',nargs='+',type=Path);a=ap.parse_args();cases={}
for p in a.qualifications:
    p=p.resolve(strict=True);assert p.is_relative_to(ROOT/'驗證紀錄/current/native77-live');q=read(p);assert q['status']=='independently_qualified_native77_C02_controller_presentation_diagnostic_scope' and q['recorded_owned_all_terminal']
    assert q['source_manifest_sha256']=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182' and q['profile'] not in cases
    cases[q['profile']]=dict(qualification=str(p),qualification_sha256=sha(p),**{k:q[k] for k in ['source_manifest_sha256','original_session_id','terminal_tool_chunk','actual_sessions','actual_requests','actual_accepted_casts','graphics_observed','both_critical_variants_observed','recorded_owned_identities','recorded_owned_all_terminal','legacy_runner_completed_responses_mislabeled_as_casts']})
    c=read(Path(q['run'])/'receipt.json');assert c['all_started_processes_terminal'] and not c['owned_processes']
    for n,h in read(Path(c['compile'])/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
assert set(cases)=={'disintegrate','disintegrate-refused','disintegrate-lawful'},'publish three independently read cases together'
positive=cases['disintegrate'];both=positive['both_critical_variants_observed']
failures=[]
for name,session,chunk in [('20261017-194618-103196',35236,'5cee8a'),('20261017-194748-160795',90733,'fbb84c'),('20261017-195153-538660',73576,'992971')]:
    p=ROOT/'驗證紀錄/current/native77-live'/name/'failure-preserved.json';f=read(p);assert f['original_session_id']==session and f['terminal_chunk']==chunk;failures.append(dict(path=str(p),sha256=sha(p),original_exit_code=1,cause=f['cause']))
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=ROOT/'驗證紀錄/current/native77-live'/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(source_manifest_sha256=positive['source_manifest_sha256'],cases=cases,actual_sessions=sum(v['actual_sessions'] for v in cases.values()),actual_requests=sum(v['actual_requests'] for v in cases.values()),actual_accepted_casts=positive['actual_accepted_casts'],both_critical_variants_observed=both,failed_probe_runs_preserved=failures,scope='Debug production-library controller and Presentation diagnostics against isolated current C02 only',normal_GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
work='來源E15907AC正常Debug／Release各31相關測試維持。現以當版Debug產品library之實際Controller／Presentation完成C02隔離77三案：成功'+str(positive['actual_accepted_casts'])+'次、MP不足與正義值不足各1請求0成功，共'+str(record['actual_sessions'])+' LOGIN／GAME sessions，原始C14／S23／扣魔70／冷卻與有限特效音效事件已獨立讀回；'+('一般11746與自然暴擊11748均實際出現。' if both else '目前僅觀察'+str(positive['graphics_observed'])+'，另一自然暴擊／一般分支仍待。')+'兩次診斷觀察失敗與一次正義值352未辨識之產品失敗／red test均封存，前兩次只改診斷觀察；352產品缺口已於E159加辨識及錄包回歸，與當版重跑資料分列。此為無視窗診斷，非正常GUI／實體音效或Release實際施法資格。完整G06／同端點、其他技能、G01–G12與G11G12仍待；登入器及素材唯讀、產物獨立，原時限及預算不變。'
for n in names[:5]:
    p=ROOT/n;j=read(p);assert j['current_product_identity']['source_manifest_sha256']==record['source_manifest_sha256']
    j.update(current_native77_live_validation=record,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,goal_complete=False,whole_goal_complete=False,progress_updated_utc=datetime.now(timezone.utc).isoformat())
    j['current_native77_lawful_application'].update(actual_Debug_controller_diagnostic_qualification=record)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-Debug-C02-diagnostic-qualified-limited',work=work,goal_complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8')
for n in names[:5]:assert read(ROOT/n)['current_native77_live_validation']==record
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before=before,after={n:sha(ROOT/n) for n in names},before_archive_sha256=sha(out/'before.zip')))
print(json.dumps(dict(checkpoint=str(out),actual_sessions=record['actual_sessions'],actual_accepted_casts=record['actual_accepted_casts'],both_critical_variants_observed=both,whole_goal_complete=False),ensure_ascii=False))
