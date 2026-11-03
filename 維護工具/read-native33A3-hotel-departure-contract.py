"""Separate the current portal contract from unverified manual guest departure."""
from pathlib import Path
import argparse,hashlib,json,re,subprocess,os
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具'
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
out=a.run.resolve(strict=True);assert out.is_relative_to(ROOT/'驗證紀錄/current/native33A3-hotel-departure-contract')
p=out/'selected-methods.json';x=read(p);assert sha(x['reference_jar'])==x['reference_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
blocks=x['methods'];portal=next(v for k,v in blocks.items()if 'enterRentalTownPortal('in k);depart=next(v for k,v in blocks.items()if 'prepareRentalHotelDeparture('in k)
assert re.search(r'130:\s+if_icmpgt\s+175',portal)and re.search(r'145:\s+if_icmpne\s+175',portal)and 'NativeLitoTownPortalCatalog.find'in portal
assert 'invalidateSharedRentalHotelLease'in depart and 'discardRentalHotelKeys'in depart
source=ROOT/'LiTo381-Modern/src/modern_lito/native_login.rs';s=source.read_text('utf-8-sig')
start=s.index('pub(crate) fn merchant_owner');end=s.index('fn merchant_target',start);owner=s[start:end];assert 'map_id'not in owner and 'r.mode==6'in owner
start=s.index('pub fn return_from_rental(');end=s.index('pub fn join_rental_room',start);body=s[start:end];assert 'vec![0x10]'in body and 'RentalRestartPending'in body
report=dict(status='read_only_contract_followup_runtime_unverified',reference_jar_sha256=x['reference_jar_sha256'],method_evidence=str(p),method_evidence_sha256=sha(p),source_files={str(source):sha(source)},findings=[dict(kind='static_control_flow',claim='enterRentalTownPortal checks distance <=3 and a catalogue route whose target map equals the current rental map before sending S5A; this method alone cannot establish a room-to-village exit',method='enterRentalTownPortal',offsets=[128,129,130,137,145,175,208]),dict(kind='static_control_flow',claim='prepareRentalHotelDeparture invalidates the shared lease for its owner; for a guest it discards guest hotel keys and clears the local lease',method='prepareRentalHotelDeparture'),dict(kind='source_readback',claim='Normal return_from_rental is available to a living non-observer in ready mode6 without a map500-only restriction; sends C10 and waits in RentalRestartPending. This is a source/character return, not evidence of room-to-village manual exit',file=str(source))],actual_sessions=0,manual_guest_exit_verified=False,runtime_verified=False,normal_source_modified=False,launcher_artifact_modified=False,next_action='After current broad and full-library completion, first execute the already prepared normal eight hotel cases. Choose a separate manual guest departure case from an actual supported current75 route; retain village-return and character-return as distinct outcomes.')
save(out/'findings.json',report);(out/'reader.py').write_bytes(Path(__file__).read_bytes())
save(out/'initial-reader-failures.json',[dict(tool_chunk='3d82fc',actual_exit_code=1,error='Exact whitespace in opcode assertion did not match javap spacing'),dict(tool_chunk='e0f069',actual_exit_code=1,error='Distance comparison offset was incorrectly 129 (iconst_3); actual if_icmpgt is offset130, observed in chunk169863. Corrected without changing selected method bytes')])
env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',PYTHONIOENCODING='utf-8',TEMP=str(ROOT/'LiTo381-Modern/target/current/scratch'),TMP=str(ROOT/'LiTo381-Modern/target/current/scratch'));rows=[]
for name in ['run-native33A3-hotel.py','qualify-native33A3-hotel-rooms.py','qualify-native33A3-hotel-keys.py','qualify-native33A3-hotel-share.py','qualify-native33A3-hotel.py','checkpoint-native33A3-hotel.py']:
    z=subprocess.run(['python','-B','-X','utf8',str(T/name),'--help'],env=env,cwd=ROOT,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=10)
    assert z.returncode==0 and not z.stderr,(name,z.stderr)
    rows.append(dict(tool=name,sha256=sha(T/name),help_exit_code=z.returncode,scope='imports and argument parser only; no runtime case'))
save(out/'tool-import-help-readback.json',rows)
print(json.dumps(dict(static_findings=str(out/'findings.json'),tool_import_and_help_passed=len(rows),actual_sessions=0,manual_exit_runtime_verified=False),ensure_ascii=False))
