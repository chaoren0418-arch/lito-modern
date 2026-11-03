from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
o=read(L/'r501-source-oracle.json');pe=Path(read(L/'r502-source-study.json')['analysis_PE']['path']).read_bytes();profiles=[]
def wire(body,counter):
 cipher,_=source_cipher(bytes(body),counter,pe);return list((len(body)+2).to_bytes(2,'little')+cipher)
for old in o['profiles']:
 env=old['environment'];b=old['send_vectors'];counter=o['seed'];sends=[]
 for name,body in [('compatibility',b[0]['plain']),('first_login',b[1]['plain']),('first_character',[201]+list(b'Role0\0')),('character_back',[15]),('retained_relogin',b[3]['plain']),('second_character',[201]+list(b'Again0\0'))]:
  after=counter_add_adc(counter,len(body));sends.append(dict(name=name,plain=body,wire=wire(body,counter),before=counter,after=after));counter=after
 rx=old['receive_vectors'][0]['after'];profiles.append(dict(environment=env,send_vectors=sends,coalesced_handshake=o['seed_wire']+old['receive_vectors'][0]['wire'],receive_after_versions=rx,first_roster=old['receive_vectors'][1],second_roster=old['receive_vectors'][2],unsupported59=dict(plain=[59,0],wire=wire([59,0],rx)),partial_roster_header=old['receive_vectors'][1]['wire'][:1]))
save(L/'r502-source-oracle.json',dict(state='R502_NEW_PLAYER_CALLBACK_SEQUENCE_SOURCE_PE_CIPHER_ORACLE',previous_R501_oracle=pin(L/'r501-source-oracle.json'),source_study=pin(L/'r502-source-study.json'),source_cipher_tool=pin(P/'tools/r495_source_oracle.py'),profiles=profiles,seed=o['seed'],seed_wire=o['seed_wire'],synthetic_credentials_only=True,PE_execution=False,GUI=0))
print('R502 sourceoracle:2callbackflows,12distinctsendvectors/coalescedhandshake/unknown59/EOFpartialheader')
