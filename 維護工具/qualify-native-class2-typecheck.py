"""Qualify metadata compilation only, never rendering or gameplay."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--compile',type=Path,required=True);ap.add_argument('--session',required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
assert a.terminal_chunk not in ['','pending','unknown']
base=a.compile.resolve(strict=True);assert base.is_relative_to(ROOT/'驗證紀錄/current/native-class2-presentation')
c=read(base/'receipt.json');assert c['status']=='compiled_current_class2_candidate_typecheck' and c['compiler']['exit_code']==0
assert '--emit' in c['compiler']['command'] and 'metadata' in c['compiler']['command'] and '--test' not in c['compiler']['command']
assert c['source_manifest_sha256']=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182'
assert sha(c['product'])==c['product_sha256']
assert sha(base/'candidate_lib.rs')==c['compiled_probe_sha256']
assert sha(base/'modern_lito_overlay.rs')==c['generated_overlay_sha256']
for name,pin in c['candidate_inputs'].items():assert sha(base/name)==c.get('snapshot_candidate_inputs',{}).get(name,pin['sha256']) and sha(pin['path'])==pin['sha256']
inputs=read(base/'compiled-inputs.json');assert len(inputs)==1031
for name,digest in inputs.items():assert sha(ROOT/name)==digest,name
q=Path(c['normal_qualification']);assert sha(q)==c['normal_qualification_sha256'] and read(q)['source_manifest_sha256']==c['source_manifest_sha256']
stderr=(base/'stderr.log').read_text(encoding='utf-8');assert not any(line.startswith('error') for line in stderr.splitlines())
out=base/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={p.name:p for p in base.iterdir() if p.is_file()};files['reader.py']=Path(__file__)
pins={name:sha(path) for name,path in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,path in files.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==digest for n,digest in pins.items())
save(out/'qualified.json',dict(status='qualified_isolated_class2_metadata_typecheck_only',source_manifest_sha256=c['source_manifest_sha256'],profile=c['profile'],
    candidate_inputs=c['candidate_inputs'],metadata_sha256=c['product_sha256'],original_session=a.session,actual_terminal_exit_code=0,terminal_chunk=a.terminal_chunk,
    compiler=c['compiler'],raw_files=pins,archive_sha256=sha(out/'raw.zip'),product_source_modified=False,actual_sessions=0,real_casts_verified=False,renderer_verified=False,whole_goal_complete=False))
print(json.dumps(dict(qualification=str(out/'qualified.json'),scope='metadata typecheck only'),ensure_ascii=False))
