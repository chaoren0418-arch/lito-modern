"""Qualify the full compiled lib inventory's default selection; retain all omissions."""
from pathlib import Path
import argparse,hashlib,json,re,struct,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('build_run',type=Path);ap.add_argument('--normal-qualification',type=Path,default=ROOT/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json');a=ap.parse_args();run=a.build_run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current')
assert not (run/'default-library-qualified.json').exists()
build=read(run/'receipt.json');product=read(run/'product-retention.json');runtime=run/'runtime-full-current';result=read(runtime/'receipt.json');pins=read(run/'source-inputs.json')
assert build['exit_code']==0 and build['sources_unchanged'] and build['features']==[] and build['test_debug_information']==0 and build['codegen_units']==256 and build['library_opt_level']==0
assert len(pins)==build['source_files']==result['source_files']==968
for name,h in pins.items():assert sha(ROOT/name)==h,name
assert sha(run/'source-inputs.json')==build['source_manifest_sha256']==result['source_manifest_sha256']
assert result['status']=='passed_pinned_library_tests' and result['exit_code']==0 and result['failed']==0 and result['filter']=='' and result['filtered_out']==0 and result['test_threads']==1 and result['test_target']=='lib'
assert product['archive_readback_verified'] and sha(product['product'])==product['product_sha256']==result['product_sha256']
assert sha(product['archive'])==product['archive_sha256']
with Path(product['product']).open('rb') as f:
 header=f.read(64);f.seek(struct.unpack_from('<I',header,60)[0]);pe=f.read(26)
assert pe[:4]==b'PE\0\0' and struct.unpack_from('<H',pe,4)[0]==0x8664 and struct.unpack_from('<H',pe,24)[0]==0x20b
inventory=read(run/'inventory-audit/inventory.json');ignored=read(run/'inventory-audit/ignored-source-reasons.json')
assert inventory['product_sha256']==product['product_sha256'] and inventory['total_tests']==2402 and inventory['default_tests_selected']==1929 and inventory['ignored_tests']==473
assert ignored['mapped_unique']==473 and ignored['mapped_ambiguous']==ignored['unmapped']==0
log=(runtime/'tests.log').read_text(encoding='utf-8-sig');matches=re.findall(r'^test (.+) \.\.\. (ok|ignored)(?:,.*)?$',log,re.M)
assert len(matches)==2402 and len({name for name,_ in matches})==2402 and {name for name,_ in matches}==set(inventory['all_test_names'])
assert {name for name,status in matches if status=='ignored'}==set(inventory['ignored_test_names'])
assert sum(status=='ok' for _,status in matches)==result['passed']==1929 and result['ignored']==473
terminals=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log);assert terminals==[('ok','1929','0','473','0','0')]
restored=read(runtime/'legacy-report-restoration.json');prior=read(runtime/'legacy-reports-before.json');assert restored['existing_reports_restored'] and prior['readback_verified'] and len(prior['files'])==1719
assert sha(runtime/'legacy-reports-before.zip')==prior['archive_sha256']==restored['before_archive_sha256']
assert sha(runtime/'legacy-generated-reports.zip')==restored['generated_archive_sha256']
legacy=PROJECT/'logs/continuation-20261003'
for name,h in prior['files'].items():assert sha(legacy/name)==h,name
fixtures=read(ROOT/'驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json');assert len(fixtures['inputs'])==19 and len(fixtures['outputs'])==8 and len(fixtures['files'])==353
for name,h in fixtures['files'].items():assert sha(ROOT/name)==h,name
normal=a.normal_qualification.resolve(strict=True);assert normal.is_relative_to(ROOT/'驗證紀錄/current');n=read(normal)
assert n['source_files']==968 and n['features']==[] and sha(normal.parent/'source-inputs.json')==n['source_manifest_sha256']==sha(run/'source-inputs.json')
for item in n['profiles'].values():assert sha(item['path'])==item['sha256']
files={}
for f in run.rglob('*'):
 if f.is_file() and f.name not in ['library-test-product.zip','default-library-evidence.zip','default-library-qualified.json','qualification-console.txt']:
  assert f.resolve().is_relative_to(run) and not f.is_symlink();files['run/'+str(f.relative_to(run)).replace('\\','/')]=f
outputs={};pid=result['pid']
for family in ['native-backend-485f-evidence','native-multisector-batch']:
 dirs=list((PROJECT/'target/current/scratch'/family).glob(f'run-{pid}-*'));assert len(dirs)==1
 for f in dirs[0].rglob('*'):
  if f.is_file():
   assert f.resolve().is_relative_to(dirs[0]) and not f.is_symlink() and f.stat().st_size<=8*1024**2
   name=family+'/'+str(f.relative_to(dirs[0])).replace('\\','/');files['owned-test-output/'+name]=f;outputs[name]=sha(f)
assert sum(name.endswith('.png') for name in outputs)==31
files['fixture-input-map.json']=ROOT/'驗證紀錄/current/native-teleport-wire/library-pet-fixture-map.json';files['normal-qualification.json']=normal;files['qualifier.py']=Path(__file__)
for name in ['inventory-pinned-library-tests.py','audit-ignored-library-sources.py','qualify-native-library-build.py','preserve-legacy-library-reports.py']:
 files['validation-sources/'+name]=ROOT/'維護工具'/name
archive=run/'default-library-evidence.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for name,f in files.items():z.write(f,name)
file_pins={name:sha(f) for name,f in files.items()}
with zipfile.ZipFile(archive) as z:
 assert set(z.namelist())==set(file_pins)
 for name,h in file_pins.items():
  with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
for name,h in pins.items():assert sha(ROOT/name)==h,name
proof={'status':'qualified_current_full_library_default_selection','source_files':968,'source_manifest_sha256':sha(run/'source-inputs.json'),'product_sha256':product['product_sha256'],'machine':'AMD64/PE32+','features':[],'compiled_test_inventory':2402,'passed':1929,'failed':0,'ignored':473,'filtered_out':0,'default_full_library_selection_verified':True,'all_2402_test_bodies_executed':False,'ignored_tests_verified':False,'ignored_source_conditions_mapped':473,'original_26_failure_cases_in_full_default_selection_passed':True,'test_debug_information':0,'codegen_units':256,'library_opt_level':0,'compiler_peak_private_bytes':product['sampled_peak_private_bytes'],'minimum_sampled_host_available_commit_kib':product['minimum_sampled_host_available_commit_kib'],'product_archive_sha256':product['archive_sha256'],'runtime_receipt_sha256':sha(runtime/'receipt.json'),'runtime_log_sha256':sha(runtime/'tests.log'),'historical_reports_restored':1719,'generated_legacy_reports_retained':len(restored['changes']),'fixed_historical_pet_input_files_checked':353,'fixed_pet_input_keys':19,'fixed_pet_output_keys':8,'fixed_pet_environment_keys':27,'current_formal_pet_race_verified':False,'owned_test_output_PNGs_retained':31,'raw_evidence_archive_sha256':sha(archive),'archive_readback_verified':True,'raw_file_pins':file_pins,'normal_mains_unchanged':{p:x['sha256'] for p,x in n['profiles'].items()},'normal_qualification_sha256':sha(normal),'full_current_TCP_regression_verified':False,'GUI_verified_by_this_run':False,'G11_acceptance_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(__file__)}
(run/'default-library-qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','compiled_test_inventory','passed','failed','ignored','historical_reports_restored','owned_test_output_PNGs_retained','all_2402_test_bodies_executed']},ensure_ascii=False))
