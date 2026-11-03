"""Bind current normal visibility scopes to original runtime identities."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser()
ap.add_argument('run',type=Path)
ap.add_argument('--session-id',required=True,type=int)
ap.add_argument('--dispatch-chunk-id',required=True)
ap.add_argument('--runner-pid',type=int)
ap.add_argument('--runner-created')
ap.add_argument('--pair',type=Path)
a=ap.parse_args()
run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
c=read(run/'cohort.json');profile=c['build_profile'];assert profile in {'Debug','Release'}
inputs=Path(c['preflight']);pins=read(inputs/'compiled-inputs.json')
assert len(pins)==1029 and all(sha(ROOT/n)==h for n,h in pins.items())
assert c['source_manifest_sha256']=='89D4B630285B2ED137EFA81CBDB429CF31FE2928C68FD2D4C9EE3180D3D83CC1'
active=c['status']==f'running_normal_{profile}_visibility_cases'
binding=run/'original-session-binding.json'
if binding.exists():
    b=read(binding);assert b['original_session_id']==a.session_id and b['dispatch_tool_chunk_id']==a.dispatch_chunk_id
else:
    assert active and a.runner_pid and a.runner_created
    ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(a.runner_pid)+"';if($p){[pscustomobject]@{pid=$p.ProcessId;created=$p.CreationDate.ToUniversalTime().ToString('o');command=$p.CommandLine}|ConvertTo-Json -Compress}"
    raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps],encoding='utf-8-sig').strip();assert raw
    identity=json.loads(raw)
    assert identity['created']==a.runner_created and 'run-native-selector-visibility-cases.py' in identity['command']
    assert str(inputs) in identity['command']
    b=dict(original_session_id=a.session_id,dispatch_tool_chunk_id=a.dispatch_chunk_id,runner_identity=identity,relaunched=False)
    save(binding,b)
qualified=run/'qualified/qualified.json';done=qualified.exists()
record=dict(run=str(run),profile=profile,original_session_id=a.session_id,status=c['status'],active=active,
    original_binding=str(binding),original_binding_sha256=sha(binding),source_manifest_sha256=c['source_manifest_sha256'],
    normal_sha256=c['normal_sha256'],independently_compared_cases=sum(v.get('status')=='executed_and_independently_compared' for v in c['cases']),
    qualified=done,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
if done:
    q=read(qualified)
    assert q['original_session_id']==a.session_id and q['observed_terminal_exit_code']==0 and q['source_manifest_sha256']==c['source_manifest_sha256']
    assert q['actual_client_sessions']==32 and q['actual_cast_requests']==14 and q['recorded_owned_all_terminal']
    record.update(qualification=str(qualified),qualification_sha256=sha(qualified),actual_client_sessions=32,accepted_casts=8,negative_cases=6)
pair=None
if a.pair:
    qp=a.pair.resolve(strict=True);assert qp.is_relative_to(run.parent.parent)
    q=read(qp)
    assert q['status']=='independently_qualified_current_normal_visibility_both_profiles'
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['actual_client_sessions']==64 and q['actual_cast_requests']==28
    assert q['accepted_casts']==16 and q['negative_cases']==12 and q['recorded_runtime_owned_all_terminal'] and q['recorded_runtime_owned_identities']==64
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    pair=dict(qualification=str(qp),qualification_sha256=sha(qp),actual_client_sessions=64,actual_cast_requests=28,accepted_casts=16,negative_cases=12,
        recorded_runtime_owned_identities=64,recorded_runtime_owned_all_terminal=True,qualified=True,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json',
    'README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:
    j=read(ROOT/n);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
    for key,value in j.get('active_workloads',{}).items():assert key=='selector-visibility' and value['original_session_id']==a.session_id
out=run/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==v for n,v in before.items())
scope='兩版八案已獨立核對64sessions、28請求／16接受／12負向案例與64組記錄程序退出' if pair else (profile+'原'+str(a.session_id)+' '+c['status']+'；目前'+str(record['independently_compared_cases'])+'案已獨立比較')
work=('來源89D4B630廣域504／500、完整library各1937過0敗473原忽略、map563兩版120sessions與十項原資源二十次均保留各自限定資格。'
    '同版正常可見性'+scope+'。來源及對應QA在本批完成前凍結；原時限／預算不變。'
    '完整G06、同端點、其餘技能／服務、GUI／實體輸入音效與可攜G11G12仍待；完整G01–G12未完成。登入器唯讀、產物獨立且替換提案撤回。')
if pair:
    work+='下一步依獨立CSharp63影格／32精確空白標記證據，建立544空白解碼隔離候選及31非空影格像素比較，作為34／38／45／46／77原生技能資源接入準備；尚無新增技能施放資格。'
for n in names[:5]:
    p=ROOT/n;j=read(p);v=j.get('current_selector_visibility',dict(profiles={}))
    v['profiles'][profile]=record
    if pair:v.update(pair=pair,qualified=True)
    else:v['qualified']=False
    j.update(current_selector_visibility=v,active_workloads={'selector-visibility':record} if active else {},
        active_workload_sessions={'selector-visibility':a.session_id} if active else {},source_and_corresponding_QA_frozen=not bool(pair),source_freeze_while_Cargo=False,
        next_task=work,next_focus=work,last_goal_turn_classification='progress' if done else 'verified_wait',last_goal_turn_evidence=pair or record,
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='source89D4-native-attack-resource-candidate' if pair else 'source89D4-current-normal-visibility',work=work,applies_to_current_source=True,goal_complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(scope=record,pair=pair,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),active=active,qualified=done,pair_qualified=bool(pair)),ensure_ascii=False))
