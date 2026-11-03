"""Bind retained canonical cold assets to primary PE type5 factory bytes only."""
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

ranges=dict(direction=(0x4b9210,0x4b92c0),entry_direction_order=(0x42f570,0x42f5d8),vector_align=(0x4b92d0,0x4b92e5));decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()};ins={i.address:i for rows in decoded.values() for i in rows}
expected={0x4b9226:('shl','edx, 1'),0x4b9230:('mov','eax, dword ptr [ebp + 0xc]'),0x4b9237:('lea','ecx, [edx + edx*4]'),0x4b923a:('lea','edx, [eax + eax*2]'),0x4b923d:('shl','ecx, 1'),0x4b923f:('shl','edx, 3'),0x4b924b:('jle','0x4b9288'),0x4b9256:('jle','0x4b9261'),0x4b926a:('jle','0x4b9275'),0x4b927c:('setle','dl'),0x4b928a:('jle','0x4b9295'),0x4b929e:('jle','0x4b92a9'),0x4b92bb:('setg','al'),0x42f5b3:('call','0x4b9210'),0x42f5c2:('call','0x4b92d0'),0x4b92e0:('and','edi, 0xfffffffe')}
for address,op in expected.items():assert (ins[address].mnemonic,ins[address].op_str)==op,(hex(address),op)
out=ROOT/'驗證紀錄/current/native-cone-cold-direction'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text(chr(10).join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+chr(10),encoding='utf-8')
record=dict(status='primary_native_direction_integer_thresholds_bound_only',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),function='0x4B9210',verified=['Direction reads raw native destination differences before vector helper even-X alignment.','The ordinate is scaled to20*dy and compared with signed24*dx,4*dx,-4*dx,-24*dx using the recorded inclusive/exclusive branches.','Coincident points return the caller-provided direction; the caller stores the low byte.','Entry42F570 computes direction before calling vector4B92D0.'],world_coordinate_conversion_qualified=False,asset_direction_image_mapping_qualified=False,normal_product_source_modified=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),new_casts=0,renderer_qualified=False),ensure_ascii=False))
