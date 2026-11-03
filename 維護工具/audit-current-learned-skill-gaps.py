"""Compare actual selected/world S1E books with native request arms; no new client sessions."""
from pathlib import Path
import json,re,hashlib,zipfile
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-current-learned-skill-gaps';assert not OUT.exists();OUT.mkdir()
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
live_path=ROOT/'驗證紀錄/current/native-shared-higher-heals/live/20261015-193452-972/qualified.json';live=json.loads(live_path.read_text(encoding='utf-8-sig'));assert live['actual_client_sessions']==60 and live['archive_readback_verified']
chosen=next(c for c in live['cases'] if c['profile']=='Release' and c['resource_profile']=='tw13081901-v1' and c['action_profile']=='heal57');case=Path(chosen['case']);receipt=case/'receipt.json';assert sha(receipt)==chosen['receipt_sha256'];d=json.loads(receipt.read_text(encoding='utf-8-sig'));cap=next((Path(d['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];books=[]
for i,row in enumerate(frames):
 if row['direction']!='server_to_client' or not re.fullmatch('[0-9A-Fa-f ]*',row.get('plain','')):continue
 b=bytes.fromhex(row['plain'])
 if not b or b[0]!=0x1e:continue
 assert len(b)>=2 and b[1]<=28 and len(b)==b[1]+2 and (b[1]<28 or b[-1]&0xf0==0)
 ids=sorted(j*8+k+1 for j,v in enumerate(b[2:]) for k in range(8) if v&(1<<k));books.append({'capture_row':i,'note':row['note'],'payload_hex':b.hex(),'learned_ids':ids})
p=ROOT/'LiTo381-Modern/src/modern_lito/native_skills.rs';text=p.read_text(encoding='utf-8');request=text[text.index('pub fn request('):text.index('pub fn reuse_millis(')];supported=set()
for m in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>',request,re.M):supported.update(map(int,re.findall(r'\d+',m.group(1))))
assert supported=={1,3,4,6,7,8,9,10,11,15,16,17,18,19,28,35,57,131};ground={58,63};roles={}
for role in ['a','b']:
 learned=d['clients'][role]['action_probe']['endpoint']['world']['learned_skills'];matches=[b for b in books if b['learned_ids']==learned]
 assert len(matches)==2 and sum(x['note']=='S_ADD_SPELL selected-character' for x in matches)==1 and sum(x['note']==f'S_ADD_SPELL learned-count={len(learned)}' for x in matches)==1
 roles[role]={'learned_ids':learned,'wire_books':matches,'native_request_supported_learned_ids':sorted(set(learned)&(supported|ground)),'learned_without_native_request':sorted(set(learned)-(supported|ground))}
assert all(i in roles['b']['learned_ids'] and i not in roles['b']['learned_without_native_request'] for i in [19,35,57]);assert 35 not in roles['a']['learned_without_native_request']
assert len(roles['a']['learned_without_native_request'])==6 and len(roles['b']['learned_without_native_request'])==52
proof={'status':'confirmed_current_actual_TW_learned_request_recognition_gaps_remaining','scope':'Reuse one current8E21/A4AB C02/TW4-session higher-heal case from qualified15-case cohort;0 new sessions. Recognition coverage only, not all recognized skills gameplay or TW presentation proof.','resource_profile':d['resource_profile'],'extended_item_details':d['extended_item_details'],'main_sha256':d['exe_sha256'],'jar_sha256':d['jar_sha256'],'receipt_sha256':sha(receipt),'capture_sha256':sha(cap),'native_request_source_sha256':sha(p),'native_request_ids':sorted(supported),'native_ground_request_ids':sorted(ground),'roles':roles,'skill19_35_57_native_request_recognized':True,'all_learned_gameplay_or_TW_rendering_verified':False,'current_higher_heal_cohort':str(live_path),'current_higher_heal_cohort_sha256':sha(live_path),'new_sessions_executed':0,'higher_heal_self_runtime_verified_by_separate60_session_cohort':True,'other_target_healing_verified':False,'all_learned_skills_native_supported':False,'whole_goal_complete':False}
(OUT/'gap.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files={'auditor.py':Path(__file__),'gap.json':OUT/'gap.json','higher-heal-qualified.json':live_path,'receipt.json':receipt,'frames.jsonl':cap,'native_skills.rs':p,'native_skill_selection.rs':ROOT/'LiTo381-Modern/src/modern_lito/native_skill_selection.rs'};fp={n:sha(f) for n,f in files.items()};a=OUT/'learned-gap-evidence.zip'
with zipfile.ZipFile(a,'x',zipfile.ZIP_DEFLATED) as z:
 for n,f in files.items():z.write(f,n)
with zipfile.ZipFile(a) as z:
 for n,h in fp.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
(OUT/'qualified.json').write_text(json.dumps({'status':proof['status'],'file_pins':fp,'archive_sha256':sha(a),'archive_readback_verified':True,'higher_heal_self_runtime_verified_by_separate60_session_cohort':True,'other_target_healing_verified':False,'whole_goal_complete':False},indent=2)+'\n',encoding='utf-8');print(json.dumps({'roles':{role:{'learned':len(x['learned_ids']),'missing_native_request':len(x['learned_without_native_request'])} for role,x in roles.items()},'higher19_35_57_request_gap_closed':True,'new_sessions':0}))
