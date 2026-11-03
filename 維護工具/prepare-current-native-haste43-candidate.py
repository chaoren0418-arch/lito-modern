"""Prepare one bounded native skill request outside the frozen product tree."""
from pathlib import Path
from datetime import datetime, timezone
import difflib
import hashlib
import json
import re
import zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
save = lambda path, value: Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
w = read(R / '工作佇列.json')
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
normal = Path(w['current_native_installed_resource_profile_normal_qualification']['run'])
inputs = read(normal / 'compiled-inputs.json')
assert len(inputs) == 1049 and all(sha(R / name) == digest for name, digest in inputs.items())
paths = V / 'native-current-skill-request-scope/handler-paths-20261020-071630-321475/paths.json'
mapping = read(paths)
assert mapping['source_manifest_sha256'] == source and mapping['formal_jar_sha256'] == '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(mapping['selected_methods']) == mapping['selected_methods_sha256']
methods = read(mapping['selected_methods'])
handler = next(block for block in methods.values() if block['method'] == 'useHasteSpell')
body = handler['text']
assert all(token in body for token in ['Protocol.objectSpellRequest', 'BattleSpellCatalog.isInNativeScreen', 'HasteSpell.effectiveMpCost',
                                      'HasteSpell.reuseDelayMillis', 'Protocol.speed', 'Protocol.effectObject', 'Protocol.manaPoint'])
definition = next(row for row in mapping['rows'] if row['skill_id'] == 43)['historical_catalogue_definitions'][0]['fields']
assert {key: definition[key] for key in ['skillId', 'row', 'column', 'effectGfx', 'mpCost', 'reuseDelayMillis', 'durationSeconds', 'maximumRange']} == dict(skillId=43, row=5, column=2, effectGfx=755, mpCost=40, reuseDelayMillis=10, durationSeconds=1200, maximumRange=-1)
relative = ['LiTo381-Modern/src/modern_lito/native_skills.rs', 'LiTo381-Modern/src/modern_lito/native_skill_selection.rs',
            'LiTo381-Modern/tests/native_skills.rs', 'LiTo381-Modern/tests/native_skill_selection.rs']
originals = {name: (R / name).read_bytes() for name in relative}
texts = {name: content.decode('utf-8-sig') for name, content in originals.items()}


def replace(name, old, new):
    assert texts[name].count(old) == 1, (name, old)
    texts[name] = texts[name].replace(old, new)


skills, selection, tests, choices = relative
replace(skills, '1 | 8 | 9 | 19 | 26 | 35 | 42 | 57 =>', '1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 =>')
replace(skills, 'matches!(skill, 8 | 9 | 26 | 42)', 'matches!(skill, 8 | 9 | 26 | 42 | 43)')
eol = '\r\n' if '\r\n' in texts[selection] else '\n'
anchor = '                        42 => "#42 力量強化（選取目標／自己）".into(),'
replace(selection, anchor, anchor + eol + '                        43 => "#43 加速術（選取目標／自己）".into(),')
eol = '\r\n' if '\r\n' in texts[tests] else '\n'
anchor = '    assert_eq!(skills::request(&w, 3, None).unwrap(), p("shield"));'
extra = '''
    assert_eq!(skills::request(&w, 43, None).unwrap(), vec![0x14, 5, 2, 1, 0, 0, 0]);
    assert_eq!(skills::request(&w, 43, Some(2)).unwrap(), vec![0x14, 5, 2, 2, 0, 0, 0]);
    // Formal native screen is asymmetric at distance nineteen. Requests remain
    // pure: effective MP cost, equipment discounts and speed are server-owned.
    for (dx, accepted) in [(18i32, true), (19, false), (-19, true), (-20, false)] {
        let mut edge = w.clone();
        let own_x = edge.actors[&1].x;
        let own_y = edge.actors[&1].y;
        let actor = edge.actors.get_mut(&2).unwrap();
        actor.x = u16::try_from(i32::from(own_x) + dx).unwrap();
        actor.y = own_y;
        let snapshot = serde_json::to_value(&edge).unwrap();
        assert_eq!(skills::request(&edge, 43, Some(2)).is_ok(), accepted);
        assert_eq!(serde_json::to_value(&edge).unwrap(), snapshot);
    }
    for change in 0..6 {
        let mut refused = w.clone();
        match change {
            0 => { refused.learned_skills.remove(&43); },
            1 => refused.mp = Some([0, 10]),
            2 => refused.hp = Some([0, 50]),
            3 => refused.actors.get_mut(&2).unwrap().dead = true,
            4 => { refused.actors.remove(&2); },
            _ => { refused.movement_blocks.insert(2); },
        }
        assert!(skills::request(&refused, 43, Some(2)).is_err());
    }
'''.strip('\n').replace('\n', eol)
replace(tests, anchor, anchor + eol + extra)
eol = '\r\n' if '\r\n' in texts[choices] else '\n'
anchor = '    let m = selected(&w, 4);' + eol + '    assert!(m.choice(7, &w, None).is_err());'
extra = '''    let haste = selected(&w, 43);
    assert!(haste.rows().iter().find(|row| row.id == 43).unwrap().label.contains("加速術"));
    let self_haste = haste.choice(7, &w, None).unwrap();
    assert_eq!(native_skills::request(&w, self_haste.skill, self_haste.target).unwrap(), vec![0x14,5,2,1,0,0,0]);
    let target_haste = haste.choice(7, &w, Target::capture(&w, 2)).unwrap();
    assert_eq!(target_haste.target, Some(2));
    assert_eq!(native_skills::request(&w, target_haste.skill, target_haste.target).unwrap(), vec![0x14,5,2,2,0,0,0]);
'''.rstrip('\n').replace('\n', eol)
replace(choices, anchor, extra + eol + anchor)
anchor = '    let heal = selected(&w, 1);'
replace(choices, anchor, anchor + eol + '    let haste = selected(&w, 43);')
anchor = '    assert!(heal.choice(7, &w, target).is_err());'
# First rejection only: retain every existing stale/dead/removal assertion.
assert texts[choices].count(anchor) == 2
texts[choices] = texts[choices].replace(anchor, anchor + eol + '    assert!(haste.choice(7, &w, target).is_err());' + eol + '    assert!(haste.choice(7, &w, None).is_ok());', 1)
for name in relative:
    assert texts[name].count('#[test]') == originals[name].decode('utf-8-sig').count('#[test]')
    assert sha(R / name) == inputs[name]
out = R / '維護工具/candidates' / ('native-haste43-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in relative:
        archive.writestr(name, originals[name])
for name in relative:
    target = out / 'files' / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(texts[name].encode('utf-8'))
assert len({(out / 'files' / name).resolve() for name in relative}) == 4
assert all((out / 'files' / name).read_bytes() == texts[name].encode('utf-8') for name in relative)
patch = ''.join(''.join(difflib.unified_diff(originals[name].decode('utf-8-sig').splitlines(True), texts[name].splitlines(True), fromfile=name, tofile='candidate/' + name)) for name in relative)
(out / 'change.diff').write_text(patch, encoding='utf-8', newline='')
save(out / 'haste43-handler.json', handler)
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
record = dict(status='prepared_native_haste43_request_and_selection_candidate_not_applied_or_executed', base_source_manifest_sha256=source,
              current_source_inputs=str(normal / 'compiled-inputs.json'), current_source_inputs_sha256=sha(normal / 'compiled-inputs.json'),
              formal_handler_paths=str(paths), formal_handler_paths_sha256=sha(paths), handler_sha256=sha(out / 'haste43-handler.json'),
              before={name: inputs[name] for name in relative}, after={name: sha(out / 'files' / name) for name in relative},
              candidate_files={name: str(out / 'files' / name) for name in relative}, distinct_written_files=4,
              files=4, runtime_files=2, existing_test_files_extended=2, existing_test_function_counts_unchanged=True,
              request_self_hex='14050201000000', request_target2_hex='14050202000000', original_reuse_millis=10,
              screen_range='existing formal asymmetric native screen predicate; no new range heuristic',
              authority='no local MP/status/speed prediction; formal original MP40 base, equipment reductions/reuse/eligibility and 1200s duration stay server-owned',
              existing_effect_graphics=755, existing_speed_envelope_opcodes=['29', '62'],
              actual_component_or_library_tests_executed=False, actual_current75_casts=0, source_applied=False,
              remaining_gates=['Review diff and handler conditions independently', 'Isolated candidate tests and prechange counterexample',
                               'Actual formal75 ordinary learned book, self and peer successful casts, malformed/stale/range/MP refusal, no prediction and duplicate pending gate',
                               'Speed/action/effect receipt on both peers, cancellation/map/owner reset and no replay',
                               'Current native effect/frame/audio data qualification and normal GUI',
                               'Apply only after current workloads terminal; rebuild and requalify affected source scopes'],
              physical_audio_verified=False, GUI_verified=False, whole_goal_complete=False)
save(out / 'candidate.json', record)
assert all(sha(R / name) == digest for name, digest in inputs.items())
print(json.dumps(dict(candidate=str(out), sha256=sha(out / 'candidate.json'), runtime_files=2, test_files=2, applied=False, tested=False), ensure_ascii=False))
