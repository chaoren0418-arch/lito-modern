"""Prepare selector-only tornado53 and its original S39 group, without running tests."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';SK='LiTo381-Modern/src/modern_lito/native_skills.rs';SEL='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';MANY='LiTo381-Modern/src/modern_lito/native_attack_many.rs'

def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_area_fireball_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous source qualification changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'previous candidate changed');base=read(q['candidate']);prior=read(base['contract']);require(sha(base['contract'])==base['contract_sha256'],'previous area contract changed');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==23 and MANY not in files,'previous source projection differs');check_pins({paths[n]:h for n,h in files.items()})
 catref=w['current_E149_skill_catalog_development'];require(sha(catref['qualification'])==catref['qualification_sha256'],'catalog changed');catalog=read(catref['qualification']);require(sha(catalog['facts'])==catalog['facts_sha256'],'catalog facts changed');facts=read(catalog['facts']);definition=next(row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id']==53);require((definition['row'],definition['column'],definition['effectGfx'],definition['mpCost'],definition['reuseDelayMillis'],definition['maximumRange'],definition['areaRadius'])==(6,4,758,40,1500,0,4),'original tornado definition differs')
 qualify="""fn qualify_tornado(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let row=c.sprites.get(&758).ok_or(HonorAssetError::Unsupported("tornado758 absent"))?;
    let a=c.resolve_action(758,ActionKey::new(0))?.animation;
    if row.graphics_id!=758||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1
        ||row.attributes.len()!=1||row.attributes[0].key!=ActionKey::new(104)||row.attributes[0].body_bytes.trim_ascii()!=b"8"
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=20
        ||a.frames.iter().enumerate().any(|(n,f)|f.image_id!=0||f.subframe!=if n==0{19}else{n as u32-1}||f.duration_ticks!=3
            ||f.events!=if n==1{vec![FrameEvent::Sound(125),FrameEvent::Hit]}else{vec![]}) {
        return Err(HonorAssetError::Unsupported("tornado758 original definition changed"));
    }
    Ok(vec![(758,0,a.clone())])
}

"""
 prepare_tornado="""        if self.tornado_bank.is_none()&&(w.learned_skills.contains(&53)||w.many_attacks.iter().any(|c|c.attack.graphics==758)||self.active.iter().any(|a|a.cue.attack.graphics==758)) {
            let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;let bank=SharedEffectBank::load_directional(&c,cancel,qualify_tornado)?;
                let clip=Clip::new(Arc::clone(bank.animation(758,0)?))?;Ok((bank,clip))})();
            match loaded{Ok((bank,clip))=>{self.tornado_bank=Some(bank);self.clips.insert(758,clip);},Err(e)=>self.error=Some(format!("{e:#}"))}
        }
"""
 audio="""                    FrameEvent::Sound(sound) if a.cue.attack.graphics==758=>{
                        let prepared=(||->Result<_>{ensure!(sound==125,"tornado sound changed");let bank=self.tornado_bank.as_ref().context("tornado758 bank absent")?;let animation=bank.animation(758,0)?;
                            let classified=super::combat_animation::classify_native_cues(&animation.frames[cue.source_frame].events)?;ensure!(classified.non_audio==[FrameEvent::Hit],"tornado758 source cue changed");
                            let group=classified.audio.context("tornado758 audio group absent")?;ensure!(group.candidates().len()==1,"tornado758 weighted cue unsupported");
                            super::native_audio_delivery::PreparedCue::from_finite_event(bank.sound_bank(),group,0,super::native_audio_delivery::FiniteEventKey{sequence:a.cue.action_sequence,graphics:758,action:0,source_frame:cue.source_frame}).map_err(Into::into)})();
                        match prepared{Ok(payload)=>self.events.push(EnteredEvent{prepared_audio:Some(payload),actor:a.cue.attack.caster,sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:"ManySound(125)".into(),sound:Some(125)}),Err(e)=>{self.error=Some(format!("{e:#}"));self.omitted+=1;a.finished=true;}}
                    },
"""
 changes={SK:[('14 | 31 | 32 | 49 | 52 | 54 | 72 | 78 | 206 =>','14 | 31 | 32 | 49 | 52 | 53 | 54 | 72 | 78 | 206 =>'),('match skill {77=>5000','match skill {53=>1500,77=>5000')],SEL:[('2|3|13|14|31|32|49|52|54|60','2|3|13|14|31|32|49|52|53|54|60'),('                        37 => "#37 解除詛咒','                        53 => "#53 龍捲風（自己周圍四格）".into(),\n                        37 => "#37 解除詛咒')],MANY:[
 ('(h(15)==3934&&b[17]==0&&b[1]==19);','(h(15)==3934&&b[17]==0&&b[1]==19)||(h(15)==758&&b[17]==0&&b[1]==18);'),
 ('#[derive(Clone)]struct Active {',qualify+'#[derive(Clone)]struct Active {'),
 ('selector_bank:Option<SharedEffectBank>,clips:','selector_bank:Option<SharedEffectBank>,tornado_bank:Option<SharedEffectBank>,clips:'),
 ('        if self.selector_bank.is_none()',prepare_tornado+'        if self.selector_bank.is_none()'),
 ('                    FrameEvent::Effect(_) if a.cue.attack.graphics==3934=>',audio+'                    FrameEvent::Effect(_) if a.cue.attack.graphics==3934=>'),
 ('let bank=if gfx==3934{self.selector_bank.as_ref()}else{self.bank.as_ref()};','let bank=if gfx==3934{self.selector_bank.as_ref()}else if gfx==758{self.tornado_bank.as_ref()}else{self.bank.as_ref()};'),
 ('bank.frame_heading(170,0,a.frame,a.finished,a.cue.attack.heading)?','bank.frame_heading(u32::from(gfx),0,a.frame,a.finished,a.cue.attack.heading)?'),
 ('self.bank.iter().chain(self.selector_bank.iter()).flat_map','self.bank.iter().chain(self.selector_bank.iter()).chain(self.tornado_bank.iter()).flat_map')]}
 template=T/'templates/native_tornado53_tests.rs';qa=template.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',qa))==8 and qa.count('#[ignore=')==1,'new QA count differs');out=owned(T/'candidates'/('native-E149-tornado53-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);(out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  path=Path(paths.get(name,str(R/name)));text=path.read_text('utf-8-sig');before[name]=sha(path)
  for old,new in replacements:require(text.count(old)==1,'source anchor differs: '+old);text=text.replace(old,new)
  if name==MANY:text+=qa
  target=out/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf-8',newline='\n');changed[name]=sha(target);files[name]=sha(target);paths[name]=str(target)
 contract=dict(schema='E149-tornado53-source-v1',formal_target_sha256=FORMAL,original_definition=definition,original_area_contract=base['contract'],original_area_contract_sha256=sha(base['contract']),raw_original_tornado_hex=prior['raw_original_effect_blocks_hex']['758'],source_changes=changes,QA_template=str(template),QA_template_sha256=sha(template),wire_prefix=[20,6,4],S39_action=18,S39_type=0,effect_graphics=758,original_sound=125,original_frames=20,original_hit_source_frame=1,damage_MP_and_radius_server_owned=True,original_pins=prior['original_pins'],tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_catalog_qualification=catref,current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=8,new_resource_tests_require_original_root=1,retained_skill_tests_written=51,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=3,effective_files=24,new_source_cases=8,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':prepare()
