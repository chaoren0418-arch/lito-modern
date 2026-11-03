"""Bind class2 coordinates to its one-shot actor-marker entry, not timer refresh."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
ORIGINAL=ROOT.parent/'lito登入器/舊版分析與驗收'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pp=ORIGINAL/'runtime/LiTo.unpacked-oep-current.exe';ap=ORIGINAL/'ida-client-oep-v2/full-disassembly.asm'
assert sha(pp)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ap)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
pe=pp.read_bytes();e=struct.unpack_from('<I',pe,60)[0];st=e+24+struct.unpack_from('<H',pe,e+20)[0]
sections=[struct.unpack_from('<IIII',pe,st+i*40+8) for i in range(struct.unpack_from('<H',pe,e+6)[0])]
def raw(va,n):
    rva=va-0x400000
    for vs,base,rs,at in sections:
        if base<=rva and rva+n<=base+min(vs,rs):return pe[at+rva-base:at+rva-base+n]
    raise ValueError(hex(va))
ranges={'one_shot_actor_marker':(0x4e45c0,0x4e45e5),'class2_entry':(0x44eda0,0x44ede0),
    'class2_timer':(0x44ed80,0x44ed91),'class2_advance':(0x44ebd0,0x44ed7c),
    'grid_unlink':(0x4e01a0,0x4e01fc),'grid_link':(0x4e0200,0x4e025e),
    'base_advance':(0x4e5b50,0x4e5bf4),'base_frame_selection':(0x4e5c00,0x4e63b0),
    'target_handle_ctor':(0x4ea8b0,0x4ea8e9)}
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);md.detail=True
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
instructions={i.address:i for rows in decoded.values() for i in rows}
assert struct.unpack('<II',raw(0x4f84b8,8))==(0x44eb90,0x44eda0)
assert instructions[0x4e45d6].op_str=='dword ptr [eax + 4]'
assert instructions[0x4e45d9].op_str=='dword ptr [esi + 0x80], 0'
expected={0x44edaf:'0x4e8160',0x44edd2:'0x4e78b0',0x44edd9:'0x44ebd0',0x44ed83:'0x4e01a0',0x44ed8a:'0x44ebd0'}
for address,operand in expected.items():assert instructions[address].mnemonic=='call' and instructions[address].op_str==operand
assert [i.op_str for i in decoded['class2_timer'] if i.mnemonic=='call']==['0x4e01a0','0x44ebd0']
def coordinate_writes(rows):
    return [i.address for i in rows if any(op.type==capstone.x86.X86_OP_MEM and op.mem.base==capstone.x86.X86_REG_ESI and op.mem.disp in [0x2c,0x30,0x3c,0x40] and op.access&capstone.CS_AC_WRITE for op in i.operands)]
assert coordinate_writes(decoded['class2_entry'])==[0x44edbb,0x44edc1,0x44edc7,0x44edcd]
for name in ['class2_timer','class2_advance','grid_unlink','grid_link','base_advance','base_frame_selection']:
    assert not coordinate_writes(decoded[name]),name
assert sum(i.mnemonic=='call' and i.op_str=='0x4e8160' for i in decoded['class2_advance'])==1
# The advance's lookup is guarded by the frame Hit bit and dispatches hurt;
# its returned actor is EDI. It never copies actor coordinates to the effect.
assert instructions[0x44ecbf].op_str=='0x4e8160'
assert [i.op_str for i in decoded['target_handle_ctor'] if i.mnemonic=='call']==['0x4f118c','0x4f118c']
out=ROOT/'驗證紀錄/current/native-class2-position'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo))
    (out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex():<24} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n',encoding='utf-8')
candidate=ROOT/'維護工具/candidates/native-class2-presentation/native_call_lightning.rs'
(out/'candidate-before.rs').write_bytes(candidate.read_bytes())
record=dict(status='primary_class2_entry_coordinate_capture_qualified',primary_PE=dict(path=str(pp),sha256=sha(pp)),primary_ASM=dict(path=str(ap),sha256=sha(ap)),
    observed=['4E45C0 invokes pending effect virtual slot4 and clears actor field80; slot4=44EDA0.',
        '44EDA0 resolves the target and copies coordinates to effect2C/30 and3C/40 before registration and advancement.',
        'Timer44ED80 only unlinks raster grids and calls44EBD0; it does not invoke44EDA0.',
        '44EBD0 has a target lookup on the Hit branch for hurt dispatch, but does not copy target coordinates.',
        'Examined grid and frame selection routines do not write effect coordinate fields; target handle constructor stores only scalar target ID and a byte, with no effect back-reference.'],
    conclusion='Sample the matching target incarnation once at the admitted actor-marker launch. Later movement, death, removal or object-ID reuse must not relocate an existing bolt; retain the captured cell until scene retirement or natural completion.',
    inference_scope='Static PE control/data flow; not a live old-client observation or whole-renderer qualification.',
    unsupported_candidate_behavior='Current draft updates Active.position from current target on every dispatch.',candidate_before_sha256=sha(candidate),
    normal_product_source_modified=False,new_sessions=0,new_renders=0,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'archive-readback.json',dict(files=pins,sha256=sha(out/'evidence.zip'),verified=True))
print(json.dumps(dict(binding=str(out/'binding.json'),candidate_coordinate_fix_required=True,normal_source_modified=False),ensure_ascii=False))
