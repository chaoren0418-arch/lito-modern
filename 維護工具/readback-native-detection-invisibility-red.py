"""Read the retained semantic red failure before applying runtime changes."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(path,h,files):
    assert sha(path)==h
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(files) and set(z.namelist())==set(files)
        for n,digest in files.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
ap=argparse.ArgumentParser();ap.add_argument('application',type=Path);ap.add_argument('--wrapper-exit-code',required=True,type=int);ap.add_argument('--exec-chunk-id',required=True);a=ap.parse_args()
app=a.application.resolve(strict=True);assert app.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
assert a.wrapper_exit_code==1 and a.exec_chunk_id=='885dc1'
r=read(app/'red-result.json');ar=read(app/'red-archive-readback.json');application=read(app/'application.json')
assert r['status']=='expected_detection_request_missing_reproduced' and r['test_exit_code']==101
archive(app/'red-raw.zip',ar['archive_sha256'],ar['files'])
archive(application['before']['path'],application['before']['sha256'],application['before']['files'])
inputs=read(app/'red-compiled-inputs.json');base=read(app/'base-compiled-inputs.json')
assert len(inputs)==len(base)==1025 and {n for n in inputs if inputs[n]!=base[n]}=={'LiTo381-Modern/tests/native_skills.rs'}
for n,h in inputs.items():assert sha(ROOT/n)==h
assert sha(app/'red-native_skills.rs')==inputs['LiTo381-Modern/tests/native_skills.rs']==r['source_sha256']
assert sha(app/'red-test.exe')==r['test_product_sha256']
data=(app/'red-test.exe').read_bytes();p=int.from_bytes(data[60:64],'little')
assert data[:2]==b'MZ' and data[p+4:p+6]==b'\x64\x86' and data[p+24:p+26]==b'\x0b\x02'
text=(app/'red-stdout.log').read_text(encoding='utf-8-sig')
messages=[json.loads(l) for l in text.splitlines() if l.startswith('{')]
assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
assert any(m.get('reason')=='compiler-artifact' and m.get('executable') and m['target']['name']=='native_skills' and m['profile']['test'] and not m['features'] for m in messages)
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('FAILED','0','1','0','0',re.search(r'(\d+) filtered out;',text).group(1))]
assert application['red_expected_reason'] in text and application['red_test_name'] in text
process=read(app/'red-process.json')
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+str(process['pid'])+"';if($p){[pscustomobject]@{pid=$p.ProcessId;created=$p.CreationDate.ToUniversalTime().ToString('o')}|ConvertTo-Json -Compress}"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode==0 and not p.stderr
if p.stdout.strip():
    live=json.loads(p.stdout.decode('utf-8-sig'));assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(process['creation_utc'])).total_seconds())>.001
normal_unchanged=all(r['normal_mains_after'][profile]==info['sha256'] for profile,info in application['normal_mains_before'].items())
q=dict(status='independently_retained_detection_missing_request_semantic_red',test_exit_code=101,observed_wrapper_exit_code=1,
    original_session_id=None,original_exec_tool_chunk_id=a.exec_chunk_id,source_inputs=1025,only_integration_test_changed=True,
    runtime_sources_unchanged=True,normal_main_bytes_unchanged=normal_unchanged,test_product_sha256=r['test_product_sha256'],
    source_sha256=r['source_sha256'],original_recorded_Cargo_identity_terminal=True,
    raw_archive_sha256=ar['archive_sha256'],raw_files=ar['files'],actual_formal_sessions=0,whole_goal_complete=False)
out=app/'red-independent-readback.json';assert not out.exists()
out.write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(app/'red-independent-reader.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:q[k] for k in ['status','test_exit_code','observed_wrapper_exit_code','normal_main_bytes_unchanged']},ensure_ascii=False))
