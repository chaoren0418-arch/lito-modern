"""Static recovered-PE evidence; does not run or attach the original client."""
from pathlib import Path
import hashlib,json
import pefile,capstone

root=Path(__file__).resolve().parent.parent
p=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest().upper()
assert digest=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(data=raw,fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase
decoder=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
def decode(start,end):
    data=pe.get_data(start-base,end-start)
    instructions=list(decoder.disasm(data,start))
    return {'start':hex(start),'end':hex(end),'bytes_sha256':hashlib.sha256(data).hexdigest().upper(),
            'instructions':[{'address':hex(i.address),'bytes':i.bytes.hex(),'mnemonic':i.mnemonic,'operands':i.op_str} for i in instructions]}
reader=decode(0x43dcc0,0x43dd38);caller=decode(0x43e84d,0x43e893)
assert sum(i['mnemonic']=='call' and i['operands']=='0x43dcc0' for i in caller['instructions'])==3
assert any(i['mnemonic']=='lea' and 'ecx + 1' in i['operands'] for i in reader['instructions'])
assert any(i['mnemonic']=='lea' and 'edi + edi*4' in i['operands'] for i in reader['instructions'])
assert not any(i['mnemonic']=='cmp' and i['operands'] in ['al, 0x2c','al, 0x2e'] for i in reader['instructions'])
# A narrow decimal model of the decoded scanner: inference, not native execution.
def scan(data,at):
    while at<len(data) and not data[at].isdigit(): at+=1
    start=at
    while at<len(data) and data[at].isdigit(): at+=1
    return int(data[start:at]),at
vectors=[]
for data in ['0.0:1[141','0,0:1[141']:
    at=0;values=[]
    for _ in range(3):value,at=scan(data,at);values.append(value)
    assert values==[0,0,1] and data[at]=='['
    vectors.append({'input':data,'values':values,'cursor':at})
out=root/'驗證紀錄/current/native-world-integration/original-sprite-comma-qualified.json'
out.write_text(json.dumps({'status':'verified_static_numeric_scanner_and_frame_caller',
    'original_pe':str(p),'original_pe_sha256':digest,'reader':reader,'frame_caller':caller,
    'inference':'Frame caller reads three integers without testing a dot; scanner advances over nondigits, including comma. The original #871 image/subframe separator is therefore equivalent to dot.',
    'modeled_vectors':vectors,'native_execution_verified':False,'gui_verified':False},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified recovered PE numeric scanner and three frame-field calls')
