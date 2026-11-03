"""Read complete-product Debug test images, normal entry and preserved red."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(path,h,files):
    assert sha(path)==h
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(files) and set(z.namelist())==set(files)
        for n,digest in files.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
def amd64(p):
    with Path(p).open('rb') as f:
        head=f.read(64);assert head[:2]==b'MZ';f.seek(int.from_bytes(head[60:64],'little'));head=f.read(26)
        assert head[:4]==b'PE\0\0' and head[4:6]==b'\x64\x86' and head[24:26]==b'\x0b\x02'
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',required=True,type=int)
ap.add_argument('--terminal-exit-code',required=True,type=int);ap.add_argument('--tool-chunk-id',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
assert a.session_id>0 and a.terminal_exit_code==0 and a.tool_chunk_id
c=read(run/'cohort.json');profile=c['profile'];assert profile in ['Debug','Release']
assert c['status']==f'executed_product_visibility_{profile}_preflight_and_unique_normal'
assert c['owned_process'] is None and len(c['process_history'])==7 and all(p['exit_code']==0 for p in c['process_history'])
if profile=='Release':
    compiler=read(run/'release-test-compiler-observation.json');compiler=[compiler] if isinstance(compiler,dict) else compiler
    assert compiler
    for row in compiler:
        assert row['ParentProcessId']==c['process_history'][0]['pid'] and row['creation_utc']
        assert all(flag in row['CommandLine'] for flag in ['opt-level=z','lto=thin','codegen-units=16','x86_64-pc-windows-msvc'])
inputs=read(run/'compiled-inputs.json');pins=read(run/'source-inputs.json')
assert len(inputs)==1025 and len(pins)==1024 and sha(run/'source-inputs.json')==c['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h
source=c['compiled_source_archive'];archive(source['path'],source['sha256'],inputs)
assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_sha256']
expected={'LiToModern','native_skills','native_skill_selection','native_actor_states'}
assert set(c['products'])==set(c['tests'])==expected
messages=[json.loads(l) for l in (run/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
artifacts={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']}
assert set(artifacts)==expected
totals=dict(passed=0,failed=0,ignored=0);names={}
for name,product in c['products'].items():
    p=Path(product['path']).resolve(strict=True);assert p.is_relative_to(PROJECT/'target/current')
    assert sha(p)==product['sha256'] and Path(artifacts[name]['executable']).resolve()==p;amd64(p)
    assert artifacts[name]['features']==[] and artifacts[name]['profile']==product['compiler_profile']
    assert product['compiler_profile']['opt_level']==('0' if profile=='Debug' else 'z')
    assert product['compiler_profile']['debug_assertions']==(profile=='Debug')
    text=(run/('test-'+name)/'stdout.log').read_text(encoding='utf-8-sig')
    rows=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',text,re.M)
    assert rows and len(rows)==len({n for n,_ in rows})
    counts={k:sum(v==status for _,v in rows) for k,status in [('passed','ok'),('failed','FAILED'),('ignored','ignored')]}
    assert counts['failed']==0
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok',str(counts['passed']),'0',str(counts['ignored']),'0','0')]
    assert all(c['tests'][name][k]==v for k,v in counts.items())
    for k,v in counts.items():totals[k]+=v
    names[name]=dict(rows)
for name in ['detection_selector_menu_guards_and_cooldown_leave_authority_unchanged',
    'detection_original749_frames_sound_once_and_generation_retirement',
    'invisibility_tick_envelope_guards_clock_and_menu_preserve_authority']:
    assert names['native_skills'][name]=='ok'
assert totals==c['counts']
products=c['test_products_archive'];archive(products['path'],products['sha256'],{n+'.exe':v['sha256'] for n,v in c['products'].items()})
normal=c['normal'];assert normal['unique_entry_exit_code']==0 and sha(normal['path'])==normal['sha256'];amd64(normal['path'])
archive(normal['archive']['path'],normal['archive']['sha256'],{'LiToModern.exe':normal['sha256']})
assert not (run/'version/stderr.log').read_bytes().strip() and b'LiToModern' in (run/'version/stdout.log').read_bytes()
app=Path(c['application']);red=read(app/'red-independent-readback.json');application=read(app/'application.json')
assert red['test_exit_code']==101 and red['observed_wrapper_exit_code']==1 and red['original_exec_tool_chunk_id']=='885dc1'
archive(app/'red-raw.zip',red['raw_archive_sha256'],red['raw_files'])
assert sha(app/'red-test.exe')==red['test_product_sha256']
phrase='assert_eq!(skills::request(&w,13,None).unwrap(),[0x14,1,4]);'
assert phrase in (app/'red-native_skills.rs').read_text(encoding='utf-8-sig') and phrase in (PROJECT/'tests/native_skills.rs').read_text(encoding='utf-8-sig')
ids=','.join(str(p['pid']) for p in c['process_history'])
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+ids+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for job in c['process_history']:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'])-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/'qualified';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'CIM.json').write_bytes(p.stdout)
files={'qualifier.py':out/'qualifier.py','CIM.json':out/'CIM.json','cohort.json':run/'cohort.json',
    'red-independent-readback.json':app/'red-independent-readback.json','apply-receipt.json':app/'apply-receipt.json'}
for p in run.rglob('*'):
    if p.is_file() and out not in p.parents and p.suffix.lower() not in ['.exe','.zip']:files['run/'+p.relative_to(run).as_posix()]=p
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
archive(out/'raw.zip',sha(out/'raw.zip'),hashes)
q=dict(status=f'independently_qualified_product_visibility_{profile}_preflight_and_unique_normal',profile=profile,
    original_session_id=a.session_id,observed_terminal_exit_code=0,terminal_tool_chunk_id=a.tool_chunk_id,
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=c['source_manifest_sha256'],
    compiled_source_archive_sha256=source['sha256'],actual_test_executables=4,counts=totals,all_test_results=names,
    normal_sha256=normal['sha256'],normal_product_archive_sha256=normal['archive']['sha256'],
    full_product_effect_module_compiled=True,unrelated_TW_prepare_not_omitted=True,
    red_regression_preserved_and_now_passed=True,red_test_product_sha256=red['test_product_sha256'],
    recorded_owned_identities=7,recorded_owned_processes_all_terminal=True,
    actual_formal_sessions=0,actual_casts=0,Release_verified=profile=='Release',live_invisibility_reveal_verified=False,GUI_verified=False,
    raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({k:q[k] for k in ['status','counts','normal_sha256','actual_casts']},ensure_ascii=False))
