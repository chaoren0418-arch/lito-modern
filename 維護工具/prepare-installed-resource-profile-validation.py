"""Prepare final test-profile normal/library tools from the immutable staged payloads."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
OLD='B0628AC904FE5AB7C2227FF176811A4A6208EAC7B5BA8892C73D0AFD99F2F2C4'
C=T/'candidates/installed-resource-profiles-20261019-201101-087897';c=read(C/'candidate.json');assert c['base_source_manifest_sha256']==OLD
base=V/'native-render-placement-normal/application-20261019-200357-854061/applied-source-inputs.json';pins=read(base);assert sha(base)==OLD
for n,h in c['candidate_files'].items():assert sha(C/Path(n).name)==h;pins[n]=h
source=hashlib.sha256((json.dumps(pins,ensure_ascii=False,indent=2)+'\n').encode('utf-8')).hexdigest().upper()
names=['run-native-render-placement-normal.py','qualify-native-render-placement-normal.py','checkpoint-native-render-placement-normal.py',
    'run-native-render-placement-library-default.py','qualify-native-render-placement-library-default.py',
    'observe-native-render-placement-library-default.py','checkpoint-native-render-placement-library-active.py','checkpoint-native-render-placement-library-completed.py']
out=V/'native-installed-resource-profile-tooling'/('validation-preparation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);records={}
for name in names:
    s=(T/name).read_text('utf-8-sig')
    if name.startswith('qualify-')and name.endswith('-normal.py'):
        start=s.index("candidate=read(application['candidate'])");end=s.index("verify_archive(app/'applied-payload.zip'",start)
        s=s[:start]+'''candidate=read(application['candidate'])
assert sha(application['candidate'])==application['candidate_sha256']
assert candidate['base_source_manifest_sha256']==application['before_source_manifest_sha256']=='B0628AC904FE5AB7C2227FF176811A4A6208EAC7B5BA8892C73D0AFD99F2F2C4'
assert sha(candidate['independent_Text_oracle'])==candidate['independent_Text_oracle_sha256']
assert candidate['product_runtime_code_changed']is False and len(candidate['new_ignored_tests'])==4
old=read(app/'before-compiled-inputs.json');assert set(old)==set(inputs)
changed={n for n,h in inputs.items()if old[n]!=h}
assert changed==set(application['changed_files'])==set(candidate['candidate_files'])and len(changed)==5
assert application['created_files']==[] and application['after_files']==candidate['candidate_files']
assert len(old)==len(inputs)==1049
'''+s[end:]
    else:s=s.replace(OLD,source)
    s=s.replace('render-placement','installed-resource-profile').replace('render_placement','installed_resource_profile')
    s=s.replace('來源B0628AC9','來源'+source[:8]).replace('繪製回傳排序／可見性標號修正版','原資源參考／現行TW資格修正版')
    ast.parse(s)
    target=T/name.replace('render-placement','installed-resource-profile');assert not target.exists()
    (out/name).write_bytes((T/name).read_bytes());target.write_text(s,encoding='utf-8',newline='\n')
    records[target.name]=dict(base=name,base_sha256=sha(T/name),prepared_sha256=sha(target),parsed=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes());(out/'predicted-source-inputs.json').write_text(json.dumps(pins,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'tooling.json').write_text(json.dumps(dict(base_source_manifest_sha256=OLD,predicted_source_manifest_sha256=source,candidate=str(C/'candidate.json'),candidate_sha256=sha(C/'candidate.json'),tools=records,applied=False,compiler_or_runtime_started=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),predicted_source_manifest_sha256=source,tools=len(records),applied=False),ensure_ascii=False))
