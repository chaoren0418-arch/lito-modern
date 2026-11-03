"""Stage native class4/ice callback clocks, typed recurring audio and retained histories."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';PRE='LiTo381-Modern/src/modern_lito/';ASSETS=PRE+'light_arrow_effect.rs';SPELL=PRE+'native_spell_projectiles.rs';SCENE=PRE+'native_world_scene.rs';VIEW=PRE+'native_world_view.rs';AUDIO=PRE+'native_audio_delivery.rs'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_S23_history_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous source changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'candidate changed');base=read(q['candidate']);require(sha(base['contract'])==base['contract_sha256'],'previous contract changed');prior=read(base['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==28 and VIEW not in files,'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 analysis=Path(prior['base_reconstruction']);require(sha(analysis)==prior['base_reconstruction_sha256'],'original callback proof changed');a=read(analysis);require(a['class4_advances_flight_before_secondary'] and a['class5_starts_secondary_before_flight'] and not a['native_instructions_executed'],'original branch scope differs')
 templates={ASSETS:T/'templates/native_S23_clock_tests.rs',SPELL:T/'templates/native_S23_clock_origin_integration_tests.rs',AUDIO:T/'templates/native_S23_latch_tests.rs'};qa={n:p.read_text('utf-8-sig') for n,p in templates.items()};require([len(re.findall(r'#\[test\]',qa[n])) for n in [ASSETS,SPELL,AUDIO]]==[7,2,3] and qa[SPELL].count('#[ignore=')==1,'QA scope differs')
 s=Path(paths[SPELL]).read_text('utf-8-sig');old_update=s[s.index('        let mut impacts = Vec::new();'):s.index('    pub fn draw(')];start=s.index('        let mut rows = Vec::new();');end=s.index('\n#[cfg(test)]');old_draw=s[start:end];require(old_draw.rstrip().endswith('}'),'draw boundary differs')
 changes={ASSETS:[
 ('// Literal threshold branches from4B9210 after converting native dx/2 to world.',(T/'templates/native_S23_clock_core.rs').read_text('utf-8-sig')+'\n// Literal threshold branches from4B9210 after converting native dx/2 to world.'),
 ('    pub(crate) fn native_area_fireball_impact_clip(&self)',(T/'templates/native_S23_clock_assets.rs').read_text('utf-8-sig')+'    pub(crate) fn native_area_fireball_impact_clip(&self)')],SPELL:[
 ('LightArrowAssets, LightArrowPath, LightArrowPlayback','LightArrowAssets, LightArrowPath, NativeSpellClock, NativeSpellSample'),
 ('    playback: LightArrowPlayback,','    playback: NativeSpellClock,\n    sample:Option<NativeSpellSample>,\n    finished:bool,'),
 ('path::Path, sync::atomic::AtomicBool, time::Duration','path::Path, sync::atomic::AtomicBool'),
 ('impl History {','impl History {\n    pub(crate) fn active_sequences(&self)->BTreeSet<u64>{self.flights.iter().filter(|f|!f.finished).map(|f|f.cue.action_sequence).collect()}'),
 ('    omitted: u64,\n}','    omitted: u64,\n    events:Vec<EnteredEvent>,non_audio:Vec<serde_json::Value>,clock_errors:Vec<serde_json::Value>,\n}'),
 ('                    let playback=assets.start_effect(&path);\n                    let playback=if secondary {playback}else{playback.without_secondary()};','                    let Ok(playback)=assets.start_native_S23(&path,secondary)else{self.omitted+=1;continue;};'),
 ('                        playback,\n                        arrived:','                        playback,sample:None,finished:false,\n                        arrived:'),
 (old_update,(T/'templates/native_S23_clock_origin_update.rs').read_text('utf-8-sig')+'    }\n'),
 (old_draw,(T/'templates/native_S23_clock_origin_draw.rs').read_text('utf-8-sig')+'    }\n}\n')],SCENE:[
 ('    pub(crate) fn for_source(&self,same_source:bool)->Self {','    pub(crate) fn active_spell_sequences(&self)->std::collections::BTreeSet<u64>{if self.restore_other_effects{self.spell_projectiles.active_sequences()}else{Default::default()}}\n    pub(crate) fn for_source(&self,same_source:bool)->Self {')],VIEW:[
 ('        self.finite_audio_history.retain_sequences(&retained);','        if let Some((owner,_,history))=&self.finite_visual_history {if self.finite_audio_history.is_world((owner.connection,owner.generation,owner.map)){retained.extend(history.active_spell_sequences());}}\n        self.finite_audio_history.retain_sequences(&retained);'),
 ('chain(world.projectiles.iter().map(|e|e.action_sequence)).collect(),environment:', 'chain(world.projectiles.iter().map(|e|e.action_sequence)).chain(self.finite_visual_history.as_ref().filter(|(owner,_,_)|*owner==key).into_iter().flat_map(|(_,_,h)|h.active_spell_sequences().into_iter())).collect(),environment:')]}
 changes[SPELL].insert(0,('pub fn supports_graphics(graphics:u16)->bool', 'fn native_clock_elapsed_ns(now:u64,start:u64)->u64 {(now/1_000_000).saturating_sub(start/1_000_000)*1_000_000}\n\npub fn supports_graphics(graphics:u16)->bool'))
 changes[ASSETS].insert(0,('//! Its one-shot animation holds terminal0 while flight continues. 219 is\n//! removed after its1..N-1 timeline. Sound97 starts once on launch, including\n//! adjacent targets whose native flight reaches219 immediately. No homing,\n//! arbitrary velocity, guessed sprites, recoloring, or new geometry collision.',
 '//! Native S23 flight cycles source1..N-1,0 on60ms movement callbacks.\n//! 219 secondary includes the native source0 terminal dwell. The older\n//! rational helpers are retained for inspection; NativeSpellClock drives\n//! runtime S23. Sound97 may re-enter each flight cycle, subject to same-ID\n//! suppression. No homing or local combat authority.'))
 changes[AUDIO]=[
  ('seen:std::collections::BTreeSet<(FiniteEventKey,Option<u64>)>,','seen:std::collections::BTreeMap<(FiniteEventKey,Option<u64>),std::sync::Weak<Payload>>,'),
  ('self.seen.retain(|(event,_)|sequences.contains(&event.sequence));','self.seen.retain(|(event,_),_|sequences.contains(&event.sequence));'),
  ('!self.seen.contains(*key)','!self.seen.contains_key(*key)'),
  ('            if self.seen.insert(key) {report.kept+=1;kept.push(cue);}'+'\n'+'            else {report.duplicates+=1;cue.discard();}',
   '            match self.seen.entry(key) {'+'\n'+
   '                std::collections::btree_map::Entry::Vacant(entry)=>{entry.insert(Arc::downgrade(&cue.0));report.kept+=1;kept.push(cue);},'+'\n'+
   '                std::collections::btree_map::Entry::Occupied(entry)=>{report.duplicates+=1;'+
   'if !entry.get().ptr_eq(&Arc::downgrade(&cue.0)){cue.discard();}}'+'\n'+
   '            }')]
 out=owned(T/'candidates'/('native-E149-S23-clock-origin-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);(out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  path=Path(paths.get(name,str(R/name)));text=path.read_text('utf-8-sig');before[name]=sha(path)
  for old,new in replacements:require(text.count(old)==1,'source anchor differs: '+old[:160]);text=text.replace(old,new)
  text+=qa.get(name,'');target=out/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf-8',newline='\n');changed[name]=sha(target);files[name]=sha(target);paths[name]=str(target)
 contract=dict(schema='E149-S23-native-clock-source-v1',formal_target_sha256=FORMAL,source_changes=changes,QA_templates={n:dict(path=str(p),sha256=sha(p)) for n,p in templates.items()},native_snapshot_contract=prior['native_snapshot_contract'],native_snapshot_contract_sha256=prior['native_snapshot_contract_sha256'],original_pins=prior['original_pins'],base_reconstruction=str(analysis),base_reconstruction_sha256=sha(analysis),profiles=[167,1583,1799,1797],movement_ms=60,class4_flight_before_secondary=True,class5_secondary_before_flight=True,flight_cycles_per_movement=True,secondary_native_terminal_dwell=True,typed_audio_occurrence_identity=True,native_global_millisecond_origin=True,pending_shared_latch_preserved=True,weak_history_owns_no_payload=True,active_audio_sequence_retention=True,existing_history_and_source_retirement_preserved=True,old_rational_inspection_APIs_retained=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=12,new_resource_tests_require_original_root=1,retained_skill_tests_written=97,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=5,effective_files=29,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
