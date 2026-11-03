"""Resolve literal summon choices through the two original bytecode helpers."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_scan import parse
T=R/'維護工具';V=R/'驗證紀錄/current';OUT=V/'native-E149-summon51-inspection/20261024-105000'
def methods(p):
 s=Path(p).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected');ref=w['current_E149_freeze50_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'predecessor differs');q=read(ref['qualification']);check_pins(q['source_pins'])
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal differs');javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap differs')
 receipt=read(OUT/'javap-invocation.json');require(receipt['formal_sha256']==FORMAL and receipt['executable_sha256']==sha(javap),'original invocation differs')
 catalog=dict(methods(OUT/'BattleSpellCatalog.stdout.txt'));helper=next(s for h,s in catalog.items() if ' summon(' in h);special=next(s for h,s in catalog.items() if ' summonSpecial(' in h)
 for body,checks in [(helper,{12:'iconst_0',13:'iload',15:'iconst_0',16:'SummonChoice."<init>":(IIIIIZZ)V'}),(special,{12:'bipush        7',14:'iconst_0',15:'iconst_1',16:'SummonChoice."<init>":(IIIIIZZ)V'})]:
  for at,needle in checks.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(needle),body,re.M),'helper argument differs '+str(at))
 choice=(OUT/'BattleSpellCatalog$SummonChoice.stdout.txt').read_text('utf-8-sig');require('int effectivePetCost();' in choice and 'Math.addExact:(II)I' in choice and 'Field petCostAdjustment:I' in choice,'capacity accessor differs')
 cq=read(w['current_E149_skill_catalog_development']['qualification']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog facts differ');facts=read(cq['facts']);table=next(v for k,v in facts['static_maps'].items() if '.SUMMON_CHOICES:' in k);entries=table['value']['entries'];require(len(entries)==31,'summon count differs')
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels={}
 for line in sql.open(encoding='utf-8-sig'):
  m=re.match(r"INSERT INTO `npc` VALUES \((812[0-9]{2}), '([^']*)',",line)
  if m and 81210<=int(m[1])<=81240:labels[int(m[1])]=m[2]
 require(len(labels)==31,'NPC labels absent');rows=[]
 for row in entries:
  require(row['kind']=='opaque_entry' and row['member']['owner']=='local/lito381/server/BattleSpellCatalog','helper owner differs');a=[v['value'] for v in row['arguments']];name=row['member']['name'];require(name in ['summon','summonSpecial'],'unknown helper')
  expected=helper if name=='summon' else special;adjust=0 if name=='summon' else 7;direct=bool(a[4]) if name=='summon' else False
  rows.append(dict(selection=a[0],npc_id=a[1],required_level=a[2],pet_cost=a[3],pet_cost_adjustment=adjust,effective_pet_cost=a[3]+adjust,direct_choice=direct,exclusive=name=='summonSpecial',base_label=labels[a[1]],initializer_call_offset=row['offset'],helper=name))
 require([x['npc_id'] for x in rows]==list(range(81210,81241)) and len({x['selection'] for x in rows})==31 and [x['effective_pet_cost'] for x in rows[-3:]]==[49,49,51],'summon mapping differs')
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs')
 selected=[(h,s) for h,s in methods(source) if any(' '+n+'(' in h for n in ['useSummonSpell','nativeMaterialSummonBody','closeSummonList','publishSummonedCompanion'])];require(len(selected)==6,'summon overload set differs')
 for i,(h,s) in enumerate(selected):(OUT/('summon-method-'+str(i)+'.txt')).write_text(s,encoding='utf-8')
 body=next(s for h,s in selected if ' nativeMaterialSummonBody(' in h)
 for at,needle in {1:'Protocol.summonSpellRequest:',48:'controlAbilityEnabled:',59:'SummonSpellRequest.selection:',105:'SummonSpell.directGrade:',586:'NativeSummonCastAction.admit:',778:'Protocol.objectAction:',791:'Protocol.effectObject:',1015:'SummonSpell.mapFailureMessage:',1099:'SummonSpell.levelFailureMessage:',1341:'SummonSpell.capacityFailureMessage:',1504:'publishSummonedCompanion:'}.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(needle),body,re.M),'summon offset differs '+str(at))
 (OUT/'choices.json').write_bytes(encoded(rows));(OUT/'helper-summon.txt').write_text(helper,encoding='utf-8');(OUT/'helper-summonSpecial.txt').write_text(special,encoding='utf-8')
 (OUT/'original-reader-failure.txt').write_text('Original tool e9eea9 exit1 after all three javap commands exited0: UnicodeDecodeError while decoding catalog as cp950. Original stdout/stderr kept. Readback393fc9 used strict UTF-8; no javap rerun, game class load or test.\n',encoding='utf-8')
 prior=read(q['contract']);primary=read(prior['inspection']);protocol=next(Path(p) for p in primary['source_pins'] if p.endswith('Protocol.stdout.txt'));parser=next(s for h,s in methods(protocol) if ' summonSpellRequest(' in h);require('invalid native summon C_USE_SPELL layout' in parser and 'Method remaining:()I' not in parser and 'Protocol$Reader.d:()J' in parser,'seven-byte parser differs');(OUT/'summon-protocol.txt').write_text(parser,encoding='utf-8')
 (OUT/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in OUT.iterdir() if p.is_file()};pins.update({str(p):sha(p) for p in [source,jar,protocol,Path(cq['facts'])]});record=dict(schema='E149-summon51-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref,source_pins=pins,external_readonly_pins={str(sql):sha(sql),str(javap):sha(javap)},definitions=[d for d in q['next_definitions'] if d['skillId']==51],choices=rows,choices_file=str(OUT/'choices.json'),wire='14 06 02 + selection_u32LE; exact7bytes',control_ability=5,non_controlled_selection_server_direct_grade=True,special_capacity_adjustment=7,material_before_action=True,action_before_map_level_capacity_failure=True,original_javap_exit_codes=[0,0,0],original_invocation_tool_chunk='e9eea9',original_outer_reader_exit_code=1,corrected_UTF8_readback_tool_chunk='393fc9',tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)
 (OUT/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(OUT/'inspection.json'),sha256=sha(OUT/'inspection.json'),choices=31,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
