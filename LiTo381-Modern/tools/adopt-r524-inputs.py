from r491_common import *
import shutil
base=P.parent.parent;dest=L/'r524-inputs';dest.mkdir();refs={};occ=[];copies=[]
roots=[(base/'lito伺服器/server-ordinary-room-settings-candidate-network.json','8C1776AA503098E09B42EFE8CA45E07E651DD4570A07CB6A8994006421CCEF41'),(base/'lito伺服器/server-ordinary-room-settings-order-candidate.json','8D969CB59FA4ECD3515EEE794397E4C94D58D97328F8A410A4EA0BBA456CCF9E'),(base/'lito脫殼資源/CDD-outbound-contract.json','7E09849B4F0D442E04457AEE7E6DBEEB3D106437D9689AE8060323CE36FCB59E')]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   p=Path(v['path']);a=refs.setdefault(str(p),pin(p));assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),p;occ.append(a)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(p,name):
 q=dest/name;q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q);copies.append(pin(q))
docs=[]
for p,sha in roots:
 assert pin(p)['sha256']==sha;d=read(p);docs.append(d);walk(d);copy(p,p.name)
network=docs[0];assert network['actual_JAR']['sha256']=='485F599DA9BA1A13504CB48DDCA3D53C6CAD91D7BAFDF738D104E65A05E1DFD5'
m=read(network['manifest']['path']);root=Path(network['evidence_root']);assert len(m['files'])==64
for e in m['files']:
 p=Path(e['path']);p=p if p.is_absolute() else root/p;a=pin(p);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']);refs[str(p)]=a;copy(p,Path('485F-settings')/p.relative_to(root))
# Inspect saved parser contract and actual new frame bodies; no class producer
# or Java process is run. Preserve inherited BIN26 UI/codec/transport STOPs.
parser=read(docs[2]['server_comparison']['static_contract']['path']);walk(parser);copy(Path(docs[2]['server_comparison']['static_contract']['path']),'static-edit-contract.json')
frames=read(network['new_network']['frames']['path']);data=read(L/'r523-test-inputs.json');data['source_ascii']='SetNetMoon'
for v in data['setup_frames']:
 if v.get('epoch')=='GAME-initial' and v.get('direction')=='S' and v.get('body_hex','').startswith('cb'):v.update(body_hex=frames[19]['body_hex'],frame_bytes=frames[19]['frame_bytes']);v.pop('encrypted_body_hex',None)
for key,i in [('CE',65),('host_CD',69),('guest_CD',70),('map514',74),('D1_before',77),('actual_C_DD_extended',79),('D1_extended_reset',80),('extended_summary',81),('extended_configuration',82),('D1_before_compact',87),('actual_C_DD_compact',89),('D1_compact_reset',90),('compact_summary',91),('compact_configuration',92),('rejected_summary',100),('rejected_configuration',101)]:data[key]=frames[i]['body_hex']
for k in ['D1_reset','summary','configuration','actual_C_DD','input_name','input_attributes','input_password','input_capacity','actual_saved_frames']:data.pop(k,None)
indices=[19,65,69,70,74,77,79,80,81,82,87,89,90,91,92,100,101];data['selected_frames']=[dict(index=i,frame=frames[i]) for i in indices];data['prior_setup']=pin(L/'r523-test-inputs.json');data['actual_saved_frames']=network['new_network']['frames'];data['context']='R524 ONE new own synthetic normal-entry true Sender485F extended thencompact; saved Host map514 as independent receive-only prerequisite (no Cmap sent), readiness D1 and per-peer owner D1→D5/0,1 reencoded; Guest rejected identical snapshots extra independent noACK probe. Normal entry grammar remains explicit inheritedAD368 structural adapter, new485F settings writer/0,1 adapters required; no actualJavaSQL/GUI, no options-derived Create/Join authorization.'
assert data['actual_C_DD_extended']=='dd536574457874656e646564526f6f6d002c70770004000201070903030608' and data['actual_C_DD_compact']=='dd536574436f6d70616374526f6f6d0028000600';save(L/'r524-test-inputs.json',data)
save(L/'r524-input-adoption.json',dict(roots=[pin(p) for p,_ in roots],full_contracts=docs,reference_occurrences=len(occ),unique_references=list(refs.values()),copies=copies,parser_contract=docs[2]['server_comparison']['static_contract'],saved_485F_payload_files=64,producer_SHA=network['actual_JAR']['sha256'],version_switch='New explicit485F settings adapters only; R523 AD368 handoff/results remain immutable. Bootstrap/CE/participant/map grammar reused structurally, no actual485Fbackend claim.',layout='Compact6+N+P no tail; extended11+N+P+K fouroptionBYTEs+KBYTE+KselectionBYTEs. ASCII common policy remains; K0..8/nonzero exact tail; options allzero withK0 ordomains7/2/1..127/31, sample2,1,7,9 and3,6,8 is edit input, not Create/Join admission.',unknowns=network['unknowns'],BIN26_unknowns=docs[2]['wire']['unknowns'],version_findings=network['versions'],test_inputs=pin(L/'r524-test-inputs.json'),Java_SQL_GUI=0,old_PASS_replays=0,BIN28_BIN29_adopted=False,mode_network_failure='Supervisor reports first post-create join SERVER_RESTRICTED: source restriction excludes fixture source72; no accepted mode-step dynamic PASS and no transfer of success. Kept pending, no new mode/ready/start writer.'))
print('R524 adopted',len(refs),'unique references;',len(occ),'pin occurrences;64 saved485F payloads;AD368 immutable;0JavaSQL')
