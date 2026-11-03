"""Retain original callback/camera paths without running or modifying them."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
ASM=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
data=ASM.read_bytes();sha=lambda b:hashlib.sha256(b).hexdigest().upper()
assert sha(data)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=data.decode('utf-8',errors='replace').split('\n')
out=ROOT/'驗證紀錄/current/native-lighting-actor-motion-static'/('water-lifecycle-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir();functions={}
def extent(name):
    a=next(i for i,s in enumerate(lines) if s.startswith('sub_'+name+' proc'))
    b=next(i for i in range(a,len(lines)) if lines[i].startswith('sub_'+name+' endp'))
    return a,b
for name in ['4874D0','487510','4D65F0','4D6870','485800','48BC80','413A80','4142F0','4A7130']:
    a,b=extent(name);functions[name]={'first_newline_line':a+1,'last_newline_line':b+1,'assembly':'\n'.join(lines[a:b+1]).replace('\r','')}
selected=[]
for name in ['4E2150','47E4B0']:
    a,b=extent(name);wanted=set()
    for i in range(a,b+1):
        if any(word in lines[i] for word in ['4874D0','487510','64BC2D']):wanted.update(range(max(a,i-12),min(b+1,i+13)))
    selected.append({'function':name,'lines':[{'newline_line':i+1,'assembly':lines[i].replace('\r','')} for i in sorted(wanted)]})
record={'status':'static_callback_multiplicity_cancel_and_dry_scroll_paths_retained',
 'original_assembly_sha256':sha(data),'functions':functions,'selected':selected,
 'findings':[
  '4874D0 unconditionally updates X=(X+dx+640)%640 and Y=(Y+dy+420)%420 with signed idiv. No water flag check in the function.',
  '4E2150 own walking calls4874D0 with negated camera frame deltas. Its only explicit64BC2D reference is later near the function end; do not limit scroll accumulation to water draws.',
  '485800 and48BC80 schedule487510 after50ms if water. They do not directly cancel487510;413A80 cleanup calls4142F0, which cancels414310 for its object, not global487510/owner0.',
  '4D65F0 inserts stable deadlines, permits duplicate callback/owner records. Repeated S28 or a water map-loader call can add another chain unless another path cancels it.',
  '4D6870 removes every matching callback/owner record, rechecking the shifted entry before advancing.',
  'S28false and47E4B0 clear water flag and cancel487510/owner0 without directly writing phase/counter.487510 with waterfalse resets both only if actually invoked.',
  'Do not substitute elapsed-time catchup, a single deduplicated timer, water-entry scroll reset, or absolute camera jump deltas for these paths.'
 ],'scope':'Static direct call/data paths only. Indirect bulk initialization, complete map-remap dispatch and native scheduler execution still require qualification.',
 'native_execution':False,'product_phase_scroll_integrated':False,'actual_game_sessions':0,'whole_goal_complete':False}
(out/'evidence.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'observer.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in ['evidence.json','observer.py']:z.write(out/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    for name in ['evidence.json','observer.py']:assert z.read(name)==(out/name).read_bytes()
(out/'readback.json').write_text(json.dumps({'status':'static_archive_readback_verified','sha256':sha((out/'raw.zip').read_bytes())},indent=2)+'\n',encoding='utf-8')
print(out)
