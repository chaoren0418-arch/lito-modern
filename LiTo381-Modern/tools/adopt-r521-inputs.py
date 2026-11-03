from r491_common import *
import shutil
base=P.parent.parent;dest=L/'r521-inputs';dest.mkdir();refs={};occ=[];copies=[];docs=[]
roots=[(base/'lito脫殼資源/D5-mode-map-receive-contract.json','9F1B9863C7F0316A4C580BA83DCD14CA92A366418555073FB4FB1181F544ABAE'),(base/'lito脫殼資源/D5-scalar-receive-contract.json','4BBBEEB521ECF20E6B27D78F03B25F503A5766412DA9E3FBEE3DC11A7C389BAA'),(base/'lito伺服器/server-room-D5-CF-consumer-supplement.json','7774EA7546EEF33CB5240A58A36FA9FFF7A465303049C3C3E238081887279BEF'),(base/'lito伺服器/server-ordinary-room-map-selection-contract.json','3D0B5BAA46238ED5589C193FBDE22B966B945EFDEAB70D2A2C3933A62CA8E15F'),(base/'lito伺服器/server-ordinary-room-map-update-candidate.json','53AEB279CE8529F37CF3535170C68EE8E3A3AA037539390FB6A59CC9929A12D1')]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   p=Path(v['path']);a=refs.setdefault(str(p),pin(p));assert (a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),(v,a);occ.append(a)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(p,name):
 q=dest/name;q.parent.mkdir(parents=True,exist_ok=True);assert not q.exists();shutil.copyfile(p,q);copies.append(pin(q))
for p,expected in roots:
 assert pin(p)['sha256']==expected;d=read(p);docs.append(d);walk(d);copy(p,p.name)
for e in docs[0]['outputs']:copy(Path(e['path']),Path('mode-map')/Path(e['path']).name)
ranges=read(base/'lito脫殼資源/D5-mode-map-receive/ranges.json');ins=read(base/'lito脫殼資源/D5-mode-map-receive/instruction-witnesses.json');walk(ranges);walk(ins)
pe=Path(ranges['fixed_source']['path']);raw=pe.read_bytes();checks=[]
def rawwalk(v):
 if isinstance(v,dict):
  if 'raw_hex' in v and ('file_offset' in v or 'va' in v):
   n=v.get('file_offset');n=int(n,16) if isinstance(n,str) else n
   if n is None:n=int(str(v['va']),16)-0x400000
   b=bytes.fromhex(v['raw_hex']);assert raw[n:n+len(b)]==b;checks.append(dict(offset=n,bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper()))
  for x in v.values():rawwalk(x)
 elif isinstance(v,list):
  for x in v:rawwalk(x)
rawwalk(ranges);rawwalk(ins)
for i,e in enumerate(ranges['inherited_necessary_raw_slices']):copy(Path(e['source_pin']['path']),Path('raw')/f'{i:02d}-{Path(e["source_pin"]["path"]).name}')
ref=read(docs[2]['reference_index']['path']);walk(ref);methods=[]
for name in ['Protocol.litoRoomMode','Protocol.litoRoomMap']:
 e=ref['archived_method_indexes'][name];p=Path(ref['selected_reference_pins']['AD368-Protocol.javap.txt']['path']);lines=p.read_text('utf-8-sig').splitlines();lo,hi=e['archived_lines_one_based'];digest=hashlib.sha256('\n'.join(lines[lo-1:hi]).encode('utf-8')).hexdigest().upper();assert digest==e['method_UTF8_sha256'];methods.append(dict(method=name,source=pin(p),span=[lo,hi],sha256=digest,matches=True))
prior=read(L/'r520-test-inputs.json');prior['computed_mode_body']='d50201000000ff';prior['computed_map_body']='d503010000000202';prior['context']='R521 ONE new own normal-entry synthetic mode/map case; mode builder structural only/trigger unqualified. Map builder/static AD368 layout, computed raw values not actual C7 Java network evidence. R520 normal scaffold/saved CE/CD/summary/config reused only to establish private CE and cross-family baseline. No old PASS executable replay.';prior['prior_setup']=pin(L/'r520-test-inputs.json');save(L/'r521-test-inputs.json',prior)
save(L/'r521-input-adoption.json',dict(roots=[pin(p) for p,_ in roots],contracts=docs,reference_occurrences=len(occ),unique_references=list(refs.values()),copies=copies,fixed_PE=pin(pe),raw_byte_checks=checks,new_ranges=0,inherited_slices=len(ranges['inherited_necessary_raw_slices']),distinct_reused_raw_pins=len({e['source_pin']['path'] for e in ranges['inherited_necessary_raw_slices']}),instruction_assertions=len(ins['assertions']),archived_Java_builder_spans=methods,mode_trigger='UNQUALIFIED: structural/synthetic raw mode BYTE only, no production caller claim',map_versions=dict(AD368='Static legacy invalid restore first default564 can differ from current selected514; old evidence unchanged.',parent0612833B='Candidate restores hub current map514; accepted seeded units only, getter/missingroom races unknown.',candidateC7B169C0='Inherits parent restore bytes, only updateMap ordering repair for known COUNTDOWN/disabled map partial mutations; focused seeded units5, not new ordinary network/auth/SQL here.',actual_new_C7_network_adopted=False),scalar_Batch25='Complete root/conditions/unknowns archived read-only future boundary, no subtype4/6..9 behavior or semantic/control added.',Java_SQL_GUI=0,old_PASS_replays=0,test_inputs=pin(L/'r521-test-inputs.json')))
print('R521 adopted',len(refs),'unique refs;',len(checks),'fixed PE raw comparisons;2 archived Java builder spans; no dynamic mode/map claim')
