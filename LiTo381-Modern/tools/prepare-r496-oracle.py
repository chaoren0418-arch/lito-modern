from r496_source_oracle import *
e=read(L/'r496-source-evidence.json');pe=Path(e['analysis_PE']['path']).read_bytes();old=read(L/'r494-event-input.json');seed=old['initial_counter'];cases=[]
for value in [-1,0x52df,0x52e0,0x75a3,0x75a4,0x7725,0x7726,0x9d7b,0x9d7c,0x9e37,0x9e38,0x9f09,0x9f0a]:
 fmt,path=schema205(value,pe);cases.append(dict(version=value,version_body=version_body(value),actor_body=actor_body(fmt,value),source_format=fmt,source_branch_path=path))
second=old['roster'].copy();assert bytes(second[1:9])==b'NewAlias';second[1:9]=b'AltAlias'
setup=[old['plain_seed_body'],version_body(0x9f0a),[3,2],old['roster'],second];counter=seed;wire=list(old['plain_seed_frame'])
for body in setup[1:]:
 cipher,_=source_cipher(bytes(body),counter,pe);wire.extend((len(body)+2).to_bytes(2,'little')+cipher);counter=counter_add_adc(counter,len(body))
vectors={}
for name,body in [('refused58',[58,1]),('ignored209',[209]+list(b'NewAlias\0')+[1]),('new_roster',[3,0]),('mode206',old['mode206']),('actor205',cases[-1]['actor_body'])]:
 cipher,_=source_cipher(bytes(body),counter,pe);vectors[name]=dict(plain=body,wire=list((len(body)+2).to_bytes(2,'little')+cipher),receive_before=counter,receive_after=counter_add_adc(counter,len(body)))
send=bytes([201])+b'NewAlias\0';cipher,steps=source_cipher(send,seed,pe)
second_send=bytes([201])+b'AltAlias\0';second_cipher,_=source_cipher(second_send,counter_add_adc(seed,len(send)),pe)
ctr=vectors['mode206']['receive_after'];body=vectors['ignored209']['plain'];c,_=source_cipher(bytes(body),ctr,pe);vectors['209_after_mode']=dict(plain=body,wire=list((len(body)+2).to_bytes(2,'little')+c),receive_before=ctr,receive_after=counter_add_adc(ctr,len(body)))
save(L/'r496-source-oracle.json',dict(state='R496_SOURCE_PE_REVISION_BRANCH_SCHEMA_AND_SYNTHETIC_TYPED_CONTROL_ORACLE',source_evidence=pin(L/'r496-source-evidence.json'),oracle_tool=pin(P/'tools/r496_source_oracle.py'),cipher_oracle_tool=pin(P/'tools/r495_source_oracle.py'),revision_cases=cases,setup_bodies=setup,setup_wire=wire,receive_after_setup=counter,initial_counter=seed,vectors=vectors,control_send_frame=list((len(send)+2).to_bytes(2,'little')+cipher),control_second_send_frame=list((len(second_send)+2).to_bytes(2,'little')+second_cipher),control_send_counter_after=counter_add_adc(seed,len(send)),source_instruction_steps=steps,actual_PE_execution=False,credentials_or_raw_capture=False))
print('R496 oracle frozen:13 sourcePE branch boundaries/2 records/typedbutton cs201/refusal/209phase0 vectors')
