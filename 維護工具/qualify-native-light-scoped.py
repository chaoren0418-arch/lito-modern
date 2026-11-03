"""Bind actual scoped Rust executions to their current source and retained EXEs."""
from pathlib import Path
import argparse, hashlib, json, re, zipfile

ROOT=Path(__file__).resolve().parents[1]
STAGE=ROOT/'驗證紀錄/current/native-light-spell'
parser=argparse.ArgumentParser()
parser.add_argument('--profile',choices=['Debug','Release'],required=True)
args=parser.parse_args()
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
profile=args.profile
output=STAGE/(profile+'-scoped-qualified.json')
assert not output.exists()
manifest=STAGE/'repaired-source-inputs.json'
pins=read(manifest)
assert len(pins)==970
for n,h in pins.items():assert sha(ROOT/n)==h,n
log=STAGE/('Debug-repaired-scoped.jsonl' if profile=='Debug' else 'Release-scoped.jsonl')
text=log.read_text(encoding='utf-8-sig')
messages=[json.loads(l) for l in text.splitlines() if l.startswith('{')]
assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
products={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m['target']['kind']==['test'] and m.get('executable')}
expected={'native_light_spell','native_skills'}
if profile=='Debug':expected|={'native_world','native_world_metadata','native_holy_weapon','native_effect_lifecycle','native_skill_selection'}
assert set(products)==expected
runs=list(re.finditer(r'Running tests[\\/]([A-Za-z0-9_]+)\.rs\s+\(',text))
results={}
for i,m in enumerate(runs):
    name=m[1];body=text[m.end():runs[i+1].start() if i+1<len(runs) else len(text)]
    terminal=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',body)
    assert len(terminal)==1,(name,terminal)
    status,passed,failed,ignored,measured,filtered=terminal[0]
    assert status=='ok' and failed==ignored==measured==filtered=='0'
    names=re.findall(r'^test (.+) \.\.\. ok$',body,re.M)
    assert len(names)==len(set(names))==int(passed)
    p=products[name];assert p['features']==[] and p['profile']['test']
    results[name]={'passed':int(passed),'test_names':names,'test_executable_sha256':sha(Path(p['executable']))}
assert set(results)==expected
files={'log.jsonl':log,'source-inputs.json':manifest,'qualifier.py':Path(__file__)}
files.update({'source/'+n:ROOT/n for n in pins})
files.update({'test-executables/'+n+'.exe':Path(p['executable']) for n,p in products.items()})
for name in ['red-qualified.json','source-qualified.json','149-selection-supplement-qualified.json','first-scoped-failure.json']:
    files['prior-evidence/'+name]=STAGE/name
generated=ROOT/'LiTo381-Modern/target/current/verification'
for p in generated.iterdir():
    if p.is_file() and p.suffix in ['.json','.png'] and p.stat().st_mtime>=log.stat().st_ctime-1:
        files['generated/'+p.name]=p
file_pins={n:sha(p) for n,p in files.items()}
archive=STAGE/(profile+'-scoped-evidence.zip')
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(file_pins)
    for n,h in file_pins.items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
summary={'status':'qualified_current_scoped_tests_only','profile':profile,'source_files':970,
    'source_manifest_sha256':sha(manifest),'targets':results,'passed':sum(v['passed'] for v in results.values()),
    'failed':0,'ignored':0,'archive_sha256':sha(archive),'archive_file_pins':file_pins,
    'archive_readback_verified':True,'actual_new_game_sessions':0,'new_normal_mains_qualified':False,
    'GUI_verified':False,'illumination_raster_verified':False,'physical_audio_verified':False,'whole_goal_complete':False}
output.write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:summary[k] for k in ['status','profile','source_files','passed','failed','ignored','archive_readback_verified']}))
