"""Isolated #70 four-node integration plus #69 unintegrated protocol/ownership core."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-mass69-fire70-20261025-001500';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development phase required');ref=w['current_E149_shape67_protection68_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);base=read(q['contract']);require(sha(q['contract'])==q['contract_sha256'],'contract changed');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()});inspection=V/'native-E149-mass69-fire70-review/20261024-160656-026436/review.json';require(sha(inspection)=='4EC05C6785BEBCA208ED119ED3CAA270EB5E4DDEE0D1232DEB916253F378141D','review differs');proof=read(inspection);require([g['world_offset']for g in proof['fire70_group']]==[[1,1],[-1,-1],[-1,1],[1,-1]],'native group differs');require(not C.exists(),'candidate exists')
 render=T/'templates/native_mass69_render_tests.rs';require(not render.exists(),'render template exists');s=(T/'templates/native_protection68_render_tests.rs').read_text('utf-8-sig').replace('protection68_render','mass69_render').replace('original_protection_visual','original_mass_teleport_visual').replace('original228/132','original2236/836').replace('228','2236').replace('132','836').replace('830','455');render.write_text(s,encoding='utf-8',newline='\n')
 changes={
 'native_skills.rs':[('54 | 59 | 62 | 72 |','54 | 59 | 62 | 70 | 72 |'),('match skill {65=>2000','match skill {65|70=>2000')],
 'native_skill_selection.rs':[('54|59|60|62|72|','54|59|60|62|70|72|'),('                        67 =>','                        70 => "#70 火風暴（自己周圍三格）".into(),\n                        67 =>')],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230|2177|760|228)','matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236)')],
 'native_attack_many.rs':[
  ('matches!(h(15),757|758|1812)','matches!(h(15),757|758|1812|1819)'),
  ('struct History {blizzard:','struct History {firestorm:super::native_fire70::History,blizzard:'),
  ('.chain(self.blizzard.active_sequences())','.chain(self.blizzard.active_sequences()).chain(self.firestorm.active_sequences())'),
  ('self.blizzard.merge_admission(&other.blizzard);','self.blizzard.merge_admission(&other.blizzard);self.firestorm.merge_admission(&other.firestorm);'),
  ('pub struct Effects {blizzard:','pub struct Effects {firestorm:super::native_fire70::Stage,blizzard:'),
  ('History{blizzard:self.blizzard.history(),','History{firestorm:self.firestorm.history(),blizzard:self.blizzard.history(),'),
  ('self.blizzard.restore(h.blizzard,same_source);','self.blizzard.restore(h.blizzard,same_source);self.firestorm.restore(h.firestorm,same_source);'),
  ('        self.blizzard.prepare(root,w,cancel,shared);','        self.blizzard.prepare(root,w,cancel,shared);self.firestorm.prepare(root,w,cancel,shared);'),
  ('if c.attack.graphics==757{continue;}','if matches!(c.attack.graphics,757|1819){continue;}'),
  ('self.active.len().saturating_add(self.blizzard.active_casts())','self.active.len().saturating_add(self.blizzard.active_casts()).saturating_add(self.firestorm.active_casts())'),
  ('        self.blizzard.update(w,events.into_iter(),now,self.active.len());','        self.blizzard.update(w,events.iter().copied(),now,self.active.len().saturating_add(self.firestorm.active_casts()));\n        self.firestorm.update(w,events.into_iter(),now,self.active.len().saturating_add(self.blizzard.active_casts()));'),
  ('        let mut published_audio=self.events.clone();','        let(mut firestorm,firestorm_audio)=self.firestorm.draw_to(target,projection);\n        let mut published_audio=self.events.clone();'),
  ('        (json!({"started":self.started,"hits":self.hits,','        if published_audio.len().checked_add(firestorm_audio.len()).is_some_and(|n|n<=32){published_audio.extend(firestorm_audio);}else{firestorm["audio_batch_omitted"]=json!(true);firestorm["audio_no_retry"]=json!(true);}\n        (json!({"started":self.started,"hits":self.hits,'),
  ('"blizzard757":blizzard,"entered_events":','"blizzard757":blizzard,"firestorm1819":firestorm,"entered_events":')]
 }
 supplements={'native_skills.rs':[T/'templates/native_fire70_request_tests.rs'],'native_skill_effects.rs':[render]};prepared={}
 for short,delta in changes.items():
  s=Path(paths[PREFIX+short]).read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==1,'source anchor differs '+short+': '+a[:100]+' count='+str(s.count(a)));s=s.replace(a,b)
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig')
  prepared[short]=s
 modules={};module_sources={}
 for name,template,test in [('native_mass69','native_mass69.rs','native_mass69_tests.rs'),('native_fire70','native_fire70_forest.rs','native_fire70_forest_tests.rs')]:module_sources[name]=(T/'templates'/template,T/'templates'/test,(T/'templates'/template).read_text('utf-8-sig')+'\n'+(T/'templates'/test).read_text('utf-8-sig'))
 C.mkdir();overrides={}
 for short,s in prepared.items():
  name=PREFIX+short;previous=Path(paths[name]);p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(previous),predecessor_sha256=sha(previous),changes=changes[short],supplements=[dict(path=str(t),sha256=sha(t))for t in supplements.get(short,[])]);files[name]=sha(p);paths[name]=str(p)
 for name,(template,test,s)in module_sources.items():
  key=PREFIX+name+'.rs';p=C/key;p.write_text(s,encoding='utf-8',newline='\n');modules[name]=dict(path=str(p),sha256=sha(p),template=str(template),template_sha256=sha(template),tests=str(test),tests_sha256=sha(test));files[key]=sha(p);paths[key]=str(p)
 s=prepared['native_skills.rs'];b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{70} and len(ids)==75 and 69 not in ids,'#69 must remain unintegrated');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=75,missing_request_ids=sorted(set(old['missing_request_ids'])-{70}),missing_request_count=105)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts changed');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core']for r in facts['definitions_by_id']if r['skill_id']in next_ids];cohort=Path(q['compile_run'])/'cohort.json';templates=[v[0]for v in module_sources.values()]+[v[1]for v in module_sources.values()]+[p for rows in supplements.values()for p in rows];cases=sum(p.read_text('utf-8-sig').count('#[test]')for p in templates);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=')for p in templates);require((len(files),cases,ignored)==(42,24,2),'case/overlay count differs')
 pins={str(p):sha(p)for p in [Path(__file__).resolve(),*templates,Path(base['effective_overlay_paths'][PREFIX+'native_blizzard59.rs']),Path(base['effective_overlay_paths'][PREFIX+'native_additional_finite_effects.rs'])]};contract=dict(schema='E149-mass69-core-fire70-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['contract'],predecessor_candidate_sha256=sha(q['contract']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],preparation_pins=pins,modules=modules,runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=24,new_resource_opt_in_cases=2,previous_written_deferred_cases=338,mass69_components_source_complete=True,mass69_controller_or_GUI_integrated=False,mass69_request_registered=False,fire70_request_and_four_node_presentation_integrated=True,fire70_root_ordinal=3,fire70_child_targets_absent=True,fire70_offsets=[g['world_offset']for g in proof['fire70_group']],per_node_constructor_clock_skew_qualified=False,direction_initializer_execution_qualified=False,global_scheduler_interleaving_qualified=False,relative_graphics_hurt_binding_complete=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),overlays=len(files),new_cases=cases,recognized=75,missing=105,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
