"""Eight compiler-only metadata commands for the isolated slow extension."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,os,re,subprocess,time,tomllib,traceback
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_owned_process import identity,kernel
V=R/'驗證紀錄/current';T=R/'維護工具'
def compile_candidate(candidate_path,plan_path):
 candidate_path=owned(candidate_path.resolve(strict=True));c=read(candidate_path)
 plan_path=owned(plan_path.resolve(strict=True));p=read(plan_path)
 require(c['formal_target_sha256']==FORMAL and c['base_source_manifest_sha256']==SOURCE,'candidate target differs')
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'compiler-only development expected')
 check_pins(p['input_pins']);check_pins(p['normal_product_pins'])
 require(all(sha(name)==h for name,h in p['external_toolchain_dependency_pins'].items()),'read-only dependency changed')
 out=owned(V/'native-E149-slow29-metadata-compile'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 products=owned(P/'target/current/verification/native-E149-slow29-metadata'/out.name);products.mkdir(parents=True);(products/'scratch').mkdir()
 original_root=Path(p['roots']['candidate']);lib=original_root.read_text('utf-8-sig')
 matches=list(re.finditer(r'#\[path = r#"([^"]+modern_lito[.]rs)"#\]',lib));require(len(matches)==1,'generated full module root differs')
 original_mod=Path(matches[0][1]);modern=original_mod.read_text('utf-8-sig');runtime=c['changed_runtime_files'];overridden=[];runtime_sources={}
 def replacement(match):
  path=Path(match[1])
  for name in runtime:
   if path.name==Path(name).name:
    expected=(Path(c['predecessor_blind20_candidate']).parent/name) if path.name!='native_additional_finite_effects.rs' else R/name
    allowed=[expected] if path.name=='native_additional_finite_effects.rs' else [expected,original_root.parent/name]
    require(path in allowed and sha(path)==c['base_files'][name],'unexpected previous runtime path/content')
    if path!=expected:require(p['input_pins'].get(str(path))==sha(path),'generated runtime copy was not pinned in original plan')
    runtime_sources[name]=dict(path=str(path),sha256=sha(path),original_candidate_or_normal=str(expected));overridden.append(name)
    return match[0].replace(match[1],(candidate_path.parent/name).as_posix())
  return match[0]
 modern=re.sub(r'#\[path = r#"([^"]+)"#\]',replacement,modern)
 require(sorted(overridden)==sorted(runtime),'complete graph did not override exactly three runtime modules')
 (out/'modern_lito.rs').write_text(modern,encoding='utf-8',newline='\n')
 (out/'lib.rs').write_text(lib.replace(matches[0][1],(out/'modern_lito.rs').as_posix()),encoding='utf-8',newline='\n')
 fixture=P/'tests/fixtures/native-world-current.json';paired={}
 for key,relative in [('blind20','LiTo381-Modern/tests/native_blind.rs'),('slow29','LiTo381-Modern/tests/native_slow29.rs')]:
  source=(candidate_path.parent/relative).read_text('utf-8-sig');needle='include_bytes!("fixtures/native-world-current.json")'
  require(source.count(needle)==1,'paired fixture include differs')
  target=out/(key+'-paired.rs');target.write_text(source.replace(needle,'include_bytes!(r#"'+fixture.as_posix()+'"#)'),encoding='utf-8',newline='\n');paired[key]=target
 (out/'compiler.py').write_bytes(Path(__file__).read_bytes());(out/'original-lib.rs').write_bytes(original_root.read_bytes());(out/'original-modern_lito.rs').write_bytes(original_mod.read_bytes())
 pins=dict(p['input_pins'])|{str(candidate_path):sha(candidate_path),str(plan_path):sha(plan_path),str(Path(__file__).resolve()):sha(__file__),
  str(fixture):sha(fixture),str(out/'lib.rs'):sha(out/'lib.rs'),str(out/'modern_lito.rs'):sha(out/'modern_lito.rs')}
 pins.update({str(candidate_path.parent/name):h for name,h in c['candidate_files'].items()});pins.update({str(path):sha(path) for path in paired.values()})
 inputs=read(p['base_compiled_inputs'])
 def guard():
  require(len(inputs)==1049 and all(sha(R/name)==h for name,h in inputs.items()),'normal source changed')
  check_pins(pins);check_pins(p['normal_product_pins'])
  require(sha(read(R/'工作佇列.json')['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 guard();record=dict(schema='E149-slow29-metadata-compile-v1',status='compiling_metadata_only',candidate=str(candidate_path),candidate_sha256=sha(candidate_path),
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[],owned_process=None,
  original_component_plan=str(plan_path),original_component_plan_sha256=sha(plan_path),input_pins=pins,external_toolchain_dependency_pins=p['external_toolchain_dependency_pins'],
  base_compiled_inputs=p['base_compiled_inputs'],normal_product_pins=p['normal_product_pins'],original_root=str(original_root),original_mod=str(original_mod),
  generated_root=str(out/'lib.rs'),generated_mod=str(out/'modern_lito.rs'),runtime_overrides=runtime,original_runtime_sources=runtime_sources,paired_tests={key:str(path) for key,path in paired.items()},
  tests_executed=0,PEs_executed=False,PEs_linked=False,normal_sources_modified=False,normal_products_modified=False,whole_goal_complete=False)
 def save():(out/'cohort.json').write_bytes(encoded(record))
 save();print(json.dumps(dict(run=str(out),scope='Debug/Release library, library-test and two integration metadata targets; no execution'),ensure_ascii=False),flush=True)
 environment={name:value for name,value in os.environ.items() if not name.startswith(('LITO_','RUST_')) and name not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']}
 package=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']
 environment.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=package['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'))
 try:
  for profile in ['Debug','Release']:
   library=products/('liblito_modern-'+profile+'.rmeta')
   for kind in ['library','library-test','blind20','slow29']:
    guard();command=list(p['compiler_templates'][profile]);require(command.count(p['roots']['candidate'])==1 and '--emit=metadata' in command and '--test' in command,'original metadata command differs')
    source=out/'lib.rs' if kind in ['library','library-test'] else paired[kind];command[command.index(p['roots']['candidate'])]=str(source)
    product=library if kind=='library' else products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind=='library':command.remove('--test');command+=['--crate-type=rlib']
    elif kind in paired:
     command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(library)]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
     child=subprocess.Popen(command,env=environment,cwd=products,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
     row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),exit_code=None,product=str(product))
     record['compiler_runs'].append(row);record['owned_process']=row['original_process'];save()
     try:child.wait(timeout=300)
     except BaseException:
      if child.poll() is None:
       require(identity(int(child._handle),child.pid)==row['original_process'],'original compiler handle identity changed');child.terminate();child.wait(timeout=20);row['exact_original_compiler_cleanup']=True
      raise
     finally:
      row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),logs={str(folder/name):sha(folder/name) for name in ['stdout.log','stderr.log']});record['owned_process']=None;save()
    require(child.returncode==0,'original metadata compiler failed; preserve source and diagnostics')
    row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();guard()
  record['status']='metadata_compilers_complete_no_tests'
 except BaseException:
  record['status']='compiler_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:record['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(run=str(out),compiler_exit_codes=[row['exit_code'] for row in record['compiler_runs']],tests_executed=0,PEs_linked=False),ensure_ascii=False))
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('candidate',type=Path);ap.add_argument('--component-plan',type=Path,required=True);args=ap.parse_args();compile_candidate(args.candidate,args.component_plan)
if __name__=='__main__':main()
