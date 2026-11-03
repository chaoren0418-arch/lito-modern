"""Compare actual selected/world S1E books with native request arms; no new client sessions."""
from pathlib import Path
import json,re,hashlib,zipfile
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-shared-healing-learned-gap';assert not OUT.exists();OUT.mkdir()
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
case=ROOT/'驗證紀錄/current/native-real-peers/20261015-164354-580';receipt=case/'receipt.json';d=json.loads(receipt.read_text(encoding='utf-8-sig'));cap=next((Path(d['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];books=[]
for i,row in enumerate(frames):
 if row['direction']!='server_to_client' or not re.fullmatch('[0-9A-Fa-f ]*',row.get('plain','')):continue
 b=bytes.fromhex(row['plain'])
 if not b or b[0]!=0x1e:continue
 assert len(b)>=2 and b[1]<=28 and len(b)==b[1]+2 and (b[1]<28 or b[-1]&0xf0==0)
 ids=sorted(j*8+k+1 for j,v in enumerate(b[2:]) for k in range(8) if v&(1<<k));books.append({'capture_row':i,'note':row['note'],'payload_hex':b.hex(),'learned_ids':ids})
p=ROOT/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=p.read_text(encoding='utf-8');request=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];supported=set()
for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',request,re.M):supported.update(map(int,re.findall(r'\d+',m.group(1))))
assert supported=={1,3,4,6,7,8,9,10,11,15,16,17,18,28,131};ground={58,63};roles={}
for role in ['a','b']:
 learned=d['clients'][role]['action_probe']['endpoint']['world']['learned_skills'];matches=[b for b in books if b['learned_ids']==learned]
 assert len(matches)==2 and sum(x['note']=='S_ADD_SPELL selected-character' for x in matches)==1 and sum(x['note']==f'S_ADD_SPELL learned-count={len(learned)}' for x in matches)==1
 roles[role]={'learned_ids':learned,'wire_books':matches,'native_request_supported_learned_ids':sorted(set(learned)&(supported|ground)),'learned_without_native_request':sorted(set(learned)-(supported|ground))}
assert all(i in roles['b']['learned_ids'] and i in roles['b']['learned_without_native_request'] for i in [19,35,57]);assert 35 in roles['a']['learned_without_native_request']
proof={'status':'confirmed_actual_TW_wire_learned_higher_heals_missing_native_requests','scope':'Reuse completed C02/TW actual basic-heal4-session capture,0 new sessions. No higher-heal cast executed.','resource_profile':d['resource_profile'],'extended_item_details':d['extended_item_details'],'main_sha256':d['exe_sha256'],'jar_sha256':d['jar_sha256'],'receipt_sha256':sha(receipt),'capture_sha256':sha(cap),'native_request_source_sha256':sha(p),'native_request_ids':sorted(supported),'native_ground_request_ids':sorted(ground),'roles':roles,'skill19_35_57_learned_but_native_request_unsupported':True,'new_sessions_executed':0,'higher_heal_runtime_verified':False,'all_learned_skills_native_supported':False,'whole_goal_complete':False}
(OUT/'gap.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(OUT/'first-audit-assumption.json').write_text(json.dumps({'observation':'An initial inline audit expected one matching S1E per role and rejected the actual two copies.','actual':'One selected-character book and one world learned-count book per role, exact bitsets match.','product_failure_claimed':False,'initial_inline_stdout_not_archived':True},indent=2)+'\n',encoding='utf-8')
files={'auditor.py':Path(__file__),'gap.json':OUT/'gap.json','first-audit-assumption.json':OUT/'first-audit-assumption.json','receipt.json':receipt,'frames.jsonl':cap,'native_skills.rs':p,'native_skill_selection.rs':ROOT/'LiTo381-Modern/src/modern_lito/native_skill_selection.rs'};fp={n:sha(f) for n,f in files.items()};a=OUT/'learned-gap-evidence.zip'
with zipfile.ZipFile(a,'x',zipfile.ZIP_DEFLATED) as z:
 for n,f in files.items():z.write(f,n)
with zipfile.ZipFile(a) as z:
 for n,h in fp.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
(OUT/'qualified.json').write_text(json.dumps({'status':proof['status'],'file_pins':fp,'archive_sha256':sha(a),'archive_readback_verified':True,'higher_heal_runtime_verified':False,'whole_goal_complete':False},indent=2)+'\n',encoding='utf-8');print(json.dumps({'roles':{role:{'learned':len(x['learned_ids']),'missing_native_request':len(x['learned_without_native_request'])} for role,x in roles.items()},'19_35_57_real_learned_gap_confirmed':True,'new_sessions':0}))
