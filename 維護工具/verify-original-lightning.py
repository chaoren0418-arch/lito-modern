"""Independent recovered PE verification for S39 lightning170 (no execution)."""
from pathlib import Path
import hashlib,json,struct
import pefile,capstone
root=Path(__file__).resolve().parent.parent
p=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest().upper();assert digest=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(data=raw,fast_load=True);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
def decode(a,b):
 data=pe.get_data(a-pe.OPTIONAL_HEADER.ImageBase,b-a)
 return {'start':hex(a),'end':hex(b),'bytes_sha256':hashlib.sha256(data).hexdigest().upper(),'instructions':[{'address':hex(i.address),'mnemonic':i.mnemonic,'operands':i.op_str,'bytes':i.bytes.hex()} for i in md.disasm(data,a)]}
ranges={name:decode(a,b) for name,a,b in [('s39_receiver',0x4a6510,0x4a6883),('class_table_zero_default',0x43e0c7,0x43e0e9),('attribute109_class_override',0x43e6b2,0x43e6e2),('many_constructor',0x44ef00,0x44ef75),('common_start_first_cases',0x44f350,0x44f391),('lightning170_default_start',0x44f62e,0x44f63e),('publish',0x45001a,0x45003b),('finite_advance',0x4e5b50,0x4e5bf2),('per_target_hit',0x44f1e3,0x44f2f7),('directional_frame_image',0x4e5d4d,0x4e5d5b)]}
def has(name,mn,ops):return any(i['mnemonic']==mn and i['operands']==ops for i in ranges[name]['instructions'])
assert has('class_table_zero_default','xor','eax, eax') and has('class_table_zero_default','rep stosb','byte ptr es:[edi], al')
assert has('attribute109_class_override','cmp','edi, 0x6d') and has('attribute109_class_override','mov','byte ptr [esi + edx], al')
assert has('s39_receiver','call','0x44ef00') and has('s39_receiver','call','0x4e0ef0')
assert has('many_constructor','mov','dword ptr [esi + 0x68], edx') and has('many_constructor','mov','dword ptr [esi + 0x2c], eax')
assert has('lightning170_default_start','jne','0x45001a')
assert has('per_target_hit','cmp','byte ptr [ecx + ebx], 0') and has('per_target_hit','call','0x4e8160') and has('per_target_hit','call','0x4e45f0')
va=0x635774+16*2;clock=struct.unpack('<H',pe.get_data(va-pe.OPTIONAL_HEADER.ImageBase,2))[0];assert clock==100
out={'status':'verified_static_s39_common_finite_lightning170_origin_and_target_hit','original_pe_sha256':digest,'ranges':ranges,'client_clock_millis':clock,'clock_table_va':hex(va),
 'inference':'No109 leaves class0 because the class array is zero-initialized. S39 default constructs44EF00 with wire caster coordinates and target list. Lightning170 bypasses the special gfx branches and publishes at45001A without the class1 two-step offset. Each positive target marker resolves that target at effect Hit; callbacks are presentation only.',
 'directionality':'The leading1 in0.lightning(1...) selects eight-way frame images; it is not an infinite loop flag. The common finite advance publishes source0 and retires after its dwell.',
 'native_execution_verified':False,'gui_verified':False}
(root/'驗證紀錄/current/native-world-integration/original-lightning-qualified.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified S39/class0 zero default/lightning finite origin/multi-target Hit/client100ms')
