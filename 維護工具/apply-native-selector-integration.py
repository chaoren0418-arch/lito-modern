"""Apply the qualified 20-file selector candidate and exact compatible regression sources."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def safe(p):
 p=p.absolute();assert p.is_relative_to(ROOT)
 for x in [p,*p.parents]:
  if x==ROOT.parent:break
  if x.exists():assert not x.is_symlink() and not x.is_junction(),str(x)
 return p
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('active_workload_sessions')
e=queue['current_selector_eight_cohort_qualification'];qp=Path(e['qualification']);assert sha(qp)==e['qualification_sha256'];q=read(qp)
assert q['status']=='independently_combined_current_candidate_eight_C02_scenario_cohorts' and q['all_recorded_processes_terminal'] and q['all_recorded_exit_codes_zero'] and not q['applied'] and not q['whole_goal_complete']
d=Path(q['candidate']);assert d.resolve().is_relative_to(ROOT/'維護工具/candidates');m=read(d/'candidate.json');assert sha(d/'candidate.json')==q['candidate_metadata_sha256'] and len(m['after'])==20 and not m['applied']
for row in q['cohorts'].values():assert sha(row['qualification'])==row['qualification_sha256'] and read(row['qualification'])['candidate_metadata_sha256']==q['candidate_metadata_sha256']
base=Path(m['base_related_run']);before=read(base/'source-inputs.json');assert len(before)==1024 and sha(base/'source-inputs.json')==m['base_source_manifest_sha256']==q['source_manifest_sha256']
for n,h in before.items():assert sha(ROOT/n)==h,n
payload={}
for n,h in m['after'].items():
 p=safe(ROOT/n);src=d/'sources'/Path(n).name;assert sha(src)==h
 if n in m['before']:assert sha(p)==m['before'][n]
 else:assert not p.exists()
 payload[n]=src.read_bytes()
compat=read(d/'qa-compatibility/preparation.json');assert compat['constructor_updates']==15 and compat['updated_files']==9 and not compat['applied'];reconstructions={}
for item in compat['files']:
 p=safe(Path(item['source']));src=Path(item['candidate']);assert sha(p)==item['source_sha256'] and sha(src)==item['candidate_sha256']
 old=p.read_bytes().decode('utf-8-sig');new=src.read_bytes().decode('utf-8-sig');matches=[]
 for label,text in [('raw-newlines',old),('universal-newlines',old.replace('\r\n','\n').replace('\r','\n'))]:
  for v in sorted(item['insertions'],key=lambda x:x['character_offset'],reverse=True):text=text[:v['character_offset']]+v['text']+text[v['character_offset']:]
  if text==new:matches.append(label)
 assert matches,p
 assertion_lines=lambda s:[v.strip() for v in s.splitlines() if re.match(r'\s*(?:assert|assert_eq|assert_ne|ensure)!',v)]
 assert assertion_lines(old)==assertion_lines(new),p
 n=p.relative_to(ROOT).as_posix();assert n in before and n not in payload;payload[n]=src.read_bytes();reconstructions[n]=matches
prep=read(d/'qa-compatibility/selector-regression-preparation.json');assert prep['tests']==4 and not prep['compiled'] and not prep['applied']
for kind in ['test','fixture']:
 p=safe(Path(prep[kind+'_destination']));src=Path(prep[kind+'_candidate']);assert not p.exists() and sha(src)==prep[kind+'_candidate_sha256'];payload[p.relative_to(ROOT).as_posix()]=src.read_bytes()
assert len(payload)==31
old={n:sha(ROOT/n) for n in payload if (ROOT/n).exists()};added=[n for n in payload if n not in old];assert len(old)==27 and len(added)==4
for profile,info in q['normal_products'].items():assert sha(info['path'])==info['sha256']
reference=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';assert sha(reference)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
app=ROOT/'驗證紀錄/current/native-selector-integration'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));app.mkdir(parents=True);(app/'apply-tool.py').write_bytes(Path(__file__).read_bytes());save(app/'base-source-inputs.json',before)
with zipfile.ZipFile(app/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n in old:z.write(ROOT/n,n)
with zipfile.ZipFile(app/'before.zip') as z:
 assert set(z.namelist())==set(old)
 for n,h in old.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
new_hashes={n:hashlib.sha256(b).hexdigest().upper() for n,b in payload.items()}
with zipfile.ZipFile(app/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,b in payload.items():z.writestr(n,b)
with zipfile.ZipFile(app/'proposal.zip') as z:
 for n,h in new_hashes.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
state=dict(status='prepared_selector_integration_application',candidate=str(d),candidate_metadata_sha256=q['candidate_metadata_sha256'],qualification=str(qp),qualification_sha256=sha(qp),base_source_manifest_sha256=q['source_manifest_sha256'],before=old,added_files=added,after=new_hashes,before_archive_sha256=sha(app/'before.zip'),proposal_archive_sha256=sha(app/'proposal.zip'),test_constructor_reconstructions=reconstructions,existing_test_assertions_preserved=True,new_registered_tests=4,normal_products_before=q['normal_products'],reference_jar_sha256=sha(reference),applied=False,normal_product_validation_complete=False,GUI_verified=False,whole_goal_complete=False)
save(app/'application.json',state)
for n,h in before.items():assert sha(ROOT/n)==h
for n,b in payload.items():safe(ROOT/n).write_bytes(b)
for n,h in new_hashes.items():assert sha(ROOT/n)==h,n
pins={n:sha(ROOT/n) for n in [*before,*sorted(added)]};assert len(pins)==1028
changed=[n for n in before if pins[n]!=before[n]];assert set(changed)==set(old)
compiled=pins|{'LiTo381-Modern/windows/app.manifest':sha(ROOT/'LiTo381-Modern/windows/app.manifest')};assert len(compiled)==1029
save(app/'applied-source-inputs.json',pins);save(app/'applied-compiled-inputs.json',compiled)
state.update(status='applied_selector_integration_pending_current_product_validation',applied=True,changed_existing_sources=changed,source_files=1028,compiled_input_files=1029,source_manifest_sha256=sha(app/'applied-source-inputs.json'),applied_utc=datetime.now(timezone.utc).isoformat())
save(app/'application.json',state);save(app.parent/'active-application.json',dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
assert sha(reference)==state['reference_jar_sha256']
print(json.dumps(dict(application=str(app),status=state['status'],changed_existing_sources=len(changed),added_files=len(added),source_files=1028,compiled_input_files=1029,source_manifest_sha256=state['source_manifest_sha256'],normal_products_rebuilt=False),ensure_ascii=False))
