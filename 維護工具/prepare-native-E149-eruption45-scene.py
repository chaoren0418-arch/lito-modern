"""Integrate received S23/129 presentation; keep local #45 request unregistered."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-eruption45-scene-20261024-091703'
INSPECTION=V/'native-E149-eruption45-scene-inspection/20261024-091703-820916/inspection.json'
NAME='LiTo381-Modern/src/modern_lito/native_spell_projectiles.rs'
def prepare():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'development-only idle expected')
 prior=w['current_E149_eruption45_core_development'];require(sha(prior['qualification'])==prior['qualification_sha256'],'core differs');q=read(prior['qualification']);check_pins(q['source_pins']);old_contract=read(q['contract'])
 require(sha(INSPECTION)=='67B3573B5134DE74EFCB237C75C031DA9577EF71BEDC8EF016F53F4A9A9006A8','scene primary differs');proof=read(INSPECTION);check_pins({p:h for p,h in proof['source_pins'].items() if Path(p).is_relative_to(R)});outside={p:h for p,h in proof['source_pins'].items() if not Path(p).is_relative_to(R)};require(outside=={str(Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe')):'09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A'} and all(sha(p)==h for p,h in {**outside,**proof['PE_original_pins']}.items()),'readonly primary/tool changed')
 base=read(old_contract['predecessor_candidate']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==30,'base overlay differs')
 before=Path(paths[NAME]);s=before.read_text('utf-8-sig');changes=[]
 def replace(a,b):
  nonlocal s
  require(s.count(a)==1,'wrapper source anchor differs '+a[:100]);changes.append([a,b]);s=s.replace(a,b)
 replace('    compositor::Canvas,','    compositor::Canvas,\n    native_eruption45_scene as eruption45,')
 replace('matches!(graphics,167|1583|1799|1797)','matches!(graphics,167|1583|1799|1797|129)')
 replace('    launched:u64,arrived:u64,omitted:u64,\n}', '    launched:u64,arrived:u64,omitted:u64,\n    eruption:eruption45::History,\n}')
 replace('.map(|f|f.cue.action_sequence).collect()}','.map(|f|f.cue.action_sequence).chain(self.eruption.active_sequences()).collect()}')
 replace('if self.generation==other.generation {self.seen.extend(other.seen.iter().copied());}', 'if self.generation==other.generation {self.seen.extend(other.seen.iter().copied());self.eruption.merge_admission(&other.eruption);}')
 replace('    generation: Option<u64>,\n    assets:', '    generation: Option<u64>,\n    eruption:eruption45::Stage,\n    assets:')
 replace('omitted:self.omitted}}','omitted:self.omitted,eruption:self.eruption.history()}}')
 replace('self.omitted=h.omitted;','self.omitted=h.omitted;self.eruption.restore_history(h.eruption,same_source);')
 replace('    }\n    pub fn update<\'a>(', '        self.eruption.prepare(root,world,cancel,shared);\n    }\n    pub fn update<\'a>(')
 replace('    ) -> Vec<Impact> {\n        if self.generation', '    ) -> Vec<Impact> {\n        let events:Vec<_>=events.collect();\n        if self.generation')
 replace('        self.flights.retain(|f| valid(world, &f.cue));', '        self.flights.retain(|f| valid(world, &f.cue));\n        self.eruption.update(world,events.iter().copied(),now,self.flights.len());')
 replace('for event in events.filter(|e| e.event == "Hit")', 'for event in events.iter().copied().filter(|e| e.event == "Hit")')
 replace('.find(|c| (c.attacker, c.action_sequence) == key && c.projectile.flight_type == 0)', '.find(|c| (c.attacker, c.action_sequence) == key && c.projectile.flight_type == 0 && c.projectile.graphics!=129)')
 replace('|| self.flights.len() >= 64','|| self.flights.len().saturating_add(self.eruption.active_casts()) >= 64')
 replace('        self.flights.retain(|f|!f.finished);\n        (serde_json::json!', '        self.flights.retain(|f|!f.finished);\n        let(eruption_receipt,eruption_events)=self.eruption.draw_to(target,projection);\n        let mut events=self.events.clone();events.extend(eruption_events);\n        (serde_json::json!')
 replace('"entered_events":self.events,','"entered_events":events,"eruption129":eruption_receipt,')
 replace('}),self.events.clone())','}),events)')
 replace('flights:vec![],launched:0,arrived:0,omitted:0}}','flights:vec![],launched:0,arrived:0,omitted:0,eruption:Default::default()}}')
 wrapper=C/NAME;require(not wrapper.exists(),'do not overwrite prepared wrapper');wrapper.parent.mkdir(parents=True);wrapper.write_text(s,encoding='utf-8',newline='\n')
 adapter=C/'native_eruption45_scene.rs';require(len(re.findall(r'#\[test\]',adapter.read_text('utf-8-sig')))==10 and adapter.read_text('utf-8-sig').count('#[ignore=')==2,'new QA count differs')
 retained={}
 for module,row in old_contract['modules'].items():
  n='LiTo381-Modern/src/modern_lito/'+module+'.rs';files[n]=row['sha256'];paths[n]=row['path'];retained[module]=row
 files[NAME]=sha(wrapper);paths[NAME]=str(wrapper);adapter_name='LiTo381-Modern/src/modern_lito/native_eruption45_scene.rs';files[adapter_name]=sha(adapter);paths[adapter_name]=str(adapter)
 require(len(files)==33,'new overlay count differs')
 contract=dict(schema='E149-eruption45-scene-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=prior['qualification'],predecessor_qualification_sha256=prior['qualification_sha256'],
  predecessor_candidate=old_contract['predecessor_candidate'],predecessor_candidate_sha256=sha(old_contract['predecessor_candidate']),predecessor_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),predecessor_metadata_cohort_sha256=sha(Path(q['compile_run'])/'cohort.json'),
  inspection=str(INSPECTION),inspection_sha256=sha(INSPECTION),original_pins=proof['PE_original_pins'],modules={'native_eruption45_scene':dict(path=str(adapter),sha256=sha(adapter))},retained_modules=retained,
  runtime_overrides={NAME:dict(path=str(wrapper),sha256=sha(wrapper),predecessor=str(before),predecessor_sha256=sha(before),changes=changes)},
  effective_overlay_files=files,effective_overlay_paths=paths,retained_effective_overlay_count=30,new_supplementary_runtime_modules=3,
  modules_integrated_into_scene=True,request_registered=False,current_source_coverage=q['current_source_coverage'],written_deferred_cases=10,previous_written_deferred_cases=165,tests_executed=0,PEs_linked=False,outputs_executed=False,
  pending=['Bind complete original relative graphics hurt family and child guard; source Hit currently diagnostic only.',
   'Finish #45 formal target eligibility/resource overrides and register request/selection; no request coverage credit for presentation alone.',
   'Original optional action0 speed scaling and uint32 timer wrap remain unqualified.',
   'After explicit test resumption, run 10 new cases (2 original-resource opt-in), latest full library and exact S23/GUI/owner/PCM checks.'],whole_goal_complete=False)
 (C/'contract.json').write_bytes(encoded(contract));print(json.dumps(dict(contract=str(C/'contract.json'),sha256=sha(C/'contract.json'),effective_overlay_files=33,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
