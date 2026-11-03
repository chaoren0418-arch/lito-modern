"""Static material-supplier admission class copy; no JVM or test invocation."""
from pathlib import Path
from datetime import datetime,timezone
import json,zipfile
from native_E149_recovery_contract import R,P,FORMAL,sha,encoded,require
from native_E149_classfile_scan import parse,instructions
def main():
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal changed');out=R/'驗證紀錄/current/native-E149-self-admission'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 with zipfile.ZipFile(jar)as z:
  classes=[n for n in z.namelist()if n.startswith('local/lito381/server/NativeSourceSkillCastAdmission')and n.endswith('.class')]
  for name in classes:
   c=parse(z.read(name));rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(name).stem+'-Code.json')).write_bytes(encoded(rows))
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins[str(jar)]=sha(jar);p=out/'admission.json';p.write_bytes(encoded(dict(schema='E149-self-admission-class-code-v1',formal_target_sha256=FORMAL,source_pins=pins,source_review_required=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(admission=str(p),sha256=sha(p),classes=len(classes),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
