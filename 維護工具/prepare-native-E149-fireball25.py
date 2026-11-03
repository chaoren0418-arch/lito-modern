"""Stage #25 request/menu, class5 S39 fanout and callback-scoped audio. No execution."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';PREFIX='LiTo381-Modern/src/modern_lito/';SK=PREFIX+'native_skills.rs';SEL=PREFIX+'native_skill_selection.rs';MANY=PREFIX+'native_attack_many.rs';AUDIO=PREFIX+'native_audio_delivery.rs'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_fireball_clock_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous source changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'previous candidate changed');base=read(q['candidate']);require(sha(base['contract'])==base['contract_sha256'],'previous contract changed');prior=read(base['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==26 and AUDIO not in files,'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 native_path=Path(q['native_snapshot_contract']);native=read(native_path);require(sha(native_path)==q['native_snapshot_contract_sha256'] and native['flight_frame_cycle']==[1,2,3,4,0] and native['arrival_secondary_for_each_resolved_recipient_including_miss_and_dead'] and native['launch_resolves_first_target_again'],'native class5 contract differs')
 catalog_ref=w['current_E149_skill_catalog_development'];require(sha(catalog_ref['qualification'])==catalog_ref['qualification_sha256'],'catalog changed');catalog=read(catalog_ref['qualification']);require(sha(catalog['facts'])==catalog['facts_sha256'],'facts changed');facts=read(catalog['facts']);definition=next(row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id']==25);require((definition['effectGfx'],definition['maximumRange'],definition['reuseDelayMillis'])==(171,6,500),'formal #25 differs')
 templates={MANY:T/'templates/native_fireball25_tests.rs',AUDIO:T/'templates/native_fireball25_audio_tests.rs'}
 qa={n:p.read_text('utf-8-sig') for n,p in templates.items()};require([len(re.findall(r'#\[test\]',qa[n])) for n in [MANY,AUDIO]]==[9,3] and qa[MANY].count('#[ignore=')==1,'QA scope differs')
 prepare_fireball='''        if self.fireball_assets.is_none()&&(w.learned_skills.contains(&25)||w.many_attacks.iter().any(|c|c.attack.graphics==171)||self.active.iter().any(|a|a.cue.attack.graphics==171)) {
            let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;let assets=super::light_arrow_effect::LightArrowAssets::load_effect_profile_with_reader(&c,171,cancel,&mut None)?;let clip=assets.native_area_fireball_impact_clip()?;Ok((assets,clip))})();
            match loaded{Ok((assets,clip))=>{self.fireball_assets=Some(assets);self.fireball_impact=Some(clip);},Err(e)=>{self.error=Some(format!("{e:#}"));return;}}
        }
'''
 launch='''            if c.attack.graphics==171 {
                let active=(||->Result<_>{fireball_launch(c,w,self.fireball_assets.as_ref().context("fireball assets absent")?,self.fireball_impact.as_ref().context("fireball impact clip absent")?,now)})();
                match active{Ok(a)=>{self.active.push(a);self.started+=1;},Err(e)=>{self.error=Some(format!("{e:#}"));self.omitted+=1;}}continue;
            }
'''
 update='''for a in &mut self.active {
            if a.finished {continue;}
            if a.fireball.is_some() {
                let result=self.fireball_assets.as_ref().context("fireball bank absent").and_then(|assets|update_fireball(a,w,assets,now,&mut self.events,&mut self.non_audio));
                match result{Ok(values)=>{self.hits+=values.len()as u64;impacts.extend(values);},Err(e)=>{self.error=Some(format!("{e:#}"));self.omitted+=1;a.finished=true;}}continue;
            }
            match a.playback.advance'''
 draw='''            if let Some(state)=&a.fireball {
                let secondary=state.arrival_ns.is_some();let gfx=if secondary{218}else{171};let source_frame=if secondary{a.frame}else{state.source_frame};
                let positions:Vec<_>=if secondary{state.recipients.iter().map(|r|r.position).collect()}else{vec![state.position]};
                let rendered=(||->Result<_>{let assets=self.fireball_assets.as_ref().context("fireball assets absent")?;let visual=assets.native_area_fireball_frame(gfx,if secondary{0}else{state.heading},source_frame,a.finished)?;
                    let drawn=!a.finished&&!visual.frame.rgba.is_empty()&&!positions.is_empty();if drawn{submit_fireball_group(target,visual.frame,&positions,projection)?;}
                    Ok(json!({"sequence":a.cue.action_sequence,"wire_sequence":a.cue.attack.sequence,"graphics":gfx,"phase":if secondary{"secondary"}else{"flight"},"positions":positions.iter().map(|p|[p.x,p.y]).collect::<Vec<_>>(),"source_frame":source_frame,"subframe":visual.subframe,"sprite":visual.sprite_name,"heading":state.heading,"movement_callbacks":state.callbacks,"arrival_ns":state.arrival_ns,"copies":positions.len(),"drawn":drawn,"finished":a.finished}))})();
                match rendered{Ok(row)=>rows.push(row),Err(e)=>{self.omitted+=1;rows.push(json!({"error":e.to_string()}));}}continue;
            }
'''
 audio_method='''    /// A movement callback or per-recipient start has a distinct occurrence.
    /// Source slots keep their original meaning; recurring source1 is never
    /// disguised as another catalog frame or a fabricated cast sequence.
    pub(crate) fn from_callback_event(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,event:FiniteEventKey,occurrence:u64)->Result<Self> {
        ensure!(occurrence!=0&&event.sequence!=0&&event.graphics!=0,"callback event identity missing");
        Self::build_with_occurrence(bank,group,selection,Some(event),Some(occurrence))
    }
'''
 changes={SK:[('20 | 22 | 28','20 | 22 | 25 | 28'),('17|22|34|38=>6','17|22|25|34|38=>6'),('20|29|30|44|49|57|72=>500','20|25|29|30|44|49|57|72=>500')],SEL:[('                        22 => "#22 寒冰氣息','                        25 => "#25 火球術（選取目標，六格內）".into(),\n                        22 => "#22 寒冰氣息')],MANY:[
 ('matches!(h(15),1804|1805)','matches!(h(15),171|1804|1805)'),
 ('if !class1_area_profile(attack.graphics) {return None;}','if !matches!(attack.graphics,171|1804|1805) {return None;}'),
 ('#[derive(Clone)]struct Active {cue:Cue,start:u64,playback:Playback,frame:usize,finished:bool}',(T/'templates/native_fireball25_many.rs').read_text('utf-8-sig')+'\n#[derive(Clone)]struct Active {cue:Cue,start:u64,playback:Playback,frame:usize,finished:bool,fireball:Option<FireballState>}'),
 ('class1_banks:BTreeMap<u16,AreaBank>,clips:','class1_banks:BTreeMap<u16,AreaBank>,fireball_assets:Option<super::light_arrow_effect::LightArrowAssets>,fireball_impact:Option<Clip>,clips:'),
 ('        for (skill,gfx) in [(22,1804u16),(30,1805u16)] {',prepare_fireball+'        for (skill,gfx) in [(22,1804u16),(30,1805u16)] {'),
 ('            let Some(clip)=self.clips.get(&c.attack.graphics)',launch+'            let Some(clip)=self.clips.get(&c.attack.graphics)'),
 ('frame:1,finished:false});self.started+=1;','frame:1,finished:false,fireball:None});self.started+=1;'),
 ('for a in &mut self.active {match a.playback.advance',update),
 ('        for a in &self.active {\n            let gfx=', '        for a in &self.active {\n'+draw+'            let gfx=')],AUDIO:[
 ('    finite_event:Option<FiniteEventKey>,','    finite_event:Option<FiniteEventKey>,\n    callback_occurrence:Option<u64>,'),
 ('    fn build(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,finite_event:Option<FiniteEventKey>)->Result<Self> {',audio_method+'''    fn build(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,finite_event:Option<FiniteEventKey>)->Result<Self> {
        Self::build_with_occurrence(bank,group,selection,finite_event,None)
    }
    fn build_with_occurrence(bank:Arc<SoundBank>,group:NativeAudioGroup,selection:u32,finite_event:Option<FiniteEventKey>,callback_occurrence:Option<u64>)->Result<Self> {'''),
 ('Payload{bank,group,selection,finite_event,scope:','Payload{bank,group,selection,finite_event,callback_occurrence,scope:'),
 ('seen:std::collections::BTreeSet<FiniteEventKey>,','seen:std::collections::BTreeSet<(FiniteEventKey,Option<u64>)>,'),
 ('self.seen.retain(|event|sequences.contains(&event.sequence));','self.seen.retain(|(event,_)|sequences.contains(&event.sequence));'),
 ('cue.finite_event().ok_or_else','cue.finite_event().map(|event|(event,cue.0.callback_occurrence)).ok_or_else')]}
 out=owned(T/'candidates'/('native-E149-fireball25-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);(out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  path=Path(paths.get(name,str(R/name)));text=path.read_text('utf-8-sig');before[name]=sha(path)
  for old,new in replacements:require(text.count(old)==1,'source anchor differs: '+old);text=text.replace(old,new)
  text+=qa.get(name,'');target=out/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf-8',newline='\n');changed[name]=sha(target);files[name]=sha(target);paths[name]=str(target)
 contract=dict(schema='E149-fireball25-integrated-source-v1',formal_target_sha256=FORMAL,source_changes=changes,QA_templates={n:dict(path=str(p),sha256=sha(p)) for n,p in templates.items()},native_snapshot_contract=str(native_path),native_snapshot_contract_sha256=sha(native_path),original_pins={**prior['original_pins'],catalog['facts']:sha(catalog['facts'])},definition=definition,wire_prefix=[20,3,0],S39_action=18,S39_type=8,flight_source_cycle=[1,2,3,4,0],movement_ms=60,flight_graphics=171,secondary_graphics=218,request_S39_world_integration=True,audio_callback_occurrence_identity=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_catalog_qualification=catalog_ref,current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=12,new_resource_tests_require_original_root=1,retained_skill_tests_written=82,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=4,effective_files=27,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
