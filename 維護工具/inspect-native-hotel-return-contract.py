"""Seal selected formal75 bytecode for owner/guest C10 hotel departure.

Static contract evidence only. No JVM/game/UI is started and no authority is
modified; runtime and GUI acceptance must be independently established.
"""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
encode=lambda value:(json.dumps(value,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
base=V/'native-source-handoff-expiry/contract-20261019-174028-047309'
proof=read(base/'readback.json');raw=base/'ClientSession.stdout.txt'
assert sha(raw)==proof['raw_files']['ClientSession.stdout.txt']=='964DC949F7EA6C575A8E725D025EA67B94B73ABF81C200BBE2BA190D72248CF1'
assert proof['formal_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
lines=raw.read_text('utf-8-sig').splitlines()
starts=[i for i,line in enumerate(lines)if re.match(r'^  \S.*\(.*\).*;$',line)]
names=['requestRestart','quitFight','completeRestartReturn','prepareRentalHotelDeparture',
    'invalidateSharedRentalHotelLease','serverRentalHotelLeaseInvalidated',
    'removeRentalTownSceneInventory','clearRentalTownRoomState']
methods={}
for name in names:
    candidates=[(k,i)for k,i in enumerate(starts)if ' '+name+'('in lines[i]]
    assert len(candidates)==1,name
    k,i=candidates[0];end=starts[k+1]if k+1<len(starts)else len(lines)
    methods[name]='\n'.join(lines[i:end])+'\n'
restart=methods['requestRestart'];complete=methods['completeRestartReturn']
assert 'invalid C_RESTART payload'in restart and 'RESTART_AWAIT_QUIT_FIGHT'in restart and 'Protocol.restart'in restart
assert complete.index('Method notifyRentalTownDeparture')<complete.index('Method removeRentalTownSceneInventory')<complete.index('Method clearRentalTownRoomState')
assert 'RESTART_AWAIT_SOURCE_REFRESH'in complete and 'Protocol.allowPostTransition'in complete
departure=methods['prepareRentalHotelDeparture']
assert 'Lease.owner:'in departure and departure.index('Method invalidateSharedRentalHotelLease')<departure.index('GameHub.discardRentalHotelKeys')
invalidate=methods['invalidateSharedRentalHotelLease']
assert 'GameHub.invalidateRentalHotelLease'in invalidate and 'connectedSessionsSnapshot'in invalidate and 'serverRentalHotelLeaseInvalidated'in invalidate
event=methods['serverRentalHotelLeaseInvalidated']
assert 'String restart-return'in event and 'Lease.owner:'in event and 'beginRentalTownMapTransition'in event
assert 'sipush        500'in event and 'Field rentalHotelLease:'in event
assert 'prepareRentalHotelDeparture'in methods['removeRentalTownSceneInventory']and 'Protocol.removeInventory'in methods['removeRentalTownSceneInventory']
assert 'Field rentalTownRoomState:Z'in methods['clearRentalTownRoomState']and 'Field rentalHotelLease:'in methods['clearRentalTownRoomState']
out=V/'native-hotel-return-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'selected-methods.json').write_bytes(encode(methods));(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='sealed_formal75_hotel_C10_departure_contract_static_only',
    formal_jar_sha256=proof['formal_jar_sha256'],original_disassembly=str(raw),original_disassembly_sha256=sha(raw),
    original_contract_readback=str(base/'readback.json'),original_contract_readback_sha256=sha(base/'readback.json'),
    selected_methods=str(out/'selected-methods.json'),selected_methods_sha256=sha(out/'selected-methods.json'),
    conclusions=[
        'C10 restart first requests quit-fight acknowledgement; completed rental return removes scene inventory, clears rental state, enters LOBBY and requests source refresh.',
        'Hotel owner departure invalidates the shared lease and its credentials for connected sessions.',
        'For the owner, restart-return is an explicit reason to suppress the hotel-to-map500 invalidation transition; the C10 return continues to source character selection.',
        'A guest departure discards that inventory credential and clears the local lease pointer; it does not take the owner invalidation branch.',
        'Other occupants affected by owner invalidation can receive the ordinary hotel-to-map500 transition; this is a separate guest-flow runtime requirement.'
    ],required_runtime_evidence=[
        'Original C10/S65/CD9/SF0/CD0/SF5/C8/character-directory order, with unchanged payloads and deadlines.',
        'Owner hotel key removals and passive lease/rental-state cleanup before normal window close.',
        'UI shows enabled return to character selection in the hotel; after return the source selector appears and the old world/dialog/skills retire.',
        'Guest departure, shared-key invalidation and natural lease expiry are separate cases; owner entry/return does not qualify them.'
    ],actual_runtime_executed=False,product_source_modified=False,GUI_verified=False,whole_goal_complete=False)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as archive:
    archive.write(out/'selected-methods.json','selected-methods.json');archive.write(out/'inspector.py','inspector.py')
record['raw_archive_sha256']=sha(out/'raw.zip');(out/'readback.json').write_bytes(encode(record))
print(json.dumps(dict(contract=str(out/'readback.json'),actual_runtime_executed=False,GUI_verified=False),ensure_ascii=False))
