"""Prepare complete request/selection/effect edits for detection; do not apply."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
out=ROOT/'維護工具/candidates/native-detection-skill'
assert not out.exists();out.mkdir()
changes={
 'native_skills.rs':[
  ('        2 => {ensure!(target.is_none(), "日光術不接受目標");}',
   '        13 => {ensure!(target.is_none(), "偵測術不接受目標");}\n        2 => {ensure!(target.is_none(), "日光術不接受目標");}'),
  ('11|17|18|57=>500','11|13|17|18|57=>500')],
 'native_skill_selection.rs':[
  ('matches!(skill,2|3|131)','matches!(skill,2|3|13|131)'),
  ('                        15 => "#15 火箭（選取目標）".into(),',
   '                        13 => "#13 偵測術（自己）".into(),\n                        15 => "#15 火箭（選取目標）".into(),')],
 'native_skill_effects.rs':[
  ('qualify(c, &[221, 744, 745])?','qualify(c, &[221, 744, 745, 749])?'),
  ('Self::Skills => &[221, 744, 745, 754,','Self::Skills => &[221, 744, 745, 749, 754,'),
  ('&& [1, 2, 3, 8, 9, 11, 18].iter()','&& [1, 2, 3, 8, 9, 11, 13, 18].iter()')],
}
source=ROOT/'LiTo381-Modern/src/modern_lito';files={}
for name,edits in changes.items():
    raw=(source/name).read_bytes();text=raw.decode('utf-8-sig').replace('\r\n','\n')
    (out/('base-'+name)).write_bytes(raw)
    for before,after in edits:assert text.count(before)==1,(name,before);text=text.replace(before,after,1)
    (out/name).write_text(text,encoding='utf-8',newline='\n')
    files[name]=dict(base_sha256=sha(source/name),candidate_sha256=sha(out/name),replacement_count=len(edits))
factory_path=ROOT/'驗證紀錄/current/native-current-skill-contract-audit/current-learned-factories.json'
f=json.loads(factory_path.read_text(encoding='utf-8-sig'))
definition=next(row for row in f['rows'] if row['skill_id']==13)
assert definition['matches'][0]['fields']['effectGfx']==749 and definition['matches'][0]['fields']['reuseDelayMillis']==500
meta=dict(status='unapplied_native_detection_request_selection_and_effect_candidate',files=files,skill_id=13,
    formal_definition=definition,formal_factory_reference=str(factory_path),formal_factory_reference_sha256=sha(factory_path),
    request_hex='140104',gameplay_cast_executed=False,original_resource_effect_qualified=False,
    candidate_applied_to_product=False,product_source_changed=False,GUI_verified=False,whole_goal_complete=False)
(out/'metadata.json').write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),skill=13,files=3,product_source_changed=False),ensure_ascii=False))
