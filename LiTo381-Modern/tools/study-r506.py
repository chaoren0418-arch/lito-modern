from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
external=P.parent.parent/'lito脫殼資源'
api=external/'priority-api-resolution.json';contract=external/'modern-F4-contract.json';host=external/'reconnect-host-audit.json'
assert pin(api)['sha256']=='D6E351E4B165296238B175D9B853BF12FFB0CA72BC38615758B71D9F1E12A2F6'
assert pin(contract)['sha256']=='E8FED5D675EE30B92F5358F6E7C88C65BB7C2569950332B41B43B15897E595F8'
adopt=read(api);pe=Path(adopt['main_PE']['path']);data=pe.read_bytes();asm=Path(read(L/'r505-source-study.json')['originals'][0]['path']);lines=asm.read_text(encoding='utf-8-sig').splitlines();root=L/'r506-source-fragments';root.mkdir(exist_ok=True)
for a,b in [(266525,266597),(251098,251330),(251510,252055),(552014,552042),(309640,309726),(349838,349984),(236158,236569),(236646,236805),(226337,226617)]:
 (root/f'asm-{a}-{b}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}'for i in range(a-1,b))+'\n',encoding='utf-8')
for va,n in [(0x47c920,0x1a0),(0x4d69f0,0x70),(0x475070,0x180),(0x475490,0x400),(0x475a90,0x190),(0x492fa0,0x70),(0x630b20,0x130),(0x4efe56,6),(0x4760f0,0x90)]:
 raw=data[va-0x400000:va-0x400000+n];(root/f'pe-{va:08X}.txt').write_text(f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n',encoding='utf-8')
pins=[]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   q=pin(v['path']);assert q['bytes']==v['bytes'] and q['sha256']==v['sha256'];pins.append(q)
  else:
   for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(adopt)
for r in adopt['resolutions']:
 b=r['branch_instruction'];va=int(b['va'],16);raw=bytes.fromhex(b['bytes']);assert data[va-0x400000:va-0x400000+len(raw)]==raw;assert int(r['historical_module']['base'],16)+int(r['current_export']['rva'],16)==int(b['target_va'],16)
save(L/'r506-API-adoption.json',dict(API_resolution=pin(api),F4_contract=pin(contract),host_audit=pin(host),external_reference_pins=pins,identity_basis='fixedPE exactbranches/targets,historymodule base+currentDLLverifiedexportRVA match;supervisorindependentPEexports confirmed;historicalDLLhash absent',no_historical_DLL_bytes_claim=True,numericIPv4='incomingdLE containsnetworkoctets;127.0.0.1=>0x0100007F,127.0.0.2=>0x0200007F;ntoaownedASCIIstring immediatelycopied beforefurther socketcalls',docs=['https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-inet_addr','https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-inet_ntoa','https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-gethostbyname','https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/sprintf-sprintf-l-swprintf-swprintf-l-swprintf-l?view=msvc-170'],originals=[pin(asm),pin(pe)],fragments=[pin(p)for p in sorted(root.glob('*'))],GUI=0))
print('R506adoptedverifiedAPIpin set;static47C9D1candidate andclock/UIrow studied,no execution')
