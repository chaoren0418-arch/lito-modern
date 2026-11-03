from r491_common import *
import shutil
base=P.parent.parent;dest=L/'r522-inputs';dest.mkdir();refs={};occ=[];copies=[]
roots=[(base/'lito伺服器/server-ordinary-room-map-network.json','3BBAA920A2B379B7562A38C3FF2CADC8842B98F7033C3A5C103174F4A28A8545'),(base/'lito伺服器/server-ordinary-room-map-selection-contract.json','3D0B5BAA46238ED5589C193FBDE22B966B945EFDEAB70D2A2C3933A62CA8E15F'),(base/'lito脫殼資源/CDD-outbound-contract.json','7E09849B4F0D442E04457AEE7E6DBEEB3D106437D9689AE8060323CE36FCB59E')]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   p=Path(v['path']);a=refs.setdefault(str(p),pin(p));assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),(v,a);occ.append(a)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(p,name):
 q=dest/name;q.parent.mkdir(parents=True,exist_ok=True);assert not q.exists();shutil.copyfile(p,q);copies.append(pin(q))
docs=[]
for p,sha in roots:
 assert pin(p)['sha256']==sha;d=read(p);docs.append(d);walk(d);copy(p,p.name)
m=read(docs[0]['manifest']['path']);root=Path(docs[0]['evidence_root']);assert len(m['files'])==57
for e in m['files']:
 p=Path(e['path']);p=p if p.is_absolute() else root/p;a=pin(p);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']);refs[str(p)]=a;copy(p,Path('network')/p.relative_to(root))
f=read(docs[0]['new_network']['frames']['path']);indices=[19,65,69,70,73,74,77,79,80,81,84,85,89,90];assert f[73]['body_hex']=='d1030202' and f[79]['body_hex']=='d103fd01' and f[84]['body_hex']=='d103ffff';assert f[81]['body_hex']==f[85]['body_hex']=='d50301000000fd01'
data=read(L/'r521-test-inputs.json');setup=data['setup_frames']
for e in setup:
 if e['epoch']=='GAME-initial' and e['direction']=='S' and e.get('body_hex','').startswith('cb'):e['body_hex']=f[19]['body_hex'];e['frame_bytes']=f[19]['frame_bytes']
out=dict(setup_frames=setup,source_ascii='MapNetMoon',room_name='MapNetRoom',CE=f[65]['body_hex'],host_CD=f[69]['body_hex'],guest_CD=f[70]['body_hex'],C514=f[73]['body_hex'],map514=f[74]['body_hex'],D1_before=f[77]['body_hex'],C509=f[79]['body_hex'],D1_reset=f[80]['body_hex'],map509=f[81]['body_hex'],CFFFF=f[84]['body_hex'],invalid_restore=f[85]['body_hex'],nonowner_summary=f[89]['body_hex'],nonowner_configuration=f[90]['body_hex'],selected_frames=[dict(index=i,frame=f[i]) for i in indices],next_real=data['next_real'],prior_setup=pin(L/'r521-test-inputs.json'),actual_saved_frames=docs[0]['new_network']['frames'],context='R522 ONE new own normal-entry synthetic writer case; current normal SOURCE/CB/CE/CD and owner map514, GuestReady1, owner map509 D1reset→D5map509, ownerFFFF restore509 from saved actual C7 frames reencoded. Later Guest nonowner summary/config solely independent noACK probe, not sameHost flow. Actual C7 offline evidence adopted; no new actual C7backend/Java/SQL/GUI run.')
save(L/'r522-test-inputs.json',out)
save(L/'r522-input-adoption.json',dict(roots=[pin(p) for p,_ in roots],full_contracts=docs,reference_occurrences=len(occ),unique_references=list(refs.values()),copies=copies,payload_files=57,selected_frames=indices,test_inputs=pin(L/'r522-test-inputs.json'),source_Java_case='Accepted actual C7 WAITING one network case 89encrypted/85visible/4masked adopted offline by pin; no rerun, no generalrace or othermode qualification.',version_boundary='AD368 firstdefault564 static discrepancy unchanged; parent0612833B restores currenthub map; C7 inherits restore andknownupdateMap ordering repair. Actual current network C7 only WAITING514→509→FFFFrestore509, perpeer D1→D5 ordering; no ACK.',BIN26_future_only='Full CDD root/VA/conditional UIvirtual/getters/serializer/transportSTOP/unknowns preserved; no C_DD or modewriter. BIN27 unaccepted and not adopted; no Native fullUI chain claim.',Java_SQL_GUI=0,old_PASS_replays=0))
print('R522 adopted',len(refs),'unique refs;57 payloads;accepted actual C7 frames offline, no new backend run')
