"""Readonly missing97 invisibility routes; no class loading or test invocation."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile,importlib.util
from native_E149_recovery_contract import R,P,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_scan import parse,instructions
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 primary=V/'native-E149-hide97-venom98-inspection/20261024-183937-685740/inspection.json';require(sha(primary)=='5CEF02A997634D5439B5B5C625F0679DEF06506EA2FCE608EF5E1CEC724E112D','primary changed');j=read(primary);check_pins(j['source_pins']);session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal changed')
 spec=importlib.util.spec_from_file_location('source_method_reader',T/'inspect-native-E149-hide97-venom98.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);selected=[(h,s)for h,s in m.methods(session)if 'invisibility'in h.split('(')[0].lower()];require(any(' useInvisibilitySpell('in h for h,s in selected)and any(' nativeMaterialInvisibilityBody('in h for h,s in selected),'missing97 routes absent')
 out=V/'native-E149-hide97-supplement'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);copied={}
 for n,(h,s)in enumerate(selected):
  p=out/(str(n)+'-'+h.split('(')[0].split()[-1]+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 with zipfile.ZipFile(jar)as z:
  classes={n for n in z.namelist()if n.endswith('.class')and 'Invisibility'in n};classes.update('local/lito381/server/'+n+'.class'for _,s in selected for n in re.findall(r'Method local/lito381/server/(Native[A-Za-z0-9_]+)\.',s))
  for n in sorted(classes):
   raw=z.read(n);c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(n).stem+'-Code.json')).write_bytes(encoded(rows));_,bindings=bootstraps(raw)if any(r and r[0]==18 for r in c['cp'])else(c,{});(out/(Path(n).stem+'-bootstraps.json')).write_bytes(encoded(bindings))
  _,bindings=bootstraps(z.read('local/lito381/server/ClientSession.class'));ids={int(i)for _,s in selected for i in re.findall(r'invokedynamic\s+#(\d+),[^\n]*(?:Predicate|Runnable|Supplier|Consumer|Function);',s)};(out/'target-bootstraps.json').write_bytes(encoded({str(i):bindings[i]for i in ids}))
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins.update({str(p):sha(p)for p in [primary,session,jar,T/'inspect-native-E149-hide97-venom98.py',T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py']});p=out/'supplement.json';p.write_bytes(encoded(dict(schema='E149-hide97-missing-invisibility-source-v1',formal_target_sha256=FORMAL,primary=str(primary),primary_sha256=sha(primary),source_pins=pins,method_files=copied,full_source_review_required=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(supplement=str(p),sha256=sha(p),methods=len(copied),classes=len(classes),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
