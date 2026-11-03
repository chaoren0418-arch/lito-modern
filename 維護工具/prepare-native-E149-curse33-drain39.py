"""Stage two actor+XY spell entrances from pinned formal bytecode; execute no game code."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';PRE='LiTo381-Modern/src/modern_lito/';SK=PRE+'native_skills.rs';SEL=PRE+'native_skill_selection.rs'
def methods(path):
 s=Path(path).read_text('utf-8-sig');heads=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return {m[0].split('(')[0].split()[-1]:s[m.start():heads[i+1].start() if i+1<len(heads) else len(s)] for i,m in enumerate(heads)}
def formal_evidence(out,w):
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','ClientSession javap changed');m=methods(source)
 api_path=V/'native-E149-area2253053-contract/inspection.json';api=read(api_path);require(api['formal_target_sha256']==FORMAL and api['game_classes_instantiated']==0 and all(r['exit_code']==0 for r in api['commands']),'Protocol inspection differs');check_pins(api['outputs']);protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';pm=methods(protocol)
 cref=w['current_E149_skill_catalog_development'];require(sha(cref['qualification'])==cref['qualification_sha256'],'catalog changed');cat=read(cref['qualification']);require(sha(cat['facts'])==cat['facts_sha256'],'catalog facts changed');facts=read(cat['facts']);definitions={str(n):next(r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id']==n) for n in [33,39]}
 require(tuple(definitions['33'][k] for k in ['row','column','maximumRange','actionId','effectGfx','reuseDelayMillis','clientCastClockMillis','warningDelayMillis','paralysisDurationMillis'])==(4,0,2,19,746,5000,5000,5000,10000),'#33 definition differs');require(tuple(definitions['39'][k] for k in ['row','column','maximumRange','actionId','effectGfx','reuseDelayMillis','hpCost'])==(4,6,4,19,2171,500,50),'#39 definition differs')
 selected=['useParalysisSpell','useClassicWorldParalysisSpell','sourcePoisonBody$useParalysisSpell','sourcePoisonBody$useClassicWorldParalysisSpell','nativeMaterialParalysisScopeBody','nativeMaterialWorldParalysisScopeBody','lambda$useClassicWorldParalysisSpell$367','useManaDrainSpell','useClassicWorldManaDrainSpell']
 expected={
 'nativeMaterialParalysisScopeBody':{1:'Protocol.attackSpellRequest:',100:'if_acmpeq     118',335:'ParalysisSpell.maximumRange:',1484:'Protocol.objectAction:',1638:'CombatPoisonLifecycle.curseActivate:',1733:'Protocol.effectObject:'},
 'nativeMaterialWorldParalysisScopeBody':{1:'Protocol.attackSpellRequest:',531:'ParalysisSpell.maximumRange:',1646:'Protocol.objectAction:',1840:'CombatPoisonLifecycle.curseActivate:',1874:'CombatMonsterCurseLifecycle.activateWorld:',1919:'Protocol.effectObject:'},
 'lambda$useClassicWorldParalysisSpell$367':{2:'if_acmpeq     29'},
 'useManaDrainSpell':{1:'Protocol.attackSpellRequest:',509:'if_acmpeq     537',1100:'ManaDrainSpell.hpCost:',1103:'if_icmple     1123',1164:'return',1344:'Protocol.objectAction:',2057:'Protocol.effectObject:'},
 'useClassicWorldManaDrainSpell':{1:'Protocol.attackSpellRequest:',223:'ifne          230',226:'aload_0',227:'goto          265',418:'if_acmpeq     429',453:'if_acmpeq     488',1223:'ManaDrainSpell.hpCost:',1226:'if_icmple     1269',1310:'return',1556:'Protocol.objectAction:',2173:'Protocol.effectObject:'}}
 pins={str(source):sha(source),str(api_path):sha(api_path),str(protocol):sha(protocol),str(Path(cat['facts'])):sha(cat['facts']),str(Path(cref['qualification'])):cref['qualification_sha256']}
 for name in selected:
  require(name in m,'exact method missing: '+name);body=m[name]
  for offset,needle in expected.get(name,{}).items():require(re.search(r'^\s*'+str(offset)+r':[^\n]*'+re.escape(needle),body,re.M),'formal offset differs '+name+':'+str(offset))
  p=out/(name+'.javap.txt');p.write_text(body,encoding='utf-8',newline='\n');pins[str(p)]=sha(p)
 for name in ['attackSpellRequest','spellSelector','objectAction','effectObject']:
  p=out/('Protocol.'+name+'.javap.txt');p.write_text(pm[name],encoding='utf-8',newline='\n');pins[str(p)]=sha(p)
 b=pm['attackSpellRequest'];require([b.count('Protocol$Reader.'+n+':') for n in ['c','d','h','remaining']]==[3,1,2,1] and 'invalid native attack C_USE_SPELL layout' in b,'11-byte reader differs')
 body=dict(schema='E149-curse33-drain39-bytecode-source-v1',formal_target_sha256=FORMAL,definitions=definitions,source_pins=pins,method_offset_assertions=expected,request_bytes=11,wire_prefixes={'33':[20,4,0],'39':[20,4,6]},target_payload='unsigned actor u32 little-endian + x/y u16 little-endian',curse_self_rejected=True,drain_self_explicitly_allowed=True,drain_unselected_client_intent='own actor ID and current XY',caster_action=19,target_effect_opcode=55,drain_hp_guard_silent_return=True,no_dedicated_HP_rejection_in_selected_guard=True,curse_PC_warning_and_duration_server_owned=True,curse_NPC_path_uses_server_duration=True,probability_resources_costs_lawful_and_state_server_owned=True,methods_executed=0,tests_executed=0)
 (out/'formal-evidence.json').write_bytes(encoded(body));return out/'formal-evidence.json'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_S23_clock_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous qualification changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'previous candidate changed');base=read(q['candidate']);require(sha(base['contract'])==base['contract_sha256'],'previous contract changed');prior=read(base['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==29 and SK in files and SEL in files,'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 out=owned(T/'candidates'/('native-E149-curse33-drain39-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);proofdir=out/'formal-source';proofdir.mkdir();proof=formal_evidence(proofdir,w)
 template=T/'templates/native_curse33_drain39_tests.rs';qa=template.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',qa))==10 and '#[ignore' not in qa,'QA scope differs')
 drain='''        // Formal #39 explicitly admits self in WORLD and BATTLE. A supplied
        // stale target never becomes self; only an absent selection does.
        39 => {
            let id=target.unwrap_or(own.id);
            let actor=world.actors.get(&id).context("技能目標已離開")?;
            ensure!(world.actor_visible(id)&&!actor.dead
                &&!super::native_obstacles::destroyed_guardian(actor)
                &&own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y))<=4,
                "法術目標不可用或超出範圍");
            b.extend_from_slice(&id.to_le_bytes());
            b.extend_from_slice(&actor.x.to_le_bytes());
            b.extend_from_slice(&actor.y.to_le_bytes());
        }
'''
 changes={SK:[('29 | 30 | 34 | 38 | 77 => {','29 | 30 | 33 | 34 | 38 | 77 => {'),('28=>4,29|30=>8','28=>4,33=>2,29|30=>8'),('        _ => anyhow::bail!("此技能的請求格式尚未接入"),',drain+'        _ => anyhow::bail!("此技能的請求格式尚未接入"),'),('53=>1500,77=>5000','53=>1500,33|77=>5000'),('20|25|29|30|44|49','20|25|29|30|39|44|49'),('match skill {10|35=>200','match skill {33=>5000,10|35=>200')],SEL:[('                        37 => "#37 解除詛咒','                        33 => "#33 木乃伊的詛咒（選取目標，兩格內；消耗 2 顆魔法寶石）".into(),\n                        39 => "#39 魔力奪取（選取目標／自己，四格內）".into(),\n                        37 => "#37 解除詛咒')]}
 (out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  p=Path(paths[name]);s=p.read_text('utf-8-sig');before[name]=sha(p)
  for a,b in replacements:require(s.count(a)==1,'source anchor differs: '+a);s=s.replace(a,b)
  if name==SK:s+=qa
  target=out/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(s,encoding='utf-8',newline='\n');changed[name]=sha(target);files[name]=sha(target);paths[name]=str(target)
 originals=dict(prior['original_pins']);originals.update(read(proof)['source_pins']);originals[str(proof)]=sha(proof)
 contract=dict(schema='E149-curse33-drain39-request-source-v1',formal_target_sha256=FORMAL,source_changes=changes,QA_templates={SK:dict(path=str(template),sha256=sha(template))},native_snapshot_contract=str(proof),native_snapshot_contract_sha256=sha(proof),original_pins=originals,formal_definitions=read(proof)['definitions'],curse_self_rejected=True,drain_self_allowed=True,drain_stale_selection_never_falls_back=True,no_local_HP_MP_materials_probability_or_control_prediction=True,existing_finite_resources_preserved=True,existing_pending_and_owner_fences_preserved=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_catalog_qualification=w['current_E149_skill_catalog_development'],current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=10,new_resource_tests_require_original_root=0,retained_skill_tests_written=109,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),formal_source=str(proof),changed_files=2,effective_files=29,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
