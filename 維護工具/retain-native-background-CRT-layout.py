"""Compare saved CRT trampolines with local x86 exports; do not load DLLs."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,zipfile

ROOT=Path(__file__).resolve().parents[1]
BASE=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
original=(BASE/'runtime/LiTo.unpacked-oep-current.exe').read_bytes()
assert digest(original)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
dll_path=Path(r'C:\Windows\SysWOW64\msvcrt.dll');dll=dll_path.read_bytes()
def parser(raw):
    e=struct.unpack_from('<I',raw,60)[0];o=e+24
    assert raw[:2]==b'MZ' and struct.unpack_from('<H',raw,o)[0]==0x10b
    base=struct.unpack_from('<I',raw,o+28)[0]
    sections=[struct.unpack_from('<IIII',raw,o+struct.unpack_from('<H',raw,e+20)[0]+40*i+8) for i in range(struct.unpack_from('<H',raw,e+6)[0])]
    def at(rva,count):
        for _,va,size,off in sections:
            if va<=rva and rva+count<=va+size:return raw[off+rva-va:off+rva-va+count]
        raise AssertionError(hex(rva))
    return base,o,at
_,optional,da=parser(dll)
export_rva=struct.unpack_from('<I',dll,optional+96)[0]
exp=da(export_rva,40);ordinal_base,n_functions,n_names,funcs,names,ordinals=struct.unpack_from('<6I',exp,16)
exports={}
for i in range(n_names):
    rva=struct.unpack('<I',da(names+i*4,4))[0]
    name=da(rva,128).split(b'\0',1)[0].decode('ascii')
    ordinal=struct.unpack('<H',da(ordinals+i*2,2))[0]
    rva=struct.unpack('<I',da(funcs+ordinal*4,4))[0]
    exports.setdefault(rva,[]).append(name)
ftol_rvas=[rva for rva,v in exports.items() if '_ftol' in v];assert len(ftol_rvas)==1
inferred_base=0x7741ac20-ftol_rvas[0];assert inferred_base==0x77380000 and inferred_base%65536==0
original_base,_,oa=parser(original)
metadata_path=BASE/'ida-client-oep-v2/functions.json';metadata_bytes=metadata_path.read_bytes();metadata=json.loads(metadata_bytes)
matches=[];within_module_unmatched=[]
named={f['address']:f['name'] for f in metadata}
# IDA leaves many CRT trampolines as data labels, so scan their actual bytes.
for address in range(0x4f1000,0x4f1800):
    b=oa(address-original_base,6)
    if b[:2]!=b'\x90\xe9':continue
    target=(address+6+struct.unpack_from('<i',b,2)[0])&0xffffffff
    rva=target-inferred_base
    if not 0<=rva<struct.unpack_from('<I',dll,optional+56)[0]:continue
    row={'original_address':address,'original_name':named.get(address),'original_bytes':b.hex(),'original_target':target,'candidate_RVA':rva,'candidate_exports':exports.get(rva,[])}
    (matches if rva in exports else within_module_unmatched).append(row)
assert any(v['original_address']==0x4f1592 and '_ftol' in v['candidate_exports'] for v in matches)
assert len(matches)==57 and within_module_unmatched==[]
assert dll==dll_path.read_bytes()
out=ROOT/'驗證紀錄/current/native-background-CRT-layout-static'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
save(out/'matching-trampolines.json',matches);save(out/'within-candidate-module-unmatched.json',within_module_unmatched)
(out/'observer.py').write_bytes(Path(__file__).read_bytes())
proof={'status':'multiple_original_CRT_trampolines_match_pinned_host_msvcrt_export_layout_static_inference',
    'original_PE_sha256':digest(original),'original_function_metadata_sha256':digest(metadata_bytes),
    'host_x86_DLL':str(dll_path),'host_DLL_sha256':digest(dll),'candidate_historical_module_base':inferred_base,
    'original_byte_scan_range':[0x4f1000,0x4f1800],
    'matching_trampolines':len(matches),'within_candidate_module_unmatched':len(within_module_unmatched),
    'conversion_symbol_inference':'sub_4F1592 target agrees with _ftol under the shared inferred module base.',
    'historical_DLL_binary_identity_verified':False,'DLL_loaded_or_executed':False,
    'full_initializer_executed':False,'background_pixels_verified':False,'whole_goal_complete':False}
save(out/'audit.json',proof)
files={p.name:digest(p.read_bytes()) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw-static.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw-static.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h,n
result={**proof,'path':str(out),'files':files,'raw_archive_sha256':digest((out/'raw-static.zip').read_bytes()),'raw_archive_readback_verified':True}
save(out/'retained-static.json',result);save(APP/'background-CRT-layout-static-current.json',result)
print(json.dumps({k:result[k] for k in ['status','path','matching_trampolines','within_candidate_module_unmatched','historical_DLL_binary_identity_verified']},ensure_ascii=False))
