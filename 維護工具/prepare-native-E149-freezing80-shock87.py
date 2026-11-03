"""Prepare isolated source only. Does not run tests, classes, clients or resources."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-freezing80-shock87-20261025-020000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development phase required');ref=w['current_E149_greater75_massslow76_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);base=read(q['contract']);require(sha(q['contract'])==q['contract_sha256'],'contract changed');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()})
 inspection=V/'native-E149-freezing80-shock87-review/20261024-172623-343534/review.json';require(sha(inspection)=='97B7BA7F5E017DC383F7ACFA686E1A387A21B4E0BA316613A4B169B8FFD16C68','review differs');proof=read(inspection);require(all(proof[n]for n in ['freezing80_selector_only_S39_type0_wire_caster_origin','freezing80_single_default_start_thirtyone_frames_overlap1353_no_Hit','shock87_actor_xy_range1_walk50_or54','shock87_physical_wire_and_S37_4434_separate_from_stun_success']) and not proof['shock87_persistent_object_loop_source_complete'],'primary contract differs');require(not C.exists(),'candidate exists')
 changes={
 'native_skills.rs':[
  ('72 | 78 | 206 =>','72 | 78 | 80 | 206 =>'),
  ('74 | 76 | 77 =>','74 | 76 | 77 | 87 =>'),
  ('11|20=>1','11|20|87=>1'),
  ('27|74|76=>3000','27|74|76|80=>3000,87=>8000'),
  ('    match skill {\n        1 |','    ensure!(skill!=87||matches!(own.walk_action,50|54),"衝擊之暈需要雙手劍步態");\n    match skill {\n        1 |')],
 'native_skill_selection.rs':[('                        75 =>','                        80 => "#80 冰雪颶風（自己周圍十格）".into(),\n                        87 => "#87 衝擊之暈（選取目標，一格內；需要雙手劍步態）".into(),\n                        75 =>')],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236|2232|3932)','matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236|2232|3932|4434)')],
 'native_additional_finite_effects.rs':[
  ('3944,230,760,3932];','3944,230,760,3932,4434];'),
  ('rows.push((3932,0,qualify_massslow76(c)?));Ok(rows)','rows.push((3932,0,qualify_massslow76(c)?));rows.push((4434,0,qualify_shock87(c)?));Ok(rows)')],
 'native_attack_many.rs':[
  ('matches!(h(15),757|758|1812|1819)','matches!(h(15),757|758|1812|1819|3933)'),
  ('class1_area_profile(gfx)||gfx==1812','class1_area_profile(gfx)||matches!(gfx,1812|3933)'),
  ('(74,762u16)]','(74,762u16),(80,3933u16)]'),
  ('762=>qualify_meteor74,_=>qualify_storm65','762=>qualify_meteor74,3933=>qualify_freezing80,_=>qualify_storm65'),
  ('matches!(a.cue.attack.graphics,1812|3924|762)','matches!(a.cue.attack.graphics,1812|3924|762|3933)'),
  ('762=>130,_=>1348','762=>130,3933=>1353,_=>1348'),
  ('if a.cue.attack.graphics==762{"MeteorPreparationRetired"}else','if a.cue.attack.graphics==3933{"FreezingBlizzardPreparationRetired"}else if a.cue.attack.graphics==762{"MeteorPreparationRetired"}else'),
  ('if a.cue.attack.graphics==3924{format!("ManyAudioOverlap({sound})")}','if matches!(a.cue.attack.graphics,3924|3933){format!("ManyAudioOverlap({sound})")}')]
 }
 supplements={'native_skills.rs':[T/'templates/native_freezing80_shock87_request_tests.rs'],'native_skill_effects.rs':[T/'templates/native_shock87_render_tests.rs'],'native_additional_finite_effects.rs':[T/'templates/native_shock87_support.rs',T/'templates/native_shock87_definition_tests.rs'],'native_attack_many.rs':[T/'templates/native_freezing80_support.rs',T/'templates/native_freezing80_tests.rs']};prepared={};repeat={'matches!(a.cue.attack.graphics,1812|3924|762)':4}
 for short,delta in changes.items():
  s=Path(paths[PREFIX+short]).read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==repeat.get(a,1),'source anchor differs '+short+': '+a+' count='+str(s.count(a)));s=s.replace(a,b)
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig')
  prepared[short]=s
 C.mkdir();overrides={}
 for short,s in prepared.items():
  name=PREFIX+short;previous=Path(paths[name]);p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(previous),predecessor_sha256=sha(previous),changes=changes[short],repeated_replacement_counts={a:repeat[a]for a,b in changes[short]if a in repeat},supplements=[dict(path=str(t),sha256=sha(t))for t in supplements.get(short,[])]);files[name]=sha(p);paths[name]=str(p)
 s=prepared['native_skills.rs'];b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 patterns=[r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)']+[r'pub fn '+kind+r'_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}'for kind in ['item','summon','mass']]
 for pat in patterns:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{80,87} and len(ids)==82,'request coverage differs');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=82,missing_request_ids=sorted(set(old['missing_request_ids'])-{80,87}),missing_request_count=98)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts changed');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core']for r in facts['definitions_by_id']if r['skill_id']in next_ids];cohort=Path(q['compile_run'])/'cohort.json';templates=[p for rows in supplements.values()for p in rows];cases=sum(p.read_text('utf-8-sig').count('#[test]')for p in templates);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=')for p in templates);require((len(files),cases,ignored)==(42,13,2),'case/overlay count differs');pins={str(p):sha(p)for p in [Path(__file__).resolve(),*templates]}
 contract=dict(schema='E149-freezing80-shock87-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['contract'],predecessor_candidate_sha256=sha(q['contract']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],preparation_pins=pins,modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=13,new_resource_opt_in_cases=2,previous_written_deferred_cases=402,freezing80_request_and_finite_source_integrated=True,shock87_request_and_impact_source_integrated=True,shock87_persistent_object_loop_source_complete=False,shock87_status_object_audio_History_pending=True,server_only_damage_cost_probability_and_status=True,relative_graphics_hurt_binding_complete=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),overlays=len(files),new_cases=cases,recognized=82,missing=98,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
