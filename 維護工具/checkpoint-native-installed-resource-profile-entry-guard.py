"""Birth-bound active/terminal checkpoint for the new opt-in API fixture."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, subprocess, time, zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
def read(path):
    for attempt in range(10):
        try:return json.loads(Path(path).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if attempt==9:raise
            time.sleep(.05)
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
encode=lambda value:(json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path);parser.add_argument('--session',type=int,required=True)
parser.add_argument('--mode',choices=['active','terminal'],required=True);parser.add_argument('--terminal-chunk')
parser.add_argument('--terminal-exit-code',type=int);parser.add_argument('--qualification',type=Path);a=parser.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-entry-guard')and a.session>0
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='603C835908762CAF643F3ECA995039745DD6BE428FE9F533261B226BA2AD744A'
for name,digest in read(run/'compiled-inputs.json').items():assert sha(R/name)==digest,name
for name,digest in c['validation_pins'].items():assert sha(R/'維護工具'/name)==digest,name
assert w['active_workload_sessions']in [{},{'rental_entry_guard':a.session}]
identities=[c['runner'],*c['process_history']];filters=' OR '.join('ProcessId='+str(row['pid'])for row in identities)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert probe.returncode==0 and not probe.stderr
live=json.loads(probe.stdout.decode('utf-8-sig'))if probe.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
utc=lambda text:datetime.fromisoformat(text.replace('Z','+00:00'))
is_live=lambda row:any(value['ProcessId']==row['pid']and abs((utc(value['created'])-utc(row['creation_utc'])).total_seconds())<.001 for value in live)
terminal=a.mode=='terminal';qualified=a.qualification is not None
if terminal:
    assert a.terminal_exit_code in [0,1]and a.terminal_chunk not in [None,'','pending','unknown']
    assert not any(is_live(row)for row in identities)and c['owned_processes']==[]
    assert c['status']in ['executed_both_current_rental_entry_guard_profiles_pending_independent_readback','failed_preserved']
else:
    assert a.terminal_exit_code is None and a.terminal_chunk is None and not qualified
    assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
    assert is_live(c['runner'])and c['status']in ['prepared_current_rental_entry_guard','compiling_current_rental_entry_guard','executing_current_rental_entry_guard']
if qualified:
    assert terminal and a.terminal_exit_code==0
    qp=a.qualification.resolve(strict=True);assert qp==(run/'qualified/qualified.json').resolve()
    q=read(qp);assert q['status']=='independently_qualified_both_current_rental_entry_guard_profiles_limited'
    assert q['source_manifest_sha256']==source and q['cohort_sha256']==sha(run/'cohort.json')
    assert q['original_session']==a.session and q['terminal_tool_chunk']==a.terminal_chunk and q['observed_terminal_exit_code']==0
    assert q['qualified']and q['actual_sessions']==8 and q['recorded_owned_processes_terminal']and q['recorded_owned_identities']==8
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/name)==digest for name,digest in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'original-owned-CIM.json').write_bytes(probe.stdout)
record=dict(status='current_rental_entry_guard_qualified_limited'if qualified else'current_rental_entry_guard_terminal_readback_pending'if terminal else'current_rental_entry_guard_running',
    run=str(run),source_manifest_sha256=source,original_session=a.session,runner=c['runner'],
    terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=a.terminal_exit_code,
    original_owned_processes_terminal=terminal,recorded_owned_identities=len(identities),qualified=qualified,
    qualification=str(qp)if qualified else None,qualification_sha256=sha(qp)if qualified else None,
    actual_sessions=q['actual_sessions']if qualified else c['actual_sessions'],profiles=q['profiles']if qualified else c['profiles'],
    hotel_return_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(encode(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name in before:archive.write(R/name,name)
with zipfile.ZipFile(out/'before.zip')as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==digest for name,digest in before.items())
work=('當版租裝入口API兩版各1案、8sessions已獨立核對：前後拒絕不送普通CA00，正常配裝／入村及C10返回選角；完整S7C物品與原協定封包另核對。旅館房內返回、正常GUI、傳送恢復、廣域與完整G01–G12仍待。'if qualified else
    '當版租裝入口API原批次已actual exit'+str(a.terminal_exit_code)+'且記錄程序均終止；獨立讀回或失敗診斷仍待，不移記成功。'if terminal else
    '當版租裝入口API原'+str(a.session)+'編譯／隔離實際封包驗證中，來源與對應QA固定；原session續行，不並行其他計時服務。')
work+='不改JAR／遊戲時限，不要求使用者操作；完整自動測試後才交人工。'
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d');staged={}
for name in before:
    path=R/name
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        if qualified:
            value['current_native_rental_entry_guard_qualification']=record
            value['current_source_goal_validation']['G06-room-service-branches'].update(complete=False,
                state='rental_entry_guard_and_formal_return_limited_GUI_pending',current_native_rental_entry_guard=record)
        value.update(active_workloads={}if terminal else{'rental_entry_guard':record},
            active_workload_sessions={}if terminal else{'rental_entry_guard':a.session},
            source_and_corresponding_QA_frozen=not terminal,source_freeze_while_Cargo=not terminal,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),
            goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='rental_entry_guard_actual_wire',work=work,applies_to_current_source=True,goal_complete=False)
        if name.endswith('next-validation-action.json'):value['next_action']=work
        if 'updated_date'in value:value['updated_date']=date
        staged[name]=encode(value)
    else:
        first,separator,rest=path.read_text('utf-8-sig').partition('\n\n');assert separator
        staged[name]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/name)==digest for name,digest in before.items())
for name,payload in staged.items():(R/name).write_bytes(payload)
(out/'readback.json').write_bytes(encode(dict(before_files=before,after_files={name:sha(R/name)for name in before},
    before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
(run.parent/'current-continuation.json').write_bytes(encode(dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),
    must_poll_original_session_before_restarting=not terminal)))
print(json.dumps(dict(checkpoint=str(out),qualified=qualified,original_session=a.session),ensure_ascii=False))
