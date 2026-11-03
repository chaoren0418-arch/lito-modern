"""Isolated #75 exact322 owned consent and #76 S37 finite source development only."""
from pathlib import Path
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-greater75-massslow76-20261025-014000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development phase required');ref=w['current_E149_decay71_meteor74_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);base=read(q['contract']);require(sha(q['contract'])==q['contract_sha256'],'contract changed');files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);check_pins({paths[n]:h for n,h in files.items()})
 inspection=V/'native-E149-greater75-massslow76-review/20261024-171220-650978/review.json';require(sha(inspection)=='40CD2E4336862688CFC7D7B1AED4377D5599C3F597BB73546BC73F6DB77355FD','review differs');proof=read(inspection);require(all(proof[n]for n in ['greater75_ask322_empty_NUL_four_bytes','normal61_ask321_three_bytes_preserved','answer_question_id_and_empty_text_checked_by_server','massslow76_S37_at_primary_target_only_after_positive_success_count']),'primary contract differs');require(not C.exists(),'candidate exists')
 control=T/'templates/native_greater75_controller_tests.rs';require(not control.exists(),'controller template exists');source=Path(paths[PREFIX+'native_login.rs']).read_text('utf-8-sig');start=source.index('#[cfg(test)]mod resurrection61_controller_development_tests {');helper=source[start:source.index('\n #[test]',start)].replace('resurrection61_controller_development_tests','greater75_controller_development_tests');helper+='''
 #[test]fn recipient_accepts_exact322_without_learned_spell_or_MP_and_interrupt_revokes(){let mut c=ready();c.state.world.learned_skills.clear();c.state.world.mp=Some([0,100]);c.accept_game(&[36,66,1,0]).unwrap();assert_eq!(c.resurrection_prompt().unwrap().question_id(),322);let first=c.resurrection_prompt().unwrap().token();c.interrupt_world_actions();assert!(c.resurrection_prompt().is_none());c.accept_game(&[36,65,1]).unwrap();assert_ne!(first.command_id(true),c.resurrection_prompt().unwrap().token().command_id(true));}
 #[test]fn greater_effect_cannot_acknowledge_dead_life_or_invent_hp_or_experience(){let mut c=ready();c.accept_game(&[36,66,1,0]).unwrap();let own=c.state.world.own_actor().unwrap().clone();let mut effect=vec![55];effect.extend(own.id.to_le_bytes());effect.extend(3944_u16.to_le_bytes());c.accept_game(&effect).unwrap();assert_eq!(c.state.world.hp,Some([0,100]));assert_eq!(c.resurrection_prompt().unwrap().question_id(),322);let mut revive=vec![17];revive.extend(own.id.to_le_bytes());revive.push(0);revive.extend(2_u32.to_le_bytes());revive.extend(own.graphics.to_le_bytes());c.accept_game(&revive).unwrap();assert!(c.resurrection_prompt().is_none());assert_eq!(c.state.world.hp,Some([0,100]));}
}
''';control.write_text(helper,encoding='utf-8',newline='\n')
 changes={
 'native_skills.rs':[('71 | 74 | 77 =>','71 | 74 | 76 | 77 =>'),('41 | 61 =>','41 | 61 | 75 =>'),('27|74=>3000','27|74|76=>3000')],
 'native_skill_selection.rs':[('if !matches!(skill,41|61)','if !matches!(skill,41|61|75)'),('                        71 =>','                        75 => "#75 終極返生術（選取屍體，畫面內；消耗 1 顆魔法寶石，玩家須同意）".into(),\n                        76 => "#76 集體緩速術（選取目標，十格內；消耗 2 顆魔法寶石）".into(),\n                        71 =>')],
 'native_resurrection61.rs':[
  ('Exact S24/321 consent','Exact S24/321 and S24/322 consent'),
  ('struct Token {serial:u16,owner:Owner}','struct Token {serial:u16,question:u16,owner:Owner}'),
  ('    pub(crate) fn waiting(&self)->bool{self.waiting}','    pub(crate) fn waiting(&self)->bool{self.waiting}\n    pub(crate) fn question_id(&self)->u16{self.token.question}'),
  ('if id!=321{return Ok(None);}ensure!(body.len()==3,"resurrection ask tail");Ok(Some(id))','if !matches!(id,321|322){return Ok(None);}ensure!((id==321&&body.len()==3)||(id==322&&body==[0x24,0x42,0x01,0]),"resurrection ask layout");Ok(Some(id))'),
  ('let Some(_)=id else','let Some(question)=id else'),
  ('Token{serial:self.serial,owner}','Token{serial:self.serial,question,owner}'),
  ('vec![0x23,0x41,0x01,u8::from(yes),0]','vec![0x23,token.question as u8,(token.question>>8)as u8,u8::from(yes),0]'),
  ('[36,66,1,0]','[36,67,1,0]')],
 'native_login_window.rs':[('else{"收到復活詢問。是否同意？"}','else if p.question_id()==322{"收到終極返生術詢問。是否同意？"}else{"收到復活詢問。是否同意？"}')],
 'native_login.rs':[],
 'native_additional_finite_effects.rs':[
  ('3944,230,760];','3944,230,760,3932];'),
  ('SPECS.iter().map(|s|Ok((s.sprite,0,qualify_one(c,s)?))).collect()','let mut rows:Vec<_>=SPECS.iter().map(|s|Ok((s.sprite,0,qualify_one(c,s)?))).collect::<Result<_,HonorAssetError>>()?;rows.push((3932,0,qualify_massslow76(c)?));Ok(rows)')],
 'native_skill_effects.rs':[('matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236|2232)','matches!(cue.graphics,3943|2230|3944|230|2177|760|228|2236|2232|3932)')]
 }
 supplements={'native_skills.rs':[T/'templates/native_greater75_massslow76_request_tests.rs'],'native_resurrection61.rs':[T/'templates/native_greater75_session_tests.rs'],'native_login.rs':[control],'native_additional_finite_effects.rs':[T/'templates/native_massslow76_support.rs',T/'templates/native_massslow76_definition_tests.rs'],'native_skill_effects.rs':[T/'templates/native_massslow76_render_tests.rs']};prepared={};repeat={'[36,66,1,0]':2}
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
 old=q['current_source_coverage'];require(ids==set(old['recognized_ids'])|{75,76} and len(ids)==80,'request coverage differs');cov=dict(old);cov.update(request_source=paths[PREFIX+'native_skills.rs'],request_source_sha256=files[PREFIX+'native_skills.rs'],recognized_ids=sorted(ids),recognized_count=80,missing_request_ids=sorted(set(old['missing_request_ids'])-{75,76}),missing_request_count=100)
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'facts changed');facts=read(cq['facts']);next_ids=cov['missing_request_ids'][:2];next_defs=[r['definitions'][0]['core']for r in facts['definitions_by_id']if r['skill_id']in next_ids];cohort=Path(q['compile_run'])/'cohort.json';templates=[p for rows in supplements.values()for p in rows];cases=sum(p.read_text('utf-8-sig').count('#[test]')for p in templates);ignored=sum(p.read_text('utf-8-sig').count('#[ignore=')for p in templates);require((len(files),cases,ignored)==(42,16,1),'case/overlay count differs');pins={str(p):sha(p)for p in [Path(__file__).resolve(),*templates]}
 contract=dict(schema='E149-greater75-massslow76-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['contract'],predecessor_candidate_sha256=sha(q['contract']),predecessor_metadata_cohort=str(cohort),predecessor_metadata_cohort_sha256=sha(cohort),inspection=str(inspection),inspection_sha256=sha(inspection),original_pins=proof['external_readonly_pins'],preparation_pins=pins,modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,current_source_coverage=cov,definitions=proof['definitions'],next_definition_ids=next_ids,next_definitions=next_defs,new_tests_written=16,new_resource_opt_in_cases=1,previous_written_deferred_cases=386,greater75_request_and_owned322_controller_window_integrated=True,normal61_exact321_preserved=True,massslow76_request_and_S37_overlap_integrated=True,server_only_hp_experience_cost_probability_and_status=True,relative_graphics_hurt_binding_complete=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False);p=C/'contract.json';p.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(p),sha256=sha(p),overlays=len(files),new_cases=cases,recognized=80,missing=100,next_ids=next_ids,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
