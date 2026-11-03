"""Preserve the real failed raster scope before fixing hidden/raw provenance boundaries."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
OUT=APP/'object-gate-tests-20261016-130245-955826'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
r=read(OUT/'receipt.json');assert r['status']=='failed_preserved' and r['owned_process'] is None
assert r['results']['lib-world-raster']['failed']==1 and r['results']['lib-world-raster']['exit_code']==101
pins=read(OUT/'compiled-inputs.json');assert len(pins)==1019
for n,h in pins.items():assert digest((ROOT/n).read_bytes())==h,n
for archive,index in [('compiled-source.zip',pins),('test-products.zip',{n+'.exe':v['sha256'] for n,v in read(OUT/'products.json').items()}),
    ('incidental-bin-products.zip',{n+'.exe':v['sha256'] for n,v in read(OUT/'incidental-bin-products.json').items()})]:
    with zipfile.ZipFile(OUT/archive) as z:
        assert set(z.namelist())==set(index)
        for n,h in index.items():assert digest(z.read(n))==h,n
metadata={p.relative_to(OUT).as_posix():p for p in OUT.rglob('*') if p.is_file() and p.suffix!='.zip'}
proof={'status':'failed_initial_gate_scope_preserved_not_qualified','observed_utc':datetime.now(timezone.utc).isoformat(),
    'source_files':1019,'test_session':94991,'observed_outer_terminal_exit_code':1,
    'integration_passed':sum(v['passed'] for n,v in r['results'].items() if not n.startswith('lib-')),
    'raster_passed':10,'raster_failed':1,'world_scene_group_executed':False,
    'actual_test_executables_retained':len(read(OUT/'products.json')),
    'failure':'A direct Hidden SPR reached blit_actor_state_fragment, which only accepts Half/Dim/Green.',
    'independent_scope_defect':'A raw diamond preview does not establish original RLE object coverage or signed source header eligibility.',
    'planned_fix':'Retain exact TIL prefix metadata; mark raw object coverage unqualified; skip Hidden submissions before tone paint.',
    'all_archives_readback_verified':True,'whole_goal_complete':False}
save(OUT/'scope-review.json',proof);metadata['scope-review.json']=OUT/'scope-review.json';metadata['reviewer.py']=Path(__file__)
hashes={n:digest(p.read_bytes()) for n,p in metadata.items()}
with zipfile.ZipFile(OUT/'failed-scope-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in metadata.items():z.write(p,n)
with zipfile.ZipFile(OUT/'failed-scope-raw.zip') as z:
    for n,h in hashes.items():assert digest(z.read(n))==h
save(OUT/'failed-scope-readback.json',{**proof,'files':hashes,'raw_archive_sha256':digest((OUT/'failed-scope-raw.zip').read_bytes())})
print(json.dumps({k:proof[k] for k in ['status','integration_passed','raster_passed','raster_failed','world_scene_group_executed','actual_test_executables_retained']}))
