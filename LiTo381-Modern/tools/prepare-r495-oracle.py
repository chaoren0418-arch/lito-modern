from r495_source_oracle import *
import re
O=P/'target/r495-native-send-study';e=read(L/'r495-source-evidence.json');pe=Path(e['analysis_PE']['path']).read_bytes();asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');ls=asm.splitlines();refs=[];proc='';names={'sub_4928C0','sub_4A33E0'}
for i,l in enumerate(ls):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if any(x in l for x in ['64BBF8','636584','5033C4'])and not l.strip().startswith(';'):
  if l.startswith(('mov     byte_64BBF8,','mov     dword_636584,','mov     dword_5033C4,','push    offset dword_636584')):refs.append(dict(procedure=proc,line=i+1,text=l,context=ls[max(0,i-4):i+7]));names.add(proc)
for n in sorted(names):
 m=re.search('^'+n+r' proc near',asm,re.M)
 if not m:continue
 z=re.search(r'^\w+ proc near',asm[m.end():],re.M);p=O/(n+'.asm');p.write_text(asm[m.start():m.end()+z.start()],encoding='utf-8')
 if not any(x.get('name')==n for x in e['fragments']):e['fragments'].append(dict(name=n,**pin(p)))
save(O/'compatibility-global-writers.json',refs);e['compatibility_writers']=pin(O/'compatibility-global-writers.json')
for sym in ['byte_64BBF8','dword_5033C4','dword_636584','aCs','aChdch']:
 i=next(i for i,l in enumerate(ls)if l.startswith(sym+' '));p=O/(sym+'.asm');p.write_text('\n'.join(ls[i:i+8])+'\n',encoding='utf-8')
 if not any(x['path']==str(p)for x in e.setdefault('contexts',[])):e['contexts'].append(pin(p))
save(L/'r495-source-evidence.json',e)
old=read(L/'r494-event-input.json');seed=old['initial_counter'];alias=b'NewAlias';body=bytes([201])+alias+b'\0';cases=[];counter=seed
for serial in [1,2]:
 cipher,steps=source_cipher(body,counter,pe);after=counter_add_adc(counter,len(body));wire=(len(body)+2).to_bytes(2,'little')+cipher
 cases.append(dict(serial=serial,counter_before=counter,counter_after=after,plaintext=list(body),frame=list(wire),source_instruction_steps=steps));counter=after
receive_plain=[[3,1],old['roster']];receive_wire=[];recv_counter=seed
for body in receive_plain:
 # Encryption is the SAMEsourceinstruction evaluator; Rustreceiver andsender
 # are checkedagainst these sourcecomputed bytevectors,notselfroundtrip.
 cipher,_=source_cipher(bytes(body),recv_counter,pe);receive_wire.extend((len(body)+2).to_bytes(2,'little')+cipher);recv_counter=counter_add_adc(recv_counter,len(body))
mode_cipher,_=source_cipher(bytes(old['mode206']),recv_counter,pe);modewire=list((len(old['mode206'])+2).to_bytes(2,'little')+mode_cipher)
record=dict(state='R495_SOURCE_INSTRUCTION_SYMBOLIC_ORACLE_SYNTHETIC_CHARACTER_CS201_NO_CAPTURE_OR_CREDENTIALS',source_PE=e['analysis_PE'],source_encrypt_instructions=pin(O/'pe-4C6700.disasm'),oracle_tool=pin(P/'tools/r495_source_oracle.py'),ordinary_serializer='4928C0 cs201 selectedrecordname;4C68D0 c lowbyte ands NULcopy;4C6AE0 LEbodylen+2',counter_add='source4C6770 lowadd/highadc beforefirstwrite;separatefromreceive',synthetic_alias='NewAlias',plain_seed_frame=old['plain_seed_frame'],initial_counter=seed,receive_setup_wire=receive_wire,receive_after_setup=recv_counter,mode206_wire=modewire,receive_after_mode=counter_add_adc(recv_counter,len(old['mode206'])),send_cases=cases,actual_PE_execution=False,actual_provider_or_socket_execution=False,actual_credentials_input=False,raw_capture_dump=False)
save(L/'r495-source-oracle.json',record)
failure=O/'runtime-patch-attempt1.txt';failure.write_text('apply_patch Runtime multi-hunks were outofsourceorder (Frame::Seed hunkbeforeearliersource_stream_snapshot);atomicverification failed,zeroRuntimechanges. Newsendmodule previousseparatepatch alreadywritten. Correctedorderedhunks beforeRustbuild;noRustbuildfail/consumerexecution.\n',encoding='utf-8')
save(L/'r495-preparation-history.json',dict(Runtime_patch_preparation_failures=1,failed_Runtime_patch_summary=pin(failure),files_changed_by_failed_Runtime_patch=0,new_send_module_already_written=True,Rust_production_build_failures=0,consumer_compile_failures=0,consumer_test_failures=0,source_oracle_failures=1,failed_oracle_source=pin(O/'oracle-attempt1.py'),failed_oracle_stderr=pin(O/'oracle-attempt1.stderr.log'),oracle_correction='800symbolicsteps enoughforcs201butnot87byterosterprelude;raiseboundedbudget3000forstrictbody<=155. NoNativeexecution orRust/consumerfailure.'))
print('R495 independent sourceinstruction oracle prepared:',len(cases),'typedcs201 frames;no credentials/capture/Nativeexecution')
