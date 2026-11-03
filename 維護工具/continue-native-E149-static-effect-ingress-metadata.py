"""Correct extern metadata basename; retain both successful original Debug compiles."""
from pathlib import Path
from datetime import datetime,timezone
import json,os,subprocess,tomllib,copy,traceback
from native_E149_recovery_contract import R,P,read,sha,encoded,require,check_pins
from native_E149_owned_process import identity,kernel
from native_E149_process_birth_readonly import original_terminal
T=R/'維護工具';V=R/'驗證紀錄/current'
CP=V/'native-E149-static-effect-ingress-metadata/20261024-201153-801200/cohort.json'

def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'source-only phase differs');c=read(CP);require(c['status']=='original_metadata_failure_preserved' and [(r['profile'],r['kind'],r['exit_code'])for r in c['compiler_runs']]==[('Debug','library',0),('Debug','library-test',0),('Debug','blind20',1)],'original failure differs');require(c['tests_executed']==0 and not c['PEs_linked'] and not c['outputs_executed'],'failure scope differs');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency differs')
 failed=c['compiler_runs'][2];diagnostic=Path(next(p for p in failed['logs']if p.endswith('stderr.log'))).read_text('utf-8-sig');require('extern location for lito_modern is of an unknown type'in diagnostic and 'file name should be lib*.rlib or *.dll'in diagnostic,'extern recipe failure differs')
 for row in c['compiler_runs']:
  require(row['original_handle_signaled'] and all(sha(p)==h for p,h in row['logs'].items()),'original compiler/log differs')
  if row['exit_code']==0:require(sha(row['product'])==row['product_sha256'],'successful original product differs')
 observations=[original_terminal(r)for r in [c['pipeline']]+[r['original_process']for r in c['compiler_runs']]];stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'native-E149-static-effect-ingress-metadata-continuation'/stamp;out.mkdir(parents=True);products=P/'target/current/verification/native-E149-static-effect-ingress-continuation'/stamp;products.mkdir(parents=True);(products/'scratch').mkdir();alias=products/'liblito_modern-Debug.rmeta';alias.write_bytes(Path(c['compiler_runs'][0]['product']).read_bytes());require(sha(alias)==c['compiler_runs'][0]['product_sha256'],'retained library alias differs')
 failure=dict(original_cohort=str(CP),original_cohort_sha256=sha(CP),observed_original_tool_chunk='62a862',observed_actual_exit_code=1,original_compiler_exit_codes=[0,0,1],original_process_observations=observations,diagnostic_cause='Extern library metadata basename lacked lib prefix; downstream type errors are not treated as a product source defect',retained_successful_debug_compiles=2,Debug_library_alias=str(alias),Debug_library_alias_sha256=sha(alias),source_and_QA_changes=0,tests_executed=0,PEs_linked=False,whole_goal_complete=False);fp=out/'original-failure.json';fp.write_bytes(encoded(failure));(out/'continuation.py').write_bytes(Path(__file__).read_bytes());pins=dict(c['input_pins']);pins.update({str(p):sha(p)for p in [CP,fp,out/'continuation.py',Path(__file__).resolve(),alias]})
 retained=copy.deepcopy(c['compiler_runs'][:2])
 for row in retained:row['retained_successful_original_compile']=True;row['retained_from_cohort']=str(CP)
 state={**{n:c[n]for n in ['candidate','candidate_sha256','normal_product_pins','base_compiled_inputs','generated_root','generated_mod','original_nested_runtime_pairs','paired_tests','external_toolchain_dependency_pins']},'schema':'E149-static-effect-ingress-metadata-continuation-v1','status':'continuing_eighteen_remaining_metadata_commands','input_pins':pins,'pipeline':identity(kernel().GetCurrentProcess(),os.getpid()),'compiler_runs':retained,'retained_original_successes':2,'remaining_compilers_started':0,'original_recipe_failure':str(fp),'original_recipe_failure_sha256':sha(fp),'retained_Debug_library_alias':str(alias),'retained_Debug_library_alias_sha256':sha(alias),'tests_executed':0,'outputs_executed':False,'PEs_linked':False,'whole_goal_complete':False}
 def save():(out/'cohort.json').write_bytes(encoded(state))
 def guard():
  latest=read(R/'工作佇列.json');require(latest['current_user_requested_goal']['test_execution_deferred_by_user'] and latest['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not latest['active_workloads'] and not latest['active_workload_sessions'],'phase changed');check_pins(pins);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'external dependency differs')
 save();print(json.dumps(dict(run=str(out),retained_original_debug_compiles=2,remaining_metadata_commands=18,tests_executed=0),ensure_ascii=False),flush=True);env={n:v for n,v in os.environ.items()if not n.startswith(('LITO_','RUST_'))and n not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']};env.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'));k=read(c['candidate']);previous=read(k['predecessor_source']['evidence']);templates=read(previous['compile_cohort'])
 try:
  for profile in ['Debug','Release']:
   template=next(r['command']for r in templates['compiler_runs']if r['profile']==profile and r['kind']=='library');library=alias if profile=='Debug'else products/'liblito_modern-Release.rmeta';kinds=list(c['paired_tests'])if profile=='Debug'else ['library','library-test',*c['paired_tests']]
   for kind in kinds:
    guard();command=list(template);require('--emit=metadata'in command and '--test'not in command,'metadata template differs');command[command.index(templates['generated_root'])]=c['generated_root']if kind in ['library','library-test']else c['paired_tests'][kind];product=library if kind=='library'else products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
    if kind in c['paired_tests']:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(library)]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb')as stdout,(folder/'stderr.log').open('xb')as stderr:
     child=subprocess.Popen(command,cwd=products,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),product=str(product),exit_code=None);state['compiler_runs'].append(row);state['remaining_compilers_started']+=1;save();row['exit_code']=child.wait();row['original_handle_signaled']=kernel().WaitForSingleObject(int(child._handle),0)==0
    row['logs']={str(folder/n):sha(folder/n)for n in ['stdout.log','stderr.log']};save();require(row['exit_code']==0 and row['original_handle_signaled'] and product.is_file(),'continuation metadata failed; no retry');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();print(profile+' '+kind+' metadata exit0',flush=True)
  guard();require(len(state['compiler_runs'])==20 and state['remaining_compilers_started']==18,'compiler count differs');state['status']='twenty_metadata_complete_two_retained_eighteen_new_no_execution'
 except BaseException:
  state['status']='continuation_failure_preserved';(out/'continuation-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(cohort=str(out/'cohort.json'),sha256=sha(out/'cohort.json'),exit_codes=[r['exit_code']for r in state['compiler_runs']],retained_original_debug_compiles=2,new_metadata_compilers=18,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':main()
