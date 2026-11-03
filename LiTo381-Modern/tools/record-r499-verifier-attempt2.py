from r491_common import *
import shutil
o=P/'target/r499-back-login-study/verification-attempt2';o.mkdir(exist_ok=False);shutil.copyfile(P/'tools/verify-r499-segment.py',o/'verify-r499-segment.py')
save(o/'failure.json',dict(state='VERIFIER_STATIC_CALL_STRING_ASSERT_FAILED_AFTER_SOURCE_PINS_CIPHER_DIFF_RUNTIME_PASS',line=66,expected='r.handle_source_character_back(control)',actual='s.runtime.borrow_mut().as_mut().unwrap().handle_source_character_back(control)',source=pin(o/'verify-r499-segment.py'),consumer=pin(L/'r499-consumer-exit.json'),consumer_rerun=False))
print('R499 verifierattempt2 preserved')
