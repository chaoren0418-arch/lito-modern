"""Stage formal reveal/taming intents, finite resource definitions and capacity feedback. No tests."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,importlib.util
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';PRE='LiTo381-Modern/src/modern_lito/';SK=PRE+'native_skills.rs';SEL=PRE+'native_skill_selection.rs';FINITE=PRE+'native_additional_finite_effects.rs';WORLD=PRE+'native_world.rs'
spec=importlib.util.spec_from_file_location('previous_source_reader',T/'prepare-native-E149-curse33-drain39.py');prev=importlib.util.module_from_spec(spec);spec.loader.exec_module(prev);methods=prev.methods
INSPECTION=V/'native-E149-reveal23-tame36-inspection/20261024-072834-535466/inspection.json'
def formal_evidence(out,w):
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','ClientSession changed');m=methods(source)
 api_path=V/'native-E149-area2253053-contract/inspection.json';api=read(api_path);require(api['formal_target_sha256']==FORMAL and api['game_classes_instantiated']==0 and all(r['exit_code']==0 for r in api['commands']),'Protocol inspection differs');check_pins(api['outputs']);protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';pm=methods(protocol)
 audit=read(INSPECTION);require(audit['formal_target_sha256']==FORMAL and audit['game_classes_instantiated']==audit['tests_executed']==0 and len(audit['commands'])==3 and all(r['exit_code']==0 for r in audit['commands']),'pet/reveal javap scope differs');check_pins({p:h for r in audit['commands'] for p,h in r['logs'].items()})
 cref=w['current_E149_skill_catalog_development'];require(sha(cref['qualification'])==cref['qualification_sha256'],'catalog changed');cat=read(cref['qualification']);require(sha(cat['facts'])==cat['facts_sha256'],'catalog facts changed');facts=read(cat['facts']);definitions={str(n):next(r['definitions'][0]['core'] for r in facts['definitions_by_id'] if r['skill_id']==n) for n in [23,36]}
 require(tuple(definitions['23'][k] for k in ['row','column','maximumRange','actionId','effectGfx','mpCost','reuseDelayMillis'])==(2,6,-1,19,2510,8,10),'#23 differs');require(tuple(definitions['36'][k] for k in ['row','column','maximumRange','actionId','effectGfx','mpCost','hpCost','reuseDelayMillis','durationSeconds','itemConsumeId','itemConsumeCount'])==(4,3,-1,19,228,30,20,10,3600,40318,1),'#36 differs')
 expected={'useRevealWeaknessSpell':{1:'Protocol.objectSpellRequest:',536:'BattleSpellCatalog.isInNativeScreen:',1163:'Protocol.objectAction:',1176:'Protocol.effectObject:',1184:'RevealWeaknessSpell.weaknessEffects:',1307:'Protocol.effectObject:'},'nativeMaterialTamingBody':{1:'Protocol.attackSpellRequest:',362:'BattleSpellCatalog.isInNativeScreen:',1031:'NativePetMaterialAction.admit:',1304:'Protocol.objectAction:',1626:'sipush        319',1633:'Protocol.messageCode:',1729:'publishClassicWorldTaming:',1810:'publishQuestTaming:',1886:'publishTimeAttackTaming:',1905:'Protocol.effectObject:'},'publishClassicWorldTaming':{13:'Protocol.removeObject:',27:'Protocol.effectObject:',90:'sendClassicWorldCompanionObject:'}}
 selected=['useRevealWeaknessSpell','useTamingSpell','nativeMaterialTamingBody','publishClassicWorldTaming','publishQuestTaming','publishTimeAttackTaming','publishTamedCompanion','sendClassicWorldCompanionObject']
 pins={str(p):sha(p) for p in [source,api_path,protocol,INSPECTION,Path(cat['facts']),Path(cref['qualification']),T/'prepare-native-E149-curse33-drain39.py',R/'LiTo381-Modern/src/modern_lito/native_skill_effects.rs']};pins.update({p:h for r in audit['commands'] for p,h in r['logs'].items()})
 for name in selected:
  require(name in m,'exact method missing: '+name);body=m[name]
  for offset,needle in expected.get(name,{}).items():require(re.search(r'^\s*'+str(offset)+r':[^\n]*'+re.escape(needle),body,re.M),'formal offset differs '+name+':'+str(offset))
  p=out/(name+'.javap.txt');p.write_text(body,encoding='utf-8',newline='\n');pins[str(p)]=sha(p)
 for name in ['objectSpellRequest','attackSpellRequest','spellSelector','objectAction','effectObject','removeObject']:
  p=out/('Protocol.'+name+'.javap.txt');p.write_text(pm[name],encoding='utf-8',newline='\n');pins[str(p)]=sha(p)
 for name,widths in [('objectSpellRequest',[3,1,0,1]),('attackSpellRequest',[3,1,2,1])]:require([pm[name].count('Protocol$Reader.'+n+':') for n in ['c','d','h','remaining']]==widths,'reader differs')
 raw_path=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(raw_path)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','original list differs');rows=raw_path.read_bytes().splitlines();effects=[];specs=[];nums=lambda values:'['+','.join(str(v) for v in values)+']'
 for gfx,sound in [(228,132),(2166,809),(2167,810),(2168,811),(2169,812)]:
  matches=[i for i,row in enumerate(rows) if re.match(rb'#'+str(gfx).encode()+rb'(?:\s|$)',row)];require(len(matches)==1,'sprite block ambiguous');start=matches[0];end=next(i for i in range(start+1,len(rows)) if rows[i].startswith(b'#'));block=rows[start:end];body=b'\n'.join(block)
  frames=[(int(a),int(b),int(t),[] if not s else [int(s)]) for a,b,t,s in re.findall(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',body)];attrs=[(int(k),list(v)) for k,v in re.findall(rb'(\d+)[.](?:type|attr)\(([^)]*)\)',body)]
  require([f[0] for f in frames]==[0]*len(frames) and [f[3] for f in frames]==[[],[sound],*([[]]*(len(frames)-2))],'original sound/image differs');require([f[2] for f in frames]==([4,2,2,4,4,2,2] if gfx==228 else [4]*11) and [f[1] for f in frames]==(list(range(7)) if gfx==228 else [10,*range(10)]),'original cadence differs');require(attrs==([(104,[56])] if gfx==228 else [(102,[48]),(104,[56])]),'original attrs differ')
  effects.append(dict(graphics=gfx,bank=gfx,sound=sound,header=list(block[0]),frames=frames,attributes=attrs,block=[list(l) for l in block],source_line_start=start+1,source_line_end=end))
  fs=','.join('('+','.join([str(a),str(b),str(t),'&'+nums(s)])+')' for a,b,t,s in frames);ats=','.join('('+str(k)+',&'+nums(v)+')' for k,v in attrs);specs.append('Spec{sprite:'+str(gfx)+',bank:'+str(gfx)+',header:&'+nums(block[0])+',attrs:&['+ats+'],frames:&['+fs+']},\n')
 result=dict(schema='E149-reveal23-tame36-bytecode-resource-source-v1',formal_target_sha256=FORMAL,definitions=definitions,source_pins=pins,method_offset_assertions=expected,request_bytes={'23':7,'36':11},wire_prefixes={'23':[20,2,6],'36':[20,4,3]},caster_action=19,target_effect_opcode=55,original_resource=str(raw_path),original_resource_sha256=sha(raw_path),finite_effects=effects,generated_specs=specs,existing_2510_action0_alias149_one_frame_no_audio_preserved=True,native_screen_not_unbounded=True,NPC_domain_and_tamability_server_owned=True,companion_creation_retirement_duration_and_costs_server_owned=True,capacity_message=319,weakness_bits_and_wire_element_order_server_owned=True,no_elements_inferred_from_English_spr_comments=True,methods_executed=0,tests_executed=0)
 (out/'formal-evidence.json').write_bytes(encoded(result));return out/'formal-evidence.json'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_curse33_drain39_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous qualification changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'candidate changed');base=read(q['candidate']);require(sha(base['contract'])==base['contract_sha256'],'contract changed');prior=read(base['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==29,'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 out=owned(T/'candidates'/('native-E149-reveal23-tame36-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);pd=out/'formal-source';pd.mkdir();proof=formal_evidence(pd,w);p=read(proof)
 template=T/'templates/native_reveal23_tame36_tests.rs';qa=template.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',qa))==10 and qa.count('#[ignore=')==1,'QA scope differs')
 branch='''        // -1 in these formal records means native screen, not unlimited range.
        // NPC type, weakness and tamability cannot be inferred from actor names.
        23 | 36 => {
            let id=target.context("請先選取怪物")?;
            let actor=world.actors.get(&id).context("技能目標已離開")?;
            ensure!(id!=own.id&&world.actor_visible(id)&&!actor.dead
                &&!super::native_obstacles::destroyed_guardian(actor)
                &&in_native_screen([own.x,own.y],[actor.x,actor.y]),
                "法術目標不可用或超出畫面");
            b.extend_from_slice(&id.to_le_bytes());
            if skill==36 {b.extend_from_slice(&actor.x.to_le_bytes());b.extend_from_slice(&actor.y.to_le_bytes());}
        }
'''
 changes={SK:[('        // Formal #39 explicitly admits self',branch+'        // Formal #39 explicitly admits self'),('            Self::Rejected(299) => "施法材料不足",','            Self::Rejected(299) => "施法材料不足",\n            Self::Rejected(319) => "可控制的同伴數量已達上限",')],SEL:[('                        33 => "#33 木乃伊的詛咒','                        23 => "#23 能量感測（選取怪物，畫面內）".into(),\n                        36 => "#36 迷魅術（選取怪物，畫面內；消耗 1 顆魔法寶石）".into(),\n                        33 => "#33 木乃伊的詛咒')],FINITE:[('2951,227,870];','2951,227,870,228,2166,2167,2168,2169];'),('const SPECS:&[Spec]=&[\n','const SPECS:&[Spec]=&[\n'+''.join(p['generated_specs']))],WORLD:[('276 | 278 | 280 | 285 | 299 | 316 | 352','276 | 278 | 280 | 285 | 299 | 316 | 319 | 352')]}
 (out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  path=Path(paths[name]);s=path.read_text('utf-8-sig');before[name]=sha(path)
  for a,b in replacements:require(s.count(a)==1,'source anchor differs: '+a);s=s.replace(a,b)
  if name==SK:s+=qa
  dest=out/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(s,encoding='utf-8',newline='\n');changed[name]=sha(dest);files[name]=sha(dest);paths[name]=str(dest)
 originals=dict(prior['original_pins']);originals.update(p['source_pins']);originals[str(proof)]=sha(proof)
 contract=dict(schema='E149-reveal23-tame36-request-finite-source-v1',formal_target_sha256=FORMAL,source_changes=changes,QA_templates={SK:dict(path=str(template),sha256=sha(template))},native_snapshot_contract=str(proof),native_snapshot_contract_sha256=sha(proof),original_pins=originals,formal_definitions=p['definitions'],finite_effects=p['finite_effects'],both_require_explicit_other_actor=True,native_screen_not_unbounded=True,NPC_domain_and_companion_state_server_owned=True,no_local_cost_probability_or_pet_creation=True,existing_2510_and_S23_sources_preserved=True,capacity_message_319_classified=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_catalog_qualification=w['current_E149_skill_catalog_development'],current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=10,new_resource_tests_require_original_root=1,retained_skill_tests_written=119,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),formal_source=str(proof),changed_files=4,effective_files=29,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
