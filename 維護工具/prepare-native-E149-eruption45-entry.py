"""Stage #45 typed actor request and native menu on the current 33-file candidate."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_scan import parse,instructions
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-eruption45-entry-20261024-094100'
INSPECTION=V/'native-E149-eruption45-entry-inspection/20261024-093643-749788/inspection.json'
SK='LiTo381-Modern/src/modern_lito/native_skills.rs';SEL='LiTo381-Modern/src/modern_lito/native_skill_selection.rs'
def prepare():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'development-only idle expected')
 ref=w['current_E149_eruption45_scene_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'scene differs');q=read(ref['qualification']);check_pins(q['source_pins']);base=read(q['candidate']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==33,'base overlay differs')
 require(sha(INSPECTION)=='E7F8CB3A8963C2045151F17B76FCC72AA5A445032E094B7EA7764E5F4434E697','primary differs');proof=read(INSPECTION);check_pins(proof['source_pins']);require(proof['base_range']==8 and proof['base_reuse_millis']==100 and proof['self_rejected'] and proof['wire_prefix']==[20,5,4] and proof['wire_bytes']==11,'entry source facts differ')
 qa=T/'templates/native_eruption45_entry_tests.rs';text=qa.read_text('utf-8-sig');require(text.count('#[test]')==10 and text.count('#[ignore=')==1,'new QA count differs')
 changes={SK:[('33 | 34 | 38 | 40 | 77 =>','33 | 34 | 38 | 40 | 45 | 77 =>'),('29|30=>8','29|30|45=>8'),('19=>100,35=>200','19|45=>100,35=>200')],SEL:[('                        41 => "#41 造屍術','                        45 => "#45 地裂術（選取目標，八格內）".into(),\n                        41 => "#41 造屍術')]}
 C.mkdir();overrides={}
 for name,delta in changes.items():
  before=Path(paths[name]);s=before.read_text('utf-8-sig')
  for a,b in delta:require(s.count(a)==1,'source anchor differs '+a);s=s.replace(a,b)
  if name==SK:s+=text
  p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');files[name]=sha(p);paths[name]=str(p)
  overrides[name]=dict(path=str(p),sha256=sha(p),predecessor=str(before),predecessor_sha256=sha(before),changes=delta,QA_template=dict(path=str(qa),sha256=sha(qa)) if name==SK else None)
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal jar differs');override_code={}
 with zipfile.ZipFile(jar) as z:
  for name,method,desc in [('ResourceDatabase','value','(IILjava/lang/String;Ljava/lang/String;)I'),('CombatDatabase','bound','(Ljava/lang/String;)Z'),('CombatRules','intField','(IILjava/lang/String;)I')]:
   data=z.read('local/lito381/server/'+name+'.class');c=parse(data);m=next(x for x in c['methods'] if x['name']==method and x['descriptor']==desc);code=m['code']['bytes'];ops=instructions(code,c['cp']);override_code[name]=dict(class_sha256=hashlib.sha256(data).hexdigest().upper(),method=method,descriptor=desc,code_sha256=hashlib.sha256(code).hexdigest().upper(),instructions=ops)
 members=[x.get('member',{}) for x in override_code['ResourceDatabase']['instructions']]
 require(all(any(m.get('owner')=='local/lito381/server/'+owner and m.get('name')==method for m in members) for owner,method in [('CombatDatabase','bound'),('CombatDatabase','value'),('ResourceDatabase','shared'),('CombatRules','intField')]),'override chain differs')
 override_path=C/'ResourceDatabase-static-chain.json';override_path.write_bytes(encoded(dict(source_jar=str(jar),source_jar_sha256=FORMAL,methods=override_code,observed_GAME_values=False,load_or_JDBC_invoked=False)))
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog differs');facts=read(cq['facts']);next_definitions=[x['definitions'][0]['core'] for x in facts['definitions_by_id'] if x['skill_id'] in [46,47]]
 require(len(next_definitions)==2,'next catalog definitions absent')
 coverage=dict(q['current_source_coverage']);recognized=set();s=Path(paths[SK]).read_text('utf-8-sig');body=s[s.index('pub fn request('):s.index('pub fn reuse_millis(')]
 for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',body,re.M):recognized.update(map(int,re.findall(r'\d+',m[1])))
 for pat in [r'ensure!\(matches!\(skill, ([0-9 |]+)\), "此地面技能尚未接入"\)',r'pub fn item_targeted\(skill:u16\)->bool \{matches!\(skill,([0-9|]+)\)\}']:
  m=re.search(pat,s);require(m is not None,'request parser differs');recognized.update(map(int,re.findall(r'\d+',m[1])))
 require(recognized==set(coverage['recognized_ids'])|{45} and len(recognized)==60,'request changes exceed45');coverage.update(request_source=paths[SK],request_source_sha256=files[SK],recognized_ids=sorted(recognized),recognized_count=60,missing_request_ids=sorted(set(coverage['missing_request_ids'])-{45}),missing_request_count=120)
 predecessor_cohort=Path(q['compile_run'])/'cohort.json';z=read(w['current_E149_zombie41_development']['qualification']);integrations=Path(z['compile_run'])/'cohort.json';require(len(read(integrations)['paired_tests'])==7,'retained integrations differ')
 contract=dict(schema='E149-eruption45-entry-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref['qualification'],predecessor_qualification_sha256=ref['qualification_sha256'],predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(predecessor_cohort),predecessor_metadata_cohort_sha256=sha(predecessor_cohort),
  integration_predecessor_cohort=str(integrations),integration_predecessor_cohort_sha256=sha(integrations),inspection=str(INSPECTION),inspection_sha256=sha(INSPECTION),override_chain=str(override_path),override_chain_sha256=sha(override_path),original_pins={},modules={},runtime_overrides=overrides,effective_overlay_files=files,effective_overlay_paths=paths,modules_integrated_into_scene=True,request_registered=True,
  current_source_coverage=coverage,wire_prefix=[20,5,4],wire_bytes=11,base_range=8,base_reuse_millis=100,literal_definition=proof['literal_definition'],next_definitions=next_definitions,written_deferred_cases=10,new_resource_opt_in_cases=1,previous_written_deferred_cases=175,
  retained_hurt_source_incomplete=True,actual_GAME_overrides_observed=False,local_cost_or_damage_predicted=False,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False)
 (C/'contract.json').write_bytes(encoded(contract));print(json.dumps(dict(contract=str(C/'contract.json'),sha256=sha(C/'contract.json'),effective_files=33,recognized_requests=60,remaining=120,new_tests_written=10,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
