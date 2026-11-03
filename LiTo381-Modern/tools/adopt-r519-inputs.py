from r491_common import *
import shutil
root=P.parent.parent/'lito伺服器/server-ordinary-room-self-alias-qualification.json';assert pin(root)['sha256']=='423A1EA9E4578E735FF3CC9717AADA57CA0434C2F1129E2F696EE372CAE6507F'
d=read(root);dest=L/'r519-inputs';dest.mkdir();cache={};occurrences=[];copies=[]
def verify(e,base=None):
 p=Path(e['path']);p=p if p.is_absolute() else base/p
 if str(p) not in cache:cache[str(p)]=pin(p)
 a=cache[str(p)];assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']),(e,a);occurrences.append(a);return p
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):verify(v)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(p,to):
 to.parent.mkdir(parents=True,exist_ok=True);assert not to.exists();shutil.copyfile(p,to);copies.append(pin(to))
walk(d);copy(root,dest/root.name);manifest=read(d['manifest']['path']);base=Path(d['evidence_root']);copy(Path(d['manifest']['path']),dest/'manifest.json')
for e in manifest['files']:
 p=verify(e,base);copy(p,dest/p.relative_to(base))
 if p.suffix=='.json':walk(read(p))
evidence=read(d['evidence']['path']);methods=[]
for key,source_key in [('CS_method_indexes','actual-CS-IL'),('Protocol_method_indexes','actual-Protocol-IL')]:
 p=Path(evidence['selected_reference_pins'][source_key]['path']);lines=p.read_text('utf-8-sig').splitlines()
 for name,items in evidence[key].items():
  for e in items if isinstance(items,list) else [items]:
   lo,hi=e['archived_lines_one_based'];digest=hashlib.sha256('\n'.join(lines[lo-1:hi]).encode('utf-8')).hexdigest().upper();assert digest==e['method_UTF8_sha256'];methods.append(dict(method=name,source=pin(p),span=[lo,hi],sha256=digest,matches=True))
frames=read(evidence['selected_reference_pins']['ordinary-frames']['path']);copy(Path(evidence['selected_reference_pins']['ordinary-frames']['path']),dest/'ordinary-frames.json')
assert all(frames[i]['direction']=='S' for i in [18,19,21,65,69,72,74]);assert frames[73]['body_hex']=='d0'
prior=read(L/'r518-test-inputs.json');setup=prior['setup_frames']
for e in setup:
 if e['epoch']=='GAME-initial' and e['direction']=='S' and e.get('body_hex','').startswith('cb'):e['body_hex']=frames[19]['body_hex'];e['frame_bytes']=frames[19]['frame_bytes']
save(L/'r519-test-inputs.json',dict(setup_frames=setup,source_ascii='RoomExitMoon',CE=frames[65]['body_hex'],host_CD=frames[69]['body_hex'],guest_CD=frames[72]['body_hex'],self_D0=frames[74]['body_hex'],self_alias_raw=d['ordinary_trace'][0]['roster_alias_raw_hex'],count_body=frames[18]['body_hex'],row_body=frames[19]['body_hex'],accepted_C8=frames[21]['body_hex'],actual_C_D0=frames[73]['body_hex'],next_real=prior['next_real'],prior_setup=pin(L/'r518-test-inputs.json'),ordinary_frames=evidence['selected_reference_pins']['ordinary-frames'],context='ONE new Modern synthetic normal-entry recombination. Exact saved ordinary Host singleton row/count/C8/CE/CD/selfD0 reused, with owned fake login; no new actual backend authentication/commit or unchanged Java flow claim. Typed login intentionally differs from issued alias.'))
maproot=P.parent.parent/'lito脫殼資源/map-table-source-contract.json';assert pin(maproot)['sha256']=='12D76285CC907373A9CBB4D19C88ADD58AB997536E5238004321F83233B803C2';copy(maproot,dest/maproot.name)
save(L/'r519-input-adoption.json',dict(root=pin(root),references=len(occurrences),unique_references=list(cache.values()),copies=copies,contract=d,archived_method_span_checks=methods,offline_encoder_examples=read(d['offline_encoder_examples']['path']),offline_encoder_scope='Existing producer offline9 examples read only; not rerun/production auth qualification. RouteB preserves issued raw bytes; no encoder implementation or alias splitting.',map_dependency_only=pin(maproot),map_runtime_expanded=False,test_inputs=pin(L/'r519-test-inputs.json'),Java_SQL_GUI=0,old_PASS_replays=0))
print('R519 adopted',len(occurrences),'references;',len(cache),'unique;',len(methods),'IL spans; saved singleton/selfD0; map dependency only')
