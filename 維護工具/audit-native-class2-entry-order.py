"""Read-only class2 actor-marker dispatch and first callback ordering evidence."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
p=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
b=p.read_bytes();assert sha(p)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
e=struct.unpack_from('<I',b,60)[0];count=struct.unpack_from('<H',b,e+6)[0];sections=e+24+struct.unpack_from('<H',b,e+20)[0]
def raw(lo,hi):
    for i in range(count):
        size,rva,physical,at=struct.unpack_from('<IIII',b,sections+i*40+8)
        if rva<=lo-0x400000 and hi-0x400000<=rva+min(size,physical):return b[at+lo-0x400000-rva:at+hi-0x400000-rva]
    raise ValueError(hex(lo))
ranges=dict(class2_vtable=(0x4f84b8,0x4f84c0),actor_marker_launch=(0x4e45c0,0x4e45e5),
    actor_hit_branch=(0x4e580a,0x4e585e),actor_store_pending=(0x4e0d28,0x4e0d60),
    class2_entry=(0x44eda0,0x44ede0),class2_callback=(0x44ebd0,0x44ec72),
    finite_advance=(0x4e5b50,0x4e5bf2),base_registration=(0x4e78b0,0x4e7950))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
rows={name:list(md.disasm(raw(lo,hi),lo)) for name,(lo,hi) in ranges.items() if name!='class2_vtable'}
ins={x.address:x for group in rows.values() for x in group}
assert struct.unpack('<II',raw(*ranges['class2_vtable']))==(0x44eb90,0x44eda0)
predicates={0x4e45d2:None,0x44edd2:'call 0x4e78b0',0x44edd9:'call 0x44ebd0',0x44ebe9:'call 0x4e5b50',0x4e5b58:'inc bl',0x4e5b5e:'mov byte ptr [ecx + 0x13], bl'}
# Keep exact addresses for established instructions; locate the virtual marker call independently.
del predicates[0x4e45d2]
for addr,text in predicates.items():assert ins[addr].mnemonic+' '+ins[addr].op_str==text,(hex(addr),text)
launch=rows['actor_marker_launch'];assert any(x.mnemonic=='call' and x.op_str=='dword ptr [eax + 4]' for x in launch)
assert any(x.mnemonic=='mov' and x.op_str=='dword ptr [esi + 0x80], 0' for x in launch)
assert any(x.mnemonic=='mov' and x.op_str=='dword ptr [esi + 0x80], ecx' for x in rows['actor_store_pending'])
out=ROOT/'驗證紀錄/current/native-class2-entry-order'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi))
    if name in rows:(out/(name+'.asm')).write_text('\n'.join(f'{x.address:08X} {x.bytes.hex():<24} {x.mnemonic} {x.op_str}' for x in rows[name])+'\n',encoding='utf-8')
save(out/'binding.json',dict(status='static_primary_PE_entry_order_only',primary_PE=str(p),primary_PE_sha256=sha(p),
    observed=['S23 constructed effect is held in attacker field80 until actor marker dispatch.',
    '4E45C0 invokes pending effect virtual slot4 then clears field80; class2 vtable slot4 is44EDA0.',
    '44EDA0 resolves current target position, invokes base registration4E78B0, then44EBD0.',
    '44EBD0 calls4E5B50; this increments initial source frame0 to1 before4E5C00 and next callback scheduling.',
    'Frame2 override runs after advance; wrapped frame0 dwells until a following callback retires effect.'],
    inference=['Clip start at source1 matches the first dispatched callback state and event order.',
    'Base registration precedes first advancement; actual compositor visibility of transient source0 remains unqualified.'],
    byte_predicates={hex(k):v for k,v in predicates.items()},product_source_modified=False,new_casts=0,new_renders=0,renderer_qualified=False,whole_goal_complete=False))
(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'archive-readback.json',dict(files=pins,archive_sha256=sha(out/'raw.zip'),verified=True))
print(json.dumps(dict(binding=str(out/'binding.json'),renderer_qualified=False),ensure_ascii=False))
