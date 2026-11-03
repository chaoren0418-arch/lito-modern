"""Data-only original71/74 method, Code, bootstrap, Text and native-start evidence."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require
from native_E149_classfile_scan import parse,instructions
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start()if i+1<len(hs)else len(s)])for i,h in enumerate(hs)]
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_mass69_integration_development'];require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and sha(ref['qualification'])==ref['qualification_sha256'],'phase differs');q=read(ref['qualification']);require(q['next_definition_ids']==[71,74],'next definitions differ')
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25' and sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D' and sha(jar)==FORMAL and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'original identities differ')
 out=V/'native-E149-decay71-meteor74-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);names=['useClassicWorldDecayPotionSpell','useDecayPotionSpell','decayPotionCasterStillAdmitted','nativeMaterialDecayPotionBody','nativeMaterialWorldDecayPotionBody','useAreaAttackSpell','useClassicWorldAreaAttackSpell'];selected=[(h,s)for h,s in methods(session)if any(' '+n+'('in h for n in names)or ('lambda$'in h and 'DecayPotionSpell$'in h)];copied={}
 for i,(h,s)in enumerate(selected):p=out/(str(i)+'-'+h.split('(')[0].split()[-1]+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 for i,(h,s)in enumerate((h,s)for h,s in methods(protocol)if any(' '+n+'('in h for n in ['attackSpellRequest','attackMany'])):p=out/('Protocol-'+str(i)+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 with zipfile.ZipFile(jar)as z:
  selected_classes={n for n in z.namelist()if n.endswith('.class')and 'DecayPotion'in n};selected_classes.update('local/lito381/server/'+name+'.class'for _,body in selected for name in re.findall(r'Method local/lito381/server/(Native[A-Za-z0-9_]+)\.',body))
  for n in sorted(selected_classes):
   raw=z.read(n);c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(n).stem+'-Code.json')).write_bytes(encoded(rows));_,bindings=bootstraps(raw)if any(r and r[0]==18 for r in c['cp'])else(c,{});(out/(Path(n).stem+'-bootstraps.json')).write_bytes(encoded(bindings))
  _,bindings=bootstraps(z.read('local/lito381/server/ClientSession.class'));ids={int(i)for _,body in selected for i in re.findall(r'invokedynamic\s+#(\d+),[^\n]*Predicate;',body)};(out/'target-bootstraps.json').write_bytes(encoded({str(i):bindings[i]for i in ids}))
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');raw=text.read_bytes();defs={}
 for gfx in [2232,762]:
  m=re.search(rb'(?m)^#'+str(gfx).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'raw definition absent');p=out/(str(gfx)+'.definition.txt');p.write_bytes(m.group());defs[str(gfx)]=dict(path=str(p),sha256=sha(p))
 start=V/'native-E149-blizzard59-inspection/20261024-115029-671040/start.bin';require(sha(start)=='6937E97AE98ACA01232582E90F70DBD2BA2D656D92E92837105A233B6A318A77','native start differs');md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);code={i.address:i for i in md.disasm(start.read_bytes(),0x44f350)};pc=0x44f370;eq=None;trace=[]
 while pc!=0x45001a:
  require(pc in code and len(trace)<40,'native route bound');i=code[pc];trace.append(dict(address=pc,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str));n=pc+i.size
  if i.mnemonic=='mov':require(i.op_str=='ax, word ptr [esi + 0x14]','unreviewed load')
  elif i.mnemonic=='cmp':a,b=i.op_str.split(', ');require(a=='ax','unreviewed cmp');eq=762==int(b,0)
  elif i.mnemonic in ['je','jne']:require(eq is not None,'missing cmp');n=int(i.op_str,0)if eq==(i.mnemonic=='je')else n
  else:raise RuntimeError('unreviewed special native route')
  pc=n
 (out/'start762-static-route.json').write_bytes(encoded(dict(graphics=762,trace=trace,default_start=pc,PE_executed=False)))
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE differs');sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip()for l in sql.open(encoding='utf-8-sig')if re.match(r'INSERT INTO `skills` VALUES \((71|74),',l)];require(len(labels)==2,'labels differ');(out/'readonly-labels.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins.update({str(p):sha(p)for p in [session,protocol,jar,start,T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py']});p=out/'inspection.json';p.write_bytes(encoded(dict(schema='E149-decay71-meteor74-primary-copy-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(text):sha(text),str(pe):sha(pe),str(sql):sha(sql)},definitions=q['next_definitions'],method_files=copied,raw_definitions=defs,meteor762_uses_default_start=True,source_review_required=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(inspection=str(p),sha256=sha(p),methods=len(copied),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
