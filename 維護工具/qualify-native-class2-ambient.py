"""Read raw names, results, pinned inputs and PE; qualify only isolated ledger checks."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--run',type=Path,required=True);ap.add_argument('--session',required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
assert a.terminal_chunk not in ('pending','', 'unknown')
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-class2-ambient')
r=read(run/'receipt.json');assert r['status']=='executed_candidate_component_checks'
compiled=Path(r['compile']);assert sha(compiled/'receipt.json')==r['compile_receipt_sha256'];c=read(compiled/'receipt.json')
assert c['compiler']['exit_code']==0 and c['candidate_only']
assert all(p['exit_code']==0 for p in r['process_history']) and len(r['process_history'])==2
for name,digest in r['raw_hashes'].items():assert sha(run/name)==digest
for name,digest in read(compiled/'compiled-inputs.json').items():assert sha(ROOT/name)==digest,name
for library in c['libraries'].values():assert sha(library['path'])==library['sha256']
assert sha(compiled/'probe.rs')==c['compiled_probe_sha256']
assert sha(compiled/'native_ambient_override.rs')==c['candidate_module_sha256']==r['candidate_module_sha256']
expected=set(re.findall(r'#\[test\]\s*fn\s+(\w+)',(compiled/'probe.rs').read_text(encoding='utf-8')))
listed=set(re.findall(r'^(\w+): test$',(run/'list.stdout.log').read_text(encoding='utf-8'),re.M))
executed=set(re.findall(r'^test (\w+) \.\.\. ok$',(run/'checks.stdout.log').read_text(encoding='utf-8'),re.M))
assert expected==listed==executed and len(expected)==10
assert 'test result: ok. 10 passed; 0 failed; 0 ignored' in (run/'checks.stdout.log').read_text(encoding='utf-8')
exe=Path(r['product']);assert sha(exe)==r['product_sha256']==c['product_sha256']
with exe.open('rb') as f:
    header=f.read(64);f.seek(struct.unpack_from('<I',header,60)[0]);signature=f.read(6)
assert signature==b'PE\0\0\x64\x86'
normalq=Path(c['normal_qualification']);assert sha(normalq)==c['normal_qualification_sha256'];n=read(normalq)
assert n['source_manifest_sha256']==r['source_manifest_sha256'] and n['passed']==31
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={}
for folder,prefix in [(run,'runtime'),(compiled,'compile')]:
    for p in folder.iterdir():
        if p.is_file():files[prefix+'/'+p.name]=p
files['reader.py']=Path(__file__)
pins={name:sha(path) for name,path in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,path in files.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
save(out/'qualified.json',dict(status='qualified_isolated_ambient_ledger_only',profile=r['profile'],source_manifest_sha256=r['source_manifest_sha256'],
    candidate_module_sha256=r['candidate_module_sha256'],product_sha256=sha(exe),passed=10,failed=0,ignored=0,test_names=sorted(expected),
    original_runtime_session=a.session,actual_terminal_exit_code=0,terminal_chunk=a.terminal_chunk,process_history=r['process_history'],
    raw_files=pins,archive_sha256=sha(out/'raw.zip'),actual_sessions=0,real_casts_verified=False,renderer_verified=False,GUI_verified=False,whole_goal_complete=False))
print(json.dumps(dict(qualification=str(out/'qualified.json'),passed=10,scope='isolated ambient ledger only'),ensure_ascii=False))
