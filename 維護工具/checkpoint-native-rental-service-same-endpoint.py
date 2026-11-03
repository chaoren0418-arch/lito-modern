"""Track exact compile/runtime jobs for the separate advertised rental route."""
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
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path)
parser.add_argument('--kind',choices=['compile','runtime'],required=True);parser.add_argument('--mode',choices=['active','terminal'],required=True)
parser.add_argument('--session',type=int);parser.add_argument('--terminal-chunk');parser.add_argument('--terminal-exit-code',type=int)
parser.add_argument('--qualification',type=Path);a=parser.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-rental-service-same-endpoint')
c=read(run/'receipt.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
assert a.session is None or a.session>0
assert w['active_workload_sessions']in [{},{'same_endpoint_rental':a.session}]
build=run if a.kind=='compile'else Path(c['compile'])
inputs=read(build/'compiled-inputs.json');assert len(inputs)==1049
for name,digest in inputs.items():assert sha(R/name)==digest,name
identities=[c['runner'],*c['process_history']]
if a.kind=='runtime':identities+=read(run/'compile-terminal-observation.json')['identities']
ids=sorted({row['pid']for row in identities});filters=' OR '.join('ProcessId='+str(pid)for pid in ids)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert probe.returncode==0 and not probe.stderr
live=json.loads(probe.stdout.decode('utf-8-sig'))if probe.stdout.strip()else[];live=[live]if isinstance(live,dict)else live
utc=lambda text:datetime.fromisoformat(text.replace('Z','+00:00'))
is_live=lambda row:any(value['ProcessId']==row['pid']and abs((utc(value['created'])-utc(row['creation_utc'])).total_seconds())<.001 for value in live)
terminal=a.mode=='terminal';qualified=a.qualification is not None
if terminal:
    assert a.terminal_exit_code in [0,1]and a.terminal_chunk not in [None,'','pending','unknown']
    assert not any(is_live(row)for row in identities)
    assert c.get('owned_process')is None and not c.get('owned_processes',[])
    if a.terminal_exit_code==0:
        expected='compiled_normal_public_API_same_endpoint_rental_probe'if a.kind=='compile'else'executed_same_endpoint_advertised_rental_pending_independent_readback'
        assert c['status']==expected
    else:assert c['status']=='failed_preserved'
else:
    assert a.session is not None and not qualified and a.terminal_exit_code is None and a.terminal_chunk is None
    assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']and is_live(c['runner'])
if qualified:
    assert terminal and a.kind=='runtime'and a.terminal_exit_code==0
    qp=a.qualification.resolve(strict=True);assert qp==(run/'qualified/qualified.json').resolve()
    q=read(qp);assert q['status']=='normal_public_API_same_endpoint_advertised_rental_scope_qualified'
    assert q['source_manifest_sha256']==source and q['profile']==c['profile']and q['actual_sessions']==6
    assert q['original_session']==a.session and q['terminal_tool_chunk']==a.terminal_chunk and q['observed_terminal_exit_code']==0
    assert q['recorded_original_owned_processes_terminal']and not q['normal_channel_continuation_verified']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']and sha(qp.parent/'product.zip')==q['product_archive_sha256']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/name)==digest for name,digest in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'original-owned-CIM.json').write_bytes(probe.stdout)
record=dict(status='same_endpoint_advertised_rental_qualified_limited'if qualified else'same_endpoint_advertised_rental_terminal_pending'if terminal else'same_endpoint_advertised_rental_running',
    run=str(run),kind=a.kind,profile=c['profile'],source_manifest_sha256=source,original_session=a.session,
    returned_without_persistent_session=a.session is None,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=a.terminal_exit_code,
    runner=c['runner'],recorded_original_owned_processes_terminal=terminal,recorded_original_identities=len(identities),
    qualified=qualified,qualification=str(qp)if qualified else None,qualification_sha256=sha(qp)if qualified else None,
    actual_sessions=6 if qualified else c.get('actual_sessions',0),normal_channel_continuation_verified=False,
    old_CA00_failure_preserved=True,nondefault_stat_continuity_verified=False,GUI_verified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(encode(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name in before:archive.write(R/name,name)
with zipfile.ZipFile(out/'before.zip')as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==digest for name,digest in before.items())
work='當版'+c['profile']+'同端點租裝服務'+('接續已獨立核對：6sessions、初次交接全物品欄位、新世代／舊frame拒絕、有效服務再入與權威移動。'if qualified else'原'+str(a.session)+'已actual exit'+str(a.terminal_exit_code)+'，獨立讀回或失敗診斷仍待。'if terminal else'原'+str(a.session)+'沿原程序編譯／執行，來源固定，不重啟。')
work+='這不是普通頻道或正常GUI資格；舊CA00失敗保留，非預設素質／其他服務與完整GUI仍待。自行完整測試後才交人工，不改JAR／時限，不要求使用者操作。'
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d');staged={}
for name in before:
    path=R/name
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        if qualified:
            value.setdefault('current_native_same_endpoint_rental_qualification',{})[c['profile']]=record
            value['current_source_goal_validation']['G06-room-service-branches'].update(complete=False,
                state='advertised_rental_same_endpoint_limited_other_services_GUI_pending',current_same_endpoint_rental=record)
        value.update(active_workloads={}if terminal else{'same_endpoint_rental':record},
            active_workload_sessions={}if terminal else{'same_endpoint_rental':a.session},source_and_corresponding_QA_frozen=not terminal,
            source_freeze_while_Cargo=not terminal,next_task=work,next_focus=work,last_goal_turn_classification='progress',
            last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='same_endpoint_advertised_rental',work=work,applies_to_current_source=True,goal_complete=False)
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
(run.parent/'current-continuation.json').write_bytes(encode(dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_session_before_restarting=not terminal)))
print(json.dumps(dict(checkpoint=str(out),qualified=qualified,kind=a.kind,original_session=a.session),ensure_ascii=False))
