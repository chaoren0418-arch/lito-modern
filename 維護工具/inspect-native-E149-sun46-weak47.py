"""Static 46/47 dispatch, raw shared definitions and native class1 branch binding."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,struct,subprocess,zipfile,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected');ref=w['current_E149_eruption45_entry_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'entry predecessor differs');q=read(ref['qualification']);check_pins(q['source_pins'])
 out=V/'native-E149-sun46-weak47-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs');selected={}
 for h,s in methods(source):
  if any(' '+n+'(' in h for n in ['useWeaknessSpell','useClassicWorldWeaknessSpell','nativeMaterialWeaknessBody','nativeMaterialWorldWeaknessBody','lambda$useClassicWorldWeaknessSpell$303','repair032$lambda$useWeaknessSpell$306']):selected[h]=s
 require(len(selected)==6,'weakness method set differs')
 for i,(h,s) in enumerate(selected.items()):(out/('method-'+str(i)+'.txt')).write_text(s,encoding='utf-8')
 battle=next(s for h,s in selected.items() if ' nativeMaterialWeaknessBody(' in h);world=next(s for h,s in selected.items() if ' nativeMaterialWorldWeaknessBody(' in h)
 for body,offsets in [(battle,{1:'Protocol.attackSpellRequest:',590:'WeaknessSpell.maximumRange:',1772:'NativeWeaknessMaterialAction.admit:',2106:'Protocol.objectAction:',2287:'BattleTimedStatus.activateWeakness:',2515:'Protocol.effectObject:'}),(world,{1:'Protocol.attackSpellRequest:',427:'WeaknessSpell.maximumRange:',1470:'NativeWeaknessMaterialAction.admit:',1790:'Protocol.objectAction:',1971:'BattleTimedStatus.activateWeakness:',2112:'Protocol.effectObject:'})]:
  for at,needle in offsets.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(needle),body,re.M),'weakness offset differs '+str(at))
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal jar differs')
 with zipfile.ZipFile(jar) as z:data=z.read('local/lito381/server/ClientSession.class')
 parsed,indys=bootstraps(data);refs={int(n) for text in [battle,world] for n in re.findall(r'invokedynamic\s+#(\d+),[^\n]*Predicate;',text)};bindings={str(n):indys[n] for n in sorted(refs)}
 require(any(any(h['name']=='repair032$lambda$useWeaknessSpell$306' for h in row['method_handles']) for row in bindings.values()) and any(any(h['name']=='lambda$useClassicWorldWeaknessSpell$303' for h in row['method_handles']) for row in bindings.values()),'active target predicates differ');(out/'weakness-bootstraps.json').write_bytes(encoded(bindings))
 javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap differs');command=[str(javap),'-J-Dfile.encoding=UTF-8','-classpath',str(jar),'-c','-p','local.lito381.server.NativeWeaknessMaterialAction'];run=subprocess.run(command,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);(out/'NativeWeaknessMaterialAction.stdout.txt').write_bytes(run.stdout);(out/'NativeWeaknessMaterialAction.stderr.txt').write_bytes(run.stderr);require(run.returncode==0,'javap failed; retain logs')
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','shared Text differs');raw=text.read_bytes();definitions={}
 for identifier in [1811,2228]:
  m=re.search(rb'(?m)^#'+str(identifier).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'raw definition absent');p=out/(str(identifier)+'.definition.txt');p.write_bytes(m.group());definitions[str(identifier)]=dict(path=str(p),sha256=sha(p),raw=list(m.group()))
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE differs');b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];ss=e+24+struct.unpack_from('<H',b,e+20)[0]
 def data_at(a,n):
  for i in range(struct.unpack_from('<H',b,e+6)[0]):
   size,start,physical,at=struct.unpack_from('<IIII',b,ss+i*40+8)
   if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return b[at+a-0x400000-start:at+a-0x400000-start+n]
  raise ValueError(hex(a))
 ranges={'start':(0x44f350,0x450028),'S23_class1':(0x4a6234,0x4a6288),'constructor':(0x44ede0,0x44eeb3)};decoded={};md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 for name,(lo,hi) in ranges.items():
  data=data_at(lo,hi-lo);rows=list(md.disasm(data,lo));require(sum(i.size for i in rows)==len(data) and rows[0].address==lo and all(a.address+a.size==b.address for a,b in zip(rows,rows[1:])),'partial instruction range '+name);decoded[name]={i.address:i for i in rows};(out/(name+'.bin')).write_bytes(data);(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows)+'\n',encoding='utf-8')
 at=0x44f370;equal=None;trace=[]
 for _ in range(32):
  if at in [0x45001a,0x44fffb]:break
  i=decoded['start'][at];next_at=at+i.size;entry=dict(address=at,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str)
  if i.mnemonic=='mov' and i.op_str=='ax, word ptr [esi + 0x14]':entry['known_graphic']=1811
  elif i.mnemonic=='cmp' and i.op_str.startswith('ax, '):equal=1811==int(i.op_str[4:],0);entry['equal']=equal
  elif i.mnemonic in ['je','jne']:
   require(equal is not None,'branch before compare');taken=equal if i.mnemonic=='je' else not equal;entry['taken']=taken
   if taken:next_at=int(i.op_str,0)
  else:raise RuntimeError('unreviewed branch '+repr(entry))
  trace.append(entry);at=next_at
 require(at==0x45001a,'1811 unexpectedly enters two-step branch')
 (out/'1811-start-trace.json').write_bytes(encoded(dict(graphics=1811,reached=at,trace=trace,offset_branch=False,PE_executed=False)))
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');names=[l.rstrip('\r\n') for l in sql.open(encoding='utf-8-sig') if re.match(r'INSERT INTO `skills` VALUES \((46|47),',l)];require(len(names)==2 and any("'烈炎術'" in l for l in names) and any("'弱化術'" in l for l in names),'names differ');(out/'readonly-SQL-names.txt').write_text('\n'.join(names)+'\n',encoding='utf-8')
 pins={str(p):sha(p) for p in out.iterdir() if p.is_file()};pins.update({str(p):sha(p) for p in [source,jar,T/'native_E149_classfile_bootstraps.py',T/'native_E149_classfile_scan.py']});(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins[str(out/'inspector.py')]=sha(out/'inspector.py')
 record=dict(schema='E149-sun46-weak47-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref,source_pins=pins,external_readonly_pins={str(text):sha(text),str(pe):sha(pe),str(sql):sha(sql),str(javap):sha(javap)},definitions=q['next_definitions'],raw_definitions=definitions,weakness_target_bootstraps=bindings,javap_command=command,javap_exit_code=0,sunburst_direct_wire_destination=True,sunburst_sound_source_frame=1,sunburst_hit_source_frame=11,weakness_mix='Multiply_RGB555_attr104_128',weakness_duration_base_seconds=64,weakness_is_server_status_not_finite_visual_duration=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),new_static_profiles=2,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
