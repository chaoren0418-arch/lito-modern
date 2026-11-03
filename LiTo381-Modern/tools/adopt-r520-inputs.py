from r491_common import *
import shutil
base=P.parent.parent;roots=[base/'lito伺服器/server-ordinary-room-DD-edit-network.json',base/'lito伺服器/server-room-D5-CF-consumer-supplement.json'];expected=['A13ECBFE9DC9BFCBA1D32649C829FB960DA995591C59C8613BB403FA4D62C168','7774EA7546EEF33CB5240A58A36FA9FFF7A465303049C3C3E238081887279BEF'];dest=L/'r520-inputs';dest.mkdir();cache={};occurrences=[];copies=[];docs=[]
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
for i,p in enumerate(roots):
 assert pin(p)['sha256']==expected[i];d=read(p);docs.append(d);walk(d);copy(p,dest/p.name);m=read(d['manifest']['path']);root=Path(d['evidence_root']);copy(Path(d['manifest']['path']),dest/str(i)/'manifest.json')
 for e in m['files']:
  q=verify(e,root);copy(q,dest/str(i)/q.relative_to(root))
  if q.suffix=='.json':walk(read(q))
reference=read(docs[1]['reference_index']['path']);methods=[];texts={}
for name,e in reference['archived_method_indexes'].items():
 cls=name.split('.')[0];p=Path(reference['selected_reference_pins'][f'AD368-{cls}.javap.txt']['path'])
 if str(p) not in texts:texts[str(p)]=p.read_text('utf-8-sig').splitlines()
 lo,hi=e['archived_lines_one_based'];digest=hashlib.sha256('\n'.join(texts[str(p)][lo-1:hi]).encode('utf-8')).hexdigest().upper();assert digest==e['method_UTF8_sha256'];methods.append(dict(method=name,source=pin(p),span=[lo,hi],sha256=digest,matches=True))
static=read(docs[0]['static_contract']['path']);f=read(docs[0]['new_network']['frames']['path']);indices=[19,65,69,70,74,77,78,79,84,85]
assert all(f[i]['direction']=='S' for i in indices)
assert f[78]['body_hex']==f[81]['body_hex']==f[84]['body_hex'] and f[79]['body_hex']==f[82]['body_hex']==f[85]['body_hex']
prior=read(L/'r519-test-inputs.json');setup=prior['setup_frames']
for e in setup:
 if e['epoch']=='GAME-initial' and e['direction']=='S' and e.get('body_hex','').startswith('cb'):e['body_hex']=f[19]['body_hex'];e['frame_bytes']=f[19]['frame_bytes']
save(L/'r520-test-inputs.json',dict(setup_frames=setup,source_ascii='EditMoon',CE=f[65]['body_hex'],host_CD=f[69]['body_hex'],guest_CD=f[70]['body_hex'],D1_before=f[74]['body_hex'],D1_reset=f[77]['body_hex'],summary=f[78]['body_hex'],configuration=f[79]['body_hex'],rejected_summary=f[84]['body_hex'],rejected_configuration=f[85]['body_hex'],selected_frames=[dict(index=i,frame=f[i]) for i in indices],next_real=prior['next_real'],prior_setup=pin(L/'r519-test-inputs.json'),actual_saved_frames=docs[0]['new_network']['frames'],context='ONE new Modern normal-entry synthetic recombination, no C_DD/C_D1 map edit/ready command. Actual saved owner D1reset→D5/0→1 and nonowner identical snapshots reused. Extra unpaired ordering/changed summary/config are synthetic only; no unchanged Java flow/atomic pair/edit ACK/server auth or new server runtime claim.'))
maproot=base/'lito脫殼資源/map-candidate-index-contract.json';assert pin(maproot)['sha256']=='04EBECA473FF0BD0FC6A0EEDDE36AF8819EC2FE59F037D08367BF7EF38B36307';copy(maproot,dest/maproot.name)
save(L/'r520-input-adoption.json',dict(roots=[pin(p) for p in roots],reference_occurrences=len(occurrences),unique_references=list(cache.values()),copies=copies,contracts=docs,static_edit_contract=static,archived_method_span_checks=methods,actual_success_and_rejected_D5_bodies_identical=True,offline_map_dependency_only=pin(maproot),map_runtime_expanded=False,Native_D5_branch_contract_adopted=False,Native_behavior_comparison_claimed=False,test_inputs=pin(L/'r520-test-inputs.json'),Java_SQL_GUI=0,old_PASS_replays=0))
print('R520 adopted',len(occurrences),'references;',len(cache),'unique;',len(methods),'archived IL spans; success/reject exact same snapshots; Java layout only')
