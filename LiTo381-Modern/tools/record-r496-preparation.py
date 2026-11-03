from r491_common import *
O=P/'target/r496-actor-study'
save(L/'r496-preparation-history.json',dict(atomic_patch_preparation_failures=1,failed_patch_reason='multiple operations targeted native_source_connection.rs in one apply_patch; atomic validation rejected before any file writes',failed_patch_files_changed=0,Rust_build_failures=0,consumer_compile_failures=0,consumer_test_failures=0))
assert not (P/'src/modern_lito/native_source_actor.rs').exists()
print('R496 atomic patch rejection preserved;zero production writes')
