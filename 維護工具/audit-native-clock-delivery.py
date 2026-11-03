"""Pin current clock packet delivery evidence; do not infer calendar API identity."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-clock-delivery'
JAR=ROOT/'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
ASM=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm')
COHORT=ROOT/'驗證紀錄/current/native-teleport-current/20261015-232637-257-TW-shared'
CONNECTION=ROOT/'LiTo381-Modern/src/modern_lito/native_connection.rs'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
assert sha(JAR)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(ASM)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
q=read(COHORT/'qualified.json'); archive=COHORT/'GUI-raw-evidence.zip'
assert sha(archive)==q['raw_archive_sha256']
OUT.mkdir(exist_ok=False)
env=os.environ.copy();env.update(PYTHONDONTWRITEBYTECODE='1',TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'))
selected={}
for classname,names in [('Protocol',['worldTime','blind','weather']),('ClientSession',['nativeTimerNotification'])]:
    args=[r'C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe','-J-Dfile.encoding=UTF-8','-c','-p','-classpath',str(JAR),'local.lito381.server.'+classname]
    result=subprocess.run(args,cwd=OUT,env=env,capture_output=True,timeout=45,creationflags=subprocess.CREATE_NO_WINDOW)
    (OUT/(classname+'-javap.stderr')).write_bytes(result.stderr)
    (OUT/(classname+'-javap.stdout')).write_bytes(result.stdout)
    assert result.returncode==0
    text=result.stdout.decode('utf-8-sig')
    headers=list(re.finditer(r'^  (?!Code:|\s)([^\n]+\([^\n]*\)[^\n]*;)\r?$',text,re.MULTILINE))
    for name in names:
        matches=[(i,m) for i,m in enumerate(headers) if re.search(r'\b'+name+r'\(',m.group(1))]
        assert len(matches)==1,(classname,name,len(matches))
        i,m=matches[0];end=headers[i+1].start() if i+1<len(headers) else len(text)
        selected[classname+'.'+name]=text[m.start():end]
assert 'bipush        33' in selected['Protocol.worldTime']
timer=selected['ClientSession.nativeTimerNotification']
assert 'bipush        32' in timer and 'ClassicLineageClock.now' in timer and 'Protocol.worldTime' in timer
assert re.search(r'51: bipush\s+32\s+53: if_icmpne\s+83',timer)
assert '56: invokestatic' in timer and '80: invokevirtual' in timer and 'Method sendEncrypted:' in timer
(OUT/'actual-C02-bytecode.json').write_text(json.dumps(selected,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')

lines=ASM.read_text(encoding='utf-8-sig').split('\n')
assembly={}
for name in ['sub_47B720','sub_4A5E00','sub_407A70']:
    a=next(i for i,l in enumerate(lines) if l.startswith(name+' proc'))
    b=next(i for i in range(a,len(lines)) if lines[i].startswith(name+' endp'))
    assembly[name]={'first_newline_line':a+1,'last_newline_line':b+1,'assembly':'\n'.join(lines[a:b+1])}
assert 'mov     ecx, 0EA60h' in assembly['sub_47B720']['assembly']
assert 'push    20h' in assembly['sub_47B720']['assembly']
assert 'call    sub_4D0470' in assembly['sub_4A5E00']['assembly']
(OUT/'selected-assembly.json').write_text(json.dumps(assembly,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
captures=[]
with zipfile.ZipFile(archive) as z:
    for profile in ['Debug','Release']:
        for c in q['profiles'][profile]['cases']:
            member=profile+'/'+c['scenario']+'/raw-capture/frames.jsonl'
            raw=z.read(member)
            assert hashlib.sha256(raw).hexdigest().upper()==q['raw_archive_files'][member]
            rows=[json.loads(line) for line in raw.decode('utf-8-sig').splitlines()]
            sends=[r for r in rows if r['direction']=='client_to_server' and r['opcode']=='0x20']
            times=[r for r in rows if r['direction']=='server_to_client' and r['opcode']=='0x21']
            assert all(bytes.fromhex(r['plain'])==bytes([0x20]) for r in sends)
            assert all(len(bytes.fromhex(r['plain']))==5 for r in times)
            captures.append({'profile':profile,'scenario':c['scenario'],'member':member,
                'capture_sha256':q['raw_archive_files'][member],'C_ALIVE_count':len(sends),
                'S_TIME_count':len(times),'S_TIME':[{'time':r['time'],'raw':int.from_bytes(bytes.fromhex(r['plain'])[1:],'little'),'note':r.get('note')} for r in times]})
assert sum(c['C_ALIVE_count'] for c in captures)==0
(OUT/'retained-capture-clock-observations.json').write_text(json.dumps(captures,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(OUT/'native_connection.rs').write_bytes(CONNECTION.read_bytes())
(OUT/'audit.py').write_bytes(Path(__file__).read_bytes())
proof={'status':'current_C02_keepalive_clock_reply_and_original_60s_callback_contract_pinned',
    'reference_jar_sha256':sha(JAR),'original_assembly_sha256':sha(ASM),'cohort_qualification_sha256':sha(COHORT/'qualified.json'),
    'cohort_raw_archive_sha256':sha(archive),'observed_scenarios':14,'observed_C_ALIVE_packets':0,
    'observed_S_TIME_packets':sum(c['S_TIME_count'] for c in captures),
    'original_C_ALIVE_opcode':32,'original_reschedule_millis':60000,
    'C02_C_ALIVE_reply':'S_TIME dword from ClassicLineageClock.now()',
    'next_work':'Add epoch-owned GAME timer delivery after authentication, verify original registration and no LOGIN/stale epoch sends, then bind S21 environment state to transport lifetime.',
    'absence_scope':'No C20 in these archived captures; this is not an exhaustive claim about all program paths or a per-socket 60s live timer test.',
    'original_calendar_UTC_or_local_resolved':False,'client_clock_local_advance_verified':False,
    'timer_product_fix_implemented':False,'new_game_sessions':0,'whole_goal_complete':False}
pins={p.name:sha(p) for p in OUT.iterdir() if p.is_file()}
packed=OUT/'clock-delivery-evidence.zip'
with zipfile.ZipFile(packed,'x',zipfile.ZIP_DEFLATED) as z:
    for name in pins:z.write(OUT/name,name)
with zipfile.ZipFile(packed) as z:
    assert z.testzip() is None
    for name,v in pins.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==v
proof.update(archive_sha256=sha(packed),archive_files=pins,archive_readback_verified=True)
(OUT/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:v for k,v in proof.items() if k!='archive_files'},ensure_ascii=False))
