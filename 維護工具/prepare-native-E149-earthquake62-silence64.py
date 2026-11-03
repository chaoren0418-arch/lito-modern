"""Add exact centered1812 and actor-XY64 request/menu/finite resource source."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-earthquake62-silence64-20261024-132800';PREFIX='LiTo381-Modern/src/modern_lito/'
def prepare():
 w=read(R/'工作佇列.json');ref=w['current_E149_finite_history_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);require(sha(q['candidate'])==q['candidate_sha256'],'candidate differs');base=read(q['candidate']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);inspection=V/'native-E149-earthquake62-silence64-inspection/20261024-132318-591306/inspection.json';proof=read(inspection);check_pins(proof['source_pins']);require(not C.exists(),'candidate exists')
 changes={
 'native_skills.rs':[('54 | 59 | 72','54 | 59 | 62 | 72'),('50 | 56 | 77 =>','50 | 56 | 64 | 77 =>'),('10|40|46|47|56=>3','10|40|46|47|56|64=>3'),('53|59=>1500','53|59|62=>1500'),('56|57|72=>500','56|57|64|72=>500')],
 'native_skill_selection.rs':[('54|59|60|72','54|59|60|62|72'),('                        61 =>','                        62 => "#62 震裂術（自己周圍三格）".into(),\n                        64 => "#64 魔法封印（選取目標，三格內；消耗 1 顆魔法寶石）".into(),\n                        61 =>')],
 'native_attack_many.rs':[
 ('matches!(h(15),757|758)','matches!(h(15),757|758|1812)'),
 ('[(22,1804u16),(30,1805u16)]','[(22,1804u16),(30,1805u16),(62,1812u16)]'),
 ('if gfx==1804{qualify_cloud}else{qualify_earthjail}','match gfx{1804=>qualify_cloud,1805=>qualify_earthjail,_=>qualify_earthquake}'),
 ('let layers=if gfx==1805 {let bindings=super::native_actor_layers::Plan::from_catalog(c.catalog(),1805)?','let layers=if matches!(gfx,1805|1812) {let bindings=super::native_actor_layers::Plan::from_catalog(c.catalog(),u32::from(gfx))?'),
 ('"earthjail layers incomplete"','"area layers incomplete"'),
 ('            if a.finished {continue;}\n            if a.fireball.is_some()', '            if a.finished {continue;}\n            if a.cue.attack.graphics==1812&&self.error.is_some()&&!self.class1_banks.contains_key(&1812){a.finished=true;self.omitted+=1;self.non_audio.push(json!({"sequence":a.cue.action_sequence,"event":"EarthquakePreparationRetired","no_retry":true}));continue;}\n            if a.fireball.is_some()'),
 ('FrameEvent::Sound(sound) if class1_area_profile(a.cue.attack.graphics)=>','FrameEvent::Sound(sound) if finite_area_profile(a.cue.attack.graphics)=>'),
 ('sound==if gfx==1804{714}else{718}','sound==match gfx{1804=>714,1805=>718,_=>716}'),
 ('prepared_audio:Some(payload),actor:a.cue.attack.caster,sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("ManySound({sound})")','prepared_audio:Some(payload),actor:if a.cue.attack.graphics==1812{0}else{a.cue.attack.caster},sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("ManySound({sound})")'),
 ('                    FrameEvent::Hit=>{self.hits+=1;', '                    FrameEvent::Hit if a.cue.attack.graphics==1812=>{self.hits+=1;self.non_audio.push(json!({"sequence":a.cue.action_sequence,"source_frame":cue.source_frame,"event":"EarthquakeHit","targets":a.cue.targets,"authority_callback":false,"relative_graphics_hurt_binding_pending":true}));},\n                    FrameEvent::Hit=>{self.hits+=1;'),
 ('        for a in &self.active {\n            if let Some(state)=&a.fireball','        for a in &self.active {\n            if a.cue.attack.graphics==1812&&a.finished&&!self.class1_banks.contains_key(&1812){rows.push(json!({"sequence":a.cue.action_sequence,"graphics":1812,"finished":true,"drawn":false,"assets_unavailable":true}));continue;}\n            if let Some(state)=&a.fireball'),
 ('else if class1_area_profile(gfx){self.class1_banks.get','else if finite_area_profile(gfx){self.class1_banks.get'),
 ('if gfx==1805{Mix::Over}else{Mix::Screen}','finite_area_mix(gfx)')],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230)','matches!(cue.graphics,3943|2230|3944|230|2177)')]
 }
 supplements={'native_skills.rs':[T/'templates/native_earthquake62_silence64_request_tests.rs'],'native_attack_many.rs':[T/'templates/native_earthquake62_support.rs',T/'templates/native_earthquake62_tests.rs'],'native_skill_effects.rs':[T/'templates/native_silence64_render_tests.rs']};prepared={}
 for short,delta in changes.items():
  s=Path(paths[PREFIX+short]).read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==1,'source anchor differs '+short+': '+a[:100]+' count='+str(s.count(a)));s=s.replace(a,b)
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig')
  prepared[short]=s
 C.mkdir();overrides={}
 for short,s in prepared.items():
  name=PREFIX+short;previous=Path(paths[name]);p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(previous),predecessor_sha256=sha(previous),changes=changes[short],supplements=[dict(path=str(t),sha256=sha(t)) for t in supplements.get(short,[])]);files[name]=sha(p);paths[name]=str(p)
 s=Path(paths[PREFIX+'native_skills.rs']).read_text('utf-8-sig');b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{62,64} and len(ids)==70,'scope exceeds62/64');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=70,missing_request_ids=sorted(set(old['missing_request_ids'])-{62,64}),missing_request_count=110);cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts differ');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id'] in next_ids];cohort=Path(q['compile_run'])/'cohort.json';cases=sum(p.read_text('utf-8-sig').count('#[test]') for rows in supplements.values() for p in rows);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=') for rows in supplements.values() for p in rows);require((len(files),cases,ignored)==(36,14,2),'scope count differs')
 contract=dict(schema='E149-earthquake62-silence64-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),integration_predecessor_cohort=str(cohort),integration_predecessor_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=14,new_resource_opt_in_cases=2,previous_written_deferred_cases=276,earthquake_fixed_wire_origin_and_original_three_layers=True,earthquake_Hit_diagnostic_only=True,earthquake_failed_preparation_retires_once=True,silence_world_audio_and_server_status_authority=True,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),effective_files=36,new_cases=14,recognized_requests=70,remaining=110,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
