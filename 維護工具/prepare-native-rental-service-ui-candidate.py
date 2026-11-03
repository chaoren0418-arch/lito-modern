"""Stage two native-source fixes against sealed rental-service contract evidence."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source=='A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
root=V/'native-source-handoff-expiry/contract-20261019-174028-047309';proof=read(root/'readback.json')
raw=root/'ClientSession.stdout.txt';assert sha(raw)==proof['raw_files']['ClientSession.stdout.txt']
lines=raw.read_text('utf-8-sig').splitlines();starts=[i for i,l in enumerate(lines)if re.match(r'^  \S.*\(.*\).*;$',l)]
methods={}
for needle in ['private boolean rentalItemSelectionContext()','private void enterChannel(byte[])','private void createRoom(byte[])']:
    start=next(i for i,l in enumerate(lines)if needle in l);end=next(i for i in starts if i>start);methods[needle]='\n'.join(lines[start:end])+'\n'
context=methods['private boolean rentalItemSelectionContext()'];channel=methods['private void enterChannel(byte[])'];create=methods['private void createRoom(byte[])']
assert 'rentalTownDirectoryAdvertised'in context and 'hasRentalBattleHandoff'in context and 'rentalBattleLoadoutActive'not in context
assert channel.index('rentalItemSelectionContext')<channel.index('selectRentalItemSet')
assert 'Phase.CHANNEL'in create and 'create-invalid-phase'in create
gui=V/'native-accessibility-labels-sky-gui/case-20261019-172310-318572-debug-rental-entry'
events=read(gui/'Sky-events.json');entry=next(e for e in events if e['role']=='a'and e['label']=='rental-directory-ready')
assert any('進入頻道'in l and 'ID: 103'in l and 'disabled'not in l for l in entry['tree'].splitlines())
old=w['qualification_before_initial_source_confirmation'];normal=read(old['normal']['qualification']);archive=Path(normal['compiled_source_archive'])
names=['LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_rental_window.rs']
window='LiTo381-Modern/src/modern_lito/native_login_window.rs'
with zipfile.ZipFile(archive)as z:
    assert z.read(window)==(R/window).read_bytes()
    assert z.read(names[1])==(R/names[1]).read_bytes()
    old_login=z.read(names[0]).decode('utf-8-sig').replace('\r\n','\n')
now_login=(R/names[0]).read_text('utf-8-sig')
section=lambda b:b[b.index('    pub fn enter_channel('):b.index('    pub fn refresh_rooms(',b.index('    pub fn enter_channel('))]
assert section(old_login)==section(now_login)
hotel=T/'candidates/native-hotel-return-20261019-171731-517856';h=read(hotel/'candidate.json')
assert sha(R/names[1])==h['before_sha256']and sha(hotel/'native_rental_window.rs')==h['candidate_sha256']
login=(R/names[0]).read_bytes().decode('utf-8-sig');nl='\r\n'if'\r\n'in login else'\n'
needle=nl.join(['            "channel selection is not ready"','        );','        let mut frames = vec![];'])
replacement=nl.join(['            "channel selection is not ready"','        );',
    '        ensure!(self.state.phase != LoginPhase::CharacterSelected',
    '            || self.state.services.as_ref().is_none_or(|directory| directory.rooms.is_empty()),',
    '            "此入口提供租裝服務，請確認配裝後進入租裝村");',
    '        let mut frames = vec![];'])
assert login.count(needle)==1;login=login.replace(needle,replacement)
rental=(hotel/'native_rental_window.rs').read_text('utf-8-sig')
needle='if entry{for id in [ROOMS,305,306,ROOM_PASSWORD,JOIN,READY,UNREADY,EXIT_ROOM,CHANGE_FORCE]'
replacement='if entry{for id in [SOURCES,304,SELECT,ROOMS,305,306,ROOM_PASSWORD,JOIN,READY,UNREADY,EXIT_ROOM,CHANGE_FORCE]'
assert rental.count(needle)==1;rental=rental.replace(needle,replacement)
out=T/'candidates'/('native-rental-service-ui-and-hotel-return-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
payloads={names[0]:login.encode('utf-8'),names[1]:rental.encode('utf-8')}
for name,payload in payloads.items():
    (out/(Path(name).name+'.before')).write_bytes((R/name).read_bytes());(out/Path(name).name).write_bytes(payload)
(out/'formal-selected-methods.json').write_text(json.dumps(methods,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='two_native_source_candidates_not_applied_or_compiled',base_source_manifest_sha256=source,
    before_files={n:sha(R/n)for n in names},candidate_files={n:sha(out/Path(n).name)for n in names},
    formal_jar_sha256=proof['formal_jar_sha256'],formal_class_disassembly=str(raw),formal_class_disassembly_sha256=sha(raw),
    selected_methods=str(out/'formal-selected-methods.json'),selected_methods_sha256=sha(out/'formal-selected-methods.json'),
    observed_GUI_evidence=str(gui/'Sky-events.json'),observed_GUI_evidence_sha256=sha(gui/'Sky-events.json'),
    GUI_evidence_source_manifest_sha256=old['source_manifest_sha256'],unchanged_current_UI_bytes_and_enter_channel_body_verified=True,
    hotel_candidate=str(hotel/'candidate.json'),hotel_candidate_sha256=sha(hotel/'candidate.json'),
    intended_behavior=['Hide ordinary channel controls while rental entry is advertised; controller rejects that unsupported request before writing CA00, while already-entered ordinary channel switching remains available.',
        'Hotel maps 501..503 expose existing C10 return to character selection in the NPC footer slot, with inventory and state eligibility gating and hotel map changes invalidating layout.'],
    scope_not_claimed=['ordinary channels supported on the rental endpoint','hotel-to-village travel','guest/owner/shared lease behavior verified','natural expiry verified','normal GUI or full product qualification'],
    required_validation=['apply only when current workloads are idle and all source pins still match','normal both-profile builds and targeted tests, full library before full GUI acceptance',
        'normal GUI rental entry hides ordinary channel controls, ordinary endpoint retains them, same endpoint retains valid rental service routes',
        'actual controller rejection emits no channel request or state change','hotel owner and guest C10 return with original reply and lease/inventory cleanup, then source character selection and normal follow-up',
        'initial GAME C8 confirmation and staged rental teleport/recovery behavior retained'],
    formal_JAR_modified=False,product_source_modified=False,compiled=False,GUI_verified=False,whole_goal_complete=False)
(out/'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert all(sha(R/n)==digest for n,digest in record['before_files'].items())
print(json.dumps(dict(candidate=str(out),status=record['status'],product_source_modified=False),ensure_ascii=False))
