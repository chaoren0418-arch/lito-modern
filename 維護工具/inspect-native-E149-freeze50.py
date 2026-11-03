"""Read-only per-skill bytecode and raw definitions; never loads game classes."""
from pathlib import Path
from datetime import datetime,timezone
import re,json,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(p):
 s=Path(p).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M))
 return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected')
 ref=w['current_E149_sun46_weak47_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'predecessor differs');q=read(ref['qualification']);check_pins(q['source_pins'])
 out=V/'native-E149-freeze50-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs')
 names=['useFreezeAttackSpell','useClassicWorldFreezeAttackSpell','repair020$lambda$useFreezeAttackSpell$343','repair020$lambda$useFreezeAttackSpell$344','lambda$useClassicWorldFreezeAttackSpell$341','lambda$useClassicWorldFreezeAttackSpell$342','placeBattleFreezeEffect','placeClassicWorldFreezeEffect','publishClassicWorldFreezePalette','useSummonSpell','nativeMaterialSummonBody']
 selected=[(h,s) for h,s in methods(source) if any(' '+n+'(' in h for n in names)];require(len(selected)==len(names),'exact method set differs')
 for i,(h,s) in enumerate(selected):(out/('method-'+str(i)+'.txt')).write_text(s,encoding='utf-8')
 battle=next(s for h,s in selected if ' useFreezeAttackSpell(' in h);world=next(s for h,s in selected if ' useClassicWorldFreezeAttackSpell(' in h)
 for body,checks in [(battle,{4:'Protocol.attackSpellRequest:',505:'ifne',796:'FreezeAttackSpell.maximumRange:',2560:'Protocol.attackSpell:',3414:'Protocol.paralysis:'}),(world,{7:'Protocol.attackSpellRequest:',612:'FreezeAttackSpell.maximumRange:',1639:'Protocol.attackSpell:',2434:'Protocol.attackSpell:',3019:'Protocol.paralysis:'})]:
  for at,needle in checks.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(needle),body,re.M),'bytecode offset differs '+str(at))
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal differs')
 with zipfile.ZipFile(jar) as z:_,indys=bootstraps(z.read('local/lito381/server/ClientSession.class'))
 refs={int(n) for s in [battle,world] for n in re.findall(r'invokedynamic\s+#(\d+),[^\n]*Predicate;',s)};bindings={str(n):indys[n] for n in sorted(refs)}
 expected={34803:'lambda$useClassicWorldFreezeAttackSpell$341',34809:'lambda$useClassicWorldFreezeAttackSpell$342',45617:'repair020$lambda$useFreezeAttackSpell$343',45622:'repair020$lambda$useFreezeAttackSpell$344'}
 require(refs==set(expected) and all(any(h['name']==expected[n] for h in bindings[str(n)]['method_handles']) for n in refs),'actual dispatch differs');(out/'target-bootstraps.json').write_bytes(encoded(bindings))
 protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';require(sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D','protocol differs')
 parsers=[(h,s) for h,s in methods(protocol) if any(' '+n+'(' in h for n in ['attackSpellRequest','attackSpell','summonSpellRequest'])];require(len(parsers)==3,'protocol methods differ')
 for i,(h,s) in enumerate(parsers):(out/('protocol-'+str(i)+'.txt')).write_text(s,encoding='utf-8')
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');raw=text.read_bytes();defs={}
 for id in [756,219]:
  m=re.search(rb'(?m)^#'+str(id).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'definition absent');p=out/(str(id)+'.definition.txt');p.write_bytes(m.group());defs[str(id)]=dict(path=str(p),sha256=sha(p))
 require(b'109.effect(4 219)' in Path(defs['756']['path']).read_bytes() and not any(c in Path(row['path']).read_bytes() for row in defs.values() for c in [b'[',b'<',b'!']),'50 class4 silent chain differs')
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip() for l in sql.open(encoding='utf-8-sig') if re.match(r'INSERT INTO `skills` VALUES \((50|51),',l)];require(len(labels)==2 and "'冰矛圍籬'" in labels[0], 'labels differ');(out/'readonly-names.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8')
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in out.iterdir() if p.is_file()};pins.update({str(p):sha(p) for p in [source,protocol,jar,T/'native_E149_classfile_bootstraps.py',T/'native_E149_classfile_scan.py']})
 record=dict(schema='E149-freeze50-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref,source_pins=pins,external_readonly_pins={str(text):sha(text),str(sql):sha(sql)},definitions=[d for d in q['next_definitions'] if d['skillId']==50],raw_definitions=defs,target_bootstraps=bindings,wire_request='14 06 01 + actor_u32LE + x_u16LE + y_u16LE',S23_wire_type=0,native_class=4,secondary_graphics=219,flight_image_count=8,loaded_folded_banks=4,flight_events=[],secondary_events=[],server_freeze_seconds_base=16,server_freeze_authority_only=True,summon51_requires_distinct_selection_field=True,summon51_not_implemented_in_this_stage=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)
 require(len(record['definitions'])==1,'definition selection differs');(out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
