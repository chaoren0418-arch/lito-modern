"""Complete missing material/visibility classes as readonly data, without loading them."""
from pathlib import Path
from datetime import datetime,timezone
import re,json,zipfile,importlib.util
from native_E149_recovery_contract import R,P,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_scan import parse,instructions
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 primary=V/'native-E149-hide97-venom98-inspection/20261024-183937-685740/inspection.json';supplement=V/'native-E149-hide97-supplement/20261024-184556-713087/supplement.json';require(sha(primary)=='5CEF02A997634D5439B5B5C625F0679DEF06506EA2FCE608EF5E1CEC724E112D' and sha(supplement)=='764C6693082A4A0A842954E9637CBD38EBFEA2882BFFFCD5F4CF352411661FD8','prior changed');j=read(primary);check_pins(j['source_pins']);check_pins(read(supplement)['source_pins']);jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal changed')
 out=V/'native-E149-hide97-venom98-routes'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 with zipfile.ZipFile(jar)as z:
  classes=[n for n in z.namelist()if n.endswith('.class')and re.match(r'local/lito381/server/(NativeLightAction|NativeCastMaterial|BattleVenomEnchantStatus|CombatPoisonLifecycle|CombatInvisibilityRules|BattleInvisibilityStatus)(?:[$.]|$)',n)]
  for n in classes:
   raw=z.read(n);c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(n).stem+'-Code.json')).write_bytes(encoded(rows));_,bindings=bootstraps(raw)if any(r and r[0]==18 for r in c['cp'])else(c,{});(out/(Path(n).stem+'-bootstraps.json')).write_bytes(encoded(bindings))
 spec=importlib.util.spec_from_file_location('copied_method_reader',T/'inspect-native-E149-hide97-venom98.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';methods={}
 for h,s in m.methods(protocol):
  if any(' '+n+'('in h for n in ['invisible','removeObject','systemMessage']):
   p=out/('Protocol-'+str(len(methods))+'.txt');p.write_text(s,encoding='utf-8',newline='\n');methods[h]=str(p)
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins.update({str(p):sha(p)for p in [primary,supplement,protocol,jar,T/'inspect-native-E149-hide97-venom98.py',T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py']});p=out/'routes.json';p.write_bytes(encoded(dict(schema='E149-hide97-venom98-material-and-visibility-source-v1',formal_target_sha256=FORMAL,source_pins=pins,method_files=methods,primary=str(primary),supplement=str(supplement),source_review_required=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(routes=str(p),sha256=sha(p),classes=len(classes),protocol_methods=len(methods),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
