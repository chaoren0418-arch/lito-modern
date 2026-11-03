"""Derive source89 resource readers with metadata-only AST equivalence."""
from pathlib import Path
from datetime import datetime, timezone
import ast
import hashlib
import json
import zipfile

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / '維護工具'
APP = ROOT / '驗證紀錄/current/native-selector-integration/application-20261017-123033-357908'

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

pairs = [
    ('run-native-current-five-resource-optins.py', 'run-native-selector-five-resource-optins.py'),
    ('qualify-native-current-five-resource-optins.py', 'qualify-native-selector-five-resource-optins.py'),
    ('qualify-native-current-ten-resource-optins.py', 'qualify-native-selector-ten-resource-optins.py'),
]
strings = {
    'independently_qualified_presentation_source_owner_both_profiles': 'independently_qualified_current_normal_selector_both_profiles',
    '驗證紀錄/current/native-presentation-source-owner': '驗證紀錄/current/native-selector-integration',
    'independently_qualified_presentation_source_owner_related_profile': 'independently_qualified_current_normal_selector_related_profile',
    'current1024-ten-bodies-': 'current1028-ten-bodies-',
    '2407': '2409',
}
numbers = {1024: 1028, 1025: 1029, 2408: 2410, 2407: 2409}

class Normalize(ast.NodeTransformer):
    def visit_Constant(self, node):
        if isinstance(node.value, str):
            node.value = strings.get(node.value, node.value)
        elif type(node.value) is int:
            node.value = numbers.get(node.value, node.value)
        return node

    def visit_FunctionDef(self, node):
        self.generic_visit(node)
        if node.name == 'verify_normal_reference':
            for item in ast.walk(node):
                if isinstance(item, ast.Compare) and ast.unparse(item.left) == "q['actual_test_executables']":
                    assert len(item.comparators) == 1 and item.comparators[0].value == 10
                    item.comparators[0].value = 29
        return node

out = APP / ('resource-optin-preparation-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
rows = []
for original, derived in pairs:
    before = (BASE / original).read_text(encoding='utf-8-sig')
    after = before
    for old, new in strings.items():
        after = after.replace(old, new)
    import re
    for old, new in numbers.items():
        after = re.sub(r'\b' + str(old) + r'\b', str(new), after)
    after = after.replace("q['actual_test_executables']==10", "q['actual_test_executables']==29")
    expected = Normalize().visit(ast.parse(before))
    assert ast.dump(expected, include_attributes=False) == ast.dump(ast.parse(after), include_attributes=False), derived
    destination = BASE / derived
    assert not destination.exists(), destination
    destination.write_text(after, encoding='utf-8', newline='\n')
    (out / original).write_bytes((BASE / original).read_bytes())
    (out / derived).write_bytes(destination.read_bytes())
    rows.append(dict(original=original, original_sha256=sha(BASE / original), derived=derived,
        derived_sha256=sha(destination), AST_equal_after_explicit_metadata_normalization=True))
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for p in out.iterdir():
        if p.suffix == '.py':
            z.write(p, p.name)
with zipfile.ZipFile(out / 'raw.zip') as z:
    for name in z.namelist():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == sha(out / name)
record = dict(status='prepared_source89_resource_optins_not_executed', helpers=rows,
    source_files=1028, compiled_input_files=1029, library_inventory=2410, library_ignored=473,
    per_selected_test_filtered=2409, normal_related_actual_test_executables=29,
    exact_original_test_names_and_assertions_preserved=True, actual_test_invocations=0,
    no_product_or_fixture_changes=True, GUI_verified=False, whole_goal_complete=False,
    raw_archive_sha256=sha(out / 'raw.zip'), preparation=str(out))
assert not (APP / 'resource-optin-preparation.json').exists()
(APP / 'resource-optin-preparation.json').write_text(json.dumps(record, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps(record, ensure_ascii=False))
