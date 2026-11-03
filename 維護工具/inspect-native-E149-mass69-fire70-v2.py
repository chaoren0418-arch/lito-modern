"""Copy original methods/classfile Code/Text for review; no class loading or execution."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require
from native_E149_classfile_scan import parse,instructions
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start()if i+1<len(hs)else len(s)])for i,h in enumerate(hs)]
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_shape67_protection68_development'];require(sha(ref['qualification'])==ref['qualification_sha256'] and w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'phase differs');q=read(ref['qualification']);require(q['next_definition_ids']==[69,70],'next definitions differ')
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25' and sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D' and sha(jar)==FORMAL and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'original identities differ')
 out=V/'native-E149-mass69-fire70-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);copied={}
 names=['useMassTeleportSpell','nativeMaterialMassTeleportBody','answerMassTeleport','nativeMassTeleportAnswerBody','nativeRejectMassTeleportQuestion','resolveMassTeleportDestination','resolveClassicWorldMassTeleportDestination','massTeleportSceneStillValid','rejectMassTeleport','beginMassTeleportPortal','reserveMassTeleportPortal','sendMassTeleportPortalRequest','answerAsk','useCenteredAreaAttackSpell','useClassicWorldCenteredAreaAttackSpell','publishClassicWorldAreaAttackSpellPacket']
 for i,(h,s)in enumerate((h,s)for h,s in methods(session)if any(' '+n+'('in h for n in names)or ('lambda$'in h and ('MassTeleport'in h or 'CenteredAreaAttackSpell'in h))):
  p=out/(str(i)+'-'+h.split('(')[0].split()[-1]+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 packet_names=['massTeleportSpellRequest','askAnswer','ask','portal','portalDestination','attackSpellRequest','attackMany']
 for i,(h,s)in enumerate((h,s)for h,s in methods(protocol)if any(' '+n+'('in h for n in packet_names)):
  p=out/('Protocol-'+str(i)+'-'+h.split('(')[0].split()[-1]+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 with zipfile.ZipFile(jar)as z:
  names=[n for n in z.namelist()if n.endswith('.class')and ('MassTeleport'in n or 'CenteredArea'in n)];(out/'class-names.json').write_bytes(encoded(names))
  for n in names:
   raw=z.read(n);c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(n).stem+'-Code.json')).write_bytes(encoded(rows));_,bindings=bootstraps(raw) if any(row and row[0]==18 for row in c['cp']) else (c,{});(out/(Path(n).stem+'-bootstraps.json')).write_bytes(encoded(bindings))
  _,bindings=bootstraps(z.read('local/lito381/server/ClientSession.class'));(out/'ClientSession-bootstraps.json').write_bytes(encoded(bindings))
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text changed');raw=text.read_bytes();defs={}
 for gfx in [2236,1819]:
  m=re.search(rb'(?m)^#'+str(gfx).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'raw definition absent');p=out/(str(gfx)+'.definition.txt');p.write_bytes(m.group());defs[str(gfx)]=dict(path=str(p),sha256=sha(p))
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip()for l in sql.open(encoding='utf-8-sig')if re.match(r'INSERT INTO `skills` VALUES \((69|70),',l)];require(len(labels)==2,'labels differ');(out/'readonly-labels.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins.update({str(p):sha(p)for p in [session,protocol,jar,T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py',T/'inspect-native-E149-mass69-fire70.py',V/'native-E149-mass69-fire70-inspection/20261024-155129-498403/tooling-failure.json']})
 p=out/'inspection.json';p.write_bytes(encoded(dict(schema='E149-mass69-fire70-primary-copy-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(text):sha(text),str(sql):sha(sql)},definitions=q['next_definitions'],method_files=copied,raw_definitions=defs,source_review_required=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(inspection=str(p),sha256=sha(p),methods=len(copied),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
