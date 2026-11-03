"""Reconcile preserved failed diagnostics; never infer audio qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import ctypes,hashlib,json,zipfile,subprocess,re
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE
k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
k.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
def terminal(j):
    h=k.OpenProcess(0x100000|0x1000,False,j['pid'])
    if not h:
        error=ctypes.get_last_error()
        if error==87:return dict(**j,original_identity_terminal=True,observation='PID absent')
        assert error==5,(j,error)
        command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter 'ProcessId="+str(j['pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
        output=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(output.decode('utf-8-sig')) if output.strip() else []
        live=[live] if isinstance(live,dict) else live
        assert not live or abs((datetime.fromisoformat(live[0]['created'].replace('Z','+00:00'))-datetime.fromisoformat(j.get('creation_utc',j.get('start_utc')).replace('Z','+00:00'))).total_seconds())>.001
        return dict(**j,original_identity_terminal=True,observation='CIM identity absent or reused after AccessDenied')
    try:
        ts=[wintypes.FILETIME() for _ in range(4)];assert k.GetProcessTimes(h,*[ctypes.byref(t) for t in ts])
        ticks=ts[0].dwLowDateTime|(ts[0].dwHighDateTime<<32)
        born=datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)
        expected=datetime.fromisoformat(j.get('creation_utc',j.get('start_utc')).replace('Z','+00:00'))
        reused=abs((born-expected).total_seconds())>.001
        assert reused or k.WaitForSingleObject(h,0)==0,j
        return dict(**j,original_identity_terminal=True,observation='PID reused' if reused else 'original process signalled')
    finally:k.CloseHandle(h)
run=ROOT/'驗證紀錄/current/native1040-teleport-legacy-audio/run-20261018-100824-998699'
c=read(run/'receipt.json');assert c['status']=='failed_preserved' and c['owned_process'] is None
case=Path(c['case']);cr=read(case/'receipt.json');runtime=Path(cr['runtime'])
assert cr['test_exit_code']==101 and cr['server_exit_code']==1 and cr['status']=='failed'
assert sha(c['product'])==c['product_sha256']==cr['test_binary_sha256']
for n,h in c['pins'].items():assert sha(Path(n))==h
build=Path(c['compile']);build_receipt=read(build/'receipt.json');compilers=build_receipt['process_history']+[build_receipt['original_runner']]
jobs=[read(run/'runner-identity.json')]+c['process_history']+cr['owned_processes']+compilers
identities=[terminal(j) for j in jobs]
for role in ['a','b']:
    state=read(runtime/(role+'-presentation-shutdown.json'))
    assert state['client_quiescent'] and state['Presentation_quiescent'] and state['shutdown_error'] is None
capture=next((runtime/'captures').glob('*/frames.jsonl'))
rows=[json.loads(line)for line in capture.read_text('utf-8-sig').splitlines()]
wire=[(number,row,bytes.fromhex(row['plain']))for number,row in enumerate(rows,1)if isinstance(row.get('plain'),str) and re.fullmatch('[a-fA-F0-9 ]*',row['plain'])]
uses=[n for n,row,data in wire if row['direction']=='client_to_server'and data[:1]==b'\x1f']
actions=[n for n,row,data in wire if row['direction']=='server_to_client'and data[:1]==b'\x20'and data[-1:]==b'\x44']
assert uses==[279] and actions==[280,281]
a=read(runtime/'a-presentation-journal.json');b=read(runtime/'b-presentation-journal.json')
assert a['last_snapshot']['phase']=='character_selected'and a['last_snapshot']['world']['status']is None
assert b['error']=='inspected wave graphics only'and 1534 in b['played']
bown=str(b['last_snapshot']['world']['status']['object_id']);assert b['last_snapshot']['world']['actors'][bown]['graphics']==61
transfers=[n for n,row,data in wire if row['direction']=='server_to_client'and data[:1]==b'\xf4'];assert transfers
for n,h in read(build/'compiled-inputs.json').items():assert sha(ROOT/n)==h
out=run/('failure-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={}
for family,folder in [('driver',run),('case',case),('runtime',runtime),('compile',build)]:
    for p in folder.rglob('*'):
        if p.is_file() and not p.is_relative_to(out) and p.suffix.lower() not in ['.exe','.zip']:
            files[family+'/'+p.relative_to(folder).as_posix()]=p
pins={name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'failed-runtime.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for name,p in files.items():archive.write(p,name)
with zipfile.ZipFile(out/'failed-runtime.zip')as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
record=dict(status='failed_preserved_B_graphics61_guard_after_A_endpoint_handoff',run=str(run),profile='Release',
    source_manifest_sha256=c['source_manifest_sha256'],candidate_effective_manifest_sha256=c['candidate_effective_manifest_sha256'],normal_source_bodies_unchanged=False,candidate_source_overlays_used=True,original_session_id=38756,original_exit_code=1,
    terminal_tool_chunk='cd125b',Rust_exit_code=101,server_exit_code=cr.get('server_exit_code'),server_cleanup_error=cr['error'],server_original_identity_terminal_now=True,product_sha256=c['product_sha256'],
    C1F_capture_lines=uses,S20_wave_capture_lines=actions,source_frames_published=True,A_new_endpoint_character_selected=True,B_remote_wave1534_delivered=True,B_actual_graphics=61,F4_capture_lines=transfers,C72_audio_qualified=False,
    cause='A sent actual C1F/S20 wave68 and reached a new endpoint character selection. B received and delivered the remote1534 cue, but its own graphics61 was rejected by a probe guard that inspected only A graphics0. The complete case and source-cue retirement remain unqualified.',
    recorded_identities=identities,recorded_owned_all_terminal=True,source_and_compiled_inputs_unchanged=True,
    raw_archive_sha256=sha(out/'failed-runtime.zip'),raw_files=pins,
    authentic_audio_handoff_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'failure-preserved.json',record)
active=list(run.glob('running-checkpoint-*/readback.json'));assert len(active)==1
before=read(active[0])['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as archive:
    for n in before:archive.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as archive:
    assert all(hashlib.sha256(archive.read(n)).hexdigest().upper()==h for n,h in before.items())
work='正常基準4E1F7A4E之各90相關、G06控制器／非同步Presentation、廣域Debug519／Release515與完整library各1963過維持限定資格，正常來源未改。一般音效生命週期候選Release原38756退出1（Rust101）：A真實C1F／S20揮手後完成端點切換並到角色選擇，B已接收播放遠端1534；但B實際角色gfx61遭僅允許gfx0的探針限制，整案及來源音效退役仍未通過。失敗與程序終止已封存；下一步先依實收B角色核對正常list.spr之wave68／WAV，再擴大診斷已知角色域並重驗，不猜音效或改時限。候選四檔尚未套正常版；Debug首畫面就緒另待定位。GUI／實體音效、其他服務及完整G06／G01–G12仍未完成，相鄰只讀。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        value['current_native1040_legacy_wave_graphics_failed_attempt']=record
        value['current_native1040_G06_audio_candidate_active'].update(status='failed_preserved_terminal',qualification_pending=False,qualification_granted=False,original_exit_code=1,terminal_tool_chunk='cd125b',failure=str(out/'failure-preserved.json'))
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'failure-preserved.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='G06-legacy-audio-wave-two-role-domain',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,separator,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and separator
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),failure_sha256=sha(out/'failure-preserved.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(status='failed_candidate_Release_B_graphics61_guard_A_handoff_observed',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],candidate_effective_manifest_sha256=c['candidate_effective_manifest_sha256'],normal_source_bodies_unchanged=False,candidate_source_overlays_used=True,original_session_id=38756,observed_terminal_exit_code=1,terminal_tool_chunk='cd125b',failure=str(out/'failure-preserved.json'),failure_sha256=sha(out/'failure-preserved.json'),checkpoint=str(out),recorded_owned_all_terminal=True,qualification_granted=False,must_poll_original_session_before_restarting=False,Release_qualified=False,Debug_qualified=False,next_action=work,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),terminal_identities=len(identities),source_audio_qualified=False),ensure_ascii=False))
