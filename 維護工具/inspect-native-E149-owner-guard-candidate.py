"""Disassemble two new QA classes without loading or invoking server code."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,json,subprocess
from native_E149_recovery_contract import R,read,sha,encoded,owned
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=owned(a.run.resolve(strict=True),R/'驗證紀錄/current/native-E149-recovery-bootstrap-compile')
c=read(run/'cohort.json');assert c['status']=='integrated_fixture_compilation_only_complete' and len(c['process_history'])==1
assert c['process_history'][0]['exit_code']==0 and len(c['classes'])==6 and c['metadata']=={}
out=run/('owner-guard-bytecode-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
candidate=read(c['candidate']);commands=[]
for name in ['NativeRecoveryReadonlySnapshot','NativeTeleportFailureHarness']:
 command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-p','-c','-constants',
  '-classpath',str(run/'classes')+';'+candidate['formal_reference'],'local.lito381.server.'+name]
 result=subprocess.run(command,capture_output=True,cwd=out,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
 logs={}
 for kind,blob in [('stdout',result.stdout),('stderr',result.stderr)]:
  path=out/(name+'.'+kind+'.txt');path.write_bytes(blob);logs[str(path)]=sha(path)
 commands.append(dict(command=command,exit_code=result.returncode,logs=logs));assert result.returncode==0
record=dict(status='owner_guard_candidate_bytecode_only',compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),
 commands=commands,outputs={str(p):sha(p) for p in out.iterdir() if p.is_file()},
 original_owner_guards_invoked=False,classes_instantiated=0,tests_executed=0,desktop_inputs=0,whole_goal_complete=False)
(out/'inspection.json').write_bytes(encoded(record));(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(inspection=str(out/'inspection.json'),javap_exit_codes=[r['exit_code'] for r in commands],tests_executed=0),ensure_ascii=False))
