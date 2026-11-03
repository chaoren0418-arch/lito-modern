"""Publish only independently qualified current request-related scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--terminal-chunk');a=ap.parse_args()
run=ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865';c=read(run/'cohort.json');inputs=read(run/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
profiles={}
for p in ['Debug','Release']:
    paths=sorted(run.glob(p+'-qualified-*/qualified.json'))
    if not paths:continue
    qp=paths[-1];q=read(qp);assert q['status']=='independently_qualified_native77_request_normal_related_profile' and q['source_manifest_sha256']==c['source_manifest_sha256'] and q['passed']==31 and q['recorded_owned_all_terminal']
    assert sha(q['normal']['path'])==q['normal']['sha256']
    profiles[p]=dict(qualification=str(qp),qualification_sha256=sha(qp),normal_sha256=q['normal']['sha256'],passed=31,failed=0,ignored=0)
assert 'Debug' in profiles
complete=bool(a.terminal_chunk)
if complete:
    assert set(profiles)=={'Debug','Release'} and c['status']=='executed_native77_request_normals_related' and c['owned_process'] is None
    assert len(c['process_history'])==24 and all(v['exit_code']==0 for v in c['process_history'])
    driver=read(run/'original-runner-process.json');identities=c['process_history']+[driver]+[dict(pid=x['ProcessId'],creation_utc=x['created']) for x in c['observed_compilers']]
    ids=','.join(str(v['pid']) for v in identities)
    ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);rows=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
    if isinstance(rows,dict):rows=[rows]
    for row in rows:
        for j in identities:
            if row['ProcessId']==j['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())>.001
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:assert read(ROOT/n)['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
out=run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='qualified_both_normals_and_direct_request_targets_only' if complete else 'qualified_Debug_normal_and_direct_request_targets_Release_running',source_manifest_sha256=c['source_manifest_sha256'],run=str(run),original_session_id=94297,profiles=profiles,scope='normal builds/version0 and 3 directly affected request/selection/attack targets only',new_sessions=0,new_casts=0,full_library_verified=False,full_G06_verified=False,GUI_verified=False,whole_goal_complete=False)
if complete:record.update(original_exit_code=0,original_terminal_tool_chunk=a.terminal_chunk,recorded_owned_identities=len({(j['pid'],j['creation_utc']) for j in identities}),recorded_owned_all_terminal=True)
work=('來源E15907AC正常Debug／Release沿唯一入口建置與version0，三個直接相關測試目標各31過0敗0忽略，原24子工作0及記錄程序皆已獨立讀回終止。僅正常與77請求相關範圍取得資格；下一步C02隔離雙玩家正義值352拒絕、實際C14／ack／扣魔／冷卻及一般與暴擊補驗。' if complete else '來源E15907AC正常Debug建置／version0與三個直接相關目標31過0敗0忽略已獨立核對；Release沿原94297繼續，來源／QA仍凍結，未移記舊版結果。')+'C831效果／素材依未變逐檔來源參考，未轉移整版資格。實際施法、其他技能、完整G06／同端點、G01–G12、GUI實體輸入音效／G11G12仍待。登入器及素材唯讀且產物獨立，原時限／預算不變。'
for n in names[:5]:
    path=ROOT/n;j=read(path);identity=j['current_product_identity'];identity.update(source_qualification_pending=not complete,qualification_scope='current_native77_normal_and_direct_request_targets_only' if complete else 'Debug_native77_normal_and_direct_request_targets_only_Release_pending')
    for p,v in profiles.items():identity.update({p+'_normal_sha256':v['normal_sha256'],p+'_normal_source_manifest_sha256':c['source_manifest_sha256'],p+'_normal_qualified_for_current_source':True,p+'_normal_state':'independently_qualified_normal_and_direct_request_targets_only'})
    j.update(current_native77_lawful_normal=record,source_and_corresponding_QA_frozen=not complete,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,goal_complete=False,whole_goal_complete=False,progress_updated_utc=datetime.now(timezone.utc).isoformat())
    j['current_native77_lawful_application'].update(normal_validation_pending=not complete,normal_related_qualification=record)
    if complete:j.update(active_workloads={},active_workload_sessions={})
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-authority-live-integration',work=work,goal_complete=False)
    save(path,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8')
for n in names[:5]:assert read(ROOT/n)['current_native77_lawful_normal']==record
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(scope=record,before=before,after={n:sha(ROOT/n) for n in names},before_archive_sha256=sha(out/'before.zip')))
if complete:save(Path(c['application'])/'normal-related-qualification.json',record)
print(json.dumps(dict(checkpoint=str(out),profiles=list(profiles),complete_for_this_scope=complete,whole_goal_complete=False),ensure_ascii=False))
