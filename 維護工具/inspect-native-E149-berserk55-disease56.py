"""Static per-skill branches, native S37 heading copy and exact raw profiles."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,struct,zipfile,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(p):
 s=Path(p).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected');ref=w['current_E149_summon51_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'predecessor differs');q=read(ref['qualification']);check_pins(q['source_pins']);out=V/'native-E149-berserk55-disease56-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs');names=['useCombatBuffSpell','lambda$useCombatBuffSpell$207','sameNonemptyClan','sameClassicParty','nativeMaterialWeaknessBody','nativeMaterialWorldWeaknessBody','repair032$lambda$useWeaknessSpell$306','lambda$useClassicWorldWeaknessSpell$303'];selected=[(h,s) for h,s in methods(source) if any(' '+n+'(' in h for n in names)];require(len(selected)==8,'method set differs')
 for i,(h,s) in enumerate(selected):(out/('method-'+str(i)+'.txt')).write_text(s,encoding='utf-8')
 buff=next(s for h,s in selected if ' useCombatBuffSpell(' in h);battle=next(s for h,s in selected if ' nativeMaterialWeaknessBody(' in h);world=next(s for h,s in selected if ' nativeMaterialWorldWeaknessBody(' in h)
 for body,checks in [(buff,{1:'Protocol.objectSpellRequest:',226:'sameNonemptyClan:',235:'sameClassicParty:',307:'isInNativeScreen:',889:'activateBerserk:',967:'Protocol.objectAction:',981:'Protocol.effectObject:'}),(battle,{1:'Protocol.attackSpellRequest:',590:'WeaknessSpell.maximumRange:',1772:'NativeWeaknessMaterialAction.admit:',2287:'activateWeakness:',2515:'Protocol.effectObject:'}),(world,{1:'Protocol.attackSpellRequest:',427:'WeaknessSpell.maximumRange:',1470:'NativeWeaknessMaterialAction.admit:',1971:'activateWeakness:',2112:'Protocol.effectObject:'})]:
  for at,needle in checks.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(needle),body,re.M),'formal offset differs '+str(at))
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal differs')
 with zipfile.ZipFile(jar) as z:_,indys=bootstraps(z.read('local/lito381/server/ClientSession.class'))
 refs={int(n) for s in [buff,battle,world] for n in re.findall(r'invokedynamic\s+#(\d+),[^\n]*Predicate;',s)};bindings={str(n):indys[n] for n in refs};require(any(any(x['name']=='lambda$useCombatBuffSpell$207' for x in b['method_handles']) for b in bindings.values()),'buff target caller differs');(out/'target-bootstraps.json').write_bytes(encoded(bindings))
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');raw=text.read_bytes();defs={}
 for id in [3943,2230]:
  m=re.search(rb'(?m)^#'+str(id).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'raw absent');p=out/(str(id)+'.definition.txt');p.write_bytes(m.group());defs[str(id)]=dict(path=str(p),sha256=sha(p))
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE differs');b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];ss=e+24+struct.unpack_from('<H',b,e+20)[0]
 def at(a,n):
  for i in range(struct.unpack_from('<H',b,e+6)[0]):
   size,start,physical,pos=struct.unpack_from('<IIII',b,ss+i*40+8)
   if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return b[pos+a-0x400000-start:pos+a-0x400000-start+n]
  raise RuntimeError(hex(a))
 md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);decoded={}
 for name,(lo,hi) in {'S37':(0x4adda0,0x4adf60),'constructor':(0x44ede0,0x44eeb3),'frame_selector':(0x4e5c00,0x4e63ba)}.items():
  data=at(lo,hi-lo);rows=list(md.disasm(data,lo));require(sum(x.size for x in rows)==len(data) and all(x.address+x.size==y.address for x,y in zip(rows,rows[1:])),'partial instruction range '+name);decoded[name]=rows;(out/(name+'.bin')).write_bytes(data);(out/(name+'.asm')).write_text('\n'.join(f'{x.address:08X} {x.bytes.hex()} {x.mnemonic} {x.op_str}' for x in rows)+'\n',encoding='utf-8')
 require(any(x.mnemonic=='movsx' and x.op_str=='edx, byte ptr [esi + 0x11]' for x in decoded['S37']) and any(x.mnemonic=='mov' and x.op_str=='byte ptr [esi + 0x11], cl' for x in decoded['constructor']) and any('[esi + 0x11]' in x.op_str for x in decoded['frame_selector']),'native heading field chain differs')
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip() for l in sql.open(encoding='utf-8-sig') if re.match(r'INSERT INTO `skills` VALUES \((55|56),',l)];require(len(labels)==2,'labels absent');(out/'readonly-labels.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8')
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in out.iterdir() if p.is_file()};pins.update({str(p):sha(p) for p in [source,jar,T/'native_E149_classfile_bootstraps.py',T/'native_E149_classfile_scan.py']});record=dict(schema='E149-berserk55-disease56-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref,source_pins=pins,external_readonly_pins={str(text):sha(text),str(pe):sha(pe),str(sql):sha(sql)},definitions=q['next_definitions'],raw_definitions=defs,buff_object_request_and_server_clan_or_party=True,disease_actor_xy_material_world_and_battle=True,S37_effect_heading_copies_receipt_actor_heading=True,heading_field_offset=17,berserk_graphics=3943,berserk_image_count=8,berserk_sound=1361,berserk_sound_source_frame=2,disease_graphics=2230,disease_Multiply_attr128=True,disease_sound=839,disease_sound_source_frame=1,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
