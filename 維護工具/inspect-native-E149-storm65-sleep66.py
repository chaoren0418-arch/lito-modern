"""Readonly classfile/Text/PE inspection for targeted storm and sleep; no game loading."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_bootstraps import bootstraps
from native_E149_classfile_scan import parse,instructions
T=R/'維護工具';V=R/'驗證紀錄/current'

def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M))
 return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]

def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_earthquake62_silence64_development'];require(sha(ref['qualification'])==ref['qualification_sha256'] and w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'phase differs');q=read(ref['qualification']);require(q['next_definition_ids']==[65,66],'next definitions differ')
 out=V/'native-E149-storm65-sleep66-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs')
 names=['useAreaAttackSpell','useClassicWorldAreaAttackSpell','useSleepSpell','useClassicWorldSleepSpell','nativeAreaWorldSleepBody','nativeAreaBattleSleepBody'];selected={h:s for h,s in methods(session) if any(' '+n+'(' in h for n in names)};require(len(selected)==8,'overload set differs')
 for i,(h,s) in enumerate(selected.items()):(out/(str(i)+'-'+h.split('(')[0].split()[-1]+'.txt')).write_text(s,encoding='utf-8',newline='\n')
 def body(name,typed=False):
  found=[s for h,s in selected.items() if ' '+name+'(' in h and (not name.endswith('AreaAttackSpell') or ('(local.' in h)==typed)];require(len(found)==1,'overload ambiguous');return found[0]
 for name,checks in [('useAreaAttackSpell',{2:'Protocol.attackSpellRequest:'}),('useClassicWorldAreaAttackSpell',{2:'Protocol.attackSpellRequest:'}),('nativeAreaWorldSleepBody',{1:'Protocol.attackSpellRequest:',1622:'NativeAreaMaterialAction.admit:',2704:'NativeCancellationSleep.activate:',2881:'Protocol.effectObject:'}),('nativeAreaBattleSleepBody',{1:'Protocol.attackSpellRequest:',2191:'NativeAreaMaterialAction.admit:',3665:'NativeCancellationSleep.activate:',3875:'Protocol.effectObject:'})]:
  for at,text in checks.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(text),body(name),re.M),'formal offset differs '+name+str(at))
 for name,at in [('useAreaAttackSpell',4203),('useClassicWorldAreaAttackSpell',6154)]:require(re.search(r'^\s*'+str(at)+r':[^\n]*Protocol.attackMany:\(IJIIIJIL',body(name,True),re.M),'S39 default type8 caller differs')
 protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';require(sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D','Protocol differs');packets={h:s for h,s in methods(protocol) if ' attackMany(' in h or ' attackSpellRequest(' in h};require(len(packets)==3,'packet overload set differs')
 for i,(h,s) in enumerate(packets.items()):(out/('Protocol-'+str(i)+'.txt')).write_text(s,encoding='utf-8',newline='\n')
 require(any(re.search(r'11: bipush\s+8',s) for h,s in packets.items() if ' attackMany(' in h),'default attackMany type differs')
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal jar differs')
 with zipfile.ZipFile(jar) as z:
  raw=z.read('local/lito381/server/NativeAreaMaterialAction.class');c=parse(raw);_,indys=bootstraps(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp'])) for m in c['methods'] if m['code']];(out/'NativeAreaMaterialAction-Code.json').write_bytes(encoded(rows));(out/'NativeAreaMaterialAction-bootstraps.json').write_bytes(encoded(indys))
  for name in ['worldSleep','battleSleep']:
   method=next(m for m in rows if m['name']==name);require(any(i.get('member',{}).get('name')=='run' for i in method['instructions']),'material owner route absent');target='lambda$'+name+('$2' if name=='worldSleep' else '$3');lambda_method=next(m for m in rows if m['name']==target);require(any(i.get('member',{}).get('name')==('nativeAreaWorldSleepBody' if name=='worldSleep' else 'nativeAreaBattleSleepBody') for i in lambda_method['instructions']),'actual material callback differs');require(any(any(h['name']==target for h in binding['method_handles']) for binding in indys.values()),'material bootstrap absent')
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');data=text.read_bytes();defs={}
 for gfx in [3924,760]:
  m=re.search(rb'(?m)^#'+str(gfx).encode()+rb'[ \t]+.*?(?=^#|\Z)',data,re.S|re.M);require(m is not None,'raw definition absent');p=out/(str(gfx)+'.definition.txt');p.write_bytes(m.group());defs[str(gfx)]=dict(path=str(p),sha256=sha(p))
 require(b'<1348' in Path(defs['3924']['path']).read_bytes() and b'109.effect(1 -1)' in Path(defs['3924']['path']).read_bytes(),'storm class/audio differs')
 class1=V/'native-E149-area2230-contract/20261024-045931-508526/contract.json';old=read(class1);require(all(old[n] for n in ['first_target_snapshot_at_packet_receipt','first_hit_flag_does_not_change_anchor','missing_first_target_fallback_one_step','zero_target_fallback','frozen_until_retirement']),'class1 original flow absent')
 pe=Path(old['PE']);require(sha(pe)==old['PE_sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE differs');source=V/'native-E149-blizzard59-inspection/20261024-115029-671040/start.bin';require(sha(source)=='6937E97AE98ACA01232582E90F70DBD2BA2D656D92E92837105A233B6A318A77','native bytes differ');raw=source.read_bytes();md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);code={i.address:i for i in md.disasm(raw,0x44f350)};pc=0x44f370;equal=None;trace=[]
 while pc!=0x45001a:
  require(len(trace)<40 and pc in code,'native compare bound');i=code[pc];trace.append(dict(address=pc,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str));next_pc=pc+i.size
  if i.mnemonic=='mov':require(i.op_str=='ax, word ptr [esi + 0x14]','unreviewed load')
  elif i.mnemonic=='cmp':a,b=i.op_str.split(', ');require(a=='ax','unreviewed cmp');equal=3924==int(b,0)
  elif i.mnemonic in ['je','jne']:require(equal is not None,'missing cmp');next_pc=int(i.op_str,0) if equal==(i.mnemonic=='je') else next_pc
  else:raise RuntimeError('unreviewed native instruction')
  pc=next_pc
 (out/'start-3924-static-route.json').write_bytes(encoded(dict(graphics=3924,trace=trace,reached_default=pc,PE_executed=False)))
 audio=R/'LiTo381-Modern/src/modern_lito/combat_animation.rs';require('FrameEvent::Effect(id) => Some((NativeAudioMode::AllowOverlappingVoices, *id))' in audio.read_text('utf-8-sig'),'original less-than audio classification differs')
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip() for l in sql.open(encoding='utf-8-sig') if re.match(r'INSERT INTO `skills` VALUES \((65|66),',l)];require(len(labels)==2,'labels differ');(out/'readonly-labels.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in out.iterdir()};pins.update({str(p):sha(p) for p in [session,protocol,jar,class1,source,audio,T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py']})
 p=out/'inspection.json';p.write_bytes(encoded(dict(schema='E149-storm65-sleep66-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(text):sha(text),str(pe):sha(pe),str(sql):sha(sql)},definitions=q['next_definitions'],raw_definitions=defs,storm_actor_xy_S39_type8=True,storm_class1_first_target_snapshot_default_start=True,storm_less_than1348_is_overlap_audio=True,storm_Hit_source_frame=5,storm_audio_source_frame=2,sleep_actor_xy_S37_primary_object_only=True,sleep_material_owner_callbacks_static_only=True,sleep_status_server_owned_seconds=32,sleep_visual_millis=5250,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(inspection=str(p),sha256=sha(p),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
