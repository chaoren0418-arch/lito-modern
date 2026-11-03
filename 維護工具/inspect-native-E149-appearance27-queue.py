"""Capture original queued S27 dispatch and formal67/68 prerequisites, without execution."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile,hashlib,csv,io,pefile,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require
from native_E149_classfile_scan import parse,instructions
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M))
 return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_storm65_sleep66_development'];require(sha(ref['qualification'])==ref['qualification_sha256'] and w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'source-only phase differs');q=read(ref['qualification']);require(q['next_definition_ids']==[67,68],'next definitions differ')
 out=V/'native-E149-appearance27-queue-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs');names=['usePolymorphSpell','nativeMaterialPolymorphBody','handleShapeChangeAction','useDamageProtectionSpell','nativeMaterialProtectionBody'];selected={h:s for h,s in methods(session) if any(' '+n+'(' in h for n in names)};require(len(selected)==5,'method set differs')
 for i,(h,s) in enumerate(selected.items()):(out/(str(i)+'-'+h.split('(')[0].split()[-1]+'.txt')).write_text(s,encoding='utf-8',newline='\n')
 protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';require(sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D','protocol differs');packets={h:s for h,s in methods(protocol) if any(' '+n+'(' in h for n in ['polymorph','closeHypertext','hypertext','hypertextActionRequest','objectSpellRequest'])};require(sum(' polymorph(' in h for h in packets)==2,'polymorph overload differs')
 for i,(h,s) in enumerate(packets.items()):(out/('Protocol-'+str(i)+'.txt')).write_text(s,encoding='utf-8',newline='\n')
 full=next(s for h,s in packets.items() if ' polymorph(long, int, int, boolean)' in h);require(all(n in full for n in ['68: bipush        39','Protocol$Writer.d:','Protocol$Writer.h:','Protocol$Writer.c:']),'S27 wire differs')
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'jar differs')
 with zipfile.ZipFile(jar) as z:
  for name in ['NativePolymorphMaterialAction','NativeProtectionCastAction','BattlePolymorphCatalog']:
   raw=z.read('local/lito381/server/'+name+'.class');c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp'])) for m in c['methods'] if m['code']];(out/(name+'-Code.json')).write_bytes(encoded(rows))
  raw=z.read('battle/polymorph-native-380.csv');(out/'polymorph-native-380.csv').write_bytes(raw);forms=list(csv.DictReader(io.StringIO('\n'.join(l for l in raw.decode('utf-8-sig').splitlines() if not l.startswith('#')))));require(forms and len({f['selection'] for f in forms})==len(forms),'form catalogue duplicates');(out/'standard-menu-forms.json').write_bytes(encoded([f for f in forms if f['standard_menu']=='1']))
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE differs');native=pefile.PE(str(pe),fast_load=True);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ranges={}
 for name,lo,hi in [('S27',0x4a70b0,0x4a712c),('set_appearance',0x4e0490,0x4e0575),('body_busy',0x4e6af0,0x4e6b6b),('queue27',0x4e6860,0x4e68d0)]:
  raw=native.get_data(lo-native.OPTIONAL_HEADER.ImageBase,hi-lo);(out/(name+'.bin')).write_bytes(raw);rows=[dict(address=i.address,mnemonic=i.mnemonic,operands=i.op_str,hex=i.bytes.hex()) for i in md.disasm(raw,lo)];ranges[name]=dict(start=lo,end=hi,sha256=hashlib.sha256(raw).hexdigest().upper(),instructions=rows)
 calls={r['operands'] for r in ranges['S27']['instructions'] if r['mnemonic']=='call'};require({'0x4e8160','0x4e6af0','0x4e6860','0x4e0490'}<=calls,'native queued/direct dispatch differs');require(any(r['mnemonic']=='mov' and r['operands']=="dword ptr [eax], 0x27" for r in ranges['queue27']['instructions']),'native queue tag differs');(out/'native-ranges.json').write_bytes(encoded(ranges));native.close()
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip() for l in sql.open(encoding='utf-8-sig') if re.match(r'INSERT INTO `skills` VALUES \((67|68),',l)];require(len(labels)==2,'labels differ');(out/'readonly-labels.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8')
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in out.iterdir()};pins.update({str(p):sha(p) for p in [session,protocol,jar,T/'native_E149_classfile_scan.py',R/'LiTo381-Modern/src/modern_lito/native_combat_animation.rs',R/'LiTo381-Modern/src/modern_lito/native_world_animation.rs']})
 p=out/'inspection.json';p.write_bytes(encoded(dict(schema='E149-appearance27-queue-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(pe):sha(pe),str(sql):sha(sql)},definitions=q['next_definitions'],S27_exact_eight_bytes=True,native_busy_queues27_instead_of_applying_now=True,native_idle_applies27_directly=True,LT_base_binding_qualified=False,prototype_queue_not_yet_world_or_GUI_integrated=True,formal_standard_forms=sum(f['standard_menu']=='1' for f in forms),tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(inspection=str(p),sha256=sha(p),standard_forms=sum(f['standard_menu']=='1' for f in forms),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
