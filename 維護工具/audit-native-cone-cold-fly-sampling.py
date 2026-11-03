"""Bind type5 fly sampling and inherited105 image/subframe submission."""
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
ranges=dict(ctor=(0x42f1f0,0x42f2f5),advance_tail=(0x42f4de,0x42f543),frame_increment=(0x4e5b50,0x4e5bf3),frame_select=(0x4e5c00,0x4e63b0),layer105_parser=(0x43e4be,0x43e52e),layer_load=(0x4e6ea0,0x4e6f6c),layer_draw=(0x4e7000,0x4e70a5),main_then_layer=(0x4e0200,0x4e025d))
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()};ins={i.address:i for rows in decoded.values() for i in rows}
expected={0x42f26d:('mov','byte ptr [esi + 0x10], bl'),0x42f270:('mov','byte ptr [esi + 0x13], bl'),0x42f4f0:('mov','ecx, 0x3c'),0x42f4f5:('call','0x4d65f0'),0x42f500:('call','0x4e5b50'),0x42f505:('mov','eax, dword ptr [esi + 0x154]'),0x42f50d:('jle','0x42f520'),0x42f511:('call','0x4e0200'),0x4e5b54:('mov','bl, byte ptr [ecx + 0x13]'),0x4e5b58:('inc','bl'),0x4e5b7d:('cmp','dl, byte ptr [edi + eax*4 + 1]'),0x4e5b81:('jl','0x4e5bdf'),0x4e5b8b:('xor','dl, dl'),0x4e5bc9:('mov','byte ptr [ecx + 0x13], dl'),0x4e5bcc:('call','0x4e5c00'),0x4e5be1:('call','0x4e5c00'),0x4e5d55:('movsx','eax, byte ptr [esi + 0x11]'),0x4e5d59:('add','edi, eax'),0x4e5f0d:('push','edi'),0x4e5f10:('call','0x4e6ea0'),0x4e604d:('mov','dl, byte ptr [ecx + edx*4 + 0x11]'),0x4e6051:('xor','dl, 0x2b'),0x4e6056:('mov','byte ptr [esi + 0x19], dl'),0x4e6ef7:('mov','edx, dword ptr [ebp + 8]'),0x4e6f02:('call','0x4406c0'),0x4e6f22:('mov','edx, dword ptr [ecx + edx*4]'),0x4e6f27:('mov','ecx, dword ptr [0x647e68]'),0x4e6f2d:('mov','dx, word ptr [ecx + esi*2]'),0x4e6f34:('mov','word ptr [ecx], dx'),0x4e021d:('call','0x4cd9d0'),0x4e022e:('call','0x4e7000'),0x4e7072:('mov','dx, word ptr [edi]'),0x4e7077:('movsx','edx, byte ptr [esi + 0x19]'),0x4e707f:('call','0x4cd9d0')}
for address,op in expected.items():assert (ins[address].mnemonic,ins[address].op_str)==op,(hex(address),op)
ops={(i.mnemonic,i.op_str) for i in decoded['layer105_parser']}
for op in [('cmp','edi, 0x69'),('mov','edx, dword ptr [0x647e48]'),('mov','dword ptr [edx + esi*8], edi'),('mov','word ptr [edx], ax')]:assert op in ops,op
binding=ROOT/'驗證紀錄/current/native-cone-cold-binding/20261018-034221-586202/binding.json';assert read(binding)['effect_type']==5
asset=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json';aq=read(asset.parent/'qualified/qualified.json');assert sha(asset)==aq['raw_files'][asset.name]
tw=next(v for v in read(asset)['explicit_TW_namespace_inspections'] if v['namespace']=='launcher/TW13081901.pak');rows={v['graphics_id']:v for v in tw['rows'] if v['graphics_id'] in [11738,11739,11740,11741]}
assert rows[11738]['directionality']=='EightWay' and rows[11738]['image_count']==8
assert [f['subframe'] for f in rows[11738]['frames']]==[2,0,1] and not any(f['events'] for f in rows[11738]['frames'])
assert dict((v['key'],v['body']) for v in rows[11738]['attributes'])[105]=='1 11739'
assert dict((v['key'],v['body']) for v in rows[11739]['attributes'])[104]=='128'
assert rows[11739]['image_count']==8 and 'MissingAction' in rows[11739]['resolution_error']
out=ROOT/'驗證紀錄/current/native-cone-cold-fly-sampling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text(chr(10).join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+chr(10),encoding='utf-8')
record=dict(status='primary_type5_fly_source_cycle_direction_image_and_inherited105_submission_bound_only',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),factory_binding=str(binding),factory_binding_sha256=sha(binding),retained_asset_inventory=str(asset),retained_asset_inventory_sha256=sha(asset),new_resource_pack_read=False,fly_source_cycle=[1,2,0],fly_subframe_cycle=[0,1,2],fly_images=list(range(8)),fly_overlay_graphics=11739,fly_overlay_flags=128,findings=['Constructor initializes action10 and source13 to0. Every advance42F300 calls4E5B50 once; that increments the source, wraps by source count, and calls4E5C00.','Type5 schedules60 independently of the selected catalogue dwell; it ignores4E5B50 returned dwell for its scheduling.','4E5C00 adds heading11 to the selected image for directional actions, then passes that exact image to105 loader4E6EA0. Canonical11738 image0 therefore selects heading0..7 for body11738 and layer11739.','Loader reads105 child IDs from647E48 and records child104 flags. Layer draw uses the parent subframe19 and current parent2C/30 with the child flags. No child action0 is required.','4E0200 submits main before inherited105. Qualified existing depth queue reverses equal-depth insertions; no new framebuffer comparison is claimed.','Advance only submits fly draw while remaining154>0. The arrival/exhaustion step advances the frame selector but retires without submitting the fly image. Impact construction remains a distinct node.'],direction_image_mapping_bound=True,parent_layer_sampling_bound=True,fly_frame_phase_bound=True,timer_units_qualified=False,asset_pixels_loaded=False,renderer_qualified=False,normal_product_source_modified=False,new_sessions=0,new_casts=0,new_renders=0,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),source_cycle=[1,2,0],inherited_layer_sampling_bound=True,new_casts=0,renderer_qualified=False),ensure_ascii=False))
