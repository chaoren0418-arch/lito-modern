"""Preserve exact type7 constructor byte folding without assuming draw semantics."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';inputs=read(normal/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h
parent=V/'native-eruption-factory-binding/20261018-064959-138715/binding.json';binding=read(parent);assert binding['type7_constructor']=='0x4238F0'
primary=Path(binding['primary_PE']['path']);pe=primary.read_bytes();assert hashlib.sha256(pe).hexdigest().upper()==binding['primary_PE']['sha256']
e=struct.unpack_from('<I',pe,60)[0];count=struct.unpack_from('<H',pe,e+6)[0];sections=e+24+struct.unpack_from('<H',pe,e+20)[0]
def raw(a,n):
    for i in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+i*40+8)
        if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return pe[at+a-0x400000-start:at+a-0x400000-start+n]
    raise ValueError(hex(a))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ranges={'constructor7':(0x4238f0,0x4239fc),'factory7_argument':(0x4a60ff,0x4a6166),'factory_fallback_virtual':(0x4a638f,0x4a63b2)}
decoded={name:list(md.disasm(raw(lo,hi-lo),lo))for name,(lo,hi)in ranges.items()};ops=[(i.mnemonic,i.op_str)for i in decoded['constructor7']]
expected=[('mov','eax, dword ptr [ebp + 0x24]'),('cmp','eax, 3'),('jle','0x423944'),('add','eax, -4'),('mov','byte ptr [esi + 0x11], al')]
indexes=[ops.index(x)for x in expected];assert indexes==sorted(indexes)
assert ('mov','dword ptr [esi], 0x4f7f30')in ops and ('ret','0x2c')in ops
assert ('call','dword ptr [edx + 4]')in[(i.mnemonic,i.op_str)for i in decoded['factory_fallback_virtual']]
table=struct.unpack('<8I',raw(0x4f7f30,32))
out=V/'native-eruption-constructor-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi)in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in decoded[name])+'\n',encoding='utf-8')
(out/'vtable.bin').write_bytes(raw(0x4f7f30,32))
record=dict(status='primary_type7_constructor_parameter_fold_bound_only',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],primary_PE=binding['primary_PE'],parent_factory_binding=dict(path=str(parent),sha256=sha(parent)),
    constructor='0x4238F0',constructor_argument_offset=0x24,stored_byte_offset=0x11,
    exact_rule='signed compare argument with3; if greater subtract4 once; store low byte. No repeated mask or modulo is inferred outside the0..7 input domain.',
    parameter_0_through_7_stored_values=[0,1,2,3,0,1,2,3],vtable_address='0x4F7F30',vtable_first_eight_pointers=[f'0x{p:08X}'for p in table],factory_fallback_virtual_call_offset=4,
    unresolved_external_call='0x77406880',wire_heading_semantic_binding_qualified=False,image_direction_consumer_qualified=False,current_TW_namespace_qualified=False,
    pending=['Bind the factory local/constructor argument to the qualified wire heading decoder.','Trace the byte11 draw consumer and later entry/timer writes before treating the constructor fold as complete image-direction behavior.','Bind current TW129/123/130/124 and type6 child timeline, target position/retirement, shadows and Effect3080 before implementation.'],
    normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_sessions=0,new_casts=0,new_renders=0,request_registered=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),parameter_fold=record['parameter_0_through_7_stored_values'],vtable=record['vtable_first_eight_pointers'],image_direction_consumer_qualified=False),ensure_ascii=False))
