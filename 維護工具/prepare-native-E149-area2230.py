"""Prepare S39 receipt-time first-target class1 area skills22/30; no execution."""
from pathlib import Path
from datetime import datetime,timezone
import re,json
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';SK='LiTo381-Modern/src/modern_lito/native_skills.rs';SEL='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';MANY='LiTo381-Modern/src/modern_lito/native_attack_many.rs';WORLD='LiTo381-Modern/src/modern_lito/native_world.rs'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_class1_origin_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous source qualification changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'previous candidate changed');base=read(q['candidate']);prior=read(base['contract']);require(sha(base['contract'])==base['contract_sha256'],'previous contract changed');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==25 and WORLD not in files,'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 native_path=V/'native-E149-area2230-contract/20261024-045931-508526/contract.json';native=read(native_path);require(native['class1_graphics']==[1804,1805] and native['first_target_snapshot_at_packet_receipt'] and native['missing_first_target_fallback_one_step'] and not native['PE_executed'] and not native['SQL_executed'],'native snapshot evidence differs');require(native['display_names']=={'22':'寒冰氣息','30':'巖牢'},'display-name reference differs');require(sha(native['PE'])==native['PE_sha256'],'original PE changed')
 catref=w['current_E149_skill_catalog_development'];require(sha(catref['qualification'])==catref['qualification_sha256'],'catalog changed');catalog=read(catref['qualification']);require(sha(catalog['facts'])==catalog['facts_sha256'],'catalog facts changed');facts=read(catalog['facts']);definitions={row['skill_id']:row['definitions'][0]['core'] for row in facts['definitions_by_id'] if row['skill_id'] in [22,30]};require([(definitions[i]['effectGfx'],definitions[i]['maximumRange'],definitions[i]['reuseDelayMillis']) for i in [22,30]]==[(1804,6,300),(1805,8,500)],'E149 definitions differ')
 capture="""fn class1_area_profile(gfx:u16)->bool {matches!(gfx,1804|1805)}
pub fn capture_target_origin(attack:&Attack,actors:&BTreeMap<u32,super::native_world::Actor>)->Option<[i32;2]> {
    if !class1_area_profile(attack.graphics) {return None;}
    let &(dx,dy)=[(0,-1),(1,-1),(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1)].get(usize::from(attack.heading))?;
    if let Some(actor)=attack.targets.first().and_then(|first|actors.get(&first.id)) {return Some([i32::from(actor.x),i32::from(actor.y)]);}
    Some([i32::from(attack.position[0])+dx,i32::from(attack.position[1])+dy])
}

"""
 qualify="""fn qualify_area2230(c:&SpriteActionCatalog,gfx:u16)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {
    let (ticks,sound,sound_frame,hits,attrs):(Vec<u32>,u32,usize,&[usize],Vec<(u32,&[u8])>)=match gfx {
        1804=>(vec![5,4,4,4,4,4,4,4,5,5,5,5,5,5,5,5,5,5,5,5,5],714,1,&[8],vec![(104,b"8"),(109,b"1 -1")]),
        1805=>(vec![4,3,3,3,3,3,3,4,4,4,4,5,5,5,5,5,4],718,2,&[4,7],vec![(101,b"1806"),(105,b"1 1807"),(109,b"1 -1")]),
        _=>return Err(HonorAssetError::Unsupported("unqualified area class1 graphic")),
    };
    let row=c.sprites.get(&u32::from(gfx)).ok_or(HonorAssetError::Unsupported("area class1 absent"))?;let a=c.resolve_action(u32::from(gfx),ActionKey::new(0))?.animation;
    if row.graphics_id!=u32::from(gfx)||row.image_count!=Some(1)||row.header_issue.is_some()||row.actions.len()!=1||row.attributes.len()!=attrs.len()
        ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1)
        ||a.directionality!=Directionality::Undirected||a.frames_per_second!=24||a.frames.len()!=ticks.len()
        ||a.frames.iter().enumerate().any(|(n,f)|{let mut events=vec![];if n==sound_frame{events.push(FrameEvent::Sound(sound));}if hits.contains(&n){events.push(FrameEvent::Hit);}
            f.image_id!=0||f.subframe!=if n==0{ticks.len()as u32-1}else{n as u32-1}||f.duration_ticks!=ticks[n]||f.events!=events}) {
        return Err(HonorAssetError::Unsupported("area class1 original definition changed"));
    }
    if gfx==1805 {for (id,attrs) in [(1806,vec![(102,b"0".as_slice())]),(1807,vec![(102,b"0".as_slice()),(104,b"8")])] {
        let row=c.sprites.get(&id).ok_or(HonorAssetError::Unsupported("earthjail layer absent"))?;
        if row.graphics_id!=id||row.image_count!=Some(1)||row.header_issue.is_some()||!row.actions.is_empty()||row.attributes.len()!=attrs.len()
            ||attrs.iter().any(|(key,body)|row.attributes.iter().filter(|v|v.key==ActionKey::new(*key)&&v.body_bytes.trim_ascii()==*body).count()!=1) {return Err(HonorAssetError::Unsupported("earthjail original layer changed"));}
    }}
    Ok(vec![(u32::from(gfx),0,a.clone())])
}
fn qualify_cloud(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {qualify_area2230(c,1804)}
fn qualify_earthjail(c:&SpriteActionCatalog)->super::tower_assets::Result<Vec<(u32,u8,Animation)>> {qualify_area2230(c,1805)}
struct AreaLayers {bank:super::native_actor_layers::Bank,bindings:super::native_actor_layers::Bindings}
struct AreaBank {bank:SharedEffectBank,layers:Option<AreaLayers>}

"""
 prepare_area="""        for (skill,gfx) in [(22,1804u16),(30,1805u16)] {
            if self.class1_banks.contains_key(&gfx)||!(w.learned_skills.contains(&skill)||w.many_attacks.iter().any(|c|c.attack.graphics==gfx)||self.active.iter().any(|a|a.cue.attack.graphics==gfx)){continue;}
            let loaded=(||->Result<_>{let c=SharedActionCatalog::reuse_for_model(root,cancel,shared)?;let bank=SharedEffectBank::load_directional(&c,cancel,if gfx==1804{qualify_cloud}else{qualify_earthjail})?;
                let animation=bank.animation(u32::from(gfx),0)?;let clip=Clip::new(Arc::clone(animation))?;
                let layers=if gfx==1805 {let bindings=super::native_actor_layers::Plan::from_catalog(c.catalog(),1805)?.bind(0)?;let mut layer_bank=super::native_actor_layers::Bank::open(root)?;
                    for frame in &animation.frames {let prepared=layer_bank.prepare(&bindings,frame.subframe as usize,cancel)?;ensure!(prepared.missing_shadow.is_none()&&prepared.missing_attachments.is_empty(),"earthjail layers incomplete");}Some(AreaLayers{bank:layer_bank,bindings})}else{None};Ok((AreaBank{bank,layers},clip))})();
            match loaded{Ok((bank,clip))=>{self.class1_banks.insert(gfx,bank);self.clips.insert(gfx,clip);},Err(e)=>{self.error=Some(format!("{e:#}"));return;}}
        }
"""
 audio="""                    FrameEvent::Sound(sound) if class1_area_profile(a.cue.attack.graphics)=>{
                        let prepared=(||->Result<_>{let gfx=a.cue.attack.graphics;ensure!(sound==if gfx==1804{714}else{718},"area class1 sound changed");let bank=&self.class1_banks.get(&gfx).context("area class1 bank absent")?.bank;let animation=bank.animation(u32::from(gfx),0)?;
                            let classified=super::combat_animation::classify_native_cues(&animation.frames[cue.source_frame].events)?;ensure!(classified.non_audio.is_empty(),"area class1 source audio frame changed");let group=classified.audio.context("area class1 audio group absent")?;ensure!(group.candidates().len()==1,"area class1 weighted cue unsupported");
                            super::native_audio_delivery::PreparedCue::from_finite_event(bank.sound_bank(),group,0,super::native_audio_delivery::FiniteEventKey{sequence:a.cue.action_sequence,graphics:gfx,action:0,source_frame:cue.source_frame}).map_err(Into::into)})();
                        match prepared{Ok(payload)=>self.events.push(EnteredEvent{prepared_audio:Some(payload),actor:a.cue.attack.caster,sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("ManySound({sound})"),sound:Some(sound)}),Err(e)=>{self.error=Some(format!("{e:#}"));self.omitted+=1;a.finished=true;}}
                    },
"""
 draw="""                    let mut layer_receipts=vec![];
                    if drawn {let submitted=(||->Result<()>{
                        if gfx==758{return submit_tornado_group(target,v.frame,position,projection);}
                        let mut frames=vec![(v.frame,if gfx==1805{Mix::Over}else{Mix::Screen})];
                        if let Some(layers)=self.class1_banks.get(&gfx).and_then(|bank|bank.layers.as_ref()) {
                            for (name,mix) in &layers.bindings.attachments {let frame=layers.bank.frame(name,v.subframe)?;let mode=match mix{super::native_actor_layers::Mix::Over=>Mix::Over,super::native_actor_layers::Mix::Screen=>Mix::Screen};frames.push((frame,mode));layer_receipts.push(json!({"kind":"clothes","sprite":name,"subframe":v.subframe,"mix":mix,"source":layers.bank.get(name)?.source}));}
                            if let Some(name)=&layers.bindings.shadow {let frame=layers.bank.frame(name,v.subframe)?;frames.push((frame,Mix::Shadow));layer_receipts.push(json!({"kind":"shadow","sprite":name,"subframe":v.subframe,"source":layers.bank.get(name)?.source}));}
                        }
                        target.submit_batch(&frames,position,projection)
                    })();if let Err(e)=submitted{self.omitted+=1;rows.push(json!({"error":e.to_string()}));continue;}}
"""
 changes={SK:[('18 | 20 | 28 | 29 | 34','18 | 20 | 22 | 28 | 29 | 30 | 34'),('29=>8,17|34|38=>6','29|30=>8,17|22|34|38=>6'),('match skill {53=>1500','match skill {22=>300,53=>1500'),('20|29|44|49|57|72=>500','20|29|30|44|49|57|72=>500')],SEL:[('                        53 => "#53 龍捲風','                        22 => "#22 寒冰氣息（選取目標，六格內）".into(),\n                        30 => "#30 巖牢（選取目標，八格內）".into(),\n                        53 => "#53 龍捲風')],WORLD:[('self.many_attacks.push(super::native_attack_many::Cue{attack:attack.clone(),','self.many_attacks.push(super::native_attack_many::Cue{target_origin:super::native_attack_many::capture_target_origin(&attack,&self.actors),attack:attack.clone(),')],MANY:[
 ('(h(15)==758&&b[17]==0&&b[1]==18);','(h(15)==758&&b[17]==0&&b[1]==18)||(matches!(h(15),1804|1805)&&b[17]==8&&b[1]==18);'),
 ('pub struct Cue {pub attack:Attack,','pub struct Cue {pub target_origin:Option<[i32;2]>,pub attack:Attack,'),
 ('fn tornado_positions(origin:WorldCell)',capture+qualify+'fn tornado_positions(origin:WorldCell)'),
 ('tornado_bank:Option<SharedEffectBank>,clips:','tornado_bank:Option<SharedEffectBank>,class1_banks:BTreeMap<u16,AreaBank>,clips:'),
 ('        if self.tornado_bank.is_none()',prepare_area+'        if self.tornado_bank.is_none()'),
 ('            let Some(clip)=self.clips.get(&c.attack.graphics)', '            if class1_area_profile(c.attack.graphics)&&c.target_origin.is_none(){self.omitted+=1;continue;}\n            let Some(clip)=self.clips.get(&c.attack.graphics)'),
 ('                    FrameEvent::Sound(sound) if a.cue.attack.graphics==758=>',audio+'                    FrameEvent::Sound(sound) if a.cue.attack.graphics==758=>'),
 ('else if gfx==758{self.tornado_bank.as_ref()}else{self.bank.as_ref()};','else if gfx==758{self.tornado_bank.as_ref()}else if class1_area_profile(gfx){self.class1_banks.get(&gfx).map(|bank|&bank.bank)}else{self.bank.as_ref()};'),
 ('let position=WorldCell{x:i32::from(a.cue.attack.position[0]),y:i32::from(a.cue.attack.position[1])};','let frozen=a.cue.target_origin.unwrap_or([i32::from(a.cue.attack.position[0]),i32::from(a.cue.attack.position[1])]);let position=WorldCell{x:frozen[0],y:frozen[1]};'),
 ('                    if drawn {let submitted=if gfx==758{submit_tornado_group(target,v.frame,position,projection)}else{target.submit(v.frame,position,projection,Mix::Screen)};if let Err(e)=submitted{self.omitted+=1;rows.push(json!({"error":e.to_string()}));continue;}}',draw),
 ('"heading":a.cue.attack.heading,"targets":a.cue.targets,','"heading":a.cue.attack.heading,"layers":layer_receipts,"target_origin":a.cue.target_origin,"targets":a.cue.targets,'),
 ('self.bank.iter().chain(self.selector_bank.iter()).chain(self.tornado_bank.iter()).flat_map','self.bank.iter().chain(self.selector_bank.iter()).chain(self.tornado_bank.iter()).chain(self.class1_banks.values().map(|bank|&bank.bank)).flat_map')]}
 template=T/'templates/native_area2230_tests.rs';qa=template.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',qa))==9 and qa.count('#[ignore=')==1,'new QA scope differs');out=owned(T/'candidates'/('native-E149-area2230-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);(out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  path=Path(paths.get(name,str(R/name)));text=path.read_text('utf-8-sig');before[name]=sha(path)
  for old,new in replacements:require(text.count(old)==1,'source anchor differs: '+old);text=text.replace(old,new)
  if name==MANY:text+=qa
  target=out/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf-8',newline='\n');changed[name]=sha(target);files[name]=sha(target);paths[name]=str(target)
 contract=dict(schema='E149-area2230-source-v1',formal_target_sha256=FORMAL,definitions=definitions,display_names=native['display_names'],native_snapshot_contract=str(native_path),native_snapshot_contract_sha256=sha(native_path),corrected_start_contract=prior['native_start_contract'],corrected_start_contract_sha256=prior['native_start_contract_sha256'],source_changes=changes,QA_template=str(template),QA_template_sha256=sha(template),S39_action=18,S39_type=8,graphics=[1804,1805],frames={'1804':21,'1805':17},sounds={'1804':714,'1805':718},hit_source_frames={'1804':[8],'1805':[4,7]},snapshot_at_packet_receipt=True,costs_damage_and_radius_server_owned=True,original_pins={**prior['original_pins'],str(native_path):sha(native_path),str(Path(catalog['facts'])):sha(catalog['facts'])},tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_catalog_qualification=catref,current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=9,new_resource_tests_require_original_root=1,retained_skill_tests_written=65,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=4,effective_files=26,new_source_cases=9,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
