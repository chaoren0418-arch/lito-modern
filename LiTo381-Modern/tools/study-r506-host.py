from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
external=P.parent.parent/'lito脫殼資源'
host=external/'host-alias-lifecycle.json';vectors=external/'api-vectors.json'
assert pin(host)['sha256']=='875055619F42C3AFB3C5257A1D2E2EDA4DF1037B51E8A0A0EE282268542AE6CC'
assert pin(vectors)['sha256']=='7E2E1B5B52BE57EF83149EC6FB8757C0CB632EAE381D9918949784810E3A586E'
v=read(host);pe=Path(v['source_PE']['path']);assert pin(pe)==v['source_PE'];data=pe.read_bytes()
root=L/'r506-host-source-fragments';root.mkdir(exist_ok=True)
checks=[]
for va,n in [(0x480B80,0x180),(0x481830,0x80),(0x481DA0,0x90),(0x47CB00,0x80),(0x47C8A0,0x180),(0x492F10,0x60),(0x4A7690,0x40)]:
 raw=data[va-0x400000:va-0x400000+n];path=root/f'pe-{va:08X}.txt';path.write_text(f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n',encoding='utf-8');checks.append(pin(path))
for va,dest in [(0x480BC1,0x64B7E4),(0x480C77,0x64B7E4),(0x481861,0x64B7E4),(0x481DDC,0x64B7E4),(0x47C9D0,0x64AD74),(0x492F42,0x64AD54)]:
 raw=data[va-0x400000:va-0x400000+5];assert raw==b'\xbf'+dest.to_bytes(4,'little'),(hex(va),raw.hex())
assert 0x64AD70-0x64AD54==28 and 0x64AE74-0x64AD74==256
save(L/'r506-host-alias-adoption.json',dict(host_contract=pin(host),conversion_vectors=pin(vectors),source_PE=pin(pe),own_fragments=checks,verified='exact MOV EDI destinations at six stated writers; local default/argument/provider/COM copy bytes and saved alias replay inspected',host_authority='static source buffer and copy destination qualified; actual startup input and private/transitive execution remain unknown',selected_alias_capacity=28,host_capacity=256,no_fixture_endpoint_as_host=True,no_original_execution=True,conversion_vectors_adopted_as_historical=False,GUI=0))
print('R506 host six direct writers verified; actual startup input remains unproved')
