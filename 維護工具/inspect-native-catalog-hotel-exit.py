"""Read actual current75 room-exit callers without starting any formal sessions."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,subprocess
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
env={k:v for k,v in os.environ.items()if k not in {'JAVA_TOOL_OPTIONS','JDK_JAVA_OPTIONS','_JAVA_OPTIONS'}};env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
cmd=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-J-XX:-UsePerfData','-J-Djava.io.tmpdir='+env['TMP'],'-p','-c','-classpath',str(jar),'local.lito381.server.ClientSession']
z=subprocess.run(cmd,env=env,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=45);assert z.returncode==0 and not z.stderr
lines=z.stdout.decode('utf-8').splitlines();heads=[i for i,s in enumerate(lines)if re.match(r'^  (?:public|private|protected) .*\(.*\)(?: throws [^;]+)?;$',s)]
blocks={lines[i].strip():'\n'.join(lines[i:heads[n+1]if n+1<len(heads)else len(lines)])for n,i in enumerate(heads)}
selected={h:b for h,b in blocks.items()if '// String room-exit'in b or 'Method beginRentalTownMapTransition:'in b or re.search(r'\b(?:useRentalHotelKey|handleRentalHotelKey|leaveRentalHotel|prepareRentalHotelDeparture|settleRequestedBattleExit)\(',h)or('Hotel'in h and re.search('Key|Use|Exit|Depart',h))}
assert selected and any('// String room-exit'in b for b in selected.values())
out=V/'native-catalog-hotel-exit-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
data=dict(status='current75_room_exit_static_only',reference_jar_sha256=sha(jar),command=cmd,javap_exit_code=z.returncode,source_manifest_sha256=json.loads((R/'工作佇列.json').read_text('utf-8-sig'))['current_product_identity']['source_manifest_sha256'],normal_login_sha256=sha(P/'src/modern_lito/native_login.rs'),methods=selected,actual_sessions=0,manual_guest_exit_runtime_verified=False,normal_source_modified=False,launcher_artifact_modified=False,whole_goal_complete=False)
(out/'methods.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(output=str(out),methods={h:len(b.splitlines())for h,b in selected.items()},actual_sessions=0),ensure_ascii=False))
