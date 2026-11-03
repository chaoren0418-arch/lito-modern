"""Current source request recognition against pinned actual original C02 books.

No new cast, session, rendering, source edit or playback acceptance is inferred.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=ROOT/'驗證紀錄/current/native-scoped-catalog-normal/validation-20261018-132210-585372'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
qualified=read(next(normal.glob('Debug-qualified-*/qualified.json')));source=qualified['source_manifest_sha256']
assert qualified['status']=='independently_qualified_scoped_catalog_normal_seventeen_targets_and_selected_library' and qualified['passed']==156

assert source=='33199F4ED6584FD4D19FBF1EC69C3EC69AB6E62B27ABA64CC59FE0AD2C986ED4'
run=ROOT/'驗證紀錄/current/native77-live/20261017-200632-752686';qp=run/'qualified-20261017-200726-409085/qualified.json';q=read(qp)
assert q['source_manifest_sha256']=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182' and q['original_exit_code']==0 and q['recorded_owned_all_terminal']
runtime=Path(read(run/'receipt.json')['runtime']);raw=qp.parent/'raw.zip';assert sha(raw)==q['raw_archive_sha256']
with zipfile.ZipFile(raw) as z:
    for n,h in q['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
contract=ROOT/'驗證紀錄/current/native-current-skill-contract-audit';cq=read(contract/'qualified.json')
formal=PROJECT/'target/current/server-reference/formal-server-C02AAAE8.jar'
assert sha(formal)==cq['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(contract/'current-skill-contract-evidence.zip')==cq['archive_sha256']
with zipfile.ZipFile(contract/'current-skill-contract-evidence.zip') as z:
    for n,h in cq['file_pins'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
factories={x['skill_id']:x for x in read(contract/'current-learned-factories.json')['rows']}
src=PROJECT/'src/modern_lito/native_skills.rs';text=src.read_text(encoding='utf-8-sig')
request=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];recognized=set()
for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',request,re.M):recognized.update(map(int,re.findall(r'\d+',match.group(1))))
ground=text[text.index('pub fn ground_request('):text.index('pub fn in_native_screen(')]
match=re.search(r'ensure!\(matches!\(skill,\s*([\d |]+)\)',ground);assert match
ground_ids=set(map(int,re.findall(r'\d+',match.group(1))));recognized|=ground_ids
captures=[n for n in q['raw_files'] if n.endswith('frames.jsonl')];assert len(captures)==1
cap=runtime/captures[0];assert sha(cap)==q['raw_files'][captures[0]]
books=[]
for line_number,line in enumerate(cap.read_text(encoding='utf-8-sig').splitlines(),1):
    row=json.loads(line)
    if row['direction']!='server_to_client' or row['opcode']!='0x1E':continue
    b=bytes.fromhex(row['plain']);assert len(b)==b[1]+2 and b[1]<=28
    ids=sorted(j*8+k+1 for j,v in enumerate(b[2:]) for k in range(8) if v&(1<<k))
    books.append(dict(line=line_number,note=row['note'],payload_hex=b.hex(),ids=ids))
roles={};learned=set()
for role in ['a','b']:
    report=runtime/(role+'-client.json');assert sha(report)==q['raw_files'][report.name]
    ids=read(report)['initial']['state']['world']['learned_skills'];matching=[b for b in books if b['ids']==ids]
    assert any(b['note']=='S_ADD_SPELL selected-character' for b in matching)
    learned.update(ids);roles[role]=dict(learned_ids=ids,matching_actual_wire=matching)
assert len(learned)==74 and learned==set(factories)
assert len(learned&recognized)==35
missing=learned-recognized;assert all(i in missing for i in [45,46]) and 38 not in missing and 34 not in missing and 77 not in missing
out=ROOT/'驗證紀錄/current/native1040-learned-request-audit'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
rows=[dict(skill_id=i,recognized_request=i in recognized,ground_request=i in ground_ids,retained_C02_definitions=factories[i]['matches'],new_runtime_qualification=False) for i in sorted(learned)]
result=dict(status='current1040_request_recognition_and_original_C02_wire_books_audited_only',source_manifest_sha256=source,native_skills_sha256=sha(src),retained_book_wire_source_manifest_sha256=q['source_manifest_sha256'],retained_book_runtime_qualification=str(qp),runtime_qualification_sha256=sha(qp),actual_book_source_scope='retained E159/C02 original class/book wire evidence only; current33199 source independently scanned; current75DA skill runtime is not inferred; no retained casts or rendering promoted to current source',capture=str(cap),capture_sha256=sha(cap),factory_qualification=str(contract/'qualified.json'),factory_qualification_sha256=sha(contract/'qualified.json'),observed_learned_ids=len(learned),recognized_learned_requests=len(learned&recognized),missing_learned_requests=len(missing),missing_ids=sorted(missing),roles=roles,rows=rows,current_cold_normal_runtime_qualification_pending=True,next_attack_candidates=[45,46],next_attack_candidate_gate='Full native canonical presentation, actual C02 positive/refused casts, request and target lifetime guards, MP and cooldown, native sound delivery and peer visibility must be implemented/verified; request recognition alone is insufficient.',new_sessions=0,new_casts=0,new_renders=0,request_recognition_is_not_gameplay_acceptance=True,all_learned_skills_native_supported=False,product_source_modified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'audit.json',result);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());save(out/'pins.json',dict(audit_sha256=sha(out/'audit.json'),auditor_sha256=sha(out/'auditor.py')))
print(json.dumps(dict(audit=str(out/'audit.json'),learned=74,recognized=len(learned&recognized),missing=len(missing),next_candidates=[45,46],new_casts=0),ensure_ascii=False))
