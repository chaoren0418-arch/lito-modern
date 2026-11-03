from r491_common import *
checks=[];cache={}
def check_group(group,path):
 path=Path(path)
 if str(path) not in cache:cache[str(path)]=path.read_text('utf-8-sig').splitlines()
 for name,items in group.items():
  for e in items if isinstance(items,list) else [items]:
   lo,hi=e['archived_lines_one_based'];text='\n'.join(cache[str(path)][lo-1:hi]);digest=hashlib.sha256(text.encode('utf-8')).hexdigest().upper();assert digest==e['method_UTF8_sha256'],name
   if 'IL' in e:assert e['IL'].strip()==text.strip(),name
   checks.append(dict(method=name,source=pin(path),span=[lo,hi],sha256=digest,matches=True))
docs=read(L/'r518-input-adoption.json')['contracts']
for d in docs[2:]:
 e=read(d['evidence']['path']);pins=e['selected_reference_pins'];cs=pins['actual-CS-archived-IL']['path']
 if 'method_indexes' in e:check_group(e['method_indexes'],cs)
 if 'reused_emission_methods' in e:check_group(e['reused_emission_methods'],cs)
 if 'new_CS_method_indexes' in e:check_group(e['new_CS_method_indexes'],cs)
 if 'new_finite_GH_IL' in e:check_group(e['new_finite_GH_IL'],pins['actual-GameHub-archived-IL']['path'])
save(L/'r518-archived-method-span-adoption.json',dict(checks=checks,count=len(checks),scope='Existing archived IL UTF8 span equality only, no javap/Java/SQL/runtime calls; no semantic test rerun.'))
print('Matched',len(checks),'archived method spans')
