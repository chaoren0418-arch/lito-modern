"""Build an unselected #59 source projection, preserving every predecessor."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-blizzard59-20261024-120000';PREFIX='LiTo381-Modern/src/modern_lito/'
def prepare():
 w=read(R/'工作佇列.json');ref=w['current_E149_berserk55_disease56_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);check_pins(q['source_pins']);base=read(q['candidate']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);inspection=V/'native-E149-blizzard59-inspection/20261024-115029-671040/inspection.json';proof=read(inspection);check_pins(proof['source_pins'])
 require(not C.exists(),'candidate exists');C.mkdir();changes={
 'native_skills.rs':[('49 | 52 | 53 | 54 | 72','49 | 52 | 53 | 54 | 59 | 72'),('53=>1500','53|59=>1500')],
 'native_skill_selection.rs':[('49|52|53|54|60','49|52|53|54|59|60'),('                        55 =>','                        59 => "#59 暴風雪（自己周圍十格；消耗由伺服器判定）".into(),\n                        55 =>')],
 'native_attack_many.rs':[
 ('(h(15)==758&&b[17]==0&&b[1]==18)','(matches!(h(15),757|758)&&b[17]==0&&b[1]==18)'),
 ('pub(crate) struct History {generation:','pub(crate) struct History {blizzard:super::native_blizzard59::History,generation:'),
 ('#[derive(Default)]\npub struct Effects {generation:','impl History {\n    pub(crate) fn active_sequences(&self)->BTreeSet<u64>{self.active.iter().filter(|a|!a.finished).map(|a|a.cue.action_sequence).chain(self.blizzard.active_sequences()).collect()}\n    pub(crate) fn merge_admission(&mut self,other:&Self){if self.generation==other.generation{self.seen.extend(other.seen.iter().copied());self.blizzard.merge_admission(&other.blizzard);}}\n}\n#[derive(Default)]\npub struct Effects {blizzard:super::native_blizzard59::Stage,generation:'),
 ('History{generation:self.generation','History{blizzard:self.blizzard.history(),generation:self.generation'),
 ('pub(crate) fn restore_history(&mut self,h:History){self.generation=h.generation;self.seen=h.seen;self.active=h.active;self.started=h.started;self.hits=h.hits;self.omitted=h.omitted;}','pub(crate) fn restore_history(&mut self,h:History){self.restore_history_with_source(h,true);}\n    pub(crate) fn restore_history_with_source(&mut self,h:History,same_source:bool){self.generation=h.generation;self.seen=h.seen;self.active=if same_source{h.active}else{vec![]};self.started=h.started;self.hits=h.hits;self.omitted=h.omitted;self.blizzard.restore(h.blizzard,same_source);}'),
 ('        if self.error.is_some(){return;}','        self.blizzard.prepare(root,w,cancel,shared);\n        if self.error.is_some(){return;}'),
 ('        self.events.clear();self.non_audio.clear();','        let events:Vec<_>=events.collect();\n        self.events.clear();self.non_audio.clear();'),
 ('w.many_attacks.iter().map(|c|(c.attack.caster,c.action_sequence)).collect();self.seen','w.many_attacks.iter().map(|c|(c.attack.caster,c.action_sequence)).chain(self.active.iter().map(|a|(a.cue.attack.caster,a.cue.action_sequence))).collect();self.seen'),
 ('for e in events.filter(|e|e.event=="Hit")','for e in events.iter().copied().filter(|e|e.event=="Hit")'),
 ('            if !self.seen.insert(key){continue;}','            if c.attack.graphics==757{continue;}\n            if !self.seen.insert(key){continue;}'),
 ('if self.active.len()>=64||','if self.active.len().saturating_add(self.blizzard.active_casts())>=64||'),
 ('        impacts\n    }','        self.blizzard.update(w,events.into_iter(),now,self.active.len());\n        impacts\n    }'),
 ('        self.active.retain(|a|!a.finished);\n        (json!','        self.active.retain(|a|!a.finished);\n        let(mut blizzard,blizzard_audio)=self.blizzard.draw_to(target,projection);\n        let mut published_audio=self.events.clone();\n        if published_audio.len().checked_add(blizzard_audio.len()).is_some_and(|n|n<=32){published_audio.extend(blizzard_audio);}else{blizzard["audio_batch_omitted"]=json!(true);blizzard["audio_no_retry"]=json!(true);}\n        (json!'),
 ('"effects":rows,"entered_events":self.events','"effects":rows,"blizzard757":blizzard,"entered_events":published_audio'),
 ('"complete":false}),self.events.clone())','"complete":false}),published_audio)')],
 'native_world_scene.rs':[
 ('.chain(self.class1.active_sequences()).collect()', '.chain(self.class1.active_sequences()).chain(self.many.active_sequences()).collect()'),
 ('self.class1.merge_admission(&other.class1);self.animation','self.class1.merge_admission(&other.class1);self.many.merge_admission(&other.many);self.animation'),
 ('        if history.restore_other_effects {','        self.many_effects.restore_history_with_source(history.many,history.restore_other_effects);\n        if history.restore_other_effects {'),
 ('            self.many_effects.restore_history(history.many);\n','')]
 }
 supplements={'native_skills.rs':[T/'templates/native_blizzard59_request_tests.rs']};overrides={}
 for short,delta in changes.items():
  name=PREFIX+short;old=Path(paths[name]);s=old.read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==1,'anchor differs '+short+': '+a[:100]);s=s.replace(a,b)
  extra=[]
  for p in supplements.get(short,[]):s+='\n'+p.read_text('utf-8-sig');extra.append(dict(path=str(p),sha256=sha(p)))
  p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');files[name]=sha(p);paths[name]=str(p);overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(old),predecessor_sha256=sha(old),changes=delta,supplements=extra)
 offsets='\nconst OFFSETS:[[i32;2];25]=['+','.join('['+','.join(map(str,r['world_offset']))+']' for r in proof['group'])+'];\n';module_template=T/'templates/native_blizzard59.rs';tests=T/'templates/native_blizzard59_tests.rs';p=C/(PREFIX+'native_blizzard59.rs');p.write_text(module_template.read_text('utf-8-sig')+offsets+'\n'+tests.read_text('utf-8-sig'),encoding='utf-8',newline='\n');modules={'native_blizzard59':dict(path=str(p),sha256=sha(p),template=str(module_template),template_sha256=sha(module_template),generated_suffix=offsets,supplements=[dict(path=str(tests),sha256=sha(tests))])};files[PREFIX+p.name]=sha(p);paths[PREFIX+p.name]=str(p)
 s=Path(paths[PREFIX+'native_skills.rs']).read_text('utf-8-sig');b=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')];ids=set()
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',b,re.M):ids.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}',r'pub fn summon_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'recognition parser differs');ids.update(map(int,re.findall(r'\d+',m[1])))
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{59} and len(ids)==67,'scope exceeds59');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=67,missing_request_ids=sorted(set(old['missing_request_ids'])-{59}),missing_request_count=113)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts differ');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[x['definitions'][0]['core'] for x in facts['definitions_by_id'] if x['skill_id'] in next_ids];cohort=Path(q['compile_run'])/'cohort.json';require(len(files)==35,'projection count differs')
 contract=dict(schema='E149-blizzard59-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),integration_predecessor_cohort=str(cohort),integration_predecessor_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],modules=modules,runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=12,new_resource_opt_in_cases=1,previous_written_deferred_cases=240,per_node_constructor_clock_skew_qualified=False,direction_initializer_execution_qualified=False,relative_graphics_hurt_binding_complete=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False)
 (C/'contract.json').write_bytes(encoded(contract));print(json.dumps(dict(contract=str(C/'contract.json'),sha256=sha(C/'contract.json'),effective_files=len(files),new_cases=12,recognized_requests=67,remaining=113,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
