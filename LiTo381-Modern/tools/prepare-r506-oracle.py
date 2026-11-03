from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
old=read(L/'r505-source-oracle.json');setup=old['setup'];pe=Path(read(L/'r506-API-adoption.json')['originals'][1]['path']).read_bytes()
def vector(body,counter):
 b=bytes(body);encrypted,steps=source_cipher(b,counter,pe);return dict(plain=list(b),wire=list((len(b)+2).to_bytes(2,'little')+encrypted),before=counter,after=counter_add_adc(counter,len(b)),source_instruction_steps=steps)
def sequence(bodies,c):
 out=[]
 for b in bodies:v=vector(b,c);out.append(v);c=v['after']
 return out
def row(name,address,port,first,last):return list(name.encode()+b'\0')+list(address.to_bytes(4,'little'))+list(port.to_bytes(2,'little'))+list(first.to_bytes(4,'little'))+list(last.to_bytes(4,'little'))
rows=[251]+list((1).to_bytes(4,'little'))+list((4).to_bytes(4,'little'))+row('Header',0x0100007F,2000,77,88)
for args in [('Arena1',0x0100007F,2000,2,5),('Full',0x0100007F,2000,5,5),('Disabled',0x0100007F,2000,0xffffffff,0),('Other',0x0200007F,2000,0,0)]:rows+=row(*args)
assert len(rows)<=155
c9=lambda c:[201,0,c]+list((0x33445566).to_bytes(4,'little'))+list((0x7788).to_bytes(2,'little'))+list((0x12345678).to_bytes(4,'little'))
same_rx=sequence([[244,0,127,0,0,1,0xd0,7],[200,0],c9(2),rows,rows,[200,0],c9(1)],setup['first_roster']['after'])
same_tx=sequence([[217],[203],[202,0],[202,2],[217],[203],[202,0],[202,1]],old['select']['after'])
redirect_tx=sequence([[217],[15]],old['select']['after'])
redirect_rx=sequence([[3,1],[203],[203],[200,0],c9(3)],setup['receive_after_versions'])
fresh_tx=sequence([old['select']['plain'],[202,3]],setup['send_vectors'][0]['after'])
save(L/'r506-source-oracle.json',dict(setup=setup,select=old['select'],same_receive=same_rx,same_send=same_tx,redirect_send=redirect_tx,redirect_receive=redirect_rx,redirect_fresh_send=fresh_tx,dynamic_redirect_input_basis='F4 cdh exact network octets; ephemeral port inserted only in test input, never supplied as64AD74',cipher_tool=pin(P/'tools/r495_source_oracle.py'),setup_only_previous_oracle=pin(L/'r505-source-oracle.json'),GUI=0))
root=L/'r506-source-fragments';data=pe;asm=Path(read(L/'r506-API-adoption.json')['originals'][0]['path']);lines=asm.read_text(encoding='utf-8-sig').splitlines()
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
for a,b in [(360100,360285),(252791,252982),(232869,232890),(317005,317129),(349728,349817)]:
 (root/f'asm-{a}-{b}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}'for i in range(a-1,b))+'\n',encoding='utf-8')
for va,n in [(0x475F40,0x350),(0x4AC7D0,0x280),(0x46EB30,0x180),(0x46ED50,0x130)]:
 raw=data[va-0x400000:va-0x400000+n];(root/f'pe-{va:08X}.txt').write_text(f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n',encoding='utf-8')
save(L/'r506-transfer-source-study.json',dict(API_adoption=pin(L/'r506-API-adoption.json'),host_alias_adoption=pin(L/'r506-host-alias-adoption.json'),fragments=[pin(p)for p in sorted(root.glob('*'))],first_row_layout='dhdd maps104,108,10C,110;114=0',second_row_layout='dhdd maps104,108,110,114;10C=0',first_row_callback_not_adopted=True,second_row_available_subset='110!=FFFFFFFF and (114==0 or110<114); same-address branch only until source startup-host authority',timer_adaptation='Modern monotonic strict>3s completion, strict>10s unresolved-host timeout conservative disconnect; noNativeQPC/u32wrap claim',selected_alias_capacity=28,GUI=0))
print('R506 symbolic source TX/RX vectors and row/branch fragments prepared')
