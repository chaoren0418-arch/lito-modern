"""Current source idle gate, independent of historical process paths."""
from pathlib import Path
import hashlib,json
SOURCE='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
def require_idle_runtime(root):
    w=json.loads((Path(root)/'工作佇列.json').read_text('utf-8-sig'))
    assert w['current_product_identity']['source_manifest_sha256']==SOURCE
    assert not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen']
    assert w['current_full_default_library_verified'] and w['current_native_catalog_library']['source_manifest_sha256']==SOURCE
    library=w['current_native_catalog_library'];p=Path(library['completion'])
    with p.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==library['completion_sha256']
    c=json.loads(p.read_text('utf-8-sig'));assert c['recorded_original_owned_processes_all_terminal'] and c['observed_terminal_exit_code']==0
    return 'current_library_terminal_and_no_reserved_workload'
