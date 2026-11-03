"""Read-only original handlers; retain evidence inside the rebuild project."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile
ROOT=Path(__file__).resolve().parents[1]
ASM=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
data=ASM.read_bytes()
sha=lambda b:hashlib.sha256(b).hexdigest().upper()
assert sha(data)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=data.decode('utf-8',errors='replace').split('\n')
out=ROOT/'驗證紀錄/current/native-lighting-actor-motion-static'/('relation-updates-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
fragments={}
for name in ['4AEFD0','4AFDC0','4E7720','4E7690','4D3A80']:
    a=next(i for i,s in enumerate(lines) if s.startswith('sub_'+name+' proc'))
    b=next(i for i in range(a,len(lines)) if lines[i].startswith('sub_'+name+' endp'))
    fragments[name]={'first_newline_line':a+1,'last_newline_line':b+1,'assembly':'\n'.join(lines[a:b+1]).replace('\r','')}
record={'status':'static_relation_update_paths_retained','original_assembly':str(ASM),'sha256':sha(data),
 'S4F':'4AEFD0 ->4E7720 replaces actor+64 master pointer without writing1F/20',
 'S69':'4AFDC0 ->4E7690 replaces actor+60 clan; active war: empty clan clears20 only; enemy writes20=2/1F=1; own clan writes20=1/1F=1 without constructor master exception; unrelated clan retains flags',
 'comparison_policy':'Original4D3A80 uses unresolved ACP lead/fold tables. Rebuild explicitly uses MS950 lead pairs plus ASCII folding; no claim of live original table initialization parity.',
 'fragments':fragments,'native_handler_execution':False,'actual_game_sessions':0,'whole_goal_complete':False}
(out/'evidence.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'observer.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in ['evidence.json','observer.py']:z.write(out/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    for name in ['evidence.json','observer.py']:assert z.read(name)==(out/name).read_bytes()
(out/'readback.json').write_text(json.dumps({'status':'static_archive_readback_verified','sha256':sha((out/'raw.zip').read_bytes()),'scope':'static only'},indent=2)+'\n',encoding='utf-8')
print(out)
