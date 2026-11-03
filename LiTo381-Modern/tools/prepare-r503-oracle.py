from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
old=read(L/'r502-source-oracle.json');setup=old['profiles'][0];pe=Path(read(L/'r503-source-study-initial.json')['originals'][1]['path']).read_bytes()
def vector(body,counter):
 encrypted,steps=source_cipher(bytes(body),counter,pe)
 return dict(plain=body,wire=list((len(body)+2).to_bytes(2,'little')+encrypted),before=counter,after=counter_add_adc(counter,len(body)),source_instruction_steps=steps)
def sequence(bodies,start):
 out=[]
 for b in bodies:
  v=vector(b,start);out.append(v);start=v['after']
 return out
def success(channel):return [201,0,channel]+list((0xDEADBEEF).to_bytes(4,'little'))+list((0x8123).to_bytes(2,'little'))+list((0x12345678).to_bytes(4,'little'))
tx=setup['send_vectors'][1]['after'];rx=setup['first_roster']['after']
normal_tx=sequence([[201]+list(b'Role0\0'),[201]+list(b'Role1\0'),[201]+list(b'Role2\0')],tx)
normal_rx=sequence([[201,1,0],[200,8],[200,7],[201,10,0],success(2),success(2),[200,0]],rx)
auto_tx=sequence([[201]+list(b'Role0\0'),[202,100]],tx)
auto_rx=sequence([[200,0],[200,0],success(3)]+[success(3)]*10,rx)
malformed_rx=sequence([[201,0,1,2,3]],rx)
save(L/'r503-source-oracle.json',dict(state='R503_NEW_ORDINARY_RETURN_STATE_SUBSET',setup_only_previous_oracle=pin(L/'r502-source-oracle.json'),source_study=pin(L/'r503-source-study-initial.json'),source_cipher_tool=pin(P/'tools/r495_source_oracle.py'),setup=setup,seed=old['seed'],normal_tx=normal_tx,normal_rx=normal_rx,auto_tx=auto_tx,auto_rx=auto_rx,malformed_rx=malformed_rx,guard_success=vector(success(2),rx),parser_formats=dict(S_C8='c',S_C9='cc then dhd on status0',C_CA='cc',C_CB='c',C_0F='c'),reachable='C_C9 selectedname -> S_C9success Lobby; optional S_C8success B8zero -> C_CA100 -> S_C9success Lobby',unreachable='page40 requires AFBC/B8!=0 connection-return state, currently no source-closed producer; no setter invented',PE_execution=False,GUI=0))
print('R503 independent source-instruction wire oracle created')
