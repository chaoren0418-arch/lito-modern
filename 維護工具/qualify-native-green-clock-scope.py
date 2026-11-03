"""Read back actual source1021 green scope products, inventories and logs."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('directory',type=Path);a=ap.parse_args()
p=a.directory.resolve(strict=True);assert p.is_relative_to(ROOT/'驗證紀錄/current/native-actor-green-clock')
read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
def sha(f):
    with Path(f).open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def entry(z,name):
    with z.open(name) as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='executed_source1021_green_shared_clock_scope' and r['compile_exit_code']==0 and r['owned_process'] is None
pins=read(p/'source-inputs.json');assert len(pins)==r['source_files']==1021
for name,h in pins.items():assert sha(ROOT/name)==h,name
assert sha(p/'compiled-source.zip')==r['compiled_source_sha256']
with zipfile.ZipFile(p/'compiled-source.zip') as z:
    assert set(z.namelist())==set(pins)
    for name,h in pins.items():assert entry(z,name)==h,name
products=read(p/'products.json');assert set(products)==set(r['targets'])==set(r['results'])
counts=dict(passed=0,failed=0,ignored=0)
with zipfile.ZipFile(p/'test-products.zip') as z:
    assert set(z.namelist())=={name+'.exe' for name in products}
    for name,v in products.items():
        result=r['results'][name];assert entry(z,name+'.exe')==sha(v['path'])==v['sha256']==result['test_sha256']
        assert result['exit_code']==0
        names=re.findall(r'^(.+): test$',(p/(name+'-inventory.stdout.log')).read_text(encoding='utf-8-sig'),re.M)
        log=p/(name+'-runtime.stdout.log');assert sha(log)==result['runtime_log_sha256']
        matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log.read_text(encoding='utf-8-sig'))
        assert len(matches)==1 and matches[0][0]=='ok'
        passed,failed,ignored,measured,filtered=map(int,matches[0][1:]);assert failed==measured==filtered==0 and len(names)==passed+ignored==result['inventory_count']
        for key,value in [('passed',passed),('failed',failed),('ignored',ignored)]:assert value==result[key];counts[key]+=value
assert counts=={key:r[key] for key in counts} and counts['failed']==0
raw=read(p/'archive-readback.json');assert raw['verified'] and sha(p/'raw.zip')==raw['raw_archive_sha256']
with zipfile.ZipFile(p/'raw.zip') as z:
    assert set(z.namelist())==set(raw['files'])
    for name,h in raw['files'].items():assert entry(z,name)==h==sha(p/name),name
q=dict(status='independently_qualified_source1021_green_shared_clock_scope',profile=r['profile'],targets=r['targets'],**counts,
    source_files=1021,source_manifest_sha256=sha(p/'source-inputs.json'),actual_test_executables=len(products),
    compiled_source_sha256=sha(p/'compiled-source.zip'),test_products_sha256=sha(p/'test-products.zip'),raw_archive_sha256=sha(p/'raw.zip'),
    scalar_native_reference_vectors=532,queue_native_reference_snapshots=16,queue_native_reference_rows=72,
    current_normal_mains_qualified=False,current_GUI_qualified=False,full_default_library_TCP_regression_verified=False,
    qualified_utc=datetime.now(timezone.utc).isoformat(),qualifier_sha256=sha(Path(__file__)),whole_goal_complete=False)
dest=p/'qualified.json';assert not dest.exists();dest.write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(p/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(q,ensure_ascii=False))
