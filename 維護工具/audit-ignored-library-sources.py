"""Associate opt-in test names with source ignore reasons; ambiguous matches stay explicit."""
from pathlib import Path
import argparse,collections,hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern'
ap=argparse.ArgumentParser();ap.add_argument('build_run',type=Path);a=ap.parse_args();run=a.build_run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current');inventory=json.loads((run/'inventory-audit/inventory.json').read_text(encoding='utf-8-sig'));records=collections.defaultdict(list)
for file in (PROJECT/'src').rglob('*.rs'):
 text=file.read_text(encoding='utf-8-sig');lines=text.splitlines()
 for index,line in enumerate(lines):
  match=re.search(r'#\[ignore(?:\s*=\s*"(.*?)")?\]',line)
  if not match:continue
  window=line[match.end():]+'\n'+'\n'.join(lines[index+1:index+14]);fn=re.search(r'\bfn\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(',window)
  if fn:records[fn.group(1)].append({'source':str(file.relative_to(ROOT)).replace('\\','/'),'ignore_line':index+1,'reason':match.group(1) or None,'source_sha256':hashlib.sha256(file.read_bytes()).hexdigest().upper()})
# Resolve the audited map-rotation test macros to their literal ignore attribute.
macro_file=PROJECT/'src/modern_lito/canonical_map_rotation_tests.rs'
macro_text=macro_file.read_text(encoding='utf-8-sig')
for definition in re.finditer(r'macro_rules!\s+(\w+)\s*\{(.*?)\n\}',macro_text,re.S):
 reason=re.search(r'#\[ignore\s*=\s*"([^"]+)"\]',definition.group(2))
 if not reason:continue
 for invocation in re.finditer(r'\b'+re.escape(definition.group(1))+r'!\(\s*(\w+)',macro_text):
  records[invocation.group(1)].append({'source':str(macro_file.relative_to(ROOT)).replace('\\','/'),'ignore_line':macro_text[:definition.start(2)+reason.start()].count('\n')+1,'reason':reason.group(1),'source_sha256':hashlib.sha256(macro_file.read_bytes()).hexdigest().upper(),'macro':definition.group(1),'macro_invocation_line':macro_text[:invocation.start()].count('\n')+1})
rows=[];reasons=collections.Counter()
for name in inventory['ignored_test_names']:
 candidates=records[name.split('::')[-1]]
 if len(candidates)>1:
  module=name.split('::')[1] if '::' in name else ''
  exact=[x for x in candidates if Path(x['source']).stem==module]
  if len(exact)==1:candidates=exact
 status='unique_source_attribute' if len(candidates)==1 else 'ambiguous_source_attributes' if candidates else 'source_attribute_unmapped'
 rows.append({'test':name,'source_mapping':status,'candidates':candidates,'body_executed':False})
 if len(candidates)==1:reasons[candidates[0]['reason'] or '(no documented reason)']+=1
raw_names=re.findall(r'^(.+): test$',(run/'inventory-audit/ignored.stdout.txt').read_text(encoding='utf-8-sig'),re.M)
assert hashlib.sha256((run/'inventory-audit/ignored.stdout.txt').read_bytes()).hexdigest().upper()==inventory['raw_pins']['ignored.stdout.txt']
assert inventory['ignored_test_names']==raw_names and len(raw_names)==len(set(raw_names))==inventory['ignored_tests']
assert len(rows)==inventory['ignored_tests'] and {row['test'] for row in rows}==set(raw_names)
out={'status':'audited_all_compiled_ignored_names_with_source_attribute_candidates','tests':rows,'mapped_unique':sum(x['source_mapping']=='unique_source_attribute' for x in rows),'mapped_ambiguous':sum(x['source_mapping']=='ambiguous_source_attributes' for x in rows),'unmapped':sum(x['source_mapping']=='source_attribute_unmapped' for x in rows),'exact_reason_counts':dict(reasons),'tests_executed_by_this_audit':0,'preconditions_satisfied_not_assumed':True,'safe_to_run_automatically_not_assumed':True,'current_protocol_compatibility_not_inferred':True,'whole_goal_complete':False,'audit_source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest().upper()}
(run/'inventory-audit/ignored-source-reasons.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print({k:out[k] for k in ['status','mapped_unique','mapped_ambiguous','unmapped']})
