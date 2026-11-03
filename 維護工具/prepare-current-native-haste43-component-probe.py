"""Prepare off-tree baseline/candidate Rust probes, without invoking a compiler."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import re

R = Path(__file__).resolve().parents[1]
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
save = lambda path, value: Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
candidate = R / '維護工具/candidates/native-haste43-20261020-074819-989309'
c = read(candidate / 'candidate.json')
assert sha(candidate / 'candidate.json') == '2026A1527B23E1309D6D4237661229349BCCAA5BAE13D602576FA4E3B4B40A5F'
assert c['distinct_written_files'] == len(set(c['candidate_files'].values())) == 4
assert not c['source_applied'] and not c['actual_component_or_library_tests_executed']
assert all(sha(R / name) == digest for name, digest in c['before'].items())
assert all(sha(c['candidate_files'][name]) == digest for name, digest in c['after'].items())
runtime = [name for name in c['before'] if '/src/' in name]
tests = [name for name in c['before'] if '/tests/' in name]
assert len(runtime) == len(tests) == 2
skills = next(name for name in runtime if name.endswith('/native_skills.rs'))
selection = next(name for name in runtime if name.endswith('/native_skill_selection.rs'))
original = (R / skills).read_text('utf-8-sig')
expected = original.replace('1 | 8 | 9 | 19 | 26 | 35 | 42 | 57 =>', '1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 =>').replace('matches!(skill, 8 | 9 | 26 | 42)', 'matches!(skill, 8 | 9 | 26 | 42 | 43)')
assert expected == Path(c['candidate_files'][skills]).read_text('utf-8-sig')
original = (R / selection).read_text('utf-8-sig')
anchor = '                        42 => "#42 力量強化（選取目標／自己）".into(),'
expected = original.replace(anchor, anchor + '\n' + '                        43 => "#43 加速術（選取目標／自己）".into(),')
assert expected == Path(c['candidate_files'][selection]).read_text('utf-8-sig')
for name in tests:
    old = (R / name).read_text('utf-8-sig').splitlines()
    new = iter(Path(c['candidate_files'][name]).read_text('utf-8-sig').splitlines())
    assert all(any(line == previous for line in new) for previous in old), 'Original test line removed/reordered'
out = candidate / ('component-preparation-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'tests/fixtures').mkdir(parents=True)
(out / 'target/current/verification').mkdir(parents=True)
fixture_pins = {}
test_pins = {}
for name in tests:
    source = Path(c['candidate_files'][name])
    text = source.read_text('utf-8-sig')
    if name.endswith('/native_skills.rs'):
        before = 'use lito_modern::modern_lito::{native_skills as skills, native_world::World};'
        after = 'use crate::candidate::{native_skills as skills, native_world::World};'
    else:
        before = 'use lito_modern::modern_lito::{'
        after = 'use crate::candidate::{'
    assert text.count(before) == 1
    text = text.replace(before, after)
    target = out / 'tests' / Path(name).name
    target.write_text(text, encoding='utf-8', newline='\n')
    assert text.count('#[test]') == source.read_text('utf-8-sig').count('#[test]')
    test_pins[str(target)] = sha(target)
    for relative in re.findall(r'include_bytes!\(\s*"([^"]+)"\s*\)', text):
        fixture = (R / name).parent / relative
        assert fixture.resolve(strict=True).is_relative_to(R / 'LiTo381-Modern/tests/fixtures')
        target_fixture = target.parent / relative
        target_fixture.parent.mkdir(parents=True, exist_ok=True)
        target_fixture.write_bytes(fixture.read_bytes())
        fixture_pins[str(target_fixture)] = dict(source=str(fixture), sha256=sha(fixture))
for mode in ['baseline', 'candidate']:
    source_root = R / 'LiTo381-Modern/src/modern_lito' if mode == 'baseline' else candidate / 'files/LiTo381-Modern/src/modern_lito'
    modules = '\n'.join('    #[path = r#"' + str(source_root / (name + '.rs')) + '"#]\n    pub mod ' + name + ';' for name in ['native_skills', 'native_skill_selection'])
    probe = 'mod candidate {\n    pub use lito_modern::modern_lito::{native_world, native_obstacles, world_map};\n' + modules + '\n}\n'
    probe += '#[path = "tests/native_skills.rs"]\nmod native_requests;\n#[path = "tests/native_skill_selection.rs"]\nmod native_selection;\n'
    (out / (mode + '.rs')).write_text(probe, encoding='utf-8', newline='\n')
record = dict(status='haste43_component_sources_and_static_review_prepared_no_compiler_or_test_run',
              candidate=str(candidate / 'candidate.json'), candidate_sha256=sha(candidate / 'candidate.json'),
              base_source_manifest_sha256=c['base_source_manifest_sha256'], normal_sources_unchanged=True,
              runtime_candidate_exactly_two_request_arm_edits_and_one_label=True, original_test_lines_preserved_in_order=True,
              unchanged_runtime_guards=['learned/MP/HP/ready/movement-block', 'target visible/live/destroyed guardian',
                                        'Target owner/generation/incarnation/revision', 'original pending and server-feedback gates',
                                        'no local MP/speed/status prediction'],
              static_formal_contract='Actual useHasteSpell objectSpellRequest + isInNativeScreen; MP40 base and original reductions/reuse/duration remain server-owned. Existing native gfx755 and S29 parser are reused; this review grants no live authority or GUI qualification.',
              probe_sources={mode: dict(path=str(out / (mode + '.rs')), sha256=sha(out / (mode + '.rs'))) for mode in ['baseline', 'candidate']},
              compatibility_test_sources=test_pins, fixtures=fixture_pins,
              compatibility_rewrite='Only the top-level request/selection module import is redirected to the baseline/candidate modules. World, rendering and all existing test assertions still use the normal source-matched crate. Diagnostic output uses an owned CARGO_MANIFEST_DIR.',
              required_extern_crates=['lito_modern', 'anyhow', 'serde', 'serde_json', 'png'],
              normal_module_reexports=['native_world', 'native_obstacles', 'world_map'],
              baseline_expected_Rust_exit=101, baseline_selected_expected_failures=[
                  'native_requests::spell_requests_use_formal_selectors_and_never_predict_mana',
                  'native_selection::selection_uses_formal_packets_and_preserves_authority',
                  'native_selection::stale_target_never_becomes_another_actor_or_implicit_self_heal'],
              candidate_expected_tests=20, normal_product_and_active_QA_not_modified=True,
              compiler_executed=False, tests_executed=False, actual_formal75_casts=0, qualification_granted=False, whole_goal_complete=False)
save(out / 'preparation.json', record)
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
assert all(sha(R / name) == digest for name, digest in c['before'].items())
print(json.dumps(dict(preparation=str(out / 'preparation.json'), baseline_expected_failures=3, candidate_expected_tests=20, compiled=False, tested=False), ensure_ascii=False))
