"""Select formal75 primary method bodies offline, including package-private timer callback."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
reference=V/'native-hotel-return-contract/20261019-192705-656308/readback.json';proof=read(reference)
p=Path(proof['original_disassembly']);assert sha(p)==proof['original_disassembly_sha256']
jar=R/'LiTo381-Modern/target/current/server-reference/formal-server-75DA69B0.jar'
assert sha(jar)==proof['formal_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
lines=p.read_text('utf-8-sig').splitlines()
starts=[i for i,line in enumerate(lines)if re.match(r'^  \S.*\(.*\).*;\s*$',line)]
names=['performRentalTownHotelRental','expireRentalHotelLeaseIfDue','serverRentalHotelExpiry','activeRentalHotelLease','getRentalHotelGroundKey','dropRentalHotelInventoryKey','sendRentalHotelGroundKey','invalidateSharedRentalHotelLease','serverRentalHotelLeaseInvalidated']
methods={}
for name in names:
 matches=[i for i in starts if (name+'(')in lines[i]];assert len(matches)==1
 start=matches[0];end=next((i for i in starts if i>start),len(lines));text='\n'.join(lines[start:end])+'\n'
 assert sum(bool(re.match(r'^  \S.*\(.*\).*;\s*$',line))for line in text.splitlines())==1
 methods[name]=dict(start_line=start+1,end_line=end,text=text,sha256=hashlib.sha256(text.encode()).hexdigest().upper())
rent=methods['performRentalTownHotelRental']['text'];timer=methods['serverRentalHotelExpiry']['text']
assert all(word in rent for word in ['Room.durationSeconds','java/lang/System.nanoTime','scheduleRentalHotelExpiry'])
assert all(word in timer for word in ['expiresAtNanos','java/lang/System.nanoTime','scheduleRentalHotelExpiry','String rental-hotel-expiry','invalidateSharedRentalHotelLease'])
assert 'serverRentalHotelExpiry(' not in methods['expireRentalHotelLeaseIfDue']['text']
assert 'Protocol.groundItem' in methods['sendRentalHotelGroundKey']['text']
assert 'serverRentalHotelLeaseInvalidated' in methods['invalidateSharedRentalHotelLease']['text']
out=V/'native-room-form-label-tooling'/('formal75-hotel-contract-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'selected-methods.json').write_text(json.dumps(methods,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
record=dict(status='offline_primary_formal75_hotel_sharing_and_natural_expiry_contract',formal_jar_sha256=sha(jar),original_disassembly=str(p),original_disassembly_sha256=sha(p),reference=str(reference),reference_sha256=sha(reference),selected_methods=str(out/'selected-methods.json'),selected_methods_sha256=sha(out/'selected-methods.json'),selected_methods_count=9,package_private_timer_callback_separately_bounded=True,previous_grouped_selection=str(V/'native-room-form-label-tooling/formal75-hotel-sharing-expiry-contract-20261020-015215-716258/readback.json'),conclusions=['Lease construction reads original catalogue durationSeconds and actual System.nanoTime, then uses original expiry scheduling.','Original package-private expiry callback compares its actual expiresAtNanos with System.nanoTime, reschedules when still early, otherwise invalidates with rental-hotel-expiry reason.','Shared invalidation calls original connected session invalidation handler; live key removal and map500 portal still require current GUI/wire/trace verification.','Ground item uses original Protocol.groundItem; no guessed object identity or constructor bypass is permitted.'],server_or_GUI_runtime_started=False,authority_mutations=0,clock_mutations=0,qualification_granted=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(contract=str(out/'readback.json'),methods=9,static_only=True),ensure_ascii=False))
