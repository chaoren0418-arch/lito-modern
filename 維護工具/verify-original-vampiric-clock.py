"""Read recovered PE cast-clock entries without running the original client."""
from pathlib import Path
import hashlib,json,struct
import pefile
root=Path(__file__).resolve().parent.parent
p=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest().upper()
assert digest=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(data=raw,fast_load=True);rows=[]
for skill,expected in [(10,200),(28,100)]:
    va=0x635774+(skill-1)*2;data=pe.get_data(va-pe.OPTIONAL_HEADER.ImageBase,2)
    value=struct.unpack('<H',data)[0];assert value==expected
    rows.append({'skill':skill,'table_index':skill-1,'va':hex(va),'bytes':data.hex(),'client_clock_millis':value})
(root/'驗證紀錄/current/native-world-integration/original-vampiric-clock-qualified.json').write_text(json.dumps(
    {'status':'verified_recovered_client_clock_table_entries','original_pe_sha256':digest,'rows':rows,
     'table':'word_635774; entries are separate from server reuse and animation completion',
     'native_execution_verified':False},indent=2)+'\n',encoding='utf-8')
print('Verified client clocks: skill10=200ms, skill28=100ms')
