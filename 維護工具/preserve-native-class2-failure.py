"""Seal a terminal failed owned diagnostic and its immutable compiler snapshot."""
from pathlib import Path
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--chunk',required=True);ap.add_argument('--status',required=True);ap.add_argument('--reason',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-class2-world')
r=read(run/'receipt.json');assert r['status']=='failed_preserved' and r['all_started_processes_terminal']
assert all(j.get('exit_code') is not None for j in r['process_history'])
assert not (run/'failure-preserved.json').exists() and not (run/'failed-runtime.zip').exists()
runtime=Path(r['runtime']);build=Path(r['compile'])
files=[(p,'run/'+p.relative_to(run).as_posix()) for p in run.rglob('*') if p.is_file()]
files.extend((p,'runtime/'+p.relative_to(runtime).as_posix()) for p in runtime.rglob('*') if p.is_file())
files.extend((p,'compile/'+p.relative_to(build).as_posix()) for p in build.iterdir() if p.is_file())
pins={name:sha(p) for p,name in files};assert len(pins)==len(files)
with zipfile.ZipFile(run/'failed-runtime.zip','x',zipfile.ZIP_DEFLATED) as z:
    for p,name in files:z.write(p,name)
with zipfile.ZipFile(run/'failed-runtime.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
record=dict(status=a.status,original_session=a.session,actual_exit_code=1,terminal_chunk=a.chunk,all_owned_terminal=True,
    archive_sha256=sha(run/'failed-runtime.zip'),files=pins,reason=a.reason,normal_product_source_changed=False,whole_goal_complete=False)
(run/'failure-preserved.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(preserved=str(run/'failure-preserved.json'),files=len(files),whole_goal_complete=False),ensure_ascii=False))
