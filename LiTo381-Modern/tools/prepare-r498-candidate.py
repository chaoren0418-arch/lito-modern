from r491_common import *
import struct,shutil
from r495_source_oracle import source_cipher,counter_add_adc
mapping=read(L/'r498-character-resource-mapping.json');skin=read(L/'r497-native-menu-skin-binding.json');skin['state']='R498_SAME_SKIN_PLUS_FOUR_SOURCE_CHARACTER_BUTTONS';skin['previous_skin']=pin(L/'r497-native-menu-skin-binding.json');skin['character_buttons']=[]
for ident in [6303,6304,6201,6202]:
 m=next(m for m in mapping['members']if m['name']==f'{ident}.img');raw=Path(m['raw']['path']).read_bytes();w,h,flag,key=struct.unpack('<hhHH',raw[:8]);rgba=bytearray()
 for(v,)in struct.iter_unpack('<H',raw[8:]):
  assert v<32768
  rgba.extend(bytes(4)if v==key else bytes(((c<<3)|(c>>2))for c in [(v>>10)&31,(v>>5)&31,v&31])+bytes([255]))
 art=P/f'target/r498-character-ui-study/{ident}.rgba';art.write_bytes(rgba);m['rgba_reference']=pin(art);skin['character_buttons'].append(dict(id=ident,bytes=m['container_bytes'],sha256=m['raw']['sha256'],reference=pin(art)))
save(L/'r498-character-resource-mapping.json',mapping);save(L/'r498-native-menu-skin-binding.json',skin)
old=read(L/'r497-player-candidate.json');O=P/'target/r498-player-entry-candidate';O.mkdir(exist_ok=True);main=O/'LiToModern-R498.rs';main.write_text(Path(old['candidate_main']['path']).read_text(encoding='utf-8').replace('r497','r498').replace('R497','R498'),encoding='utf-8')
entry=read(L/'r497-entry-source-contract.json');entry.update(state='R498_SOURCE_CHARACTER_SELECT_VS_SUBMIT_AND_ORIGINAL_BUTTON_ART_CANDIDATE',candidate_main=pin(main),skin_binding=pin(L/'r498-native-menu-skin-binding.json'),previous_R497_player_record=pin(L/'r497-player-candidate.json'),UI_assets='IMG6303/6304 submit and6201/6202 back;source51x23button rects translated by Modern scene origin24/232 andDPI. Four slot buttons remain Modern geometry becauseIMG79/80 dimensions andPNG art absent. Singleclick stagesselection;doubleclick/opaqueSubmit buildsownedcs201;sceneexit/roster/epoch retires.')
save(L/'r498-entry-source-contract.json',entry)
for oldn,newn in [('build-r497-player-candidate.ps1','build-r498-player-candidate.ps1'),('build-r497-ui-entry-consumer.ps1','build-r498-character-ui-consumer.ps1')]:
 s=(P/'tools'/oldn).read_text(encoding='utf-8').replace('r497','r498').replace('R497','R498').replace('r498a-library-build','r498b-library-build')
 if 'consumer'in newn:s=s.replace('ui-entry-consumer','character-ui-consumer').replace('ui_entry_consumer','character_ui_consumer').replace('r498b-fixed-event-consumer','r498-fixed-event-consumer').replace('r496-candidate.json','r497-candidate.json').replace('R496','R497').replace('195','198').replace("native_city_presentation.rs'))","native_character_selection.rs'))")
 (P/'tools'/newn).write_text(s,encoding='utf-8')
v=read(L/'r496-source-oracle.json');pe=Path(read(L/'r498-source-study.json')['analysis_PE']['path']).read_bytes();fmt=b'sscchhhcccccccccddddddddddcdddddc';bodies=[[3,4]]
for slot in range(4):
 body=bytearray([203]);body.extend(f'Role{slot}'.encode()+b'\0T\0')
 for i,c in enumerate(fmt[2:]):
  n={99:1,104:2,100:4}[c];val=slot if i==0 else (slot%2 if i==1 else 1);body.extend(val.to_bytes(n,'little'))
 bodies.append(list(body))
# Use pinned R496 source1 seed frame, then independently encrypt new 4-slot bodies.
assert v['setup_bodies'][0][0]==1
seedbody=bytes(v['setup_bodies'][0]);assert len(seedbody)==5;seed=(0x87546CA1<<32)|int.from_bytes(seedbody[1:],'little');wire=list((len(seedbody)+2).to_bytes(2,'little')+seedbody);counter=seed
for body in bodies:
 cipher,_=source_cipher(bytes(body),counter,pe);wire+=list((len(body)+2).to_bytes(2,'little')+cipher);counter=counter_add_adc(counter,len(body))
replacement=[3,4];cipher,_=source_cipher(bytes(replacement),counter,pe);rewire=list((len(replacement)+2).to_bytes(2,'little')+cipher)
save(L/'r498-source-oracle.json',dict(state='R498_NEW_FOUR_SOURCE_SLOT_FIXTURE_FROM_PINNED_SOURCE_SCHEMA_AND_CIPHER',previous=pin(L/'r496-source-oracle.json'),seed=seed,setup_bodies=[list(seedbody)]+bodies,setup_wire=wire,receive_after=counter,replacement_wire=rewire,synthetic_source_inputs=True,no_setters_or_provider_or_socket=True,GUI=0))
progress('R498_active_character_callback_source_mapping_and_runtime_scene','R497 finite accepted20JSON1177pins775unique. R498 closedsourceconstructor/callback/slotanchors/classvariantPNGtables, IMG51x23submit/back pinned. Singleclick stagesprivateScene;doubleclick/opaqueSubmit usesactualRuntime sender. Missing79/80IMG andPNG sprites remainexactgaps;sourcegeometrypartial. Newvalidation/freeze pending;GUI0/overallunfinished16of40.')
print('R498 new binding/candidate/source fixture prepared;198pairs612sources expected')
