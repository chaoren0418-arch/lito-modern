"""Compile baseline metadata or baseline/candidate linked test PE. Never execute PE."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,ctypes,json,os,subprocess,tomllib,traceback
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,check_pins,require

def validate(path):
 p=read(owned(path,R/'維護工具/candidates'))
 require(p['schema']=='E149-recovery-component-matrix-plan-v1' and p['status']=='development_plan_only','component plan scope differs')
 require(p['source_manifest_sha256']==SOURCE and p['formal_target_sha256']==FORMAL,'component source/target differs')
 check_pins(p['input_pins']);check_pins(p['normal_product_pins'])
 require(all(sha(n)==h for n,h in p['external_toolchain_dependency_pins'].items()),'read-only toolchain changed')
 inputs=read(p['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal inputs changed')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'active timed workload/source freeze')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 return p

def compile_matrix(path,metadata_only):
 path=owned(path.resolve(strict=True));p=validate(path)
 out=R/'驗證紀錄/current/native-E149-component-matrix-compile'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 products=P/'target/current/verification/native-E149-component-matrix'/out.name;products.mkdir(parents=True);(products/'scratch').mkdir()
 (out/'compiler.py').write_bytes(Path(__file__).read_bytes())
 k=ctypes.WinDLL('kernel32',use_last_error=True);k.GetCurrentProcess.restype=ctypes.c_void_p
 k.GetProcessTimes.argtypes=[ctypes.c_void_p]+[ctypes.c_void_p]*4
 def identity(handle,pid):
  times=[ctypes.c_uint64() for _ in range(4)]
  require(k.GetProcessTimes(handle,*[ctypes.byref(x) for x in times]),'original compiler birth unavailable')
  return dict(pid=pid,creation_filetime_ticks=times[0].value,start_epoch_millis=(times[0].value-116444736000000000)//10000)
 record=dict(schema='E149-recovery-component-matrix-compile-v1',status='preparing_compilers_only',
  plan=str(path),plan_sha256=sha(path),metadata_only=metadata_only,pipeline=identity(k.GetCurrentProcess(),os.getpid()),
  compiler_runs=[],owned_process=None,input_pins=p['input_pins'],external_toolchain_dependency_pins=p['external_toolchain_dependency_pins'],
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,products=str(products),normal_product_pins=p['normal_product_pins'],
  tests_executed=0,PEs_executed=False,GUI_inputs=0,normal_source_modified=False,normal_products_modified=False,whole_goal_complete=False)
 def save():(out/'cohort.json').write_bytes(encoded(record))
 save();print(json.dumps(dict(run=str(out),mode='baseline_metadata_only' if metadata_only else 'four_library_test_links_only'),ensure_ascii=False),flush=True)
 environment={n:v for n,v in os.environ.items() if not n.startswith('LITO_') and n not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']}
 configuration=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))
 environment.update(TEMP=str(products/'scratch'),TMP=str(products/'scratch'),CARGO_MANIFEST_DIR=str(P),
  CARGO_PKG_VERSION=configuration['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1')
 try:
  for profile in ['Debug','Release']:
   for variant in (['baseline'] if metadata_only else ['baseline','candidate']):
    validate(path);command=list(p['compiler_templates'][profile])
    before=p['roots']['candidate'];require(command.count(before)==1,'original library-test root differs')
    command[command.index(before)]=p['roots'][variant]
    suffix='.rmeta' if metadata_only else '.exe';product=products/(profile+'-'+variant+suffix)
    command[command.index('-o')+1]=str(product)
    require(command.count('--emit=metadata')==1 and '--test' in command,'not an original metadata test command')
    if not metadata_only:
     command[command.index('--emit=metadata')]='--emit=link'
     for native in p['native_link_search_paths'][profile]:command+=['-L',native]
    folder=out/(profile+'-'+variant);folder.mkdir()
    with (folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
     child=subprocess.Popen(command,cwd=products,env=environment,stdout=stdout,stderr=stderr,
      creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
     row=dict(profile=profile,variant=variant,command=command,product=str(product),
      original_process=identity(int(child._handle),child.pid),exit_code=None)
     record['compiler_runs'].append(row);record['owned_process']=row['original_process'];record['status']='compiler_running';save()
     try:child.wait(timeout=900 if not metadata_only else 300)
     except BaseException:
      if child.poll() is None:
       require(identity(int(child._handle),child.pid)==row['original_process'],'compiler original handle identity differs')
       child.terminate();child.wait(timeout=20);row['exact_original_compiler_cleanup']=True
      raise
     finally:
      row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),
       logs={str(folder/n):sha(folder/n) for n in ['stdout.log','stderr.log']})
      record['owned_process']=None;save()
    require(child.returncode==0,'original component compiler failed; preserve outputs')
    require(product.is_file(),'compiler product missing');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();validate(path)
  record['status']='baseline_metadata_complete_no_tests' if metadata_only else 'linked_test_images_complete_no_execution'
 except BaseException:
  record['status']='compiler_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:record['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(run=str(out),compiler_exit_codes=[x['exit_code'] for x in record['compiler_runs']],metadata_only=metadata_only,tests_executed=0),ensure_ascii=False))

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('plan',type=Path)
 group=ap.add_mutually_exclusive_group(required=True);group.add_argument('--metadata-only',action='store_true');group.add_argument('--link-only',action='store_true')
 a=ap.parse_args();compile_matrix(a.plan,a.metadata_only)
if __name__=='__main__':main()
