"""Preserve the pre-LOGIN stale version guard failure, including missing Java exit code."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];run=ROOT/'驗證紀錄/current/native44FE-teleport-audio/run-20261018-163721-922344'
def read(p):return json.loads(p.read_text('utf-8-sig'))
def sha(p):
    with p.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
r=read(run/'receipt.json');case=Path(r['case']);cr=read(case/'receipt.json');build=Path(r['compile'])
assert r['status']=='failed_preserved'and r['process_history'][0]['exit_code']==1 and cr['test_exit_code']==101 and cr.get('server_exit_code')is None
text=(case/'test.stderr.txt').read_text('utf-8-sig');assert '75DA69B0'in text and 'C02AAAE8'in text and 'assertion `left == right` failed'in text and(case/'server-trace.jsonl').stat().st_size==0
runner=read(run/'runner-identity.json');identities=[(runner['pid'],runner['creation_utc'])]+[(p['pid'],p['creation_utc'])for p in r['process_history']]+[(p['pid'],p['start_utc'])for p in cr['owned_processes']]
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+','.join(str(pid)for pid,_ in identities)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress";c=subprocess.run(['pwsh','-NoProfile','-Command',command],capture_output=True);assert c.returncode==0;live=json.loads(c.stdout.decode('utf-8-sig'))if c.stdout.strip()else[]
if isinstance(live,dict):live=[live]
for row in live:
    for pid,birth in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(birth)).total_seconds())>.001
out=run/'failed-preflight';out.mkdir();files={}
for folder,prefix in[(run,'run/'),(case,'case/')]:
    for p in folder.iterdir():
        if p.is_file():files[prefix+p.name]=p
for n in['success.rs','failure.rs','diagnostic_wave_api.rs','diagnostic-inputs.json','receipt.json','compile-terminal-observation.json']:files['compile/'+n]=build/n
files['preparation.before-repair.json']=ROOT/'維護工具/candidates/native44FE-teleport-audio/preparation.json';files['actual-test.exe']=Path(cr['test_binary']);pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
record=dict(status='preserved_stale_probe_JAR_guard_failed_before_LOGIN',original_session_id=3752,original_tool_exit_code=1,terminal_tool_chunk='f863cf',child_PowerShell_exit_code=1,Rust_exit_code=101,server_exit_code_recorded=cr.get('server_exit_code'),owned_processes_all_terminal=True,owned_identities=len(set(identities)),no_client_LOGIN_started=True,wire_trace_bytes=0,resource_first_frame_budget_executed=False,source_manifest_sha256=r['source_manifest_sha256'],expected_current_reference=cr['jar_sha256'],stale_probe_guard='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539',raw_archive_sha256=sha(out/'raw.zip'),raw_files=pins,normal_source_modified=False,gameplay_qualification_granted=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'failure.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');(out/'process-CIM.json').write_bytes(c.stdout);(out/'reader.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(failure=str(out/'failure.json'),owned_terminal=len(set(identities)),actual_LOGIN_started=False),ensure_ascii=False))
