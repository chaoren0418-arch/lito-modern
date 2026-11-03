from r491_common import *
import shutil
base=P.parent.parent;dest=L/'r525-inputs';dest.mkdir();refs={};occ=[];copies=[]
roots=[(base/'lito伺服器/server-ordinary-room-mode-gate-network.json','14B6DA5301356A26D038A97A7D0D42FD0F8C2DA3D68DD7ED0A0D13EB9409945E'),(base/'lito脫殼資源/CD1-mode-step-contract.json','1BB37D61C8E25EC195EA2084941EFDC89D2E6F2760119583357A088A04C3EC8F'),(base/'lito伺服器/server-ordinary-room-mode-step-boundary.json','BC57E2930FE6DA6B46F2EF9DECE6C6A5A43423B31952384EE03CDD2039EE7DAB')]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   p=Path(v['path']);assert p.suffix!='.pyc';a=refs.setdefault(str(p),pin(p));assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),p;occ.append(a)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(p,name):
 q=dest/name;q.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,q);copies.append(pin(q))
docs=[]
for p,sha in roots:
 assert pin(p)['sha256']==sha;d=read(p);docs.append(d);walk(d);copy(p,p.name)
network,native,failed=docs;assert network['actual_JAR']['sha256']=='485F599DA9BA1A13504CB48DDCA3D53C6CAD91D7BAFDF738D104E65A05E1DFD5'
m=read(network['manifest']['path']);root=Path(network['evidence_root']);assert len(m['files'])==78
for e in m['files']:
 p=Path(e['path']);p=p if p.is_absolute() else root/p;a=pin(p);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']);refs[str(p)]=a;copy(p,Path('485F-mode')/p.relative_to(root))
for e in native['outputs']:copy(Path(e['path']),Path('BIN28')/Path(e['path']).name)
ranges=read(base/'lito脫殼資源/CD1-mode-step/ranges.json');instructions=read(base/'lito脫殼資源/CD1-mode-step/instruction-witnesses.json');slices=read(base/'lito脫殼資源/CD1-mode-step/source-IL-witnesses.json');walk(ranges);walk(instructions);walk(slices)
pe=Path(native['wire']['source_PE']['path']);raw=pe.read_bytes();checks=[]
def rawwalk(v):
 if isinstance(v,dict):
  if 'raw_hex' in v and ('file_offset' in v or 'va' in v):
   n=v.get('file_offset');n=int(n,16) if isinstance(n,str) else n
   if n is None:n=int(str(v['va']),16)-0x400000
   b=bytes.fromhex(v['raw_hex']);assert raw[n:n+len(b)]==b;checks.append(dict(offset=n,bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper()))
  for x in v.values():rawwalk(x)
 elif isinstance(v,list):
  for x in v:rawwalk(x)
rawwalk(ranges);rawwalk(instructions);slice_checks=[]
for e in slices:
 text=Path(e['source']['path']).read_text('utf-8-sig').splitlines();start,end=e['lines_one_based'];span='\n'.join(text[start-1:end]);assert span.splitlines()==e['text'].splitlines();slice_checks.append(dict(source=e['source'],lines_one_based=[start,end],text_sha256=hashlib.sha256(span.encode()).hexdigest().upper(),matches=True))
preflight=read(network['preflight_contract']['path']);walk(preflight);copy(Path(network['preflight_contract']['path']),'mode-step-preflight-contract.json')
frames=read(network['new_network']['frames']['path']);data=read(L/'r524-test-inputs.json');data['source_ascii']='GateNetMoon'
for v in data['setup_frames']:
 if v.get('epoch')=='GAME-initial' and v.get('direction')=='S' and v.get('body_hex','').startswith('cb'):v.update(body_hex=frames[19]['body_hex'],frame_bytes=frames[19]['frame_bytes']);v.pop('encrypted_body_hex',None)
mapping=[('CE',65),('host_CD',68),('bot_CD',69),('bot_D0',70),('guest_CD',71),('map514',75),('D1_before',78),('actual_C_NEXT',80),('NEXT_D1_reset',81),('mode5',82),('map510',83),('D1_before_previous',88),('actual_C_PREVIOUS',90),('PREVIOUS_D1_reset',91),('mode1',92),('map564',93),('D1_before_nonowner_probe',98),('nonowner_summary',101),('nonowner_configuration',102)]
for k in ['actual_C_DD_extended','actual_C_DD_compact','D1_extended_reset','extended_summary','extended_configuration','D1_before_compact','D1_compact_reset','compact_summary','compact_configuration','rejected_summary','rejected_configuration']:data.pop(k,None)
for key,i in mapping:data[key]=frames[i]['body_hex']
data['selected_frames']=[dict(index=i,frame=frames[i]) for i in [19]+[i for _,i in mapping]];data['prior_setup']=pin(L/'r524-test-inputs.json');data['actual_saved_frames']=network['new_network']['frames'];data['context']='R525 ONE new own synthetic normal-entry actual485F relative NEXT/PREVIOUS Sender. Saved sameHost branch: CE/CDHost/CDbot/foreignbotD0/CDGuest/map514/D1ready, then actualmodeC read before reencoded corresponding HostD1reset→D5/2→D5/3. No actualCmap/Create-options authorization: ownCreate compactoptionsNone, source admission remains server authority. Guest0/1 later separate nonowner receiptprobe notHost ACK. Earlier exactE1 pluslateMode/Map remainunsupported boundary; no JavaSQLGUI or olderPASS.'
assert data['actual_C_NEXT']=='d10201' and data['actual_C_PREVIOUS']=='d10202';save(L/'r525-test-inputs.json',data)
save(L/'r525-input-adoption.json',dict(roots=[pin(p) for p,_ in roots],full_contracts=docs,reference_occurrences=len(occ),unique_references=list(refs.values()),copies=copies,saved_485F_payload_files=78,fixed_PE=pin(pe),BIN28_raw_checks=checks,BIN28_instruction_witnesses=len(instructions),BIN28_source_IL_span_checks=slice_checks,selected_method_evidence='Bounded saved Java class/method and same-version preflight indexes adopted by fixed-file SHA as data only; no class producer/Java/disassembler rerun. Source/IL13 spans and fixedPE raw independently compared.',producer_SHA=network['actual_JAR']['sha256'],old_failure_scope='BC57 prior485F first Guestjoin SERVER_RESTRICTED due source72 excluded preserved as failed fixture. New14B6 successfulsame485F preflight/admission/options2,0,1,31 selections72 and auth two-player WAITING1→5→1 network separately accepted; oldfailure does not summarize newstate.',BIN28_historical_scope='Original Native mode_dynamic0/C7map-only/handoffgap and AD/C7failureorder text retained historical. New14B6 success485F separately pinned, not full mode/failure/wrap/teamtoggle/GUI qualification.',unknowns=network['unknowns'],BIN28_unknowns=native['conditional_boundaries'],test_inputs=pin(L/'r525-test-inputs.json'),Java_SQL_GUI=0,old_PASS_replays=0,BIN29_BIN30_team_toggle_adopted=False,R524_queue_late_wording_corrected=pin(L/'r525-r524-progress-wording-correction.json')))
print('R525 adopted',len(refs),'unique references;',len(occ),'pin occurrences;78saved485F payloads;',len(checks),'rawchecks/21instructions/13spans;0newbackend')
