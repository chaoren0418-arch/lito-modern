"""Retain original producer flags; static evidence is distinct from selector execution."""
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
out=ROOT/'驗證紀錄/current/native-object-producer-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
spans=[]
for name in ['sub_4E0200','sub_4E7000','sub_4CD9D0','sub_483770']:
    a=next(i for i,s in enumerate(lines) if s.startswith(name+' proc near'))
    b=next(i for i in range(a+1,len(lines)) if lines[i].startswith(name+' endp'))
    spans.append({'name':name,'first_original_LF_line':a+1,'last_original_LF_line':b+1})
    (out/(name+'.txt')).write_text('\n'.join(f'{i+1}:{lines[i].rstrip(chr(13))}' for i in range(a,b+1))+'\n',encoding='utf-8')
(out/'TIL-object-flags.txt').write_text('\n'.join(f'{i}:{lines[i-1].rstrip(chr(13))}' for a,b in [(281230,281274),(281357,281399),(282593,282633),(282716,282758)] for i in range(a,b+1))+'\n',encoding='utf-8')
assert 'or      edx, 5' in (out/'sub_4E0200.txt').read_text(encoding='utf-8')
attachment=(out/'sub_4E7000.txt').read_text(encoding='utf-8')
for text in ['test    al, 2','test    al, 40h','test    ah, 1','or      al, 40h','or      ecx, 1']:assert text in attachment
(out/'observer.py').write_bytes(Path(__file__).read_bytes())
proof={'status':'static_original_object_producer_flag_contract_retained','path':str(out),'original_ASM_sha256':digest(raw),
    'functions':spans,'body_flags':'actor+1A word passed unchanged','shadow_flags':'(body_flags & 0x0fff) | 5',
    'attachment_flags':'skip on body bit2; own flags | 0x40 if body bit40, else own flags | 1 if body bit100, else own flags',
    'TIL_object_flags':'bit10 -> 8; else bit4 -> 0x100; else (type & 0x20) << 2',
    'currently_accepted_catalog_flags':{'body':[0,1],'attachment':[0,8],'shadow':[0]},
    'gate_derived_from_final_alpha':False,'producer_machine_code_executed':False,'all_unsupported_producer_flags_complete':False,
    'whole_scene_gate_verified':False,'normal_map303_GUI_verified':False,'game_sessions':0,'whole_goal_complete':False}
save(out/'contract.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h
save(APP/'object-producer-contract-current.json',{**proof,'files':files,'archive_sha256':digest((out/'raw.zip').read_bytes()),'archive_readback_verified':True})
assert digest(ASM.read_bytes())==digest(raw)
print(json.dumps({'status':proof['status'],'path':str(out),'archive_readback_verified':True},ensure_ascii=False))
