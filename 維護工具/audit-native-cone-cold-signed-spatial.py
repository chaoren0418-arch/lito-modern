"""Bind S23 byte offsets and signed native coordinate transform; no rendering."""
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
ranges=dict(header=(0x4a5ed0,0x4a5f4d),tail=(0x4a5f4d,0x4a5fdf),type5=(0x4a61cd,0x4a6234),decoder=(0x4a2a50,0x4a2b71),constructor=(0x42f1f0,0x42f295),vector=(0x4b92d0,0x4b9356),rounding=(0x42f31f,0x42f39f))
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
ins={i.address:i for rows in decoded.values() for i in rows}
assert raw(0x634390,7)==b'cddccd\0' and raw(0x634384,9)==b'hchhhhcs\0'
decoder_cases={}
for ch,expected in [('c',0x4a2a8a),('h',0x4a2a9a),('d',0x4a2ab9),('s',0x4a2ae7)]:
    index=raw(0x4a2b90+ord(ch)-0x53,1)[0]
    target=struct.unpack('<I',raw(0x4a2b78+index*4,4))[0]
    assert target==expected
    decoder_cases[ch]=dict(case=hex(target),scalar_bytes={'c':1,'h':2,'d':4}.get(ch),little_endian=ch in 'hd')
expected={0x4b9320:('shl','eax, 0x10'),0x4b9323:('cdq',''),0x4b9324:('idiv','esi'),0x4b932a:('shl','eax, 0x10'),0x4b932d:('cdq',''),0x4b932e:('idiv','ecx'),0x42f359:('shr','edx, 0x10'),0x42f364:('shr','eax, 0x10'),0x4a5f0b:('lea','eax, [esi + 1]'),0x4a5f0e:('push','0x634390'),0x4a5f1b:('call','0x4a2a50'),0x4a5f75:('add','esi, 0x10'),0x4a5f78:('push','0x634384'),0x4a5f7e:('call','0x4a2a50'),0x4a5f83:('mov','eax, dword ptr [ebp - 0x4c]'),0x4a5f86:('mov','edx, dword ptr [ebp - 0x48]'),0x4a5f89:('and','eax, 0xffff'),0x4a5f8e:('mov','edi, dword ptr [ebp - 0x40]'),0x4a5f91:('mov','esi, dword ptr [ebp - 0x44]'),0x4a5f94:('and','edx, 0xffff'),0x4a5f9a:('lea','ecx, [eax + eax - 0x8000]'),0x4a5fa4:('lea','ebx, [edx + edx - 0x8000]'),0x4a5fb1:('mov','dword ptr [ebp - 0x2c], ecx'),0x4a5fc1:('and','edi, 0xffff'),0x4a5fc9:('and','esi, 0xffff'),0x4a6212:('mov','ecx, dword ptr [ebp - 0x2c]'),0x4a6215:('push','esi'),0x4a6216:('push','ebx'),0x4a6217:('push','edi'),0x4a6218:('push','ecx'),0x4a622a:('call','0x42f1f0'),0x42f21f:('mov','eax, dword ptr [ebp + 0x14]'),0x42f219:('mov','ecx, dword ptr [ebp + 0x18]'),0x42f228:('mov','dword ptr [esi + 0x2c], eax'),0x42f22b:('mov','dword ptr [esi + 0x30], ecx'),0x42f240:('mov','edx, dword ptr [ebp + 0x1c]'),0x42f24f:('mov','eax, dword ptr [ebp + 0x20]'),0x42f258:('mov','dword ptr [esi + 0x150], eax'),0x42f263:('mov','dword ptr [esi + 0x14c], edx')}
for address,op in expected.items():assert (ins[address].mnemonic,ins[address].op_str)==op,(hex(address),op)
header=[('action',1,'c','3c'),('attacker',2,'d','34'),('target',6,'d','14'),('damage_marker',10,'c','1c'),('heading',11,'c','28'),('projectile_id',12,'d','18')]
tail=[('graphics',16,'h','20'),('flight_type',18,'c','24'),('source_x',19,'h','4c'),('source_y',21,'h','40'),('destination_x',23,'h','48'),('destination_y',25,'h','44'),('tail_byte',27,'c','38'),('tail_string',28,'s','cc')]
assert ''.join(v[2] for v in header)=='cddccd' and ''.join(v[2] for v in tail)=='hchhhhcs'
modern_path=ROOT/'LiTo381-Modern/src/modern_lito/native_combat.rs';modern=modern_path.read_text('utf-8-sig')
for fragment in ['action: b[1]','b[2..6]','b[6..10]','damage_marker: b[10]','heading: b[11]','b[12..16]','let graphics = h(16)','flight_type: b[18]','from: [h(19), h(21)]','to: [h(23), h(25)]','b.len() == 29 && b[27..29] == [0, 0]']:assert fragment in modern,fragment
for n,h in inputs.items():assert sha(ROOT/n)==h
out=ROOT/'驗證紀錄/current/native-cone-cold-signed-spatial'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text(chr(10).join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+chr(10),encoding='utf-8')
record=dict(status='primary_S23_signed_conversion_vector_division_and_unsigned_rounding_bound_only',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),header_format_address='0x634390',header_format='cddccd',tail_format_address='0x634384',tail_format='hchhhhcs',decoder_cases=decoder_cases,header_fields=header,tail_fields=tail,normal_decoder=dict(path=str(modern_path),sha256=sha(modern_path)),native_x_formula='2 * zero_extend_u16(wire_x) - 32768 (signed32)',native_y_formula='zero_extend_u16(wire_y)',native_x_full_wire_domain=[-32768,98302],safe_unsigned16_candidate_wire_x_domain=[16384,49151],candidate_domain_note='Existing isolated spatial candidate accepts native u16 only; it does not qualify the full signed coordinate domain. Outside the recorded safe domain no clamp/wrap conversion is authorized.',signed_vector_shift_before_idiv=True,large_minor_caveat='4B92D0 shifts a signed32 numerator before cdq/idiv; minor>=32768 must not use unsigned division. Retained u16-only candidates tested short vectors, not the complete unsigned16 domain.',findings=['The two primary format strings and decoder case table give exact S23 field offsets independently of the modern parser.','Native source and destination X are transformed after u16 masking; Y is zero-extended without scaling.','Factory type5 pushes these source/destination values directly into42F1F0 coordinate fields.','Modern decode_attack preserves the wire world cells; conversion belongs at the native spatial boundary, not by altering protocol fields.'],world_coordinate_conversion_bound=True,full_signed_spatial_candidate_qualified=False,timer_units_qualified=False,asset_direction_image_mapping_qualified=False,normal_product_source_modified=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(binding=str(out/'binding.json'),wire_conversion_bound=True,full_signed_spatial_candidate_qualified=False,new_casts=0),ensure_ascii=False))
