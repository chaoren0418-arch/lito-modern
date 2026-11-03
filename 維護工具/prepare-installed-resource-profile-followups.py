"""Rebind untouched API/GUI/controller fixtures to the applied final source."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
SOURCE='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
assert read(R/'工作佇列.json')['current_product_identity']['source_manifest_sha256']==SOURCE
names=['run-native-rental-service-entry-return-library-resource-optin.py','checkpoint-native-rental-service-entry-return-library-resource-optin.py',
    'prepare-native-rental-service-entry-return-sky-gui.py','start-native-rental-service-entry-return-sky-gui-case.py','checkpoint-native-rental-service-entry-return-sky-gui.py',
    'qualify-native-rental-service-entry-return-GUI-rental-entry.py','qualify-native-rental-service-entry-return-GUI-hotel-entry.py','qualify-native-rental-service-entry-return-GUI-hotel-return.py',
    'run-native-rental-service-entry-return-controller.py','qualify-native-rental-service-entry-return-controller-case.py','qualify-native-rental-service-entry-return-controller-cohort.py',
    'observe-native-rental-service-entry-return-controller-completion.py','checkpoint-native-rental-service-entry-return-controller-active.py','checkpoint-native-rental-service-entry-return-controller-complete.py',
    'run-native-rental-entry-guard.py','qualify-native-rental-entry-guard.py','checkpoint-native-rental-entry-guard.py',
    'compile-native-rental-service-same-endpoint.py','run-native-rental-service-same-endpoint.py','qualify-native-rental-service-same-endpoint.py','checkpoint-native-rental-service-same-endpoint.py',
    'prepare-native-rental-service-nondefault-character-fixture.py']
def rename(s):
    return(s.replace('native-rental-service-entry-return','native-installed-resource-profile')
        .replace('native_rental_service_entry_return','native_installed_resource_profile')
        .replace('native-rental-entry-guard','native-installed-resource-profile-entry-guard')
        .replace('native-rental-service-same-endpoint','native-installed-resource-profile-same-endpoint')
        .replace('native-rental-service-nondefault-character-fixture','native-installed-resource-profile-nondefault-character-fixture'))
out=V/'native-installed-resource-profile-tooling'/('followups-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);records={}
for name in names:
    s=(T/name).read_text('utf-8-sig');s=rename(s)
    s=s.replace('rental_service_entry_return_normal_related','installed_resource_profile_normal_related')
    s=s.replace('BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B',SOURCE).replace('BC2CA7E9',SOURCE[:8])
    ast.parse(s)
    dest=T/rename(name);assert dest!=T/name and not dest.exists()
    (out/name).write_bytes((T/name).read_bytes());dest.write_text(s,encoding='utf-8',newline='\n')
    records[dest.name]=dict(base=name,base_sha256=sha(T/name),prepared_sha256=sha(dest),parsed=True)
(out/'generator.py').write_bytes(Path(__file__).read_bytes())
(out/'tooling.json').write_text(json.dumps(dict(source_manifest_sha256=SOURCE,tools=records,compiler_or_runtime_started=False,GUI_verified=False,whole_goal_complete=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),prepared_tools=len(records),executed=False),ensure_ascii=False))
