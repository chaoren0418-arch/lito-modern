"""Bind type5 child impact constructor and finite one-shot default path."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950';c=read(normal/'cohort.json');assert c['source_manifest_sha256']=='86E65C7B9D9D77FF73FF154B59F28465B75BA1B3C1D1C6EF315D4990584B45AE'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
for n,h in inputs.items():assert sha(ROOT/n)==h
original=ROOT.parent/'lito登入器/舊版分析與驗收';pe_path=original/'runtime/LiTo.unpacked-oep-current.exe';asm_path=original/'ida-client-oep-v2/full-disassembly.asm';pe=pe_path.read_bytes();asm=asm_path.read_bytes()
assert hashlib.sha256(pe).hexdigest().upper()=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert hashlib.sha256(asm).hexdigest().upper()=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',pe,60)[0];assert pe[e:e+4]==b'PE\0\0' and struct.unpack_from('<H',pe,e+4)[0]==0x14c
count=struct.unpack_from('<H',pe,e+6)[0];sections=e+24+struct.unpack_from('<H',pe,e+20)[0];assert struct.unpack_from('<I',pe,e+24+28)[0]==0x400000
def raw(a,length):
    for i in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+i*40+8)
        if start<=a-0x400000 and a-0x400000+length<=start+min(size,physical):return pe[at+a-0x400000-start:at+a-0x400000-start+length]
    raise ValueError(hex(a))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges=dict(arrival=(0x42f47e,0x42f4de),ctor=(0x44ede0,0x44eeb3),entry=(0x44f350,0x450032),advance=(0x44ef80,0x44f322),timer=(0x44f330,0x44f341))
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()};ins={i.address:i for rows in decoded.values() for i in rows}
expected={0x42f49f:('push','0'),0x42f4a1:('push','0'),0x42f4af:('push','0'),0x42f4b7:('call','0x44ede0'),0x44ee12:('mov','dword ptr [esi], 0x4f84c0'),0x44ee2f:('mov','cl, byte ptr [ebp + 0x14]'),0x44ee4e:('mov','byte ptr [esi + 0x12], cl'),0x44ee5b:('mov','edi, dword ptr [ebp + 0x20]'),0x44ee66:('je','0x44ee95'),0x44ee95:('mov','dword ptr [esi + 0x68], ebx'),0x44ee9b:('mov','byte ptr [esi + 0x13c], bl'),0x45001c:('call','0x4e78b0'),0x450023:('call','0x44ef80')}
for address,op in expected.items():assert (ins[address].mnemonic,ins[address].op_str)==op,(hex(address),op)
assert struct.unpack('<I',raw(0x4f84c4,4))[0]==0x44f350
# This is a bounded read of the selector's static compare/jump edges. No PE
# instruction is executed and no native process is opened or attached.
a=0x44f370;comparison=None;path=[];tests=[]
while a!=0x45001a:
    i=ins[a];path.append(dict(address=hex(a),mnemonic=i.mnemonic,operands=i.op_str));a+=i.size
    if i.mnemonic=='mov':assert i.op_str=='ax, word ptr [esi + 0x14]'
    elif i.mnemonic=='cmp':
        assert i.op_str.startswith('ax, ');value=int(i.op_str.split(', ')[1],0);tests.append(value);comparison=11740-value
    elif i.mnemonic in ['je','jne']:
        assert comparison is not None
        if (comparison==0)==(i.mnemonic=='je'):a=int(i.op_str,0)
    else:raise AssertionError(('unexpected selector edge',i.mnemonic,i.op_str))
    assert len(path)<=20
assert tests==[0x9b3,0x9b4,0x2f6,0x71b,0x2f5,0x8d5]
assert [(i.mnemonic,i.op_str) for i in decoded['timer'] if i.mnemonic=='call']==[('call','0x4e01a0'),('call','0x44ef80')]
advance_ops={(i.mnemonic,i.op_str) for i in decoded['advance']}
for op in [('call','0x4e5b50'),('mov','byte ptr [esi + 0x13c], 1'),('mov','ecx, dword ptr [ebp - 0x14]'),('mov','edx, 0x44f330'),('call','0x4d65f0'),('call','0x4e0200'),('mov','al, byte ptr [esi + 0x12]'),('mov','edx, dword ptr [esi + 0x68]')]:assert op in advance_ops,op
out=ROOT/'驗證紀錄/current/native-cone-cold-impact'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text(chr(10).join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+chr(10),encoding='utf-8')
record=dict(status='primary_type5_impact_default_entry_no_second_hurt_or_ambient_bound_only',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),constructor='0x44EDE0',vtable='0x4F84C0',virtual_entry='0x44F350',advance='0x44EF80',timer='0x44F330',selector_path=path,selector_compared_graphics=tests,findings=['Type5 arrival passes null target-list arguments20/24 and zero ambient byte14 into impact44EDE0. The constructor stores68=null and12=0.','Graphic11740 follows the bounded static entry selector directly to45001A, registering4E78B0 and immediately advancing44EF80. No direction offset special case is selected.','The impact has no target list, so its Hit source event cannot dispatch a second target hurt; type5 already dispatched conditional hurt before constructing this child.','Ambient branch is disabled by byte12=0. This impact must not acquire a class1/class2 flash lease.','44EF80 advances source1..7 then0. The wrap sets13C but still schedules and submits the terminal source; only the following callback retires.','Each impact reschedule uses returned source dwell from4E5B50, unlike the fly branch literal60; callbacks unlink first and never resample the target position.'],impact_anchor='target current coordinates copied at flight arrival, then immutable',impact_source_order=[1,2,3,4,5,6,7,0],impact_terminal_dwell_preserved=True,impact_duplicate_hurt_forbidden=True,impact_ambient_lease=False,timer_external_units_qualified=False,normal_product_source_modified=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),duplicate_hurt_forbidden=True,impact_ambient_lease=False,renderer_qualified=False),ensure_ascii=False))
