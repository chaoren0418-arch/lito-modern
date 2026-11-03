"""Eighteen metadata-only commands: S27 ordered-body source component only, not yet World/GUI integrated."""
from pathlib import Path
from datetime import datetime, timezone
import json, os, re, subprocess, tomllib, traceback
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_owned_process import identity,kernel

T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-appearance27-queue-20261024-141000'
def stamp():return datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
def compile_core():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'development-only idle phase expected')
 contract=read(C/'contract.json');require(contract['base_source_manifest_sha256']==SOURCE and contract['formal_target_sha256']==FORMAL,'identity differs')
 require(sha(contract['predecessor_metadata_cohort'])==contract['predecessor_metadata_cohort_sha256'],'predecessor compilation changed');old=read(contract['predecessor_metadata_cohort'])
 require(sha(contract['integration_predecessor_cohort'])==contract['integration_predecessor_cohort_sha256'],'integration predecessor differs');paired=read(contract['integration_predecessor_cohort'])['paired_tests'];require(set(paired)=={'blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'},'retained integration set differs')
 pins=dict(old['input_pins']);external=dict(old['external_toolchain_dependency_pins'])
 proof=read(contract['inspection']);require(sha(contract['inspection'])==contract['inspection_sha256'],'inspection changed')
 for p,h in proof['source_pins'].items():
  if Path(p).is_relative_to(R):pins[p]=h
  else:external[p]=h
 pins.update({row['path']:row['sha256'] for row in contract['runtime_overrides'].values()})
 pins[str(T/'prepare-native-E149-appearance27-queue-tools.py')]=sha(T/'prepare-native-E149-appearance27-queue-tools.py')
 for p,h in contract['original_pins'].items():
  if Path(p).is_relative_to(R):pins[p]=h
  else:external[p]=h
 pins.update({row['path']:row['sha256'] for row in contract['modules'].values()})
 for p in [Path(contract['integration_predecessor_cohort']),T/'templates/native_appearance27_queue.rs',*map(Path,paired.values()),C/'contract.json',Path(contract['inspection']),Path(__file__).resolve(),T/'prepare-native-E149-appearance27-queue.py',T/'native-E149-appearance27-queue.md',Path(contract['predecessor_metadata_cohort']),Path(contract['predecessor_candidate']),Path(contract['predecessor_qualification'])]:pins[str(p)]=sha(p)
 inputs=read(old['base_compiled_inputs']);require(len(inputs)==1049,'normal input count differs');pins.update({str(R/n):h for n,h in inputs.items()})
 out=owned(V/'native-E149-appearance27-queue-metadata'/stamp());out.mkdir(parents=True)
 products=owned(P/'target/current/verification/native-E149-appearance27-queue-metadata'/out.name);products.mkdir(parents=True);(products/'scratch').mkdir()
 modern=Path(old['generated_mod']).read_text('utf-8-sig');require(len(re.findall(r'#\[path = r#"',modern))==367,'predecessor module count differs')
 require(all('mod '+name+';' not in modern for name in contract['modules']),'module name collision')
 for name,row in contract['runtime_overrides'].items():
  require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor changed')
  needles=list(dict.fromkeys([row['predecessor'],Path(row['predecessor']).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'runtime predecessor mapping differs');modern=modern.replace(found[0],Path(row['path']).as_posix())
 for name,row in contract['modules'].items():modern+='\n#[path = r#"'+Path(row['path']).as_posix()+'"#]\npub(crate) mod '+name+';\n'
 require(len(re.findall(r'#\[path = r#"',modern))==368,'supplementary module count differs')
 (out/'modern_lito.rs').write_text(modern,encoding='utf-8',newline='\n')
 lib=Path(old['generated_root']).read_text('utf-8-sig');needle=Path(old['generated_mod']).as_posix();require(lib.count(needle)==1,'root module differs');(out/'lib.rs').write_text(lib.replace(needle,(out/'modern_lito.rs').as_posix()),encoding='utf-8',newline='\n')
 (out/'compiler.py').write_bytes(Path(__file__).read_bytes())
 for p in [out/'lib.rs',out/'modern_lito.rs',out/'compiler.py']:pins[str(p)]=sha(p)
 def guard():
  latest=read(R/'工作佇列.json');require(latest['current_user_requested_goal']['test_execution_deferred_by_user'] and not latest['active_workloads'] and not latest['active_workload_sessions'],'phase/workload changed');check_pins(pins);check_pins(old['normal_product_pins']);require(all(sha(p)==h for p,h in external.items()),'readonly dependency changed')
 guard()
 state=dict(schema='E149-appearance27-queue-metadata-v1',status='compiling_metadata',contract=str(C/'contract.json'),contract_sha256=sha(C/'contract.json'),input_pins=pins,external_toolchain_dependency_pins=external,
  normal_product_pins=old['normal_product_pins'],base_compiled_inputs=old['base_compiled_inputs'],predecessor_cohort=contract['predecessor_metadata_cohort'],predecessor_cohort_sha256=sha(contract['predecessor_metadata_cohort']),
  generated_root=str(out/'lib.rs'),generated_mod=str(out/'modern_lito.rs'),module_count=368,paired_tests=paired,original_nested_runtime_pairs=old['original_nested_runtime_pairs'],
  historical_compiler_failure=old.get('original_tooling_failure'),pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[],owned_process=None,tests_executed=0,PEs_linked=False,outputs_executed=False,scene_integrated=True,whole_goal_complete=False)
 def save():(out/'cohort.json').write_bytes(encoded(state))
 save();print(json.dumps(dict(run=str(out),scope='eighteen metadata commands only; no test execution'),ensure_ascii=False),flush=True)
 env={k:v for k,v in os.environ.items() if not k.startswith(('LITO_','RUST_')) and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']};env.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'))
 try:
  for profile in ['Debug','Release']:
   template=next(row['command'] for row in old['compiler_runs'] if row['profile']==profile and row['kind']=='library')
   for kind in ['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448']:
    guard();command=list(template);require('--emit=metadata' in command and '--test' not in command,'metadata template differs');command[command.index(old['generated_root'])]=str(out/'lib.rs') if kind in ['library','library-test'] else paired[kind];product=products/('liblito_modern-'+profile+'.rmeta') if kind=='library' else products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
    if kind in paired:
     command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(products/('liblito_modern-'+profile+'.rmeta'))]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
     child=subprocess.Popen(command,cwd=products,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
     row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),product=str(product),exit_code=None);state['compiler_runs'].append(row);state['owned_process']=row['original_process'];save()
     try:child.wait(timeout=300)
     except BaseException:
      if child.poll() is None:require(identity(int(child._handle),child.pid)==row['original_process'],'compiler identity changed');child.terminate();child.wait(timeout=20);row['original_handle_cleanup']=True
      raise
     finally:row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),logs={str(folder/n):sha(folder/n) for n in ['stdout.log','stderr.log']});state['owned_process']=None;save()
    require(child.returncode==0,'metadata compiler failure; preserve original diagnostics');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();guard()
  state['status']='eighteen_metadata_compilers_complete_no_tests'
 except BaseException:state['status']='original_compile_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(run=str(out),exit_codes=[r['exit_code'] for r in state['compiler_runs']],tests_executed=0),ensure_ascii=False))

if __name__=='__main__':compile_core()
