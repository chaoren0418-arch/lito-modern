from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
old=read(L/'r504-source-oracle.json');setup=old['setup'];pe=Path(read(L/'r505-source-study-final-gates.json')['originals'][1]['path']).read_bytes()
def vector(body,counter):
 b=bytes(body);encrypted,steps=source_cipher(b,counter,pe);return dict(plain=list(b),wire=list((len(b)+2).to_bytes(2,'little')+encrypted),before=counter,after=counter_add_adc(counter,len(b)),source_instruction_steps=steps)
def table(values):return [243,len(values)]+[b for v in values for b in v.to_bytes(4,'little')]
def request(slot):return [244,slot,127,0,0,1,0xD0,7]
values=[0xFFFFFFFF]+list(range(1,9))+[0xFFFFFFFF]
bodies=[table(values),table([7,8]),[243,0xFF],request(9),request(0)]
counter=setup['first_roster']['after'];rx=[]
for body in bodies:
 v=vector(body,counter);rx.append(v);counter=v['after']
tail=vector(old['new_success']['plain'],counter)
save(L/'r505-source-oracle.json',dict(setup=setup,setup_only_previous_oracle=pin(L/'r504-source-oracle.json'),source_study=pin(L/'r505-source-study-final-gates.json'),cipher_tool=pin(P/'tools/r495_source_oracle.py'),receive=rx,discarded_tail=tail,select=old['old_tx'],malformed=vector(request(0)+[0],setup['first_roster']['after']),table_values=values,minimum_channel_path_achieved=False,GUI=0))
