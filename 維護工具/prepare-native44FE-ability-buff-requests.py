"""Stage object-target ability buffs from the current formal handler and catalogue."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
base=V/'native44FE-skill-book-audit/20261018-175440-043382';shape=base/'request-shapes-effects-20261018-180850-512952'
proof=read(shape/'readback.json');assert sha(shape/'computed-shapes-effects.json')==proof['data_sha256']
rows={r['skill_id']:r for r in read(shape/'computed-shapes-effects.json')['results']['tw13081901-v1']}
for skill,gfx in [(26,750),(42,6342)]:
    row=rows[skill];assert row['factory']=='abilityBuffSpell'and row['computed_request_shape']=='OBJECT_TARGET'and row['actual_S37_wire_gfx']==gfx and row['reuseDelayMillis']==10
handler=V/'native44FE-same-endpoint/20261018-171204-385221-debug/static-75DA/skill-helpers/useAbilityBuffSpell.javap.txt'
text=handler.read_text('utf-8-sig');assert all(x in text for x in ['Protocol.objectSpellRequest','sameNonemptyClan','BattleSpellCatalog.isInNativeScreen','Protocol.mageDexterity','Protocol.mageStrength'])
normal=V/'native-model-generation-normal/validation-20261018-160708-195207'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
out=ROOT/'維護工具/candidates/native44FE-ability-buff-requests';out.mkdir()
for name in ['native_skills.rs','native_skill_selection.rs']:
    source=P/'src/modern_lito'/name;raw=source.read_bytes();s=raw.decode('utf-8-sig');(out/(name+'.original')).write_bytes(raw)
    if name=='native_skills.rs':
        old='1 | 8 | 9 | 19 | 35 | 57 => {';assert s.count(old)==1;s=s.replace(old,'1 | 8 | 9 | 19 | 26 | 35 | 42 | 57 => {')
        old='matches!(skill, 8 | 9)';assert s.count(old)==1;s=s.replace(old,'matches!(skill, 8 | 9 | 26 | 42)')
    else:
        old='                        14 => "#14 負重強化術（自己）".into(),';assert s.count(old)==1
        s=s.replace(old,old+'\n                        26 => "#26 敏捷強化（選取目標／自己）".into(),\n                        42 => "#42 力量強化（選取目標／自己）".into(),')
    (out/name).write_text(s,encoding='utf-8',newline='\n')
state=dict(status='isolated_ability_buff_request_candidate_not_applied',source_manifest_sha256=read(normal/'cohort.json')['source_manifest_sha256'],
    skills=[26,42],wire_gfx={26:750,42:6342},handler=str(handler),handler_sha256=sha(handler),computed_shapes=str(shape/'computed-shapes-effects.json'),computed_shapes_sha256=sha(shape/'computed-shapes-effects.json'),
    files={n:sha(out/n)for n in ['native_skills.rs','native_skill_selection.rs']},base_files={n:sha(P/'src/modern_lito'/n)for n in ['native_skills.rs','native_skill_selection.rs']},
    behavior='Use existing live-caster/learned/visible-target guards and seven-byte object form; default to own actor, preserve explicit target. Server decides clan membership, costs, buffs and ability scores.',
    normal_source_modified=False,candidate_applied=False,typecheck_verified=False,packet_guard_component_verified=False,actual_sessions=0,actual_casts=0,SPR_decode_verified=False,GUI_verified=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),skills=state['skills'],normal_modified=False),ensure_ascii=False))
