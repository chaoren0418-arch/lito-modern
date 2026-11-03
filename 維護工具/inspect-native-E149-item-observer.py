"""javap the observer and copy primitives, without class initialization."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess
R=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(R/'驗證紀錄/current/native-E149-recovery-bootstrap-compile')
c=read(run/'cohort.json');assert c['status']=='integrated_fixture_compilation_only_complete' and len(c['process_history'])==1
candidate=read(c['candidate']);jar=candidate['formal_reference']
classes=['local.lito381.server.NativeRecoveryReadonlySnapshot','local.lito381.server.NativeTeleportFailureHarness',
 'java.util.Date','java.sql.Timestamp','java.util.EnumMap','java.util.AbstractMap','java.util.concurrent.atomic.AtomicLong','java.util.concurrent.atomic.AtomicInteger']
out=run/('nativefields-bytecode-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
commands=[]
for name in classes:
 command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-p','-c','-constants',
  '-classpath',str(run/'classes')+';'+jar,name]
 result=subprocess.run(command,capture_output=True,cwd=out,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
 prefix=name.rsplit('.',1)[-1];(out/(prefix+'.stdout.txt')).write_bytes(result.stdout);(out/(prefix+'.stderr.txt')).write_bytes(result.stderr)
 commands.append(dict(command=command,exit_code=result.returncode));assert result.returncode==0,name
record=dict(status='observer_and_JDK_copy_bytecode_readback_only',compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),
 commands=commands,output_files={str(p):sha(p) for p in out.iterdir() if p.is_file()},
 tests_executed=0,classes_instantiated=0,GUI_inputs=0,original_initializer_invoked=False,whole_goal_complete=False)
(out/'inspection.json').write_bytes(data(record));(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(inspection=str(out/'inspection.json'),disassembled_classes=len(classes),classes_instantiated=0,tests_executed=0),ensure_ascii=False))
