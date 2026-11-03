"""Disassemble fixed formal75 Protocol bytes; do not initialize a server class."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import os
import subprocess
import zipfile
R = Path(__file__).resolve().parents[1]
P = R / 'LiTo381-Modern'
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
jar = P / 'target/current/server-reference/formal-server-75DA69B0.jar'
assert sha(jar) == '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert not any(os.environ.get(name) for name in ['JAVA_TOOL_OPTIONS', 'JDK_JAVA_OPTIONS', '_JAVA_OPTIONS'])
out = R / '驗證紀錄/current/native-current-skill-request-scope' / ('protocol75-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
tool = Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe')
env = dict(os.environ, TEMP=str(P / 'target/current/scratch'), TMP=str(P / 'target/current/scratch'))
command = [str(tool), '-J-Dfile.encoding=UTF-8', '-J-Djava.io.tmpdir=' + env['TMP'], '-J-Duser.dir=' + str(out),
           '-p', '-c', '-s', '-classpath', str(jar), 'local.lito381.server.Protocol']
result = subprocess.run(command, cwd=out, env=env, capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW, timeout=45)
(out / 'Protocol.stdout.txt').write_bytes(result.stdout)
(out / 'Protocol.stderr.txt').write_bytes(result.stderr)
assert result.returncode == 0 and not result.stderr
assert b'local.lito381.server.Protocol' in result.stdout
with zipfile.ZipFile(jar) as archive:
    class_sha = hashlib.sha256(archive.read('local/lito381/server/Protocol.class')).hexdigest().upper()
record = dict(status='formal75_Protocol_bytecode_disassembly_only', jar=str(jar), jar_sha256=sha(jar), Protocol_class_sha256=class_sha,
              javap=str(tool), javap_sha256=sha(tool), command=command, actual_exit_code=result.returncode,
              disassembly=str(out / 'Protocol.stdout.txt'), disassembly_sha256=sha(out / 'Protocol.stdout.txt'),
              JVM_class_initialization_or_server_started=False, new_sessions=0, new_casts=0, normal_source_or_active_QA_modified=False,
              protocol_or_runtime_qualification_granted=False, whole_goal_complete=False)
(out / 'readback.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
(out / 'inspector.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(disassembly=record['disassembly'], class_sha256=class_sha, actual_exit_code=0, server_started=False), ensure_ascii=False))
