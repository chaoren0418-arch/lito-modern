"""Disassemble resource initialization only; never initialize a target class."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(path):
 with Path(path).open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest().upper()
data=lambda value:(json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];jar=Path(ref['cached_reference']);assert sha(ref['formal'])==sha(jar)==ref['formal_sha256']
out=V/'native-E149-resource-initialization-closure'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
classes=['SharedClientResources','InventoryCatalog','NativeDeItemTemplate','NativeDeItemAbility','NativeDeItemMetadata']
commands=[]
for name in classes:
 command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-p','-c','-constants',
  '-classpath',str(jar),'local.lito381.server.'+name]
 result=subprocess.run(command,cwd=out,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
 (out/(name+'.stdout.txt')).write_bytes(result.stdout);(out/(name+'.stderr.txt')).write_bytes(result.stderr)
 commands.append(dict(command=command,exit_code=result.returncode,stdout_sha256=sha(out/(name+'.stdout.txt')),stderr_sha256=sha(out/(name+'.stderr.txt'))))
 assert result.returncode==0,(name,result.returncode)
record=dict(status='resource_initialization_and_raw_item_declarations_disassembled_only',formal_reference=str(jar),formal_target_sha256=sha(jar),
 commands=commands,output_files={str(p):sha(p) for p in out.iterdir() if p.is_file()},
 classes_instantiated=0,original_factories_invoked=False,SQL_executed=False,database_started=False,tests_executed=0,desktop_inputs=0,
 whole_goal_complete=False)
(out/'inspection.json').write_bytes(data(record));assert sha(ref['formal'])==ref['formal_sha256']
print(json.dumps(dict(inspection=str(out/'inspection.json'),disassembled_classes=classes,classes_instantiated=0,tests_executed=0),ensure_ascii=False))
