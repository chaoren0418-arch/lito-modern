from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
from r496_source_oracle import version_body
import shutil,difflib
study=read(L/'r499-source-study.json');pe=Path(study['analysis_PE']['path']).read_bytes();old=read(L/'r498-source-oracle.json')
seedbody=old['setup_bodies'][0];seed=old['seed'];records=old['setup_bodies'][2:];assert len(records)==4
setup=[seedbody,version_body(0x9e38),[3,4]]+records
counter=seed;setupwire=[]
def frame(body,counter):
 raw=bytes(body);cipher,_=source_cipher(raw,counter,pe);return list((len(raw)+2).to_bytes(2,'little')+cipher)
for i,body in enumerate(setup):
 setupwire+=list((len(body)+2).to_bytes(2,'little')+bytes(body))if i==0 else frame(body,counter)
 if i:counter=counter_add_adc(counter,len(body))
after_setup=counter;late=version_body(0x9f0a);latewire=frame(late,counter);counter=counter_add_adc(counter,len(late));after_late=counter
new_records=[]
for i,body in enumerate(records):
 assert bytes(body[1:7])==f'Role{i}\0'.encode();new_records.append([203]+list(f'Again{i}\0'.encode())+body[7:])
renew=[[3,4]]+new_records;renewwire=[]
for body in renew:renewwire+=frame(body,counter);counter=counter_add_adc(counter,len(body))
tx=seed;sends=[]
for name,body in [('first_select',[201]+list(b'Role0\0')),('first_back',[15]),('second_select',[201]+list(b'Again0\0')),('second_back',[15])]:
 after=counter_add_adc(tx,len(body));sends.append(dict(name=name,plain=body,wire=frame(body,tx),before=tx,after=after));tx=after
oracle=dict(state='R499_INDEPENDENT_PINNED_PE_CIPHER_NEW_BACK_ROUNDTRIP_ORACLE',source_study=pin(L/'r499-source-study.json'),fixture_schema_reference=pin(L/'r498-source-oracle.json'),cipher_oracle=pin(P/'tools/r495_source_oracle.py'),version_schema_tool=pin(P/'tools/r496_source_oracle.py'),seed=seed,setup_bodies=setup,setup_wire=setupwire,receive_after_setup=after_setup,late_version_body=late,late_version_wire=latewire,receive_after_late=after_late,new_roster_bodies=renew,new_roster_wire=renewwire,receive_after_roster=counter,refusal_after_setup=dict(plain=[58,1],wire=frame([58,1],after_setup)),send_vectors=sends,synthetic_source_inputs=True,actual_credentials=False,PE_execution=False,source_setters=False,GUI=0)
mode=read(L/'r496-source-oracle.json')['vectors']['mode206']['plain'];oracle['late_mode_after_setup']=dict(plain=mode,wire=frame(mode,after_setup),before=after_setup,after=counter_add_adc(after_setup,len(mode)))
oracle['refusal_after_mode']=dict(plain=[58,1],wire=frame([58,1],oracle['late_mode_after_setup']['after']))
save(L/'r499-source-oracle.json',oracle)
mapping=[
 dict(fragment='493150(receiver6) ->496E30 ->46B670',effect='sendc15 first;clearwidgets/capture/hover/focus;world/mapcleanup;64BBD4=5;496DA0loginpanel',modern='opaqueBack requirescurrentowner/epoch/roster/scene andno pending;reserve1byte15 throughsameSender thenretirelogicalUIbindings;phase5 action-owned'),
 dict(fragment='46B670 ordinary success;46D2E0;47E4B0;478680',effect='directtracednormalpaths containno reset8317C0/C4 RX,8317C8/CC TX,63D54Cversion,8343E8socket or62FAA0rawroster',modern='retainEpoch/receiverpartialassembly/version/Sendercounter;retainreceivedrawrecords butclearselection authority;awaitactualnew3/203 beforecontrols return',limit='Bounded directsource trace;dynamicvirtualdestructors/externaltransitiveeffects notclosed. Noall-Native-teardownclaim.'),
 dict(fragment='4C65F0 vs46B670',effect='4C65F0 closes8343E8 through4EFE9E,clears8317D0 andWSACleanupbranch;ordinaryBack doesnotdirectlycallit',modern='Back normaldoesnotdisconnect;writefailure/refused58/Close stillretirewholecandidateEpoch aspreviousModernpolicy'),
 dict(fragment='496DA0',effect='clear6414A4/6414A0/64149C;callbacks496D40/60/495E40;externalrand%10 IMG1B12..1B1B;table631F68 count1group20;495230show',modern='Entry candidate only;actualLogininputpanel/backgroundrandom andcredentials notimplemented'),
 dict(fragment='49A510(ECX1)',effect='760054index/760058table ->external5B10318;not64BBD4 phase setter',modern='noinventedphase1/phase7;onlyexplicit46B6A4 write5 implemented'),
 dict(fragment='1/d ->4AE050 ->480560',effect='seedRX/TX;chdch(0,8033,636584,FromCD,5033C4);FromCD2if64BBF8elseregistrySoftware\\NC Soft\\Lineage DWORDlowbyte/default0;PEinitial636584=FFFFFFFF(-1),5033C4=114F8;58locale branchmayreplace636584',modern='compatibilitysendremainsdenied;registry/provenance/currentlocaleversionauthority notqualified;noOSregistry/auth called'),
 dict(fragment='58 ->4A33E0',effect='c1refusal;ordinaryversion63D54C andlocale/environment;LTspelldur.tbl/834960/834A60 dependencies;75F755 nonzero ->getsockname8343E8 ->cssd(1,641348,6412A0,addressDword)',modern='58updatesversion only;neverloginACK;authbuffers/flagproducer/localaddress/comparator tables OPEN;nofakecredentials'),
 dict(fragment='48FAA0 then3/4A7450 and203/4A7550',effect='75F755/56 gatedvariable72byte list75F758;possiblec53/cs200;3initializes4rawrecordsFF;203actualrecordrebuild/count complete ->497010',modern='newactual3replacesrosterandclearsawaitgate;complete203activatesfreshscenecontrols;noauthstageorphase7 invented',limit='48FAA0 dispatcheropcode andlist meaning/flagproducer notqualified;fullauth-to-rosterproducer remainsOPEN')]
save(L/'r499-call-state-mapping.json',dict(state='R499_SOURCE_BACK_VS_MODERN_RETIREMENT_AND_RELOGIN_DEPENDENCY_MAP',source=pin(L/'r499-source-study.json'),prior_R498_Back='wholeepochretirement supersededforcurrentopaquecharacterBack bysourceprovennormalc15/phase5connectionretention',mapping=mapping,next_smallest_closed_fragment='normalopaqueBackc15/phase5 plus source3/203reselect onretainedepoch; executableofflinefixture only',next_open_dependency='75F755/75F756 producer andqualified480560/58environment/locale/authbufferauthority,variablelistdispatch andcomparatortables;thenactualnormalSocketreader/writer;never58==authsuccess',GUI=0))
oldprod=read(L/'r498-production-source-snapshot.json');delta=[];diffs=[]
for q in oldprod['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_source_send.rs','native_source_identity.rs','native_mode_state_flow.rs','native_character_selection.rs','native_city_presentation.rs','native_city_runtime.rs','native_city_window.rs'},delta
files=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(files)==612
diff=L/'r499-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');out=P/'target/r499-production-source';out.mkdir(exist_ok=False);pins=[]
for p in files:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r499-production-source-snapshot.json',dict(state='R499_ALL612_SOURCE_FROZEN_BEFORE_NEW_CONSUMER',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r498-source-scope.json');scope.pop('changed_from_fixed_R497',None);scope.pop('added_from_fixed_R497',None);scope.update(state='R499_NORMAL_BACK_RETAINED_EPOCH_AND_NEW_ROSTER_CONTROL_SCOPE',source_files=612,changed_from_fixed_R498=delta,added_from_fixed_R498=[],authorization='R499 delegatedcontinuation sourceBack/relogin/reselection dependencies;Modernownedoutputs pluspreviousModernASCII scratch',scope='7changed/605unchanged/noaddedproduction;originalresourcesreadonly;sourcec15/phase5 only;noauth/provider/socket/JavaSQL/server/sharedBIN/GUI')
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r499-source-scope.json',scope);save(L/'source-change-scope.json',scope)
oldplayer=read(L/'r498-player-candidate.json');root=P/'target/r499-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R499.rs';main.write_text(Path(oldplayer['candidate_main']['path']).read_text(encoding='utf-8').replace('r498','r499').replace('R498','R499'),encoding='utf-8')
entry=read(L/'r498-entry-source-contract.json');entry.update(state='R499_OPAQUE_SOURCE_BACK15_RETAINED_CONNECTION_CANDIDATE',candidate_main=pin(main),skin_binding=pin(L/'r498-native-menu-skin-binding.json'),previous_R498_player_record=pin(L/'r498-player-candidate.json'),UI_assets='sameR498skin;opaqueBack pendingdisable/currentepoch scene+roster revision;source15/phase5;oldrosterhidden untilnew3/203;actualLoginUI/auth OPEN')
save(L/'r499-entry-source-contract.json',entry)
for oldname,newname in [('build-r498-player-candidate.ps1','build-r499-player-candidate.ps1'),('build-r498-character-ui-consumer.ps1','build-r499-back-roundtrip-consumer.ps1')]:
 s=(P/'tools'/oldname).read_text(encoding='utf-8').replace('r498','r499').replace('R498','R499').replace('r499d-library-build','r499b-library-build')
 if 'consumer'in newname:
  s=s.replace('character-ui-consumer','back-roundtrip-consumer').replace('character_ui_consumer','back_roundtrip_consumer').replace("'r497-candidate.json'","'r498-candidate.json'").replace('R497 source snapshot','R498 source snapshot').replace('198','200')
  s=s.replace(",$source,(Join-Path $project 'src\\modern_lito\\native_character_selection.rs'))",',$source)')
 (P/'tools'/newname).write_text(s,encoding='utf-8')
print('R499 oracle/call-state map/frozen612sources/7changes/diff',diff.stat().st_size,'bytes/200pairs prepared')
