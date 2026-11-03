"""Prepare isolated source only. Does not run tests, classes, clients or resources."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-decay71-meteor74-20261025-012000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development phase required');ref=w['current_E149_mass69_integration_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);base=read(q['contract']);require(sha(q['contract'])==q['contract_sha256'],'contract changed');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()})
 inspection=V/'native-E149-decay71-meteor74-review/20261024-165448-673361/review.json';require(sha(inspection)=='44450BCC73881F3D87955C86F3093885F6F95E076C507009C6F74B1399D017E9','review differs');proof=read(inspection);require(all(proof[n]for n in ['decay_actor_xy_range3_self_excluded','decay_material_commit_pair_actual_callbacks','decay_S37_only_after_accepted_status','meteor_actor_xy_range10_S39_type8','meteor_default_start_class1_first_target_snapshot','meteor_missing_first_fallback_never_chooses_second','meteor_Hit_diagnostic_only']),'primary contract differs');require(not C.exists(),'candidate exists')
 render=T/'templates/native_decay71_render_tests.rs';require(not render.exists(),'render template exists');s=(T/'templates/native_mass69_render_tests.rs').read_text('utf-8-sig').replace('mass69_render','decay71_render').replace('original_mass_teleport_visual','original_decay_visual').replace('2236','2232').replace('836','838').replace('455','1952');render.write_text(s,encoding='utf-8',newline='\n')
 changes={
 'native_skills.rs':[
  ('64 | 65 | 66 | 77 =>','64 | 65 | 66 | 71 | 74 | 77 =>'),
  ('10|40|46|47|56|64=>3','10|40|46|47|56|64|71=>3'),
  ('51=>1000,27=>3000','51=>1000,27|74=>3000'),
  ('50|56|57|64|72=>500','50|56|57|64|71|72=>500'),
  ('18|57|72=>500','18|57|71|72=>500')],
 'native_skill_selection.rs':[('                        70 =>','                        71 => "#71 藥水霜化術（選取目標，三格內；消耗 2 顆魔法寶石）".into(),\n                        74 => "#74 流星雨（選取目標，十格內；周圍三格）".into(),\n                        70 =>')],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236)','matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236|2232)')],
 'native_additional_finite_effects.rs':[],
 'native_attack_many.rs':[
  ('matches!(h(15),171|1804|1805|3924)','matches!(h(15),171|1804|1805|3924|762)'),
  ('matches!(gfx,1804|1805|3924)','matches!(gfx,1804|1805|3924|762)'),
  ('matches!(attack.graphics,171|1804|1805|3924)','matches!(attack.graphics,171|1804|1805|3924|762)'),
  ('(65,3924u16)]','(65,3924u16),(74,762u16)]'),
  ('1812=>qualify_earthquake,_=>qualify_storm65','1812=>qualify_earthquake,762=>qualify_meteor74,_=>qualify_storm65'),
  ('matches!(a.cue.attack.graphics,1812|3924)','matches!(a.cue.attack.graphics,1812|3924|762)'),
  ('1812=>716,_=>1348','1812=>716,762=>130,_=>1348'),
  ('if a.cue.attack.graphics==3924{"StormPreparationRetired"}else','if a.cue.attack.graphics==762{"MeteorPreparationRetired"}else if a.cue.attack.graphics==3924{"StormPreparationRetired"}else'),
  ('if a.cue.attack.graphics==3924{"StormHit"}else','if a.cue.attack.graphics==762{"MeteorHit"}else if a.cue.attack.graphics==3924{"StormHit"}else')]
 }
 supplements={'native_skills.rs':[T/'templates/native_decay71_meteor74_request_tests.rs'],'native_skill_effects.rs':[render],'native_additional_finite_effects.rs':[T/'templates/native_decay71_definition_tests.rs'],'native_attack_many.rs':[T/'templates/native_meteor74_support.rs',T/'templates/native_meteor74_tests.rs']};prepared={};repeat={'matches!(a.cue.attack.graphics,1812|3924)':4}
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
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{71,74} and len(ids)==78,'request coverage differs');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=78,missing_request_ids=sorted(set(old['missing_request_ids'])-{71,74}),missing_request_count=102)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts changed');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core']for r in facts['definitions_by_id']if r['skill_id']in next_ids];cohort=Path(q['compile_run'])/'cohort.json';templates=[p for rows in supplements.values()for p in rows];cases=sum(p.read_text('utf-8-sig').count('#[test]')for p in templates);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=')for p in templates);require((len(files),cases,ignored)==(42,14,2),'case/overlay count differs');pins={str(p):sha(p)for p in [Path(__file__).resolve(),*templates]}
 contract=dict(schema='E149-decay71-meteor74-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['contract'],predecessor_candidate_sha256=sha(q['contract']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],preparation_pins=pins,modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=14,new_resource_opt_in_cases=2,previous_written_deferred_cases=372,decay71_request_and_finite_visual_integrated=True,meteor74_request_and_class1_first_target_integrated=True,server_only_cost_probability_and_status=True,relative_graphics_hurt_binding_complete=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),overlays=len(files),new_cases=cases,recognized=78,missing=102,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
