"""Read primary type5 movement/arrival byte flow; no renderer qualification."""
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
ranges=dict(parser109=(0x43e6b2,0x43e6e2),factory_selector=(0x4a5fab,0x4a5fe0),factory_type5=(0x4a61cd,0x4a6234))
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
parser=[(i.mnemonic,i.op_str) for i in decoded['parser109']]
for op in [('cmp','edi, 0x6d'),('mov','edx, dword ptr [0x647e5c]'),('mov','byte ptr [esi + edx], al'),('mov','ecx, dword ptr [0x647e50]'),('mov','word ptr [ecx + esi*2], ax')]:assert op in parser,op
selector=[(i.mnemonic,i.op_str) for i in decoded['factory_selector']]
for op in [('mov','cl, byte ptr [eax + edx]'),('dec','eax'),('jmp','dword ptr [eax*4 + 0x4a63c4]')]:assert op in selector,op
table=struct.unpack('<8I',raw(0x4a63c4,32));assert table[1]==0x4a5fe0 and table[4]==0x4a61cd
branch=[(i.mnemonic,i.op_str) for i in decoded['factory_type5']]
for op in [('push','0x160'),('mov','ecx, dword ptr [0x647e50]'),('movsx','ecx, word ptr [ecx + edx*2]'),('call','0x42f1f0')]:assert op in branch,op

upstream=ROOT/'驗證紀錄/current/native-cone-cold-binding/20261018-034221-586202/binding.json';u=read(upstream);assert u['effect_type']==5 and u['constructor']=='0x42F1F0'
assert sha(upstream)==read(upstream.parent/'readback.json')['files']['binding.json']
ranges=dict(ctor=(0x42f1f0,0x42f2f5),advance=(0x42f300,0x42f543),timer=(0x42f550,0x42f561),entry=(0x42f570,0x42f614),vector=(0x4b92d0,0x4b9356))
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()};ins={i.address:i for rows in decoded.values() for i in rows}
# Derive exact call operands separately; the payload bytes are archived below.
assert ins[0x42f222].mnemonic=='mov' and ins[0x42f222].op_str=='dword ptr [esi], 0x4f8054'
assert struct.unpack('<I',raw(0x4f8058,4))[0]==0x42f570
for address,op in {0x42f585:('call','0x4e8160'),0x42f591:('mov','dword ptr [esi + 0x14c], ecx'),0x42f59a:('mov','dword ptr [esi + 0x150], edx'),0x42f5c2:('call','0x4b92d0'),0x42f60a:('call','0x42f300'),0x42f361:('and','edx, 0xfffffffe'),0x42f4f0:('mov','ecx, 0x3c'),0x42f4f5:('call','0x4d65f0'),0x42f3ce:('call','0x4e8160'),0x42f44b:('call','0x4e63c0'),0x42f479:('call','0x4e45f0'),0x42f499:('mov','ecx, dword ptr [edi + 0x30]'),0x42f49c:('mov','edx, dword ptr [edi + 0x2c]'),0x42f4b7:('call','0x44ede0')}.items():
    assert (ins[address].mnemonic,ins[address].op_str)==op,(hex(address),op)
assert [(i.mnemonic,i.op_str) for i in decoded['timer'] if i.mnemonic=='call']==[('call','0x4e01a0'),('call','0x42f300')]
assert sum(i.mnemonic=='call' and i.op_str=='0x4e8160' for i in decoded['advance'])==1
assert ins[0x4b92fa].mnemonic=='sar' and ins[0x4b92fa].op_str=='esi, 1'
assert ins[0x4b9347].mnemonic=='shl' and ins[0x4b9347].op_str=='ecx, 1'
out=ROOT/'驗證紀錄/current/native-cone-cold-motion'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n',encoding='utf-8')
record=dict(status='primary_type5_motion_and_arrival_control_flow_bound_only',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),upstream_factory_binding=str(upstream),upstream_factory_binding_sha256=sha(upstream),vtable='0x4F8054',virtual_entry='0x42F570',advance='0x42F300',timer='0x42F550',scheduler_argument=60,
    findings=['Entry42F570 resolves the target once into destination14C/150, derives direction/vector, registers and immediately advances.','Timer42F550 only unlinks and advances; it never re-enters42F570 or refreshes the captured destination.','Advance adds signed vector144/148 to fixed16 positions13C/140; rounded X is forced even.','The remaining bound154 decrements each advance; arrival compares rounded position to the captured destination.','Only the arrival branch re-resolves target handles. Conditional hurt is dispatched before constructing the impact node.','Impact44EDE0 receives the looked-up target current2C/30 coordinates and child graphic15C, then its virtual entry is called.','Non-arrival schedules callback42F550 with literal argument60, not a duration taken from the fly catalogue frames.','Vector4B92D0 normalizes using max(abs(dx)/2,abs(dy)), restores signs and doubles the X fixed-point component.'],
    pending=['Bind wire/world-cell to native coordinate conversion and all eight direction boundaries.','Confirm scheduler units/from-now semantics and exact fly frame selection/overlay sample phase before selecting a Rust clock.','Bind target incarnation/death/removal and derived impact audio identities without local HP/MP mutation.','Run actual canonical C02 positive/refused and moving-target cases only after candidate implementation.'],normal_product_source_modified=False,source_or_QA_override_used=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),scheduler_argument=60,new_casts=0,renderer_qualified=False,whole_goal_complete=False),ensure_ascii=False))
