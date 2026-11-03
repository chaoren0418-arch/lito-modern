"""Read original skill18 picker/clock without executing or modifying the client."""
from pathlib import Path
import hashlib,json,struct
import pefile,capstone
root=Path(__file__).resolve().parent.parent
p=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe');raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest().upper();assert digest=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(data=raw,fast_load=True);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
def decode(a,b):
 data=pe.get_data(a-pe.OPTIONAL_HEADER.ImageBase,b-a)
 return {'start':hex(a),'end':hex(b),'bytes_sha256':hashlib.sha256(data).hexdigest().upper(),'instructions':[{'address':hex(i.address),'mnemonic':i.mnemonic,'operands':i.op_str,'bytes':i.bytes.hex()} for i in md.disasm(data,a)]}
r=decode(0x4cb755,0x4cb78c);assert any(i['mnemonic']=='call' and i['operands']=='0x4ca6e0' for i in r['instructions']);assert any(i['mnemonic']=='mov' and i['operands']=='edx, 0xa' for i in r['instructions'])
va=0x635774+17*2;data=pe.get_data(va-pe.OPTIONAL_HEADER.ImageBase,2);clock=struct.unpack('<H',data)[0];assert clock==500
out={'status':'verified_static_original_skill18_shared_picker_range10_and_clock500ms','original_pe_sha256':digest,'picker':r,'clock_table_va':hex(va),'clock_bytes':data.hex(),'client_clock_millis':clock,'native_picker_range':10,'server_action':19,'client_preselection_action_query':18,'inference':'Shared target producer4CA6E0 uses range10; skill18 clock entry is500ms. This does not equate the client preselection action query18 with authoritative server action19.','native_execution_verified':False,'gui_verified':False}
(root/'驗證紀錄/current/native-world-integration/original-turn-undead-qualified.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified original skill18 picker and500ms clock')
