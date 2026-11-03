"""Resume the preserved metadata filename failure; reuse the two terminal outputs."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,copy,json,os,subprocess,tomllib,traceback
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_owned_process import identity,kernel
T=R/'維護工具';V=R/'驗證紀錄/current'
def resume(previous):
 previous=owned(previous.resolve(strict=True),V/'native-E149-slow29-metadata-compile');old=read(previous/'cohort.json')
 require(old['status']=='compiler_failure_preserved' and old['owned_process'] is None and len(old['compiler_runs'])==3,'one preserved original filename failure required')
 require([(row['profile'],row['kind'],row['exit_code']) for row in old['compiler_runs']]==[('Debug','library',0),('Debug','library-test',0),('Debug','blind20',1)],'unexpected prior compiler outcomes')
 failed=old['compiler_runs'][-1];check_pins(failed['logs'])
 require('extern location for lito_modern is of an unknown type:' in (previous/'Debug-blind20/stderr.log').read_text('utf-8-sig'),'different compiler failure; do not auto-resume')
 ids=','.join(str(row['pid']) for row in [old['pipeline'],*[entry['original_process'] for entry in old['compiler_runs']]])
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',query],creationflags=subprocess.CREATE_NO_WINDOW)
 live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live
 for row in [old['pipeline'],*[entry['original_process'] for entry in old['compiler_runs']]]:
  require(not any(item['ProcessId']==row['pid'] and abs((datetime.fromisoformat(item['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for item in live),'original compiler/pipeline is still live')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'another workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'metadata development only')
 out=owned(V/'native-E149-slow29-metadata-compile'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 products=owned(P/'target/current/verification/native-E149-slow29-metadata'/out.name);products.mkdir(parents=True);(products/'scratch').mkdir()
 (out/'original-terminal-CIM.json').write_bytes(observed);(out/'continuation.py').write_bytes(Path(__file__).read_bytes())
 pins=dict(old['input_pins']);compiler=str(T/'compile-native-E149-slow29-candidate.py');original_compiler_pin=pins.pop(compiler)
 require(sha(previous/'compiler.py')==original_compiler_pin,'original compiler control snapshot changed');pins[str(previous/'compiler.py')]=original_compiler_pin
 pins[compiler]=sha(compiler);pins[str(Path(__file__).resolve())]=sha(__file__);pins[str(previous/'cohort.json')]=sha(previous/'cohort.json')
 check_pins(pins);check_pins(old['normal_product_pins']);p=read(old['original_component_plan'])
 require(sha(old['original_component_plan'])==old['original_component_plan_sha256'],'original component plan changed')
 copied=products/'liblito_modern-Debug.rmeta';original=Path(old['compiler_runs'][0]['product'])
 require(sha(original)==old['compiler_runs'][0]['product_sha256'],'original successful Debug metadata changed')
 with copied.open('xb') as file:file.write(original.read_bytes())
 require(sha(copied)==sha(original),'renamed library copy differs');pins[str(copied)]=sha(copied)
 for row in old['compiler_runs'][:2]:
  check_pins(row['logs']);require(sha(row['product'])==row['product_sha256'],'successful original output changed')
 record=copy.deepcopy(old);record.update(status='continuing_metadata_filename_repair_only',pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),
  compiler_runs=copy.deepcopy(old['compiler_runs'][:2]),input_pins=pins,owned_process=None,
  previous_original_cohort=str(previous/'cohort.json'),previous_original_cohort_sha256=sha(previous/'cohort.json'),
  previous_original_terminal_tool_chunk='68c631',previous_original_actual_exit_code=1,previous_failed_compiler=failed,
  reused_original_successful_compilers=2,previous_control_snapshot=str(previous/'compiler.py'),previous_control_snapshot_sha256=original_compiler_pin,
  dependency_library_copy=dict(original=str(original),copy=str(copied),sha256=sha(copied)),new_compilers_started=0,
  preparation_failure=str(V/'native-E149-slow29-metadata-compile/20261024-003622-696035/failure.json'))
 for index,row in enumerate(record['compiler_runs']):row['reused_from_original_cohort']=str(previous/'cohort.json');row['original_entry_index']=index
 def save():(out/'cohort.json').write_bytes(encoded(record))
 inputs=read(old['base_compiled_inputs'])
 def guard():
  check_pins(pins);check_pins(old['normal_product_pins'])
  require(len(inputs)==1049 and all(sha(R/name)==h for name,h in inputs.items()),'normal source changed')
  require(sha(read(R/'工作佇列.json')['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 environment={name:value for name,value in os.environ.items() if not name.startswith(('LITO_','RUST_')) and name not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']}
 environment.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],
  CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'))
 guard();save();print(json.dumps(dict(run=str(out),reused_successful_compilers=2,new_compilers_planned=6,tests_executed=0),ensure_ascii=False),flush=True)
 try:
  for profile in ['Debug','Release']:
   library=copied if profile=='Debug' else products/'liblito_modern-Release.rmeta'
   for kind in (['blind20','slow29'] if profile=='Debug' else ['library','library-test','blind20','slow29']):
    guard();command=list(p['compiler_templates'][profile]);source=old['generated_root'] if kind in ['library','library-test'] else old['paired_tests'][kind]
    command[command.index(p['roots']['candidate'])]=source;product=library if kind=='library' else products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind=='library':command.remove('--test');command+=['--crate-type=rlib']
    elif kind in old['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(library)]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
     child=subprocess.Popen(command,env=environment,cwd=products,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
     row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),exit_code=None,product=str(product));record['compiler_runs'].append(row)
     record['owned_process']=row['original_process'];record['new_compilers_started']+=1;save()
     try:child.wait(timeout=300)
     except BaseException:
      if child.poll() is None:
       require(identity(int(child._handle),child.pid)==row['original_process'],'original compiler handle changed');child.terminate();child.wait(timeout=20);row['exact_original_compiler_cleanup']=True
      raise
     finally:row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),logs={str(folder/name):sha(folder/name) for name in ['stdout.log','stderr.log']});record['owned_process']=None;save()
    require(child.returncode==0,'continued original compiler failed; preserve without retry');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();guard()
  record['status']='metadata_compilers_complete_no_tests'
 except BaseException:
  record['status']='compiler_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:record['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(run=str(out),successful_compiler_exit_codes=[row['exit_code'] for row in record['compiler_runs']],previous_failed_compilers=1,new_compilers_started=record['new_compilers_started'],tests_executed=0),ensure_ascii=False),flush=True)
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('previous',type=Path);args=ap.parse_args();resume(args.previous)
if __name__=='__main__':main()
