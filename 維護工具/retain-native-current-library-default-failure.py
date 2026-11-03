"""Preserve the actual full-default failure before any test-only correction."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def entry(z,n):
    with z.open(n) as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('cohort',type=Path);a=ap.parse_args();run=a.cohort.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-actor-green-clock')
c=read(run/'cohort.json');assert c['status']=='failed_preserved' and c['owned_process'] is None and set(c['profiles'])=={'Debug'}
d=run/'Debug';runtime=d/'runtime-full-current';r=read(runtime/'receipt.json');pins=read(run/'source-inputs.json')
assert (r['exit_code'],r['passed'],r['failed'],r['ignored'],r['filtered_out'])==(101,1934,1,473,0)
assert r['sources_unchanged'] and len(pins)==1021
for name,h in pins.items():assert sha(ROOT/name)==h,name
inventory=read(d/'inventory-audit/inventory.json');ignored=read(d/'inventory-audit/ignored-source-reasons.json')
assert inventory['total_tests']==2408 and ignored['mapped_unique']==473 and ignored['mapped_ambiguous']==ignored['unmapped']==0
log=(runtime/'tests.log').read_text(encoding='utf-8-sig')
rows=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log,re.M)
assert len(rows)==len({n for n,_ in rows})==2408 and {n for n,_ in rows}==set(inventory['all_test_names'])
assert {n for n,s in rows if s=='ignored'}==set(inventory['ignored_test_names'])
assert [sum(s==key for _,s in rows) for key in ['ok','FAILED','ignored']]==[1934,1,473]
failed=[n for n,s in rows if s=='FAILED'];assert failed==['modern_lito::native_login::teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner']
product=read(d/'product-retention.json');assert sha(Path(product['product']))==product['product_sha256']==r['product_sha256']
assert sha(Path(product['archive']))==product['archive_sha256']
with zipfile.ZipFile(product['archive']) as z:assert entry(z,product['retained_archive_member'])==product['product_sha256']
prior=read(runtime/'legacy-reports-before.json');restored=read(runtime/'legacy-report-restoration.json')
assert prior['readback_verified'] and restored['existing_reports_restored']
assert sha(runtime/'legacy-reports-before.zip')==prior['archive_sha256']==restored['before_archive_sha256']
with zipfile.ZipFile(runtime/'legacy-reports-before.zip') as z:
    assert set(z.namelist())==set(prior['files'])
    for name,h in prior['files'].items():assert entry(z,name)==h==sha(ROOT/'LiTo381-Modern/logs/continuation-20261003'/name)
assert sha(runtime/'legacy-generated-reports.zip')==restored['generated_archive_sha256']
with zipfile.ZipFile(runtime/'legacy-generated-reports.zip') as z:
    for row in restored['changes']:assert entry(z,row['name'])==row['generated_sha256']
fixtures=read(run/'fixture-input-map.json');assert len(fixtures['files'])==353
for name,h in fixtures['files'].items():assert sha(ROOT/name)==h,name
red=read(run/'malformed-target-red/receipt.json');assert red['exit_code']==101 and red['product_sha256']==product['product_sha256']
before=run/'native_teleport_controller_tests-before.rs';before.write_bytes((ROOT/'LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs').read_bytes())
assert sha(before)==pins['LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs']
files={str(f.relative_to(run)).replace('\\','/'):f for f in run.rglob('*') if f.is_file() and f.suffix.lower()!='.zip'}
outputs={}
for family in ['native-backend-485f-evidence','native-multisector-batch']:
    folders=list((ROOT/'LiTo381-Modern/target/current/scratch'/family).glob(f"run-{r['pid']}-*"));assert len(folders)==1,(family,folders)
    for f in folders[0].rglob('*'):
        if f.is_file():
            assert f.resolve().is_relative_to(folders[0]) and not f.is_symlink() and f.stat().st_size<=8*1024**2
            name='owned-output/'+family+'/'+str(f.relative_to(folders[0])).replace('\\','/');files[name]=f;outputs[name]=sha(f)
wire=ROOT/'LiTo381-Modern/target/current/verification/native-teleport-wire/Debug'
for f in wire.glob('*.json'):files['wire-reports-retained-with-own-timestamps/'+f.name]=f
files['retainer.py']=Path(__file__)
archive=run/'failure-raw-evidence.zip';assert not archive.exists()
hashes={name:sha(f) for name,f in files.items()}
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,f in files.items():z.write(f,name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(hashes)
    for name,h in hashes.items():assert entry(z,name)==h,name
q=dict(status='independently_retained_source1021_Debug_full_default1934_pass1_fail473_ignored',source_files=1021,
    source_manifest_sha256=sha(run/'source-inputs.json'),original_unified_session_id=76012,original_unified_exit_code=1,Rust_exit_code=101,
    inventory=2408,passed=1934,failed=1,ignored=473,filtered_out=0,failed_test=failed[0],Release_default_executed=False,
    unchanged_target_red_Rust_exit_code=101,actual_test_sha256=product['product_sha256'],actual_test_archive=product['archive'],actual_test_archive_sha256=product['archive_sha256'],
    historical_reports_restored=len(prior['files']),generated_changed_reports=len(restored['changes']),historical_pet_input_files_verified=353,
    owned_output_PNGs=sum(n.endswith('.png') for n in outputs),raw_archive_sha256=sha(archive),raw_archive_files=hashes,
    legacy_before_archive_sha256=prior['archive_sha256'],legacy_generated_archive_sha256=restored['generated_archive_sha256'],
    failed_test_loop_completed=False,GUI_verified=False,full_current_TCP_verified=False,whole_goal_complete=False,retainer_sha256=sha(Path(__file__)),observed_utc=datetime.now(timezone.utc).isoformat())
(run/'failure-readback.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({k:q[k] for k in ['status','historical_reports_restored','owned_output_PNGs','Release_default_executed']},ensure_ascii=False))
