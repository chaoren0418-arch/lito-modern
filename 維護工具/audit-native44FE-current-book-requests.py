"""Bind actual 75DA books to current request recognition; no cast or completion claims."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current';P=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
source='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
reference='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
current=P/'target/current/server-reference/formal-server-75DA69B0.jar';previous=P/'target/current/server-reference/formal-server-C02AAAE8.jar'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(current)==sha(formal)==reference
assert sha(previous)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
normal=V/'native-model-generation-normal/validation-20261018-160708-195207'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
contract=V/'native-current-skill-contract-audit';catalog=contract/'current-learned-factories.json';cq=read(contract/'qualified.json')
assert cq['archive_readback_verified'] and sha(catalog)==cq['file_pins']['current-learned-factories.json']
assert sha(contract/'current-skill-contract-evidence.zip')==cq['archive_sha256']
with zipfile.ZipFile(contract/'current-skill-contract-evidence.zip')as z:
    for n,h in cq['file_pins'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
families={'BattleSpellCatalog','ClientSession','Protocol','MemoryAccountRepository'};bridge={}
with zipfile.ZipFile(previous)as a,zipfile.ZipFile(current)as b:
    def wanted(n):return n.endswith('.class')and n.rsplit('/',1)[-1].split('$')[0].removesuffix('.class')in families
    names={n for n in a.namelist()if wanted(n)};assert names=={n for n in b.namelist()if wanted(n)}
    assert all(any(n.endswith('/'+family+'.class')for n in names)for family in families)
    for n in sorted(names):
        raw=a.read(n);assert raw==b.read(n),n;bridge[n]=hashlib.sha256(raw).hexdigest().upper()
src=P/'src/modern_lito/native_skills.rs';text=src.read_text('utf-8-sig');part=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')]
recognized=set()
for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',part,re.M):recognized.update(map(int,re.findall(r'\d+',m.group(1))))
ground=text[text.index('pub fn ground_request('):text.index('pub fn in_native_screen(')]
m=re.search(r'ensure!\(matches!\(skill,\s*([\d |]+)\)',ground);assert m
ground_ids=set(map(int,re.findall(r'\d+',m.group(1))));recognized|=ground_ids
paths={'Debug':V/'native44FE-world-tree/run-20261018-173508-298197/qualified-20261018-173641-037877/qualified.json',
       'Release':V/'native44FE-world-tree/run-20261018-173810-370193/qualified-20261018-173924-489690/qualified.json'}
files={'native_skills.rs':src,'retained-C02-factories.json':catalog};profiles={};union=set()
for profile,qp in paths.items():
    q=read(qp);assert q['source_manifest_sha256']==source and q['reference_jar_sha256']==reference and q['actual_sessions']==4 and q['recorded_owned_all_terminal']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    run=Path(q['run']);r=read(run/'receipt.json');runtime=Path(r['runtime']);assert not r['source_overlays_used'] and not r['diagnostic_API_extension']
    cap=list((runtime/'source/captures').glob('*/frames.jsonl'));assert len(cap)==1
    frames=[json.loads(x)for x in cap[0].read_text('utf-8-sig').splitlines()];books=[]
    assert q['raw_pins'][str(cap[0])]==sha(cap[0])
    for line,row in enumerate(frames,1):
        if row['direction']!='server_to_client'or row['opcode']!='0x1E':continue
        b=bytes.fromhex(row['plain']);assert len(b)==b[1]+2 and b[1]<=28
        ids=sorted(i*8+j+1 for i,v in enumerate(b[2:])for j in range(8)if v&(1<<j));books.append(dict(line=line,payload_hex=b.hex(),ids=ids))
    roles={}
    for role in ['a','b']:
        p=runtime/(role+'-result.json');j=read(p);assert q['raw_pins'][str(p)]==sha(p)
        ids=j['before']['world']['learned_skills'];matches=[row for row in books if row['ids']==ids];assert matches
        union.update(ids);roles[role]=dict(character=j['before']['selected_character'],learned_ids=ids,matching_actual_book_rows=matches,
            recognized_ids=sorted(set(ids)&recognized),missing_ids=sorted(set(ids)-recognized))
        files[profile+'/'+role+'-result.json']=p
    files[profile+'/qualification.json']=qp;files[profile+'/frames.jsonl']=cap[0]
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),actual_sessions=4,roles=roles)
assert profiles['Debug']['roles']==profiles['Release']['roles']or all(profiles['Debug']['roles'][role]['learned_ids']==profiles['Release']['roles'][role]['learned_ids']for role in ['a','b'])
assert len(union)==74 and len(union&recognized)==35 and len(union-recognized)==39
factories={x['skill_id']:x for x in read(catalog)['rows']};assert union<=factories.keys()
out=V/'native44FE-skill-book-audit'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
save(out/'catalogue-byte-bridge.json',dict(previous_sha256=sha(previous),actual_sha256=reference,byte_identical_classes=bridge,whole_protocol_compatibility_claimed=False))
files['reader.py']=Path(__file__);files['catalogue-byte-bridge.json']=out/'catalogue-byte-bridge.json'
matrix=[dict(skill_id=i,current_request_recognized=i in recognized,ground_request=i in ground_ids,definitions=factories[i]['matches'],factory_errors=factories[i]['factory_errors'])for i in sorted(union)]
record=dict(status='current_actual_books_and_request_recognition_audited_no_new_casts',source_manifest_sha256=source,reference_jar_sha256=reference,profiles=profiles,
    observed_unique_learned_ids=74,recognized_request_ids=sorted(union&recognized),missing_request_ids=sorted(union-recognized),recognized_count=35,missing_count=39,
    initial_repository_fixture='The WORLD fixture initially adds131 to both memory books; it changes no other learned IDs. These are isolated reference books, not SQL/player-table coverage.',
    current_catalogue_definition_scope='Retained C02 factory rows are reused only after comparing all BattleSpellCatalog/ClientSession/Protocol/MemoryAccountRepository classes byte-for-byte against75DA.',
    matrix=matrix,actual_new_sessions=0,actual_new_casts=0,current_recognition_is_not_successful_cast_count=True,
    UI_verified=False,physical_audio_verified=False,complete_gameplay_matrix_verified=False,whole_goal_complete=False)
save(out/'matrix.json',record);files['matrix.json']=out/'matrix.json';pins={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:
    for n,h in pins.items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
save(out/'readback.json',dict(raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),matrix_sha256=sha(out/'matrix.json'),source_unchanged=sha(src)==inputs[src.relative_to(ROOT).as_posix()],scope='Static request recognition using current qualified captured books; not gameplay completion.'))
print(json.dumps(dict(audit=str(out),actual_observed_unique_ids=74,request_recognized=35,missing=39,byte_identical_class_count=len(bridge),new_casts=0),ensure_ascii=False))
