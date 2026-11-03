"""Bind a worker decode receipt to static S37; compile metadata only, no tests run."""
from pathlib import Path
from datetime import datetime,timezone
import json,os,re,subprocess,tomllib,traceback
from native_E149_recovery_contract import R,P,FORMAL,read,sha,encoded,require,check_pins
from native_E149_owned_process import identity,kernel
T=R/'維護工具';V=R/'驗證紀錄/current'

def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'source-only phase differs');ref=w['current_E149_hide97_static_draw_bridge_development'];require(sha(ref['evidence'])==ref['evidence_sha256'],'base evidence differs');q=read(ref['evidence']);old=read(q['compile_cohort']);require(sha(q['compile_cohort'])==q['compile_cohort_sha256'],'base metadata differs');base=read(old['candidate']);require(sha(old['candidate'])==old['candidate_sha256'],'base candidate differs');prior=read(q['previous_qualified_18_metadata']['qualification']);full=read(Path(prior['compile_run'])/'cohort.json');require(sha(Path(prior['compile_run'])/'cohort.json')==prior['compile_cohort_sha256'],'original paired graph differs');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal differs')
 stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');candidate=T/'candidates'/('native-E149-static-effect-ingress-'+stamp);candidate.mkdir(parents=True);out=V/'native-E149-static-effect-ingress-metadata'/stamp;out.mkdir(parents=True);products=P/'target/current/verification/native-E149-static-effect-ingress'/stamp;products.mkdir(parents=True);(products/'scratch').mkdir()
 inputs=read(old['base_compiled_inputs']);require(len(inputs)==1049,'normal count differs');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);modern=Path(old['generated_mod']).read_text('utf-8-sig');overrides={};pairs=dict(full['original_nested_runtime_pairs']);pins=dict(old['input_pins']);pins.update({str(R/n):h for n,h in inputs.items()})
 changes={
 'native_connection.rs':[
  ('        rx_counter: u64,\n    },\n}', '        rx_counter: u64,\n        receipt: IngressReceipt,\n    },\n}'),
  ('                        if let Some(body) = session.poll_body(&worker_cancel)? {\n                            emit_running(', '                        if let Some(body) = session.poll_body(&worker_cancel)? {\n                            let receipt = IngressReceipt::decoded(epoch, session.rx.counter());\n                            emit_running('),
  ('                                    rx_counter: session.rx.counter(),\n', '                                    rx_counter: session.rx.counter(),\n                                    receipt,\n')],
 'native_world.rs':[
  ('    pub effects: Vec<super::native_skills::Effect>,', '    pub effects: Vec<super::native_skills::Effect>,\n    /// Actual worker decode receipts for static2941, bounded by the effect ring.\n    #[serde(skip)]\n    pub static_effect_receipts: BTreeMap<u64, StaticEffectReceipt>,'),
  ('                    let retired=self.effects.remove(0);', '                    let retired=self.effects.remove(0);\n                    self.static_effect_receipts.remove(&retired.sequence);')],
 'native_login.rs':[
  ('    fn accept_game(&mut self, body: &[u8]) -> Result<()> {', '    fn accept_game(&mut self, body: &[u8]) -> Result<()> {\n        self.accept_game_received(body, None)\n    }\n    fn accept_game_received(&mut self, body: &[u8], ingress: Option<super::native_connection::IngressReceipt>) -> Result<()> {\n        if let Some(receipt) = ingress { ensure!(receipt.connection_epoch()==self.transport.snapshot().epoch, "world receipt connection retired"); }'),
  ('        if self.state.world.accept(body)? {', '        let world_accepted = match ingress {\n            Some(receipt) => self.state.world.accept_received(body, receipt)?,\n            None => self.state.world.accept(body)?,\n        };\n        if world_accepted {'),
  ('                Event::Packet { epoch, body, .. } => {', '                Event::Packet { epoch, body, rx_counter, receipt } => {'),
  ('                    if self.game {\n                        self.accept_game(&body)', '                    if receipt.connection_epoch()!=epoch || receipt.rx_counter()!=rx_counter {\n                        Err(anyhow::anyhow!("native packet receipt identity differs"))\n                    } else if self.game {\n                        self.accept_game_received(&body, Some(receipt))')],
 }
 templates={'native_connection.rs':['native_static_effect_ingress.rs','native_static_effect_ingress_tests.rs'],'native_world.rs':['native_world_static_effect_ingress.rs'],'native_login.rs':[]}
 for filename,edits in changes.items():
  name='LiTo381-Modern/src/modern_lito/'+filename;source=Path(paths[name])if name in paths else R/name;expected=files.get(name,inputs[name]);require(sha(source)==expected,'source predecessor differs');text=source.read_text('utf-8-sig')
  for a,b in edits:require(text.count(a)==1,'ambiguous source delta '+filename+' '+a);text=text.replace(a,b)
  supplements=[]
  for n in templates[filename]:
   p=T/'templates'/n;contents=p.read_text('utf-8-sig');text+='\n'+contents;supplements.append(dict(path=str(p),sha256=sha(p)));pins[str(p)]=sha(p)
  replacement=candidate/filename;replacement.write_text(text,encoding='utf-8',newline='\n');row=dict(predecessor=str(source),predecessor_sha256=expected,path=str(replacement),sha256=sha(replacement),changes=edits,supplements=supplements);overrides[name]=row;files[name]=sha(replacement);paths[name]=str(replacement);pins[str(replacement)]=sha(replacement)
  mapped=source
  if name in pairs:
   pair=pairs[name];require(pair['source']==str(source) and pair['source_sha256']==expected and sha(pair['path'])==pair['sha256'],'nested source predecessor differs');mapped=Path(pair['path'])
   for rw in pair['rewrites']:require(text.count(rw['original_attribute'])==1,'nested path differs');text=text.replace(rw['original_attribute'],rw['compiled_attribute']);pins[rw['resolved_path']]=sha(rw['resolved_path'])
   compiled=out/('paired-runtime-'+filename);compiled.write_text(text,encoding='utf-8',newline='\n');pins[str(compiled)]=sha(compiled);pairs[name]=dict(path=str(compiled),sha256=sha(compiled),source=str(replacement),source_sha256=sha(replacement),rewrites=pair['rewrites'],predecessor_pair_path=pair['path'],predecessor_pair_sha256=pair['sha256']);replacement=compiled
  needle=mapped.as_posix()if mapped.as_posix()in modern else str(mapped);require(modern.count(needle)==1,'compiled module mapping differs '+filename);modern=modern.replace(needle,replacement.as_posix())
 require(len(files)==43 and len(re.findall(r'#\[path = r#"',modern))==371 and sum(len(v['rewrites'])for v in pairs.values())==6,'graph inventory differs');(out/'modern_lito.rs').write_text(modern,encoding='utf-8',newline='\n');lib=Path(old['generated_root']).read_text('utf-8-sig');needle=Path(old['generated_mod']).as_posix();require(lib.count(needle)==1,'root mapping differs');(out/'lib.rs').write_text(lib.replace(needle,(out/'modern_lito.rs').as_posix()),encoding='utf-8',newline='\n');(out/'developer.py').write_bytes(Path(__file__).read_bytes())
 k=dict(schema='E149-static-effect-ingress-source-v1',predecessor_source=ref,predecessor_candidate=old['candidate'],predecessor_candidate_sha256=sha(old['candidate']),runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,overlay_count=43,runtime_files=29,Rust_QA_files=11,Java_files=3,module_count=371,original_nested_paths_preserved=6,new_tests_written=4,new_resource_opt_in_cases=0,previous_metadata_cases=459,network_decode_to_event_to_login_to_world_source_bound=True,static2941_cue_receipt_bound=True,clock_domain='std::time::Instant after full packet decode, before event backpressure',native_PE_clock_equivalence_qualified=False,scene_history_and_retirement_queue_integrated=False,production_applied=False,tests_executed=0,assets_decoded=False,whole_goal_complete=False);kp=candidate/'contract.json';kp.write_bytes(encoded(k));pins[str(kp)]=sha(kp)
 for p in [out/'modern_lito.rs',out/'lib.rs',out/'developer.py',Path(__file__).resolve(),Path(ref['evidence']),Path(old['candidate']),Path(q['compile_cohort']),Path(prior['compile_run'])/'cohort.json']:pins[str(p)]=sha(p)
 paired=dict(full['paired_tests']);paired['network_ingress']=str(P/'tests/native_network.rs');pins[paired['network_ingress']]=sha(paired['network_ingress']);external=dict(old['external_toolchain_dependency_pins']);state=dict(schema='E149-static-effect-ingress-metadata-v1',status='twenty_metadata_compiling',candidate=str(kp),candidate_sha256=sha(kp),input_pins=pins,external_toolchain_dependency_pins=external,normal_product_pins=old['normal_product_pins'],base_compiled_inputs=old['base_compiled_inputs'],generated_root=str(out/'lib.rs'),generated_mod=str(out/'modern_lito.rs'),original_nested_runtime_pairs=pairs,paired_tests=paired,pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[],tests_executed=0,outputs_executed=False,PEs_linked=False,whole_goal_complete=False)
 def save():(out/'cohort.json').write_bytes(encoded(state))
 def guard():
  latest=read(R/'工作佇列.json');require(latest['current_user_requested_goal']['test_execution_deferred_by_user'] and latest['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not latest['active_workloads'] and not latest['active_workload_sessions'],'phase changed');check_pins(pins);check_pins(old['normal_product_pins']);require(all(sha(p)==h for p,h in external.items()),'external dependency differs')
 save();print(json.dumps(dict(run=str(out),candidate=str(kp),scope='20 metadata commands; 4 new cases written, none executed'),ensure_ascii=False),flush=True);env={n:v for n,v in os.environ.items()if not n.startswith(('LITO_','RUST_'))and n not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']};env.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'))
 try:
  for profile in ['Debug','Release']:
   template=next(r['command']for r in old['compiler_runs']if r['profile']==profile and r['kind']=='library')
   for kind in ['library','library-test',*paired]:
    guard();command=list(template);require('--emit=metadata'in command and '--test'not in command,'metadata template differs');command[command.index(old['generated_root'])]=str(out/'lib.rs')if kind in ['library','library-test']else paired[kind];product=products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
    if kind in paired:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(products/(profile+'-library.rmeta'))]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb')as stdout,(folder/'stderr.log').open('xb')as stderr:
     child=subprocess.Popen(command,cwd=products,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),product=str(product),exit_code=None);state['compiler_runs'].append(row);save();row['exit_code']=child.wait();row['original_handle_signaled']=kernel().WaitForSingleObject(int(child._handle),0)==0
    row['logs']={str(folder/n):sha(folder/n)for n in ['stdout.log','stderr.log']};save();require(row['exit_code']==0 and row['original_handle_signaled'] and product.is_file(),'metadata compiler failed; no retry');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();print(profile+' '+kind+' metadata exit0',flush=True)
  guard();state['status']='twenty_metadata_complete_no_execution'
 except BaseException:
  state['status']='original_metadata_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(cohort=str(out/'cohort.json'),sha256=sha(out/'cohort.json'),candidate=str(kp),exit_codes=[r['exit_code']for r in state['compiler_runs']],tests_executed=0),ensure_ascii=False))

if __name__=='__main__':main()
