from r491_common import *
import shutil
roots=[Path(r'D:\lineage開發\3-lito開發\lito伺服器\server-room-D5-CF-consumer-supplement.json'),Path(r'D:\lineage開發\3-lito開發\lito伺服器\server-room-last-member-delete-reset-network.json'),Path(r'D:\lineage開發\3-lito開發\lito脫殼資源\D5-CF-receive-contract.json')]
expected=['7774EA7546EEF33CB5240A58A36FA9FFF7A465303049C3C3E238081887279BEF','4CB29A6DDCB95B4EEE9389AA1B36BF179F3415878F7361E7A7149ABE7F066517','C8E20CD020392C0D26E21F8FB1B830B9009998B942CABF65EBDF59AA355EDA2C']
cache={};verified=[];raw=[];copied=[]
def verify(e,base=None):
 q=Path(e['path']);q=q if q.is_absolute() else base/q
 if str(q) not in cache:cache[str(q)]=pin(q)
 a=cache[str(q)];assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']),(e,a);verified.append(a);return q
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):verify(v)
  if v.get('file_offset') and v.get('raw_hex'):raw.append(v)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
root=L/'r517-inputs';root.mkdir();docs=[]
def copy(q,destination):
 destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(q,destination);copied.append(pin(destination))
for i,q in enumerate(roots):
 assert pin(q)['sha256']==expected[i];d=read(q);docs.append(d);walk(d);copy(q,root/q.name)
 if 'manifest' in d:
  manifest=read(d['manifest']['path']);base=Path(d['evidence_root'])
  for e in manifest['files']:verify(e,base)
  copy(Path(d['manifest']['path']),root/str(i)/'manifest.json')
native=docs[2];pe=Path(native['sources'][0]['path']).read_bytes()
for v in raw:
 b=bytes.fromhex(v['raw_hex']);at=int(v['file_offset'],16);assert pe[at:at+len(b)]==b
new_raw=[]
for e in native['reusable_outputs']:
 q=verify(e);copy(q,root/'native'/q.name)
 if q.suffix=='.bin':
  va=int(q.name.split('-')[0],16);b=q.read_bytes();assert pe[va-0x400000:va-0x400000+len(b)]==b;new_raw.append(dict(pin=pin(q),va=va,matches=True))
reference=read(docs[0]['reference_index']['path']);walk(reference);copy(Path(docs[0]['reference_index']['path']),root/'reference-index.json')
methods=[];text_cache={}
for name,e in reference['archived_method_indexes'].items():
 cls=name.split('.')[0];path=Path(reference['selected_reference_pins'][f'AD368-{cls}.javap.txt']['path'])
 if str(path) not in text_cache:text_cache[str(path)]=path.read_text('utf-8-sig').splitlines()
 lo,hi=e['archived_lines_one_based'];selected='\n'.join(text_cache[str(path)][lo-1:hi]);actual=hashlib.sha256(selected.encode('utf-8')).hexdigest().upper();assert actual==e['method_UTF8_sha256'],name;methods.append(dict(method=name,span=[lo,hi],sha256=actual,matches=True))
dynamic=docs[1]
for k in ['new_network','wire_poststate','write_boundaries','poststates','cleanup','protected','accounting']:
 e=dynamic[k]['frames'] if k=='new_network' else dynamic[k];q=verify(e);copy(q,root/'dynamic'/q.name)
prior=read(L/'r516-test-inputs.json');old_frames=read(L/'r516-inputs/dynamic/frames.json');wire=docs[0]['wire']
assert all(old_frames[i]['direction']=='S' for i in [71,72,75,77])
assert all(old_frames[i]['body_hex'].startswith('cd') for i in [71,72])
assert old_frames[75]['body_hex'].startswith('d0') and old_frames[77]['body_hex']==wire['owner_D5']['body_hex']
save(L/'r517-test-inputs.json',dict(setup_frames=prior['setup_frames'],CE=prior['CE'],host_CD=old_frames[71]['body_hex'],guest_CD=old_frames[72]['body_hex'],peer_D0=old_frames[75]['body_hex'],owner_D5=wire['owner_D5']['body_hex'],CF=dynamic['new_network']['nondefault_delete']['CF_body_hex'],next_CC=dynamic['new_network']['default_reset']['CC_body_hex'],old_setup=pin(L/'r516-test-inputs.json'),old_owner_trace=pin(L/'r516-inputs/dynamic/frames.json'),new_delete_reset_trace=dynamic['new_network']['frames'],context='Existing saved wire bodies recombined in ONE new Modern synthetic normal-entry case; not actual Java run/cross-peer ordering proof. S_CC remains unsupported.'))
save(L/'r517-input-adoption.json',dict(roots=[pin(q) for q in roots],matching_reference_occurrences=len(verified),unique_references=list(cache.values()),copied=copied,root_raw_occurrences=len(raw),unique_root_raw=len({(e['file_offset'],e['raw_hex']) for e in raw}),new_raw_slices=new_raw,archived_method_UTF8_checks=methods,native_wire=native['wire'],native_D5_owner=native['D5_owner'],native_CF_delete=native['CF_delete'],native_consumer_gap=native['consumer_gap'],Native54_instruction_assertions='Producer result only; raw witnesses independently matched PE, no instruction-test rerun.',server_owner_routing=docs[0]['owner_routing'],server_CF=docs[0]['CF_contract'],dynamic_network=dynamic['new_network'],dynamic_findings=dynamic['findings'],dynamic_unknowns=dynamic['unknowns'],test_inputs=pin(L/'r517-test-inputs.json'),new_Java_SQL_Native_GUI_calls=0,old_PASS_replays=0))
print('Adopted',len(verified),'refs;',len(cache),'unique;',len(raw),'root raw;',len(new_raw),'new raw slices;',len(methods),'archived method span SHA checks')
