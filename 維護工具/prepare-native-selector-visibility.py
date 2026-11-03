"""Prepare visibility helpers without modifying frozen source or old QA."""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json, re, zipfile
ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT/'維護工具'
APP = ROOT/'驗證紀錄/current/native-selector-integration/application-20261017-123033-357908'
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()
pairs = [
 ('prepare-native-visibility-current-normal-inputs.py','prepare-native-selector-visibility-inputs.py'),
 ('run-native-visibility-Debug-cases.py','run-native-selector-visibility-cases.py'),
 ('qualify-native-visibility-Debug-cases.py','qualify-native-selector-visibility-cases.py'),
 ('qualify-native-current-normal-visibility-pair.py','qualify-native-selector-visibility-pair.py'),
 ('verify-real-peer-visibility.py','verify-selector-peer-visibility.py'),
 ('verify-native-real-peers.ps1','verify-native-selector-peers.ps1'),
]
replacements = {
 'native-default110':'native-default111',
 'independently_qualified_current_default110':'independently_qualified_current_default111',
 'presentation-related':'selector-related',
 '驗證紀錄/current/native-presentation-source-owner':'驗證紀錄/current/native-selector-integration',
 'independently_qualified_presentation_source_owner_both_profiles':'independently_qualified_current_normal_selector_both_profiles',
 'verify-real-peer-visibility.py':'verify-selector-peer-visibility.py',
 'verify-native-real-peers.ps1':'verify-native-selector-peers.ps1',
 '維護工具/verify-native-real-peers.ps1':'維護工具/verify-native-selector-peers.ps1',
}
numbers = {1024:1028,1025:1029,111:112,26:64}
def transform_string(s):
    for old,new in replacements.items(): s=s.replace(old,new)
    return s
class Normalize(ast.NodeTransformer):
    def visit_Constant(self,node):
        if isinstance(node.value,str): node.value=transform_string(node.value)
        elif type(node.value) is int: node.value=numbers.get(node.value,node.value)
        return node
out=APP/('visibility-preparation-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
rows=[]
for original,derived in pairs:
    before=(BASE/original).read_text(encoding='utf-8-sig')
    after=transform_string(before)
    if original.endswith('.py'):
        for old,new in numbers.items(): after=re.sub(r'\b'+str(old)+r'\b',str(new),after)
        assert ast.dump(Normalize().visit(ast.parse(before)),include_attributes=False)==ast.dump(ast.parse(after),include_attributes=False),derived
    else:
        assert after==before.replace('verify-real-peer-visibility.py','verify-selector-peer-visibility.py')
    p=BASE/derived; assert not p.exists(),p
    p.write_text(after,encoding='utf-8',newline='\n')
    (out/original).write_bytes((BASE/original).read_bytes())
    (out/derived).write_bytes(p.read_bytes())
    rows.append(dict(original=original,original_sha256=sha(BASE/original),derived=derived,derived_sha256=sha(p),
        AST_metadata_or_exact_PS_validator_path_equivalence=True))
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for p in out.iterdir():
        if p.suffix in {'.py','.ps1'}: z.write(p,p.name)
with zipfile.ZipFile(out/'raw.zip') as z:
    for n in z.namelist(): assert hashlib.sha256(z.read(n)).hexdigest().upper()==sha(out/n)
record=dict(status='prepared_selector_visibility_not_executed',helpers=rows,source_files=1028,compiled_input_files=1029,
    broad_images_per_profile=112,upstream_normal_recorded_identities=64,original_runtime_assertions_preserved=True,
    original_helpers_unchanged=True,actual_sessions=0,actual_casts=0,GUI_verified=False,whole_goal_complete=False,
    preparation=str(out),raw_archive_sha256=sha(out/'raw.zip'))
p=APP/'visibility-preparation.json';assert not p.exists()
p.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(preparation=str(out),helpers=len(rows),runtime_executed=False),ensure_ascii=False))
