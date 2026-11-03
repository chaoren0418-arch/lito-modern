"""Add targeted65/66 and exact original finite presentation, retaining G06 History."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-storm65-sleep66-20261024-135000';PREFIX='LiTo381-Modern/src/modern_lito/'

def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_earthquake62_silence64_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);require(sha(q['candidate'])==q['candidate_sha256'],'candidate differs');base=read(q['candidate']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()});inspection=V/'native-E149-storm65-sleep66-inspection/20261024-134253-376985/inspection.json';proof=read(inspection);check_pins(proof['source_pins']);require(not C.exists(),'candidate exists')
 render=T/'templates/native_sleep66_render_tests.rs';template=(T/'templates/native_silence64_render_tests.rs').read_text('utf-8-sig').replace('silence64_render','sleep66_render').replace('original_silence_visual_is_world_audio_and_not_a_status_timer','original_sleep_visual_is_world_audio_and_not_a_status_timer').replace('original2177/808','original760/136').replace('2177','760').replace('808','136').replace('2822','5250');require(not render.exists() or render.read_text('utf-8-sig')==template,'existing render template differs');render.write_text(template,encoding='utf-8',newline='\n')
 raw=Path(proof['raw_definitions']['760']['path']).read_bytes();header=raw.splitlines()[0];frames=','.join('(0,'+str(20 if i==0 else i-1)+',6,&'+('[136]' if i==1 else '[]')+')' for i in range(21));spec='Spec{sprite:760,bank:760,header:&'+str(list(header)).replace(' ','')+',attrs:&[(104,&[56])],frames:&['+frames+']},\n'
 changes={
 'native_skills.rs':[('56 | 64 | 77 =>','56 | 64 | 65 | 66 | 77 =>'),('51=>1000,27=>3000','65=>2000,51=>1000,27=>3000'),('53|59|62=>1500','53|59|62|66=>1500')],
 'native_skill_selection.rs':[('                        62 =>','                        65 => "#65 雷霆風暴（選取目標，十格內；周圍三格）".into(),\n                        66 => "#66 沉睡之霧（選取目標，十格內；消耗 1 顆魔法寶石）".into(),\n                        62 =>')],
 'native_attack_many.rs':[
 ('matches!(h(15),171|1804|1805)','matches!(h(15),171|1804|1805|3924)'),
 ('fn class1_area_profile(gfx:u16)->bool {matches!(gfx,1804|1805)}','fn class1_area_profile(gfx:u16)->bool {matches!(gfx,1804|1805|3924)}'),
 ('matches!(attack.graphics,171|1804|1805)','matches!(attack.graphics,171|1804|1805|3924)'),
 ('[(22,1804u16),(30,1805u16),(62,1812u16)]','[(22,1804u16),(30,1805u16),(62,1812u16),(65,3924u16)]'),
 ('1805=>qualify_earthjail,_=>qualify_earthquake','1805=>qualify_earthjail,1812=>qualify_earthquake,_=>qualify_storm65'),
 ('a.cue.attack.graphics==1812&&self.error.is_some()&&!self.class1_banks.contains_key(&1812)','matches!(a.cue.attack.graphics,1812|3924)&&self.error.is_some()&&!self.class1_banks.contains_key(&a.cue.attack.graphics)'),
 ('"event":"EarthquakePreparationRetired"','"event":if a.cue.attack.graphics==3924{"StormPreparationRetired"}else{"EarthquakePreparationRetired"}'),
 ('FrameEvent::Sound(sound) if finite_area_profile(a.cue.attack.graphics)=>','FrameEvent::Sound(sound)|FrameEvent::Effect(sound) if finite_area_profile(a.cue.attack.graphics)=>'),
 ('sound==match gfx{1804=>714,1805=>718,_=>716}','sound==match gfx{1804=>714,1805=>718,1812=>716,_=>1348}'),
 ('actor:if a.cue.attack.graphics==1812{0}else{a.cue.attack.caster}','actor:if matches!(a.cue.attack.graphics,1812|3924){0}else{a.cue.attack.caster}'),
 ('actor:if matches!(a.cue.attack.graphics,1812|3924){0}else{a.cue.attack.caster},sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:format!("ManySound({sound})")','actor:if matches!(a.cue.attack.graphics,1812|3924){0}else{a.cue.attack.caster},sequence:a.cue.action_sequence,action:0,frame:cue.source_frame,event:if a.cue.attack.graphics==3924{format!("ManyAudioOverlap({sound})")}else{format!("ManySound({sound})")}'),
 ('FrameEvent::Hit if a.cue.attack.graphics==1812=>','FrameEvent::Hit if matches!(a.cue.attack.graphics,1812|3924)=>'),
 ('"event":"EarthquakeHit"','"event":if a.cue.attack.graphics==3924{"StormHit"}else{"EarthquakeHit"}'),
 ('a.cue.attack.graphics==1812&&a.finished&&!self.class1_banks.contains_key(&1812)','matches!(a.cue.attack.graphics,1812|3924)&&a.finished&&!self.class1_banks.contains_key(&a.cue.attack.graphics)'),
 ('"sequence":a.cue.action_sequence,"graphics":1812,"finished":true','"sequence":a.cue.action_sequence,"graphics":a.cue.attack.graphics,"finished":true')],
 'native_additional_finite_effects.rs':[('2230,3944,230];','2230,3944,230,760];'),('const SPECS:&[Spec]=&[\n','const SPECS:&[Spec]=&[\n'+spec)],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230|2177)','matches!(cue.graphics,3943|2230|3944|230|2177|760)')]
 }
 supplements={'native_skills.rs':[T/'templates/native_storm65_sleep66_request_tests.rs'],'native_attack_many.rs':[T/'templates/native_storm65_support.rs',T/'templates/native_storm65_tests.rs'],'native_additional_finite_effects.rs':[T/'templates/native_sleep66_definition_tests.rs'],'native_skill_effects.rs':[render]};prepared={}
 for short,delta in changes.items():
  s=Path(paths[PREFIX+short]).read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==1,'source anchor differs '+short+': '+a[:95]+' count='+str(s.count(a)));s=s.replace(a,b)
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig')
  prepared[short]=s
 C.mkdir();overrides={}
 for short,s in prepared.items():
  name=PREFIX+short;previous=Path(paths[name]);p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(previous),predecessor_sha256=sha(previous),changes=changes[short],supplements=[dict(path=str(t),sha256=sha(t)) for t in supplements.get(short,[])]);files[name]=sha(p);paths[name]=str(p)
 s=Path(paths[PREFIX+'native_skills.rs']).read_text('utf-8-sig');b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{65,66} and len(ids)==72,'scope exceeds65/66');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=72,missing_request_ids=sorted(set(old['missing_request_ids'])-{65,66}),missing_request_count=108);cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts differ');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id'] in next_ids];cohort=Path(q['compile_run'])/'cohort.json';cases=sum(p.read_text('utf-8-sig').count('#[test]') for rows in supplements.values() for p in rows);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=') for rows in supplements.values() for p in rows);require((len(files),cases,ignored)==(36,15,2),'scope count differs')
 contract=dict(schema='E149-storm65-sleep66-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),integration_predecessor_cohort=str(cohort),integration_predecessor_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=15,new_resource_opt_in_cases=2,previous_written_deferred_cases=290,storm_class1_first_target_origin_and_overlap_audio=True,storm_Hit_diagnostic_only=True,storm_failed_preparation_retires_once=True,sleep_world_audio_and_server_status_authority=True,prior_G06_finite_history_unchanged=True,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),effective_files=36,new_cases=15,recognized_requests=72,remaining=108,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
