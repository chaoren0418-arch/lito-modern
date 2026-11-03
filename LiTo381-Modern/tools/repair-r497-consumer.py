from r491_common import *
import shutil
O=P/'target/r497-ui-entry-study/attempt1';O.mkdir(exist_ok=False)
names=['r497-consumer-candidate.json','r497-consumer-exit.json','r497-consumer.stdout.log','r497-consumer.stderr.log','r497-event-partial.json','r497-source-oracle.json']
for n in names:shutil.copyfile(L/n,O/n)
p=P/'tools/r497-ui-entry-consumer.rs';shutil.copyfile(p,O/p.name)
save(L/'r497-preparation-history.json',dict(consumer_compile_failures=0,consumer_test_failures=1,failure='testexpectedmode1 but pinned206 fixture actualmode3;production opened3 correctly',failure_executable={k:read(O/'r497-consumer-candidate.json')[k]for k in ['path','bytes','sha256']},failure_artifacts=[pin(O/n)for n in names]+[pin(O/p.name)],already_verified_flows=4,correction='resume remaining6flows;first4 assertion batches notrerun. Fresh Runtime reconstruction setup only usesactualsource bytes/typed intent;production source andfixedlibrary unchanged.',production_build_failures=0))
s=p.read_text(encoding='utf-8');start=s.index('fn main()->anyhow::Result<()>');resume=s.index(' rt.feed_source_stream_candidate(&e,&bytes(&v["mode206_wire"]))?',start)
prefix='''fn main()->anyhow::Result<()>{let start=Instant::now();let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let v:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r497-source-oracle.json"))?)?;let mut rt=Runtime::start_with_menu_skin(inputs(&p,"r486-map-binding.json"),inputs(&p,"r497-native-menu-skin-binding.json"))?;wait(||{rt.poll();rt.page()==UiPage::Menu});let mut trace:Vec<Value>=serde_json::from_slice(&fs::read(p.join("target/r497-ui-entry-study/attempt1/r497-event-partial.json"))?)?;assert_eq!(trace.len(),4);for item in &mut trace{item["execution"]=json!(1);}let mut idle=500;let mut gate=Gate::default();changed(&rt,&mut gate,true);
 //Reconstruct authority forremainingflows;do notrerun firstfourassertion batches.
 let e=rt.begin_source_connection_candidate()?;rt.feed_source_stream_candidate(&e,&bytes(&v["setup_wire"]))?;let control=rt.source_character_control(0).unwrap();rt.handle_source_character_control(&control)?;let ticket=rt.source_pending_write_ticket(&e)?.unwrap();let length=rt.source_write_view(&e,&ticket)?.bytes()?.len();rt.report_source_write_candidate(&e,&ticket,0,WriteOutcome::Written(length))?;changed(&rt,&mut gate,true);let selected=rt.source_identity().selected().cloned();
'''
s=s[:start]+prefix+s[resume:];s=s.replace('Filter::Mode{mode:1,alternate:false}','Filter::Mode{mode:3,alternate:false}',1).replace('assert!(rt.source_identity().selected().is_none());','assert_eq!(rt.source_identity().selected(),selected.as_ref());',1)
s=s.replace('"flow_cases":trace.len(),','"flow_cases":trace.len(),"current_execution_new_flows":6,"previous_failed_execution_verified_flows":4,"consumer_executions":2,"consumer_test_failures":1,"firstfourassertion_batches_rerun":false,')
s=s.replace('"trace":trace,','"trace":trace,"execution_trace_note":"first4cases frompinnedfailedattempt1;last6fromresumeexecution2",')
p.write_text(s,encoding='utf-8')
ps=P/'tools/build-r497-ui-entry-consumer.ps1';script=ps.read_text(encoding='utf-8').replace('target\\r497-fixed-event-consumer','target\\r497b-fixed-event-consumer');ps.write_text(script,encoding='utf-8')
print('R497 failure archived;resume6remainingflows,first4passedbatchesnotrerun;production/libraryunchanged')
