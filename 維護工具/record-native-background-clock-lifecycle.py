"""Retain static callback start/cancel/restart and direct phase writers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
ASM=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
raw=ASM.read_bytes();assert digest(raw)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=raw.decode('utf-8',errors='replace').split('\n')
references=[];writes=[];function=None
for i,line in enumerate(lines):
    if ' proc near' in line:function=line.split()[0]
    if 'dword_62F4C0' in line or 'dword_62F4C4' in line:
        references.append({'original_LF_line':i+1,'function':function,'text':line.rstrip('\r')})
        if line.startswith('mov     dword_62F4C'):writes.append(i+1)
assert writes==[287814,287815]
assert lines[551678].startswith('dword_62F4C0 dd 0FFCCh') and lines[551684].startswith('dword_62F4C4 dd 0FFFEh')
spans=[(269148,269163),(269384,269414),(287804,287823),(354111,354162)]
for a,b in spans:assert all(0<i<=len(lines) for i in [a,b])
out=ROOT/'驗證紀錄/current/native-background-clock-lifecycle'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'selected-assembly.txt').write_text('\n'.join(f'{i}:{lines[i-1].rstrip(chr(13))}' for a,b in spans for i in range(a,b+1))+'\n',encoding='utf-8')
(out/'observer.py').write_bytes(Path(__file__).read_bytes());save(out/'phase-references.json',references)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h
proof={'status':'static_background_phase_callback_lifecycle_retained_not_executed','path':str(out),
    'original_ASM_sha256':digest(raw),'raw_initial_phases':[65484,65534],
    'callback_increment_before_enqueue':True,'enqueue_delay_ms':64,
    'direct_start_caller':'sub_47CF00','cancel_caller':'sub_47E4B0','cancel_then_immediate_restart_caller':'sub_4A98A0',
    'all_named_direct_phase_writes_are_in_callback':True,'explicit_zero_reset_observed':False,
    'indirect_writes_ruled_out':False,'callback_lifecycle_executed':False,'modern_clock_policy_qualified':False,
    'files':files,'raw_archive_sha256':digest((out/'raw.zip').read_bytes()),'archive_readback_verified':True,
    'game_sessions':0,'whole_goal_complete':False}
save(out/'observation.json',proof);save(APP/'background-clock-lifecycle-current.json',proof)
assert digest(ASM.read_bytes())==digest(raw)
print(json.dumps({k:proof[k] for k in ['status','path','archive_readback_verified']},ensure_ascii=False))
