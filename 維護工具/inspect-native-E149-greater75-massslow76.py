"""Read-only classfile/method/Text evidence; no game classes or programs executed."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require
from native_E149_classfile_scan import parse,instructions
from native_E149_classfile_bootstraps import bootstraps
from importlib.util import spec_from_file_location,module_from_spec
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start()if i+1<len(hs)else len(s)])for i,h in enumerate(hs)]
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_decay71_meteor74_development'];require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and sha(ref['qualification'])==ref['qualification_sha256'],'phase differs');q=read(ref['qualification']);require(q['next_definition_ids']==[75,76],'next definitions differ')
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25' and sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D' and sha(jar)==FORMAL and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'original differs')
 out=V/'native-E149-greater75-massslow76-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 selected=[(h,s)for h,s in methods(session)if re.search('resurrection|massslow',h.split('(')[0],re.I)or ' answerAsk('in h];copied={}
 for i,(h,s)in enumerate(selected):
  p=out/(str(i)+'-'+h.split('(')[0].split()[-1]+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 for i,(h,s)in enumerate((h,s)for h,s in methods(protocol)if any(' '+n+'('in h for n in ['attackSpellRequest','targetSpellRequest','effectObject','ask','askAnswer'])):
  p=out/('Protocol-'+str(i)+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 with zipfile.ZipFile(jar)as z:
  classes={n for n in z.namelist()if n.endswith('.class')and re.search('Resurrection|MassSlow',n)};classes.update('local/lito381/server/'+name+'.class'for _,body in selected for name in re.findall(r'Method local/lito381/server/(Native[A-Za-z0-9_]+)\.',body))
  for n in sorted(classes):
   raw=z.read(n);c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(n).stem+'-Code.json')).write_bytes(encoded(rows));_,bindings=bootstraps(raw)if any(r and r[0]==18 for r in c['cp'])else(c,{});(out/(Path(n).stem+'-bootstraps.json')).write_bytes(encoded(bindings))
  _,bindings=bootstraps(z.read('local/lito381/server/ClientSession.class'));ids={int(i)for _,body in selected for i in re.findall(r'invokedynamic\s+#(\d+),[^\n]*(?:Predicate|Runnable|Supplier|Consumer|Function);',body)};(out/'target-bootstraps.json').write_bytes(encoded({str(i):bindings[i]for i in ids}))
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');raw=text.read_bytes();defs={}
 for gfx in [3944,3932]:
  m=re.search(rb'(?m)^#'+str(gfx).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'raw definition absent');p=out/(str(gfx)+'.definition.txt');p.write_bytes(m.group());defs[str(gfx)]=dict(path=str(p),sha256=sha(p))
 sql=Path('D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/db/lito_static.sql');labels=[l.strip()for l in sql.open(encoding='utf-8-sig')if re.match(r'INSERT INTO `skills` VALUES \((75|76),',l)];require(len(labels)==2,'labels differ');(out/'readonly-labels.txt').write_text('\n'.join(labels)+'\n',encoding='utf-8');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins.update({str(p):sha(p)for p in [session,protocol,jar,T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py']});p=out/'inspection.json';p.write_bytes(encoded(dict(schema='E149-greater75-massslow76-primary-copy-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(text):sha(text),str(sql):sha(sql)},definitions=q['next_definitions'],method_files=copied,raw_definitions=defs,source_review_required=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(inspection=str(p),sha256=sha(p),methods=len(copied),classes=len(classes),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
