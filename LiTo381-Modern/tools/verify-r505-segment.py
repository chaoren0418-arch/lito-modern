from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
import difflib,struct
seen={};instances=0
def check(q,path=None):
 global instances
 p=Path(path or q['path']);instances+=1
 if str(p)not in seen:seen[str(p)]=pin(p)
 assert seen[str(p)]['sha256']==q['sha256'] and seen[str(p)]['bytes']==q['bytes'],str(p)
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):check(v)
  else:
   for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
snap=read(L/'r505-production-source-snapshot.json');assert len(snap['source_pins'])==619
for q in snap['source_pins']:check(q);check(q,q['snapshot'])
old=read(L/'r504-production-source-snapshot.json');diff=[];changed=[]
for q in old['source_pins']:
 check(q,q['snapshot']);current=pin(q['path'])
 if current['sha256']!=q['sha256']:
  changed.append(Path(q['path']).name);diff.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(changed)=={'native_city_runtime.rs','native_source_connection.rs'}
new=P/'src/modern_lito/native_entry_transfer.rs';diff.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
assert (L/'r505-source-difference.patch').read_text(encoding='utf-8')=='\n'.join(diff)
consumer=read(L/'r505-transfer-consumer-candidate.json');assert len(consumer['source_pins'])==221
for q in consumer['source_pins']:check(q);check(q,q['snapshot'])
check({k:consumer[k]for k in ('path','bytes','sha256')});check(consumer['production_library'])
study=read(L/'r505-source-study.json');walk(study);lines=Path(study['originals'][0]['path']).read_text(encoding='utf-8-sig').splitlines();data=Path(study['originals'][1]['path']).read_bytes();asm_count=0;pe_count=0
for q in study['fragments']:
 p=Path(q['path']);text=p.read_text(encoding='utf-8').splitlines()
 if p.name.startswith('asm-'):
  for line in text:
   n,s=line.split(': ',1);assert lines[int(n)-1]==s,(p,n)
  asm_count+=1
 else:
  va=int(text[0].split()[1],16);raw=bytes.fromhex(text[1]);assert data[va-0x400000:va-0x400000+len(raw)]==raw
  expected=[f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va)];assert expected==text[2:];pe_count+=1
assert data[0x502f2c-0x400000:0x502f2c-0x400000+4]==b'cdh\0'
assert data[0x62c6a4-0x400000:0x62c6a4-0x400000+10]==b'127.0.0.1\0'
assert data[0x64ad74-0x400000:0x64ad74-0x400000+256]==bytes(256)
assert next(Cs(CS_ARCH_X86,CS_MODE_32).disasm(data[0x4efe56-0x400000:0x4efe56-0x400000+5],0x4efe56)).op_str=='0x758f57e0'
entry=read(L/'r505-entry-source-contract.json');player=read(L/'r505-player-candidate.json');walk(entry);walk(player);assert player['candidate_executions']==0
main=Path(entry['candidate_main']['path']);orig=Path(entry['original_main']['path']);entry_diff=''.join(difflib.unified_diff(orig.read_text(encoding='utf-8').splitlines(True),main.read_text(encoding='utf-8').splitlines(True),fromfile=str(orig),tofile=str(main)));(L/'r505-player-entry-difference.patch').write_bytes(entry_diff.encode('utf-8'))
details=read(L/'r505-event-details.json');assert len(details['flows'])==6 and all(q['passed']for q in details['flows']);assert details['complete_socket_TX_frames']==7;assert details['IO_workers_joined']==details['endpoint_threads_joined']==3;assert details['Runtime_Closed']==2;assert not details['minimum_channel_path_achieved']
erratum=read(L/'r505-count-erratum.json');assert erratum['correct_setup_TX_frames']==6 and erratum['correct_character_TX_frames']==1 and not erratum['rerun'];walk(erratum)
exit_record=read(L/'r505-transfer-consumer-exit.json');assert exit_record['exit_code']==0 and exit_record['executions']==1;walk(exit_record)
assert '6 flows /512signedboundaries /7 actualTXframes' in Path(exit_record['stdout']['path']).read_text(encoding='utf-8-sig')
history=read(L/'r505-execution-history.json');assert history['functional_failures']==0 and history['metadata_count_errata']==1 and history['player_executions']==0;walk(history)
for name in ['r505-source-oracle.json','r505-open-gaps.json','r505-unpacker-minimum-dependency.json']:
 v=read(L/name);walk(v);check(pin(L/name))
assert not read(L/'r505-open-gaps.json')['minimum_channel_path_achieved']
runtime=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');mod=new.read_text(encoding='utf-8');assert runtime.count('self.entry_transfer.retire()?')==2;assert 'prepare_body'not in mod and 'prepare_fixture_body'not in mod and 'AFBC=1'not in mod
max_poll=max(q['details'].get('max_poll_us',q['details'].get('IO',{}).get('max_poll_us',0))for q in details['flows'])
save(L/'r505-independent-contract.json',dict(state='R505_TRANSFER_PREFIX_FINITE_PASS_MINIMUM_CHANNEL_PATH_OPEN',source_files=619,source_pairs=221,changed=2,added=1,unchanged=616,actual_diff_bytes=(L/'r505-source-difference.patch').stat().st_size,entry_diff_bytes=len(entry_diff.encode()),ASM_fragments_verified=asm_count,PE_fragments_verified=pe_count,pin_instances=instances,unique_paths=len(seen),flows=6,signed_boundaries=512,complete_socket_TX_frames=7,setup_TX_frames=6,character_TX_frames=1,navigation_TX_frames=0,IO_workers_joined=3,endpoint_threads_joined=3,Runtime_Closed=2,elapsed_seconds=details['elapsed_seconds'],max_observed_Runtime_poll_us=max_poll,source_count_erratum=pin(L/'r505-count-erratum.json'),minimum_channel_path_achieved=False,no_flags_or_arbitrary_phase_setters=True,historical_PASS_reruns=0,GUI=0,source_snapshot=pin(L/'r505-production-source-snapshot.json'),source_difference=pin(L/'r505-source-difference.patch'),entry_difference=pin(L/'r505-player-entry-difference.patch'),unpacker_minimum_dependency=pin(L/'r505-unpacker-minimum-dependency.json')))
print(f'R505 independent {instances}pins/{len(seen)}unique;{asm_count}ASM/{pe_count}PE;6flows7TX;channelOPEN')
