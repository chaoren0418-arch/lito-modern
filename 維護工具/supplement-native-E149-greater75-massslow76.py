"""Supplement full actual areaSlow bodies and exact resurrection codecs, data only."""
from pathlib import Path
from datetime import datetime,timezone
import json
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
from importlib.util import spec_from_file_location,module_from_spec
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 primary=V/'native-E149-greater75-massslow76-inspection/20261024-170921-768554/inspection.json';require(sha(primary)=='3F68733FE7B482277A00492D8E614AD78BD513A00D6AC365EA6346EEFC598B7A','primary changed');j=read(primary);check_pins(j['source_pins']);spec=spec_from_file_location('source_copy',T/'inspect-native-E149-greater75-massslow76.py');m=module_from_spec(spec);spec.loader.exec_module(m)
 out=V/'native-E149-greater75-massslow76-supplement'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);copied={}
 for source,names in [(V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt',['nativeAreaWorldSlowBody','nativeAreaBattleSlowBody']),(V/'native-E149-area2253053-contract/Protocol.stdout.txt',['objectSpellRequest','resurrectionAsk','greaterResurrectionAsk','askAnswer','moveSpeed'])]:
  for h,s in m.methods(source):
   if any(' '+n+'('in h for n in names):
    p=out/(str(len(copied))+'-'+h.split('(')[0].split()[-1]+'.txt');p.write_text(s,encoding='utf-8',newline='\n');copied[h]=str(p)
 require(len(copied)>=6,'missing full bodies/codecs');(out/'supplementer.py').write_bytes(Path(__file__).read_bytes());pins=dict(j['source_pins']);pins.update({str(p):sha(p)for p in out.iterdir()});pins[str(primary)]=sha(primary);p=out/'supplement.json';p.write_bytes(encoded(dict(primary=str(primary),primary_sha256=sha(primary),source_pins=pins,external_readonly_pins=j['external_readonly_pins'],method_files={**j['method_files'],**copied},definitions=j['definitions'],raw_definitions=j['raw_definitions'],tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(supplement=str(p),sha256=sha(p),added_methods=len(copied)),ensure_ascii=False))
if __name__=='__main__':main()
