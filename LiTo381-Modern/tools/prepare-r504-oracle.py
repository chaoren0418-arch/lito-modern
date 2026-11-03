from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
old=read(L/'r503-source-oracle.json');setup=old['setup'];pe=Path(read(L/'r504-source-study.json')['originals'][1]['path']).read_bytes()
def vector(body,counter):
 encrypted,steps=source_cipher(bytes(body),counter,pe);return dict(plain=body,wire=list((len(body)+2).to_bytes(2,'little')+encrypted),before=counter,after=counter_add_adc(counter,len(body)),source_instruction_steps=steps)
def sequence(bodies,counter):
 out=[]
 for b in bodies:v=vector(b,counter);out.append(v);counter=v['after']
 return out
def success():return [201,0,2]+list((0xDEADBEEF).to_bytes(4,'little'))+list((0x8123).to_bytes(2,'little'))+list((0x12345678).to_bytes(4,'little'))
old_rx=sequence([[200,2],[201,2,0],[201,1,0]],setup['first_roster']['after'])
fresh=[]
for i,b in enumerate(setup['first_roster']['bodies']):
 if i==0:fresh.append(b)
 else:
  n=b.index(0,1);fresh.append([203]+list(f'AgainA{i-1}'.encode())+b[n:])
fresh_vectors=sequence(fresh,setup['receive_after_versions']);wire=[b for v in fresh_vectors for b in v['wire']]
new_success=vector(success(),fresh_vectors[-1]['after']);late=vector(success(),old_rx[-1]['after']);assert late['wire']==new_success['wire']
save(L/'r504-source-oracle.json',dict(state='R504_KNOWN_ALERT_UNKNOWN_IGNORE_IDENTICAL_DELAYED_OLD_C9',setup_only_previous_oracle=pin(L/'r503-source-oracle.json'),source_study=pin(L/'r504-source-study.json'),source_cipher_tool=pin(P/'tools/r495_source_oracle.py'),setup=setup,old_rx=old_rx,old_tx=vector([201]+list(b'Role0\0'),setup['send_vectors'][1]['after']),old_late=late,fresh_roster=dict(bodies=fresh,wire=wire,before=setup['receive_after_versions'],after=fresh_vectors[-1]['after'],vectors=fresh_vectors),new_tx=vector([201]+list(b'AgainA0\0'),setup['send_vectors'][1]['after']),new_success=new_success,late_equals_new_success_wire=True,known_alerts=dict(C8=[7,8],C9=[1,10]),Native_disconnect_on_alert=False,Modern_requires_new_epoch_on_alert=True,GUI=0))
print('R504 old/new successwire identical atsameRXcounter; freshfour names each2byteslonger ->counter+8 matchesoldunknown2+3+knownalert3')
