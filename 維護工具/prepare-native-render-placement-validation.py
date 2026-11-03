"""Rebind existing normal/library orchestration to the one-file repair."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source=='B0628AC904FE5AB7C2227FF176811A4A6208EAC7B5BA8892C73D0AFD99F2F2C4'
names=['run-native-rental-service-entry-return-normal.py','qualify-native-rental-service-entry-return-normal.py','checkpoint-native-rental-service-entry-return-normal.py',
    'run-native-rental-service-entry-return-library-default.py','qualify-native-rental-service-entry-return-library-default.py',
    'observe-native-rental-service-entry-return-library-default.py','checkpoint-native-rental-service-entry-return-library-active.py','checkpoint-native-rental-service-entry-return-library-completed.py']
out=V/'native-render-placement-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
records={}
for name in names:
    s=(T/name).read_text('utf-8-sig')
    if name.startswith('qualify-')and name.endswith('-normal.py'):
        start=s.index("candidate=read(application['candidate']);test=")
        end=s.index("verify_archive(app/'applied-payload.zip'",start)
        s=s[:start]+'''candidate=read(application['candidate'])
assert sha(application['candidate'])==application['candidate_sha256']
assert candidate['base_source_manifest_sha256']==application['before_source_manifest_sha256']=='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
old=read(app/'before-compiled-inputs.json');assert set(old)==set(inputs)
changed={n for n,h in inputs.items()if old[n]!=h}
assert changed==set(application['changed_files'])==set(candidate['candidate_files'])=={'LiTo381-Modern/src/modern_lito/world_scene.rs'}
assert application['created_files']==[] and application['after_files']==candidate['candidate_files']
assert len(old)==len(inputs)==1049
'''+s[end:]
    s=s.replace('rental-service-entry-return','render-placement').replace('rental_service_entry_return','render_placement')
    # The normal reader deliberately retains the before-source identity in its application proof.
    if not(name.startswith('qualify-')and name.endswith('-normal.py')):
        s=s.replace('BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B',source)
    s=s.replace('來源BC2CA7E9','來源'+source[:8]).replace('租裝入口／旅館返回修正版','繪製回傳排序／可見性標號修正版')
    s=s.replace('A9FE原完整library','BC2CA原完整library').replace('A9FE成果與4F2F來源到期失敗','BC2CA資源65過7敗與A9FE／4F2F歷史成果及失敗')
    ast.parse(s)
    target=T/name.replace('rental-service-entry-return','render-placement');assert not target.exists()
    (out/name).write_bytes((T/name).read_bytes());target.write_text(s,encoding='utf-8',newline='\n')
    records[target.name]=dict(base=name,base_sha256=sha(T/name),prepared_sha256=sha(target),parsed=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes())
(out/'tooling.json').write_text(json.dumps(dict(source_manifest_sha256=source,tools=records,compiler_or_runtime_started=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(records),executed=False),ensure_ascii=False))
