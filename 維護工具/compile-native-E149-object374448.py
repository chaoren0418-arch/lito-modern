"""Eighteen metadata-only compiler commands for actor spells 37, 44 and 48."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,os,re,subprocess,tomllib,traceback
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_owned_process import identity,kernel
T=R/'維護工具';V=R/'驗證紀錄/current'
# Compiler loop derived from develop-native-E149-magical73.py SHA256 D3CD42F1CF93EB6C346FC72170BE75B2F1FDD8B73771965378442C6B98F01D65
def stamp():return datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
def idle():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'workload/source freeze active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only phase expected')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed');return w
def compile_candidate(cp):
 w=idle();cp=owned(cp.resolve(strict=True));candidate=read(cp);old=read(candidate['original_metadata_cohort'])
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'candidate identity differs')
 baseline_ref=w['current_development_typecheck_qualification'];require(sha(baseline_ref['qualification'])==baseline_ref['qualification_sha256'],'normal metadata baseline changed');baseline=read(baseline_ref['qualification']);require(baseline['source_manifest_sha256']==SOURCE and baseline['normal_product_pins']==old['normal_product_pins'],'normal baseline identity differs');base_inputs=Path(baseline['run'])/'base-compiled-inputs.json';require(len(read(base_inputs))==1049,'normal input count differs')
 pins=dict(old['input_pins']);pins.update({str(R/n):h for n,h in read(base_inputs).items()});pins[str(base_inputs)]=sha(base_inputs);pins[baseline_ref['qualification']]=baseline_ref['qualification_sha256']
 pins.update({candidate['effective_overlay_paths'][n]:h for n,h in candidate['effective_overlay_files'].items()})
 contract=read(candidate['contract']);owned_original={p:h for p,h in contract['original_pins'].items() if Path(p).is_relative_to(R)};external=dict(old['external_toolchain_dependency_pins']);check_pins(owned_original);pins.update(owned_original)
 readonly_original={p:h for p,h in contract['original_pins'].items() if p not in owned_original};require(readonly_original=={str(Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr')):'523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127'},'unreviewed external source');external.update(readonly_original)
 for p in [cp,Path(candidate['base_overlay']),Path(candidate['contract']),Path(candidate['original_metadata_cohort']),Path(__file__).resolve(),T/'templates/native_object374448.rs',T/'native-E149-object374448.md']:pins[str(p)]=sha(p)
 out=owned(V/'native-E149-object374448-metadata'/stamp());out.mkdir(parents=True);products=owned(P/'target/current/verification/native-E149-object374448-metadata'/out.name);products.mkdir(parents=True);(products/'scratch').mkdir()
 require(sha(candidate['base_overlay'])==candidate['base_overlay_sha256'],'base overlay changed')
 modern=Path(old['generated_mod']).read_text('utf-8-sig');paired_runtime={}
 controller='LiTo381-Modern/src/modern_lito/native_login.rs'
 prior=w['current_native_item_spells_development']['paired_runtime_sources'][controller]
 require(sha(prior['path'])==prior['sha256'] and sha(prior['source'])==prior['source_sha256']==candidate['base_files'][controller],'paired controller predecessor changed')
 raw=Path(prior['source']).read_text('utf-8-sig');restored=Path(prior['path']).read_text('utf-8-sig')
 for rewrite in prior['rewrites']:
  require(restored.count(rewrite['compiled_attribute'])==1,'nested predecessor differs');restored=restored.replace(rewrite['compiled_attribute'],rewrite['original_attribute'])
 require(restored==raw and len(prior['rewrites'])==5,'paired predecessor differs beyond nested paths')
 newraw=(cp.parent/controller).read_text('utf-8-sig');compiled=newraw
 for rewrite in prior['rewrites']:
  require(compiled.count(rewrite['original_attribute'])==1,'new nested path differs');compiled=compiled.replace(rewrite['original_attribute'],rewrite['compiled_attribute'])
 pair=out/'paired-runtime-native_login.rs';pair.write_text(compiled,encoding='utf-8',newline='\n')
 paired_runtime[controller]=dict(path=str(pair),sha256=sha(pair),source=str(cp.parent/controller),source_sha256=sha(cp.parent/controller),rewrites=prior['rewrites'],predecessor=prior)
 pins[str(pair)]=sha(pair)
 for n in candidate['changed_runtime_files']:
  previous=candidate['base_overlay'];base=read(previous)['candidate_paths'][n]
  require(candidate['base_files'][n]==sha(base),'raw runtime predecessor differs')
  oldpath=Path(prior['path']) if n==controller else Path(base);newpath=pair if n==controller else cp.parent/n
  require(modern.count(oldpath.as_posix())==1,'full module predecessor differs')
  modern=modern.replace(oldpath.as_posix(),newpath.as_posix())
 require(len(re.findall(r'#\[path = r#"',modern))==362,'complete module count differs')
 (out/'modern_lito.rs').write_text(modern,encoding='utf-8',newline='\n');lib=Path(old['generated_root']).read_text('utf-8-sig');needle=Path(old['generated_mod']).as_posix();require(lib.count(needle)==1,'library root differs');(out/'lib.rs').write_text(lib.replace(needle,(out/'modern_lito.rs').as_posix()),encoding='utf-8',newline='\n')
 paired=dict(old['paired_tests']);fixture=P/'tests/fixtures/native-world-current.json'
 for kind,name in [('object374448','native_object374448.rs')]:
  source=cp.parent/'LiTo381-Modern/tests'/name;text=source.read_text('utf-8-sig');needle='include_bytes!("fixtures/native-world-current.json")';require(text.count(needle)==1,'QA fixture path differs');p=out/(kind+'-paired.rs');p.write_text(text.replace(needle,'include_bytes!(r#"'+fixture.as_posix()+'"#)'),encoding='utf-8',newline='\n');paired[kind]=str(p)
 for p in [out/'lib.rs',out/'modern_lito.rs',*map(Path,paired.values())]:pins[str(p)]=sha(p)
 (out/'developer.py').write_bytes(Path(__file__).read_bytes())
 def guard():
  idle();check_pins(pins);check_pins(old['normal_product_pins']);require(all(sha(p)==h for p,h in external.items()),'compiler dependencies or read-only original source changed')
 guard();state=dict(schema='E149-object374448-metadata-v1',status='compiling_metadata',candidate=str(cp),candidate_sha256=sha(cp),input_pins=pins,
  original_metadata_cohort=candidate['original_metadata_cohort'],original_metadata_cohort_sha256=sha(candidate['original_metadata_cohort']),normal_product_pins=old['normal_product_pins'],
  external_toolchain_dependency_pins=external,base_compiled_inputs=str(base_inputs),generated_root=str(out/'lib.rs'),generated_mod=str(out/'modern_lito.rs'),paired_tests=paired,paired_runtime_sources=paired_runtime,
  pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[],owned_process=None,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False)
 def save():(out/'cohort.json').write_bytes(encoded(state))
 save();print(json.dumps(dict(run=str(out),scope='eighteen metadata compiler commands; no test execution'),ensure_ascii=False),flush=True)
 env={k:v for k,v in os.environ.items() if not k.startswith(('LITO_','RUST_')) and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']};env.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'))
 try:
  for profile in ['Debug','Release']:
   library=products/('liblito_modern-'+profile+'.rmeta');template=next(r['command'] for r in old['compiler_runs'] if r['profile']==profile and r['kind']=='library')
   for kind in ['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448']:
    guard();command=list(template);require('--emit=metadata' in command and '--test' not in command,'library template differs');command[command.index(old['generated_root'])]=str(out/'lib.rs') if kind in ['library','library-test'] else paired[kind];product=library if kind=='library' else products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
    if kind in paired:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(library)]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
     child=subprocess.Popen(command,cwd=products,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),product=str(product),exit_code=None);state['compiler_runs'].append(row);state['owned_process']=row['original_process'];save()
     try:child.wait(timeout=300)
     except BaseException:
      if child.poll() is None:require(identity(int(child._handle),child.pid)==row['original_process'],'compiler handle changed');child.terminate();child.wait(timeout=20);row['original_handle_cleanup']=True
      raise
     finally:row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),logs={str(folder/n):sha(folder/n) for n in ['stdout.log','stderr.log']});state['owned_process']=None;save()
    require(child.returncode==0,'metadata compiler failed; preserve original source and diagnostics');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();guard()
  state['status']='eighteen_metadata_compilers_complete_no_tests'
 except BaseException:state['status']='original_compile_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(run=str(out),exit_codes=[r['exit_code'] for r in state['compiler_runs']],tests_executed=0),ensure_ascii=False))
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('candidate',type=Path);args=parser.parse_args();compile_candidate(args.candidate)
