from r491_common import *
import collections
def warnings(n):
 rows=[json.loads(s)for s in (L/f'{n}-library-build.jsonl').read_text('utf-8-sig').splitlines()if s.startswith('{')]
 assert any(v.get('reason')=='build-finished'and v.get('success')for v in rows)
 messages=[v['message']for v in rows if v.get('reason')=='compiler-message'];assert not any(v['level']=='error'for v in messages)
 descriptors=collections.Counter(json.dumps([v.get('code'),v['message'],sorted(s['file_name']for s in v.get('spans',[])if s['is_primary'])],sort_keys=True)for v in messages if v['level']=='warning')
 return descriptors
old=warnings('r525');new=warnings('r526');assert old==new and sum(new.values())==288
consumer_stderr=(L/'r526-consumer-build.stderr.log').read_text('utf-8-sig');player_stderr=(L/'r526-player-build.stderr.log').read_text('utf-8-sig');unit_stderr=(L/'r526-new-targeted-tests.stderr.log').read_text('utf-8-sig')
assert not consumer_stderr.strip();assert 'warning: 1 warning emitted' in player_stderr and 'Outcome' in player_stderr;assert '89 warnings (13 duplicates)' in unit_stderr
save(L/'r526-compiler-warning-verification.json',dict(library_warning_messages=288,prior_saved_warning_messages=288,new_warning_descriptors=[],removed_warning_descriptors=[],errors=0,consumer_warnings=0,player_inherited_Outcome_warning=1,lib_test_inherited_warnings=89,lib_test_duplicate_warnings=13,lib_test_inherited_R524_naming_warning=1,new_product_warnings=0))
trace=read(L/'r526-new-wire-producer-trace.json');chains=[];ciphers=[];epoch=0;masked=0;rounds=set()
for i,v in enumerate(trace):
 rounds.add(v['round'])
 if 'C_cipher_hex'in v and v['C'][0]==0:
  epoch+=1;client_counter=server_counter=0x87546CA100000000|([0x12345678,0x10203040][(epoch-1)%2])
 if 'C_login'in v:
  masked+=1;client_counter=(client_counter+v['body_bytes'])&((1<<64)-1)
 elif 'C'in v:
  assert v['counter_before']==client_counter;client_counter=(client_counter+len(v['C']))&((1<<64)-1);assert v['counter_after']==client_counter
  chains.append(dict(index=i,epoch=epoch,direction='C',before=v['counter_before'],after=client_counter))
 elif 'S'in v:
  before=server_counter;server_counter=(server_counter+len(v['S']))&((1<<64)-1)
  if 'cipher_hex'in v:assert v['counter_before']==before and v['counter_after']==server_counter
  chains.append(dict(index=i,epoch=epoch,direction='S',before=before,after=server_counter,pre_case_plaintext_only='cipher_hex'not in v))
 direction='C'if'C_cipher_hex'in v else'S'if'cipher_hex'in v else None
 if direction:
  body=bytes(v[direction]);raw=bytes.fromhex(v['C_cipher_hex'if direction=='C'else'cipher_hex']);key=v['counter_before'].to_bytes(8,'little');previous=0;plain=bytearray()
  for n,b in enumerate(raw):plain.append(b^key[n&7]^previous);previous=b
  assert bytes(plain)==body and v['frame_bytes']==len(body)+2 and v['counter_after']-v['counter_before']==len(body)
  ciphers.append(dict(index=i,epoch=epoch,round=v['round'],direction=direction,body_hex=body.hex(),body_bytes=len(body),frame_bytes=v['frame_bytes'],counter_before=v['counter_before'],counter_after=v['counter_after'],matches=True))
assert epoch==4 and masked==4 and rounds=={0,1}
S=[v for v in ciphers if v['direction']=='S'];C=[v for v in ciphers if v['direction']=='C'];ready=[v for v in C if v['body_hex'].startswith('d2')];ticks=[v for v in S if v['body_hex'].startswith('e1')];d1=[v for v in S if v['body_hex'].startswith('d1')]
assert len(S)==18 and len(C)==17 and [v['body_hex']for v in ready]==['d202','d201','d200'] and all(v['frame_bytes']==4 and v['body_bytes']==2 and v['counter_after']-v['counter_before']==2 for v in ready)
assert [v['body_hex']for v in ticks]==['e103','e102','e101','e100','e103','e102'] and len(d1)==4 and S[-1]['body_hex']=='28340200'
assert not any(bytes.fromhex(v['body_hex'])[0]in[208,209,221]for v in C)
for value in ready:assert value['body_hex']!='d2'
save(L/'r526-independent-wire-verification.json',dict(trace=pin(L/'r526-new-wire-producer-trace.json'),new_S_cipher_frames=len(S),nonsecret_C_cipher_frames=len(C),participant_requests=ready,E1_receipts=ticks,D1_receipts=d1,all_cipher_frames=ciphers,seed_all_body_counter_chain=chains,own_transport_epochs=epoch,normal_entry_rounds=2,masked_own_LOGIN_bodies=masked,first_unsupported_WORLD=S[-1],same_epoch_stale_detection_proved=False,level='Successful secondattempt ownsynthetic only; savedgenuine AD368 WORLD body as cutoff, no Java/backend execution'))
chain=read(L/'r526-inputs/ready-chain-comparison.json');shell=read(L/'r526-inputs/class-shell-comparison.json');methods=[dict(class_name=c['class_name'],**m)for c in chain['classes']for m in c['selected_methods']];same=[v for v in methods if v['Code_attribute_same']];changed=[v['class_name']+'.'+v['name']for v in methods if not v['Code_attribute_same']]
assert len(methods)==99 and len(same)==96 and set(changed)=={'ClientSession.changeRoomMap','GameHub.updateMap','GameHub.updateSettings'};assert len(shell)==23 and all(v['non_Code_class_shell_same']for v in shell)and all(c['constant_pool_same']for c in chain['classes'])
for m in methods:assert m['Code_attribute_same']==(m['AD368_Code_sha256']==m['candidate_Code_sha256'])
save(L/'r526-version-chain-verification.json',dict(selected_classes=len(chain['classes']),class_shells=len(shell),selected_methods=len(methods),Code_same=len(same),changed=changed,constant_pool_same=True,non_Code_shell_same=True,opcodes=chain['opcode_ConstantValue'],actual485F_newcountdownwire=False,comparison_root=pin(L/'r526-inputs/ready-chain-comparison.json'),source_Batch33_anchor_checks=pin(L/'r526-BIN33-source-anchor-verification.json')))
source=read(L/'r526-production-source-snapshot.json');checked=[]
for e in source['source_pins']:
 for p in [e['path'],e['snapshot']]:
  actual=pin(p);assert actual['bytes']==e['bytes']and actual['sha256']==e['sha256'];checked.append(actual)
prior=read(L/'r525-production-source-snapshot.json')
for e in prior['source_pins']:
 actual=pin(e['snapshot']);assert actual['bytes']==e['bytes']and actual['sha256']==e['sha256'];checked.append(actual)
assert len(source['changes'])==6 and len(source['source_pins'])==639
adopt=read(L/'r526-input-adoption.json')
for e in adopt['verified_pins']:
 actual=pin(e['path']);assert actual['bytes']==e['bytes']and actual['sha256']==e['sha256'];checked.append(actual)
consumer=read(L/'r526-consumer-candidate.json');player=read(L/'r526-player-candidate.json');assert consumer['production_library']['sha256']==player['production_library']['sha256'];assert player['candidate_executions']==0
for e in [consumer['executable'],consumer['production_library'],player['executable'],player['production_library']]:assert pin(e['path'])==e;checked.append(e)
scratch=read(L/'r526-scratch-cleanup.json');assert scratch['empty_directories_preserved']==5 and not scratch['deletion_attempted']and all(not any(Path(e['path']).iterdir())for e in scratch['paths'])
save(L/'r526-final-verification.json',dict(state='PASS',checked_pins=checked,total_checked=len(checked),source_changes_after_network=0,source_files=639,changed_files=6,same_final_regular_rlib=consumer['production_library'],old_PASS_executed=0,actualJava_SQL_GUI=0,failures_retained=[pin(L/'r526-consumer-repair.json'),pin(L/'r526-adoption-repair.json')],allfive_scratch_empty_preserved=True))
print('R526 verificationPASS',len(checked),'pins;18S/17C;4seeds/4maskedLOGIN;639sources6changes;288samewarnings')
