"""Read original morph-entry bytecode. Never initialize game classes."""
from pathlib import Path
from datetime import datetime,timezone
import json,subprocess
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require
T=R/'維護工具';V=R/'驗證紀錄/current'

def main():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'original workload/source freeze active')
 jar=owned(P/'target/current/server-reference/formal-server-E1498871.jar')
 require(sha(jar)==FORMAL==sha(w['current_formal_reference_identity']['formal']),'fixed original target changed')
 javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe')
 require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','read-only javap tool changed')
 out=owned(V/'native-E149-morph-denial-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());rows=[]
 for short in ['NativeDeWorldKinds$Item','NativeDeItemTemplate','NativeDeItemInventory','NativeDeItemStateCopy']:
  command=[str(javap),'-J-Dfile.encoding=UTF-8','-p','-c','-constants','-classpath',str(jar),'local.lito381.server.'+short]
  with (out/(short+'.stdout.txt')).open('xb') as stdout,(out/(short+'.stderr.txt')).open('xb') as stderr:
   process=subprocess.run(command,stdout=stdout,stderr=stderr,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
  rows.append(dict(class_name=short,command=command,exit_code=process.returncode))
  (out/'commands.json').write_bytes(encoded(rows));require(process.returncode==0,'original bytecode inspection failed; preserve output')
 require(sha(jar)==FORMAL,'original cache changed during bytecode read')
 record=dict(status='original_morph_entry_bytecode_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  jar=str(jar),javap=str(javap),javap_sha256=sha(javap),commands=rows,
  outputs={str(path):sha(path) for path in out.iterdir() if path.is_file()},
  tests_executed=0,game_classes_instantiated=0,mutations_attempted=0,desktop_inputs=0,
  full_native_owner_qualified=False,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),javap_exit_codes=[r['exit_code'] for r in rows],tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
