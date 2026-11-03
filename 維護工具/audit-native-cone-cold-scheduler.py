"""Bind relative queue arithmetic; retain unresolved external clock identity."""
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

ranges=dict(scheduler=(0x4d65f0,0x4d66ec),clock=(0x4d69f0,0x4d6a40),clock_init=(0x4d6360,0x4d6390));decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
ops={name:[(i.mnemonic,i.op_str) for i in rows] for name,rows in decoded.items()}
for op in [('mov','esi, ecx'),('call','0x4d69f0'),('add','esi, edi'),('mov','dword ptr [ebp - 0x10], esi'),('mov','dword ptr [esi], edx'),('mov','dword ptr [esi + 4], eax')]:assert op in ops['scheduler'],op
for op in [('push','0x3e8'),('call','0x4f11f0'),('call','0x4f1230'),('mov','ecx, dword ptr [0x834bec]'),('mov','ecx, dword ptr [0x834be8]')]:assert op in ops['clock'],op
for op in [('mov','dword ptr [0x834bf0], eax'),('mov','dword ptr [0x834be8], ecx'),('mov','dword ptr [0x834bec], edx')]:assert op in ops['clock_init'],op
asm_lines=asm.split(b'\n');quote_lines=[]
for lo,hi in [(448355,448375),(449425,449462)]:
    quote_lines.extend({'line':i,'text':asm_lines[i-1].decode('utf-8-sig').strip('\r')} for i in range(lo,hi+1))
# ASM is LF-separated with retained CR bytes; avoid universal-newline renumbering.
if len(asm_lines)<449462:
    asm_lines=asm.split(bytes([10]));quote_lines=[]
    for lo,hi in [(448355,448375),(449425,449462)]:quote_lines.extend({'line':i,'text':asm_lines[i-1].decode('utf-8-sig').rstrip(chr(13))} for i in range(lo,hi+1))
assert any('5A10C4E' in row['text'] for row in quote_lines)
external={name:[dict(address=hex(i.address),target=i.op_str) for i in rows if i.mnemonic=='call' and i.op_str.startswith('0x') and int(i.op_str,16)>0x01000000] for name,rows in decoded.items()}
assert any(row['target']!='0x5a10c4e' for row in external['clock'])
out=ROOT/'驗證紀錄/current/native-cone-cold-scheduler'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text(chr(10).join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+chr(10),encoding='utf-8')
save(out/'retained-ASM-clock-lines.json',quote_lines)
record=dict(status='primary_relative_scheduler_arithmetic_bound_external_clock_mapping_pending',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),scheduler='0x4D65F0',clock='0x4D69F0',clock_initializer='0x4D6360',external_calls_in_current_PE=external,
    verified=['Delay ECX is preserved before querying the native clock.','The newly queried current clock value is added to the delay before queue insertion.','The stored queue row contains the computed deadline, callback and caller parameter.','The high-resolution branch multiplies a two-word counter result by1000 and divides by the initialized two-word denominator.'],
    unresolved=['API trampoline targets differ between retained ASM and current PE; their external counter/frequency/fallback identities are not established by this audit.','Physical milliseconds, fallback behavior, tick wrapping and global cross-object dispatch order remain unqualified.'],from_now_relative_deadline_arithmetic_verified=True,physical_time_unit_qualified=False,global_timer_wrap_qualified=False,global_callback_order_qualified=False,normal_product_source_modified=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),from_now_arithmetic_verified=True,physical_time_unit_qualified=False,whole_goal_complete=False),ensure_ascii=False))
