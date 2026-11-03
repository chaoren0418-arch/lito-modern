"""Reuse the already exercised resource runner with a distinct reviewed pair."""
from pathlib import Path
from datetime import datetime, timezone
import ast
import hashlib
import json
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
save = lambda path, value: Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
original_plan = V / 'native-current-library-haste-resource2/plan-20261020-082158-005287/plan.json'
assert sha(original_plan) == '221023700E8B891962188F3F22E57F610ED7108F352B53FD905D97764E9B7740'
plan = read(original_plan)
assert plan['plan_does_not_execute_tests'] and plan['expected_tests_per_profile'] == 2
old_runner = R / '維護工具/run-current-library-motion-resource2.py'
old_reader = R / '維護工具/checkpoint-current-library-motion-resource2.py'
text = old_runner.read_text('utf-8-sig')
start = text.index('names = [\n')
end = text.index('\n]', start) + 2
text = text[:start] + 'names = ' + repr(plan['names']) + text[end:]
start = text.index('if args.prepare_only:\n')
end = text.index('\nassert args.plan is not None', start)
text = text[:start] + text[end:]
text = text.replace("parser.add_argument('--prepare-only', action='store_true')\n", '')
text = text.replace('native-current-library-motion-resource2', 'native-current-library-haste-resource2')
text = text.replace('motion_resource', 'haste_resource')
text = text.replace("'shared footsteps: 144 qualified families'", "'HASTE43 sources='")
text = text.replace("'3768 walk/attack source'", "'GREATER_HASTE sources='")
text = text.replace('Two additionally reviewed ignored bodies', 'Two additionally reviewed Haste effect resource bodies')
assert '--prepare-only' not in text and 'real_shared_player_footsteps' not in text
assert 'HASTE43 sources=' in text and 'GREATER_HASTE sources=' in text
ast.parse(text)
runner = R / '維護工具/run-current-library-haste-resource2.py'
if runner.exists():assert runner.read_text('utf-8-sig')==text
else:runner.write_text(text, encoding='utf-8', newline='\n')
text = old_reader.read_text('utf-8-sig')
text = text.replace('native-current-library-motion-resource2', 'native-current-library-haste-resource2')
text = text.replace('motion_resource', 'haste_resource')
text = text.replace("'shared footsteps: 144 qualified families'", "'HASTE43 sources='")
text = text.replace("'3768 walk/attack source'", "'GREATER_HASTE sources='")
text = text.replace('prior_mutually_exclusive_resource_tests=81', 'prior_mutually_exclusive_resource_tests=83')
text = text.replace('total_mutually_exclusive_resource_tests=83 if qualified else 81', 'total_mutually_exclusive_resource_tests=85 if qualified else 83')
text = text.replace('remaining_ignored_bodies_unexecuted=405 if qualified else 407', 'remaining_ignored_bodies_unexecuted=403 if qualified else 405')
text = text.replace('144玩家族腳步時序／PCM與十種NPC3768影格／cue／圖層', '#43與#54原SPR／各六影格RGBA／原播放時序與結束狀態')
text = text.replace('互斥資源合計83、餘405', '互斥資源合計85、餘403')
anchor = "assert len(names) == 2 and plan['expected_tests_per_profile'] == c['expected_tests_per_profile'] == 2\n"
assert text.count(anchor) == 1
literal='{'+', '.join(repr(name)for name in sorted(plan['names']))+'}'
text = text.replace(anchor, anchor + "motion = w['current_library_motion_resource2']\nassert motion['qualified'] and motion['source_manifest_sha256'] == source\nassert sha(motion['qualification']) == motion['qualification_sha256']\nassert not names & set(motion['names'])\nassert names == " + literal + "\n")
ast.parse(text)
reader = R / '維護工具/checkpoint-current-library-haste-resource2.py'
if reader.exists():assert reader.read_text('utf-8-sig')==text
else:reader.write_text(text, encoding='utf-8', newline='\n')
prior_plans=list(original_plan.parent.parent.glob('execution-plan-*/plan.json'))
assert not list(original_plan.parent.parent.glob('run-*'))
out = original_plan.parent.parent / ('execution-plan-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
plan.update(original_reviewed_plan=str(original_plan), original_reviewed_plan_sha256=sha(original_plan),
            runner_sha256=sha(runner), reader=str(reader), reader_sha256=sha(reader),
            library_completion_sha256=sha(Path(plan['library'])/'completion-observation.json'))
save(out / 'plan.json', plan)
for old in prior_plans:
    marker=old.parent/'superseded.json'
    if not marker.exists()and'library_completion_sha256'not in read(old):save(marker,dict(status='superseded_unexecuted_plan_missing_library_completion_pin',old_plan_sha256=sha(old),new_plan=str(out/'plan.json'),new_plan_sha256=sha(out/'plan.json'),runtime_started=False))
save(out / 'preparation.json', dict(status='haste_resource_pair_runner_and_independent_reader_prepared_not_executed',
                                  original_plan_sha256=sha(original_plan), plan_sha256=sha(out / 'plan.json'),
                                  template_runner_sha256=sha(old_runner), template_reader_sha256=sha(old_reader),
                                  runner=str(runner), runner_sha256=sha(runner), reader=str(reader), reader_sha256=sha(reader),
                                  changes='Distinct exact pair, reviewed plan and output family, corresponding diagnostic literals, independent existing-motion exclusion and exact Haste names; all current-source/library/PE/resource/terminal/archive guards retained.',
                                  compiler_or_runtime_started=False, qualification_granted=False, whole_goal_complete=False))
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(plan=str(out / 'plan.json'), runner=str(runner), reader=str(reader), executed=False), ensure_ascii=False))
