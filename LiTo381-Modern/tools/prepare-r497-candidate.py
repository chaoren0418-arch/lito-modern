from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
import struct
e=read(L/'r497-source-evidence.json');v=read(L/'r496-source-oracle.json');pe=Path(e['analysis_PE']['path']).read_bytes();raw=Path(e['entry_members'][1]['raw']['path']).read_bytes();bgra=bytearray()
for (p,)in struct.iter_unpack('<H',raw[8:]):bgra.extend(tuple(((c<<3)|(c>>2))for c in [p&31,(p>>5)&31,(p>>10)&31])+(255,))
art=P/'target/r497-ui-entry-study/6279.bgra';art.write_bytes(bgra)
ctr=v['vectors']['mode206']['receive_after'];deltas=[]
for mode,ident in [(255,777),(1,777),(3,778)]:
 body=[213,2]+list(ident.to_bytes(4,'little'))+[mode];c,_=source_cipher(bytes(body),ctr,pe);after=counter_add_adc(ctr,7);deltas.append(dict(plain=body,wire=list((9).to_bytes(2,'little')+c),before=ctr,after=after));ctr=after
save(L/'r497-source-oracle.json',dict(state='R497_SOURCE_RESOURCE_PIXEL_AND_TAGGED_RECEIVE_UI_REFRESH_ORACLE',previous_source_oracle=pin(L/'r496-source-oracle.json'),setup_wire=v['setup_wire'],setup_bodies=v['setup_bodies'],mode206_wire=v['vectors']['mode206']['wire'],deltas=deltas,entry_bgra=pin(art),BGRA_width=11,BGRA_height=11,synthetic_source_inputs=True,Native_GUI_or_socket_executed=False))
O=P/'target/r497-player-entry-candidate';O.mkdir(exist_ok=False);old=read(L/'r496-player-candidate.json');main=O/'LiToModern-R497.rs';main.write_text(Path(old['candidate_main']['path']).read_text(encoding='utf-8').replace('R496','R497').replace('r496','r497'),encoding='utf-8')
entry=read(L/'r496-entry-source-contract.json');entry['candidate_main']=pin(main);entry['skin_binding']=pin(L/'r497-native-menu-skin-binding.json');entry['state']='R497_VISIBLE_SOURCE_MAP_ENTRY_AND_RESOURCE_BADGE_REVISION_REFRESH_GATE';entry['UI_assets']='source6279 11x11badge+sameR478 map skin;actual Modern4910 BUTTON/WM_COMMAND. Immutablepresentation key gate prevents idlelabels/token/Invalidate;originalcharacter art/controls mapping notcomplete.';entry.pop('previous_R495_player_record',None);entry['previous_R496_player_record']=pin(L/'r496-player-candidate.json');save(L/'r497-entry-source-contract.json',entry)
for oldn,newn in [('build-r496-player-candidate.ps1','build-r497-player-candidate.ps1'),('build-r496-source-actor-consumer.ps1','build-r497-ui-entry-consumer.ps1')]:
 s=(P/'tools'/oldn).read_text(encoding='utf-8').replace('R496','R497').replace('r496','r497').replace('r497b-library-build','r497a-library-build')
 if 'consumer'in newn:s=s.replace('r495-candidate.json','r496-candidate.json').replace('R495 source snapshot','R496 source snapshot').replace('192','195').replace('source-actor-consumer','ui-entry-consumer').replace('source_actor_consumer','ui_entry_consumer').replace("native_source_actor.rs'))","native_city_presentation.rs'))")
 (P/'tools'/newn).write_text(s,encoding='utf-8')
progress('R497_active_source6279_map_entry_and_actual4910_invoker_revision_gated_refresh','R496 finiteaccepted19JSON1136pins742unique. R497 source46D148 receiver2 IMG1885/1887=>4764F0=>46B7D0 pinned;6277/6279 exist11x11. Sourcecharacter art/hit/control construction mappingOPEN;genericrolebuttons fallback. Actual4910 visibleModernbutton+source6279badge/sameR478skin;scope+skin+phase2+snapshot+mode0..5 guard. SharedpresentationKey Gate skips idlelabels/tokens/Invalidate;roster revision retained;newconsumer/oracle/snapshotpending. NativeSocket/auth/GUI0/full16of40 unchanged.')
print('R497 oracle/player scripts prepared195pairs611production expected;no fakeGUI/nativeUI claim')
