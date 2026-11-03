"""Prepare nine exact selector intents outside the frozen product source."""
from pathlib import Path
import hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

base = ROOT/'驗證紀錄/current/native-presentation-source-owner/validation-20261017-060440-458259'
pins = read(base/'compiled-inputs.json')
assert len(pins) == 1025 and all(sha(ROOT/name) == h for name,h in pins.items())
q = read(base/'completion-observation.json')
assert q['observed_terminal_exit_code'] == 0 and q['recorded_owned_processes_all_terminal']
review = ROOT/'驗證紀錄/current/native-skill-selector-only-contract-review/20261017-055306-815433/review.json'
contract = read(review)
rows = contract['selector_only_missing_batch']
assert {row['skill_id'] for row in rows} == {14,31,32,49,52,54,72,78,206}
codec_root = ROOT/'驗證紀錄/current/native-current-skill-contract-audit'
cq = read(codec_root/'selector-codec-qualified.json')
assert cq['positive_selector_parser_cases'] == 22 and cq['invalid_extra_target_or_truncated_layout_rejections'] == 44
assert sha(codec_root/'selector-codec-evidence.zip') == cq['archive_sha256']
with zipfile.ZipFile(codec_root/'selector-codec-evidence.zip') as z:
    for name,h in cq['file_pins'].items():
        with z.open(name) as f:
            assert hashlib.file_digest(f,'sha256').hexdigest().upper() == h
mapping = read(codec_root/'factory-request-decoder-map.json')
session = Path(mapping['retained_ClientSession_javap'])
assert session.resolve().is_relative_to(ROOT) and sha(session) == mapping['retained_ClientSession_javap_sha256']
text = session.read_bytes().decode('utf-8-sig')
signature = '  private local.lito381.server.ClientSession resolveSelectorOnlyConcentrationTarget('
start = text.index(signature)
end = text.index('  private ', start+len(signature))
target_contract = text[start:end]
assert all(token in target_contract for token in ['SELF_SELECTOR', 'lastCombatTargetObjectId', 'maximumRange', 'ArrayList.size', 'areturn'])
out = ROOT/'維護工具/candidates/native-selector-nine-intents'
out.mkdir()
(out/'c02-concentration-target.javap.txt').write_bytes(target_contract.encode('utf-8'))
paths = ['LiTo381-Modern/src/modern_lito/native_skills.rs', 'LiTo381-Modern/src/modern_lito/native_skill_selection.rs']
before = {name:sha(ROOT/name) for name in paths}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in paths:
        z.write(ROOT/name,name)

def patch(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old,new)

skills = (ROOT/paths[0]).read_bytes().decode('utf-8')
eol = '\r\n' if '\r\n' in skills else '\n'
anchor = '        13 => {ensure!(target.is_none(), "偵測術不接受目標");}'
skills = patch(skills, anchor, '        14 | 31 | 32 | 49 | 52 | 54 | 72 | 78 | 206 => {ensure!(target.is_none(), "此技能不接受指定目標");}'+eol+anchor)
skills = patch(skills, 'match skill {60=>0,11|13|17|18|57=>500,19=>100,35=>200,28=>50,_=>10}',
    'match skill {60=>0,11|13|17|18|49|57|72=>500,78=>12000,206=>1000,19=>100,35=>200,28=>50,_=>10}')
skills = patch(skills, 'match skill {10|35=>200,19|28=>100,2|16=>10,17=>100,18|57=>500,_=>0}',
    'match skill {10|35=>200,19|28=>100,2|16|32=>10,17=>100,18|57|72=>500,206=>1200,_=>0}')
(out/'native_skills.rs').write_bytes(skills.encode('utf-8'))
selection = (ROOT/paths[1]).read_bytes().decode('utf-8')
selection = patch(selection, 'matches!(skill,2|3|13|60|131)', 'matches!(skill,2|3|13|14|31|32|49|52|54|60|72|78|131|206)')
eol = '\r\n' if '\r\n' in selection else '\n'
anchor = '                        13 => "#13 偵測術（自己）".into(),'
labels = [(14,'負重強化術（自己）'),(31,'魔法屏障（自己，消耗 1 顆魔法寶石）'),(32,'冥想術（自己）'),
    (49,'集體治癒術（周圍角色）'),(52,'神聖疾走（自己）'),(54,'強力加速術（自己）'),
    (72,'反隱身屏障（周圍範圍）'),(78,'絕對屏障（自己，消耗 2 顆魔法寶石）'),(206,'專注（伺服器判定三格內對象）')]
selection = patch(selection, anchor, eol.join('                        '+str(skill)+' => "#'+str(skill)+' '+label+'".into(),' for skill,label in labels)+eol+anchor)
(out/'native_skill_selection.rs').write_bytes(selection.encode('utf-8'))
after = {paths[0]:sha(out/'native_skills.rs'), paths[1]:sha(out/'native_skill_selection.rs')}
assert all(sha(ROOT/name) == h for name,h in pins.items())
save(out/'candidate.json', dict(status='prepared_nine_selector_intents_pending_isolated_execution',
    base_related_run=str(base), base_source_manifest_sha256=q['source_manifest_sha256'], before=before, after=after,
    wire_vectors={str(row['skill_id']):row['request_hex'] for row in rows},
    cooldowns={str(row['skill_id']):dict(reuse_ms=row['fields']['reuseDelayMillis'], client_cast_clock_ms=row['fields'].get('clientCastClockMillis',0)) for row in rows},
    selector_codec_qualification=str(codec_root/'selector-codec-qualified.json'), selector_codec_qualification_sha256=sha(codec_root/'selector-codec-qualified.json'),
    retained_C02_session_sha256=sha(session), concentration_target_contract_sha256=sha(out/'c02-concentration-target.javap.txt'),
    concentration206=dict(client_supplies_no_object_id=True,server_targeting='eligible recent combat target, otherwise sole eligible nearby player, self if none, ambiguous group rejected',
        range=3,self_only_inferred=False,live_behavior_verified=False),
    product_source_changed=False, applied=False, actual_new_sessions=0, actual_new_casts=0,
    graphics_and_native_audio_pipeline_connected=False, GUI_verified=False, whole_goal_complete=False))
print(json.dumps(dict(candidate=str(out),candidate_files=2,product_source_changed=False,skill206_self_only_inferred=False),ensure_ascii=False))
