"""Read-only queue binding for the observed walk/hurt camera regression."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,difflib,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pe_path=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
assert sha(pe_path)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pe_path.read_bytes();e=struct.unpack_from('<I',pe,60)[0];st=e+24+struct.unpack_from('<H',pe,e+20)[0]
sections=[struct.unpack_from('<IIII',pe,st+i*40+8) for i in range(struct.unpack_from('<H',pe,e+6)[0])]
def raw(va,n):
    rva=va-0x400000
    for vs,base,rs,at in sections:
        if base<=rva and rva+n<=base+min(vs,rs):return pe[at+rva-base:at+rva-base+n]
    raise ValueError(hex(va))
ranges={'active_walk_predicate':(0x4e6b70,0x4e6b8a),'hurt_dispatch':(0x44ed1c,0x44ed5e),'queued_body_action':(0x4e63c0,0x4e647a)}
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
allins={i.address:i for rows in decoded.values() for i in rows}
expected={0x4e6b76:'cmp al, dl',0x4e6b7d:'test al, al',0x44ed1e:'call 0x4e6b70',0x44ed2b:'call 0x4e63c0',0x4e6400:'mov dword ptr [eax], 0x20',0x4e6406:'mov dword ptr [eax + 4], esi'}
for va,text in expected.items():assert allins[va].mnemonic+' '+allins[va].op_str==text
out=ROOT/'驗證紀錄/current/native-class2-motion-queue'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo))
    (out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex():<24} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n',encoding='utf-8')
files={}
for name in ['native_world_animation.rs','native_combat_animation.rs']:
    before=ROOT/'LiTo381-Modern/src/modern_lito'/name;after=ROOT/'維護工具/candidates/native-class2-presentation'/name
    (out/('before-'+name)).write_bytes(before.read_bytes());(out/('candidate-'+name)).write_bytes(after.read_bytes())
    (out/(name+'.diff')).write_text(''.join(difflib.unified_diff(before.read_text(encoding='utf-8-sig').splitlines(True),after.read_text(encoding='utf-8-sig').splitlines(True),fromfile=str(before),tofile=str(after))),encoding='utf-8')
    files[name]=dict(product_before_sha256=sha(before),candidate_sha256=sha(after))
record=dict(status='primary_running_walk_queues_hurt_binding',primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),
    observed=['4E6B70 tests current action equals walking stance and source frame is nonzero.',
        '44ED1E tests4E6B70; the walking branch pushes action2 and calls4E63C0.',
        '4E63C0 appends a typed body-action command20h and suppresses consecutive queued hurts.'],
    candidate_fix='Keep motion sampling live; hold the separate combat playback clock until the running walk boundary, so queued hurt cannot snap camera to the destination then restart a partial walk. Authoritative death retains its immediate path.',
    files=files,water_history_rewind_guard_removed=False,normal_product_source_modified=False,new_sessions=0,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'archive-readback.json',dict(files=pins,sha256=sha(out/'evidence.zip'),verified=True))
print(json.dumps(dict(binding=str(out/'binding.json'),normal_source_modified=False,rewind_guard_preserved=True),ensure_ascii=False))
