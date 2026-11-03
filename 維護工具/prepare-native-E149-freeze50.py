"""Create immutable freeze50 overlay; retain every predecessor and nested pairing."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-freeze50-20261024-103539'
INSPECTION=V/'native-E149-freeze50-inspection/20261024-103539-283280/inspection.json'
PREFIX='LiTo381-Modern/src/modern_lito/'
def prepare():
 w=read(R/'工作佇列.json');ref=w['current_E149_sun46_weak47_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'predecessor differs');q=read(ref['qualification']);check_pins(q['source_pins']);base=read(q['candidate']);proof=read(INSPECTION);check_pins(proof['source_pins']);require(proof['flight_events']==proof['secondary_events']==[],'silent profile differs')
 files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths'])
 changes={
 'native_skills.rs':[('40 | 45 | 46 | 47 | 77 =>','40 | 45 | 46 | 47 | 50 | 77 =>'),('17|22|25|34|38=>6','17|22|25|34|38|50=>6'),('46|47|49|57|72=>500','46|47|49|50|57|72=>500'),('match skill {33=>5000','match skill {50=>1000,33=>5000')],
 'native_skill_selection.rs':[('                        46 => "#46 烈炎術','                        50 => "#50 冰矛圍籬（選取目標，六格內）".into(),\n                        46 => "#46 烈炎術')],
 'native_spell_projectiles.rs':[('167|1583|1799|1797|129','167|1583|1799|1797|756|129'),('(7,1799),(15,1583)]','(7,1799),(15,1583),(50,756)]'),('.map(|c| (c.attacker, c.action_sequence))\n            .collect();','.map(|c| (c.attacker, c.action_sequence))\n            .chain(self.flights.iter().filter(|f|!f.finished).map(|f|(f.cue.attacker,f.cue.action_sequence)))\n            .collect();'),('            if flight.finished{continue;}','            if flight.finished{continue;}\n            if self.errors.contains_key(&flight.cue.projectile.graphics)&&!self.assets.contains_key(&flight.cue.projectile.graphics){flight.finished=true;self.omitted+=1;continue;}'),('if flight.cue.damage_marker>0&&','if flight.cue.projectile.graphics==756 {self.non_audio.push(serde_json::json!({"sequence":flight.cue.action_sequence,"event":"Arrival","relative_graphics_binding_pending":true,"authority_callback":false}));}\n                    if flight.cue.projectile.graphics!=756&&flight.cue.damage_marker>0&&')],
 'light_arrow_effect.rs':[
 ('167=>(219,4,97,Directionality::EightWay),1583=>(218,4,97,Directionality::EightWay),1799=>(1800,5,719,Directionality::Undirected),1797=>(1798,4,719,Directionality::EightWay)', '167=>(219,4,Some(97),Directionality::EightWay),1583=>(218,4,Some(97),Directionality::EightWay),1799=>(1800,5,Some(719),Directionality::Undirected),1797=>(1798,4,Some(719),Directionality::EightWay),756=>(219,4,None,Directionality::EightWay)'),
 ('f.events==if n==1{vec![FrameEvent::Sound(sound)]}else{vec![]}', 'f.events==if n==1{sound.map(FrameEvent::Sound).into_iter().collect::<Vec<_>>()}else{vec![]}'),
 ('    group: NativeAudioGroup,','    group: Option<NativeAudioGroup>,'),
 ('matches!(graphics,167|1583|1799)','matches!(graphics,167|1583|1799|756)'),
 ('            167=>(qualify_effect(&catalog)?,qualify_impact(&catalog)?),','            756=>(qualify_freeze50_effect(&catalog)?,qualify_impact(&catalog)?),\n            167=>(qualify_effect(&catalog)?,qualify_impact(&catalog)?),'),
 ('let (launch_sound,impact_sound)=match graphics {167=>(97,None),1583=>(97,Some(99)),1799=>(719,Some(711)),1797=>(719,Some(713)),171=>(98,Some(99)),_=>unreachable!()};', 'let (launch_sound,impact_sound)=match graphics {167=>(Some(97),None),1583=>(Some(97),Some(99)),1799=>(Some(719),Some(711)),1797=>(Some(719),Some(713)),171=>(Some(98),Some(99)),756=>(None,None),_=>unreachable!()};'),
 ('        let group = classify_native_cues(&[FrameEvent::Sound(launch_sound)])\n            .map_err(|_| LightArrowError::Unsupported("sound cue classification"))?\n            .audio\n            .ok_or(LightArrowError::Unsupported("missing sound cue group"))?;\n        let mut groups=vec![group.clone()];', '        let group = launch_sound.map(|sound|classify_native_cues(&[FrameEvent::Sound(sound)])\n            .map_err(|_| LightArrowError::Unsupported("sound cue classification"))?\n            .audio.ok_or(LightArrowError::Unsupported("missing sound cue group"))).transpose()?;\n        // 756/219 have no markers: no fallback WAV or fabricated silent cue.\n        let mut groups:Vec<_>=group.iter().cloned().collect();'),
 ('    pub fn sound_group(&self) -> &NativeAudioGroup {\n        &self.group\n    }', '    pub fn sound_group(&self) -> Option<&NativeAudioGroup> {\n        self.group.as_ref()\n    }'),
 ('(167,219)|(1583,218)|(1799,1800)|(1797,1798)', '(167,219)|(1583,218)|(1799,1800)|(1797,1798)|(756,219)'),
 ('fn qualify_effect(catalog: &SpriteActionCatalog) -> Result<BoundSequence> {', 'fn qualify_freeze50_effect(catalog:&SpriteActionCatalog)->Result<BoundSequence> {\n    qualify_fixed(catalog,756,Directionality::EightWay,8,&[(104,b"8"),(109,b"4 219")],\n        &[(3,2),(0,2),(1,2),(2,2)],None,None)\n}\nfn qualify_effect(catalog: &SpriteActionCatalog) -> Result<BoundSequence> {')]
 }
 supplements={'native_skills.rs':[T/'templates/native_freeze50_request_tests.rs'],'light_arrow_effect.rs':[T/'templates/native_freeze50_clock_tests.rs'],'native_spell_projectiles.rs':[T/'templates/native_freeze50_scene_tests.rs']}
 counts=[(p.read_text('utf-8-sig').count('#[test]'),p.read_text('utf-8-sig').count('#[ignore=')) for ps in supplements.values() for p in ps];require(counts==[(6,0),(5,0),(1,1)],'case source counts differ')
 require(not C.exists(),'candidate already exists');C.mkdir();overrides={}
 for short,delta in changes.items():
  name=PREFIX+short;before=Path(paths[name]);s=before.read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==1,'source anchor differs '+short+': '+a[:90]);s=s.replace(a,b)
  extra=[]
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig');extra.append(dict(path=str(p),sha256=sha(p)))
  p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');files[name]=sha(p);paths[name]=str(p);overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(before),predecessor_sha256=sha(before),changes=delta,supplements=extra)
 s=Path(paths[PREFIX+'native_skills.rs']).read_text('utf-8-sig');body=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{50} and len(ids)==63,'scope exceeds50');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=63,missing_request_ids=sorted(set(old['missing_request_ids'])-{50}),missing_request_count=117)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);facts=read(cq['facts']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog differs');next_ids=cov['missing_request_ids'][:2];next_defs=[x['definitions'][0]['core'] for x in facts['definitions_by_id'] if x['skill_id'] in next_ids]
 cohort=Path(q['compile_run'])/'cohort.json';contract=dict(schema='E149-freeze50-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),integration_predecessor_cohort=str(cohort),integration_predecessor_cohort_sha256=sha(cohort),inspection=str(INSPECTION),inspection_sha256=sha(INSPECTION),original_pins=proof['external_readonly_pins'],modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=12,new_resource_opt_in_cases=1,previous_written_deferred_cases=202,silent_flight_and_secondary=True,sound_group_is_optional=True,freeze50_hurt_deferred=True,freeze_status_server_owned=True,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False)
 (C/'contract.json').write_bytes(encoded(contract));print(json.dumps(dict(contract=str(C/'contract.json'),sha256=sha(C/'contract.json'),changed_runtime_files=4,new_cases=12,recognized_requests=63,remaining=117,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
