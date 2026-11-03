"""Retain the original fault protocol and replace synchronous model samples with a persistent Presentation."""
from pathlib import Path
import re,json,hashlib
ROOT=Path(__file__).resolve().parents[1]
src=ROOT/'LiTo381-Modern/tests/native_teleport_failure.rs';success=ROOT/'維護工具/candidates/native-teleport-presentation/probe.rs';d=ROOT/'維護工具/candidates/native-teleport-failure-presentation';d.mkdir(exist_ok=True)
original=src.read_text(encoding='utf-8');common=success.read_text(encoding='utf-8').split('fn player(',1)[0];s=original[original.index('fn failure_player('):]
s=s.replace('let mut c=LoginController::default();let cancel=AtomicBool::new(false);let start=Instant::now();','let mut p=Peer::default();').replace('wait(&mut c,','wait(&mut p,')
s=re.sub(r'\bc\.', 'p.c.',s).replace('p.c.poll();','p.tick()?;')
start=s.index('  let mut model=Model::load(');end=s.index('\n  p.c.rental_npc_talk',start);s=s[:start]+'  let npc=1812987996;let steps=p.walk(npc)?;'+s[end:]
needle='  let before=p.c.snapshot().clone();let old_epoch=p.c.transport().snapshot().epoch;';assert needle in s
s=s.replace(needle,'''  p.ready(8)?;
  let before=p.c.snapshot().clone();let old_epoch=p.c.transport().snapshot().epoch;
  let old_surface=p.view.frame.as_ref().unwrap().input_surface(&before.world).context("source input surface")?;
  let old_cues=p.view.frame.as_ref().unwrap().native_audio.clone();
  let source_picture=p.picture(root,role,"source")?;''')
needle='  if cancel_expiry||cancel_retry {\n   ensure!(failed.phase';assert needle in s
s=s.replace(needle,'''  let retirement_start=Instant::now();
  while !p.view.quiescent() {p.tick()?;ensure!(retirement_start.elapsed()<Duration::from_secs(3),"existing3s Presentation retirement bound");thread::sleep(Duration::from_millis(10));}
  ensure!(p.view.frame.is_none()&&!old_surface.input_ready(&failed.world),"failed destination retained published source surface");
  let delivered=p.sink.played.len();let scope=p.view.audio_scope(failed_epoch,failed.world.generation);let mut old_cue_outcomes=vec![];
  for cue in &old_cues {old_cue_outcomes.push(format!("{:?}",cue.submit(&scope,&mut p.sink)?));}
  ensure!(p.sink.played.len()==delivered,"retired source audio delivered");
  let failure_retirement=json!({"frame_absent":p.view.frame.is_none(),"view_quiescent":p.view.quiescent(),"old_surface_rejected":true,"retirement_ms":retirement_start.elapsed().as_millis(),"retained_source_cues":old_cues.len(),"cue_outcomes":old_cue_outcomes,"audio_deliveries_before":delivered,"audio_deliveries_after":p.sink.played.len()});
  drop(old_cues);
  if cancel_expiry||cancel_retry {
   ensure!(failed.phase''')
needle='p.c.snapshot().inventory.owner.is_none(),"cancelled handoff resumed in background"';assert needle in s
s=s.replace(needle,'p.c.snapshot().inventory.owner.is_none()&&p.view.frame.is_none()&&p.view.quiescent(),"cancelled handoff resumed in background"')
start=s.index('  let mut recovered_model=Model::load(');end=s.index('  let nav=recovered_model.navigation();',start)
s=s[:start]+'''  p.wait(20,|p|p.view.frame.as_ref().is_some_and(|f|f.input_ready(&p.c.snapshot().world)))?;
  p.ready(8)?;
  ensure!(!old_surface.input_ready(&p.c.snapshot().world),"source surface authorized recovered map500 input");
'''+s[end:];s=s.replace('let nav=recovered_model.navigation();','let nav=p.view.frame.as_ref().unwrap().navigation.clone();')
start=s.index('  let end=Instant::now()+Duration::from_secs(8);loop{p.tick()?;let frame=recovered_model.draw_at');end=s.index('  std::fs::write(root.join(format!("{role}-failure.json"))',start)
s=s[:start]+'''  p.ready(8)?;
  let recovered_picture=p.picture(root,role,"recovered")?;
  let presentation_recovered_snapshot=p.c.snapshot().clone();
'''+s[end:]
needle='"full_game_reentry_verified":true';assert needle in s
s=s.replace(needle,needle+',"source_picture":source_picture,"failure_retirement":failure_retirement,"recovered_picture":recovered_picture,"presentation_recovered_snapshot":presentation_recovered_snapshot,"actual_Presentation_executed":true,"presentation_trace":p.trace,"old_surface_rejected_after_reentry":true,"GUI_acceptance":false,"physical_audio_verified":false')
start=s.rindex(' p.c.cancel();let end=Instant::now()+Duration::from_secs(3);');end=s.index('\n}',start)
s=s[:start]+''' std::fs::write(root.join(format!("{role}-presentation-journal.json")),serde_json::to_vec_pretty(&json!({"scenario":scenario,"error":result.as_ref().err().map(|e|format!("{e:#}")),"trace":p.trace,"played":p.sink.played,"GUI_acceptance":false}))?)?;
 let closed=p.close();
 std::fs::write(root.join(format!("{role}-presentation-shutdown.json")),serde_json::to_vec_pretty(&json!({"client_quiescent":p.c.is_quiescent(),"Presentation_quiescent":p.view.quiescent(),"shutdown_error":closed.as_ref().err().map(|e|format!("{e:#}"))}))?)?;
 result?;closed'''+s[end:]
s=s.replace('std::env::var("LITO_TELEPORT_REFERENCE_SHA").unwrap_or_else(|_|"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43".into())','std::env::var("LITO_TELEPORT_REFERENCE_SHA").expect("owned current C02 reference")')
s=s.replace('assert_eq!(ready["jar_sha256"],expected);','assert_eq!(ready["jar_sha256"],expected);assert_eq!(expected.to_uppercase(),"C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539");')
assert 'Model::' not in s and 'recovered_model' not in s and 'AtomicBool' not in s and not re.search(r'(?<!p\.)\bc\.',s)
helper='''
use m::{native_login::{LoginPhase as P,Snapshot},native_connection::Limits,native_world};
fn wait(p:&mut Peer,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()>{p.wait(seconds,|p|ok(p.c.snapshot()))}
'''
assert not (d/'probe.rs').exists(),'retain current candidate before replacing it'
(d/'original_failure.rs').write_bytes(src.read_bytes());(d/'probe.rs').write_text(common+helper+s,encoding='utf-8',newline='\n');(d/'generator.py').write_bytes(Path(__file__).read_bytes())
prep=dict(status='prepared_actual_Presentation_failure_recovery_candidate',base_source_manifest_sha256='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182',original_test_sha256=hashlib.sha256(src.read_bytes()).hexdigest().upper(),common_Peer_prefix_sha256=hashlib.sha256(common.encode()).hexdigest().upper(),scenario_names=['timeout','refused','wrong-service','cancel-retry','cancel-expiry'],product_modified=False,compiled=False,executed=False,original_controller_and_natural_expiry_bounds_preserved=True,GUI_verified=False,whole_goal_complete=False)
(d/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(d)
