"""Stage a normal-window hotel C10 return candidate; never apply or compile it."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re

R = Path(__file__).resolve().parents[1]
T = R / '維護工具'
P = R / 'LiTo381-Modern'
source = P / 'src/modern_lito/native_rental_window.rs'
read = lambda p: json.loads(p.read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
queue = read(R / '工作佇列.json')
base = queue['current_product_identity']['source_manifest_sha256']
assert base == '4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
proof = R / '驗證紀錄/current/native-keyboard-focus-sky-gui/case-20261019-162732-054619-release-rental-hotel-entry/Sky-events.json'
rooms = [row for row in read(proof) if row['label'] == 'hotel-room-loaded']
assert len(rooms) == 2
window_source = P / 'src/modern_lito/native_login_window.rs'
return_id = int(re.search(r'const RETURN: u16 = (\d+);',window_source.read_text('utf-8-sig')).group(1))
assert return_id == 107
assert all('ID: 129' in row['tree'] and f'ID: {return_id}' not in row['tree'] for row in rooms)
body = source.read_text('utf-8-sig')
changes = [
    ('layout_key:Option<(bool,bool,bool,bool)>',
     'layout_key:Option<(bool,bool,bool,bool,bool)>'),
    ('impl Ui{\n    pub(super) unsafe fn draw_rental_choice',
     'fn hotel_room(s:&crate::modern_lito::native_login::Snapshot)->bool{\n'
     '    rental(s)&&matches!(s.world.map_id,Some(501..=503))\n}\n'
     'impl Ui{\n    pub(super) unsafe fn draw_rental_choice'),
    ('let panel=town&&self.rental.open;',
     'let panel=town&&self.rental.open;let hotel=hotel_room(s);'),
    ('TOGGLE=>town,_=>panel', 'TOGGLE=>town&&!hotel,_=>panel'),
    ('if town{let _=ShowWindow(self.control(RETURN),SW_HIDE);let _=MoveWindow(self.control(TOGGLE),px(160),area.bottom-px(45),px(130),px(32),true);}',
     'if rental(s)&&self.world_view{\n'
     '            let _=ShowWindow(self.control(RETURN),if town&&hotel{SW_SHOW}else{SW_HIDE});\n'
     '            if hotel{self.text(RETURN,"返回選角");}\n'
     '            let _=MoveWindow(self.control(TOGGLE),px(160),area.bottom-px(45),px(130),px(32),true);\n'
     '        }'),
    ('let key=(entry(s),rental(s),self.rental.open,self.inventory_view);',
     'if rental(s){let _=EnableWindow(self.control(RETURN),ready&&hotel_room(s)&&self.world_view&&!self.inventory_view);}\n'
     '        let key=(entry(s),rental(s),self.rental.open,self.inventory_view,hotel_room(s));'),
    ('match id{\n            SET|ENTER=>{',
     'match id{\n'
     '            RETURN if hotel_room(self.controller.snapshot())=>{\n'
     '                ensure!(!self.closing&&self.world_view&&!self.inventory_view,"目前不可從旅館返回選角");\n'
     '                self.controller.return_from_rental()?;\n'
     '                self.held.clear();self.held_attack.clear();self.rental.open=false;\n'
     '                self.rental.binding=None;self.painted=None;\n'
     '            },\n'
     '            SET|ENTER=>{'),
]
for before, after in changes:
    assert body.count(before) == 1, before
    body = body.replace(before, after)
stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = T / 'candidates' / ('native-hotel-return-' + stamp)
out.mkdir(parents=True)
(out / 'before.rs').write_bytes(source.read_bytes())
(out / 'native_rental_window.rs').write_text(body, encoding='utf-8', newline='\n')
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
record = dict(status='source_candidate_only_not_applied_or_compiled', base_source_manifest_sha256=base,
              target=str(source), before_sha256=sha(source), candidate_sha256=sha(out/'native_rental_window.rs'),
              GUI_evidence=str(proof), GUI_evidence_sha256=sha(proof), changes=changes,
              observed_missing_return_control_id=return_id,
              return_control_declaration_source=str(window_source), return_control_declaration_sha256=sha(window_source),
              supersedes_preparation_only=[str(T/'candidates/native-hotel-return-20261019-171257-436882/candidate.json'),str(T/'candidates/native-hotel-return-20261019-171319-679649/candidate.json')],
              preparation_correction='Resolve RETURN107 from the pinned window source, and keep the return footer hidden while the inventory overlays a rental scene. All prior candidates remain unapplied and uncompiled.',
              intended_behavior='Hotel maps 501..503 expose existing C10 return to source character selection in the NPC footer slot; pending/dead/observer/closing/inventory states remain gated. Hotel map changes invalidate the layout key.',
              scope_not_claimed=['hotel-to-village travel', 'voluntary guest return verified', 'owner/shared-key invalidation verified', 'natural expiry', 'normal GUI or full-product qualification'],
              required_validation=['apply only while current workloads are idle and source identity still matches',
                                   'unique-entry normal Debug and Release builds',
                                   'existing rental return protocol tests and both full library selections',
                                   'actual normal GUI owner and guest C10 return, original wire replies, character list, lease/inventory cleanup and subsequent actions',
                                   'hotel map change, inventory open/close, pending input, living/observer eligibility and unchanged ordinary battle withdrawal'],
              product_source_modified=False, compiled=False, GUI_verified=False, whole_goal_complete=False)
(out / 'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert sha(source)==record['before_sha256'] and sha(out/'before.rs')==record['before_sha256']
print(json.dumps(dict(candidate=str(out),status=record['status'],product_source_modified=False),ensure_ascii=False))
