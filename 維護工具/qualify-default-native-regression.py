"""Validate exact default Rust products/logs without rerunning or inflating scope."""
from pathlib import Path
import argparse,datetime,hashlib,json,re,zipfile
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--allow-harness-only-delta',action='store_true');ap.add_argument('--normal-qualification',type=Path,default=root/'驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json');a=ap.parse_args()
run=a.run.resolve(strict=True);output=a.output.resolve();assert run.is_relative_to(root/'驗證紀錄/current/default-native-regression') and output.is_relative_to(root)
assert not output.exists(),'Preserve prior qualifications; choose new output.'
r=read(run/'receipt.json');assert r['status']=='verified_current_default90_integration_targets_and_main_units'
assert r['features']==[] and not r['library_unit_tests'] and r['sources_unchanged'] and r['fixture_inputs_unchanged']
assert r['compile_exit_code']==0 and r['failed']==0
old=read(run/'source-inputs.json');assert len(old)==r['source_files']==968 and sha(run/'source-inputs.json')==r['source_manifest_sha256']
normal_path=a.normal_qualification.resolve(strict=True);assert normal_path.is_relative_to(root/'驗證紀錄/current')
normal=normal_path.parent
q=read(normal_path);current=read(normal/'source-inputs.json');assert set(old)==set(current)
for name,h in current.items():assert sha(root/name)==h,name
changed={name for name in old if old[name]!=current[name]}
if changed:
 assert a.allow_harness_only_delta
 assert changed=={'LiTo381-Modern/tests/oracle/'+n for n in ['NativeLoginHarness.java','NativeGameHarness.java','NativeRecoveryHarness.java']}
assert r['normal_mains_before']==r['normal_mains_after']=={p:q['profiles'][p]['sha256'] for p in ['Debug','Release']}
for item in q['profiles'].values():assert sha(item['path'])==item['sha256']
meta=read(run/'metadata/stdout.log');package=next(p for p in meta['packages'] if Path(p['manifest_path']).resolve()==root/'LiTo381-Modern/Cargo.toml')
targets=[t for t in package['targets'] if 'test' in t['kind']];suites={t['name'] for t in targets if not t.get('required-features')}
excluded={t['name']:t['required-features'] for t in targets if t.get('required-features')}
assert len(suites)==90 and excluded=={'native_teleport_controller':['teleport-wire-test']}
assert set(r['suites'])==suites and set(r['results'])==suites|{'main_unit'}
retained=read(run/'product-retention.json');assert retained['readback_verified'] and set(retained['products'])==set(r['results'])
assert sha(retained['archive'])==retained['archive_sha256']
counts={'passed':0,'failed':0,'ignored':0};log_pins={}
for name,result in r['results'].items():
 case=run/'cases'/name;product=retained['products'][name];assert sha(product['path'])==product['sha256']==result['product_sha256']
 if changed:
  dep=Path(product['path']).with_suffix('.d');assert dep.exists();text=dep.read_text(encoding='utf-8',errors='replace')
  assert not any(Path(n).name in text for n in changed),name
 inv=(case/'inventory/stdout.log').read_text(encoding='utf-8-sig');test_names=re.findall(r'^(.+): test$',inv,re.M)
 log=case/'runtime/stdout.log';text=log.read_text(encoding='utf-8-sig');assert sha(log)==result['runtime_log_sha256']
 m=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text);assert len(m)==1
 terminal,passed,failed,ignored,measured,filtered=m[0];passed,failed,ignored,measured,filtered=map(int,[passed,failed,ignored,measured,filtered])
 assert terminal=='ok' and failed==filtered==measured==0 and result['exit_code']==0 and not result['timed_out']
 assert len(test_names)==result['inventory_count']==passed+failed+ignored
 assert (passed,failed,ignored)==(result['passed'],result['failed'],result['ignored'])
 for k,v in zip(['passed','failed','ignored'],[passed,failed,ignored]):counts[k]+=v
 log_pins[name]={'inventory_sha256':sha(case/'inventory/stdout.log'),'runtime_sha256':sha(log),'product_sha256':product['sha256']}
for k,v in counts.items():assert v==r[k]
fixtures=read(run/'pet-fixture-inputs.json');assert len(fixtures['files'])==353
for name,h in fixtures['files'].items():assert sha(root/name)==h,name
result={'status':'qualified_retained_default_Rust_results_after_Java_harness_only_delta' if changed else 'qualified_current_default_Rust_results','profile':r['profile'],'run':str(run),'receipt_sha256':sha(run/'receipt.json'),'results_rerun':False,'integration_targets':90,'products_retained':91,'features':[],**counts,'normal_mains':r['normal_mains_after'],'original_source_manifest_sha256':sha(run/'source-inputs.json'),'current_source_manifest_sha256':sha(normal/'source-inputs.json'),'Java_only_changes_outside_Rust_dependencies':sorted(changed),'current_target_inventory':sorted(suites),'excluded_required_feature_targets':excluded,'per_target_pins':log_pins,'fixed_fixture_inputs_checked':353,'product_archive_sha256':retained['archive_sha256'],'full_library_unit_regression_verified':False,'full_current_TCP_regression_verified':False,'GUI_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(__file__),'qualified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:result[k] for k in ['status','profile','passed','failed','ignored','results_rerun']},ensure_ascii=False))
