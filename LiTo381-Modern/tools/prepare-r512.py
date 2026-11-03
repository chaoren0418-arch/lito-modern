from r491_common import *
import shutil,difflib
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源')
fields=read(B/'S0C-status-owner/fields.json')['fields'];vectors=[]
for f in ['AD368-r509-line043.plain.bin','AD368-r509-line073.plain.bin']:
 b=(B/'S0C-status-owner/frames'/f).read_bytes();vectors.append(dict(body_hex=b.hex(),expected={x['name_from_existing_server_method']:int.from_bytes(b[x['body_offset']:x['body_offset']+x['width']],'little',signed=x['server_semantic_type'].startswith('i')) for x in fields}))
b=bytearray([12])+bytearray(36)
for i,x in enumerate(fields):
 n=x['width'];v=[0x89abcdef,201,0xfedcba98,11,22,33,44,55,66,0x1234,0x5678,0x9abc,0xdef0,-17,0x87654321,29,7,-12345,-1,-127,12,99][i]
 b[x['body_offset']:x['body_offset']+n]=v.to_bytes(n,'little',signed=x['server_semantic_type'].startswith('i'))
vectors.append(dict(body_hex=b.hex(),expected={x['name_from_existing_server_method']:int.from_bytes(b[x['body_offset']:x['body_offset']+x['width']],'little',signed=x['server_semantic_type'].startswith('i')) for x in fields}))
c=read(L/'r512-inputs/server-channel-return-network-lifecycle.json');frames=read(c['network']['frames']['path']);save(L/'r512-test-inputs.json',dict(status_vectors=vectors,frames=frames,original_frames_pin=pin(c['network']['frames']['path']),oracle='Python fields/schema0 offsets; supplied saved encrypted bodies, no Rust output or Java/SQL execution'))
prior={e['path']:e for e in read(L/'r511-production-source-snapshot.json')['source_pins']};root=P/'target/r512-production-source';root.mkdir();pins=[];changes=[];diff=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);e=pin(p);e['snapshot']=str(d);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:changes.append(str(p));diff.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r512-source-difference.patch';patch.write_text('\n'.join(diff),'utf-8');save(L/'r512-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R511=pin(L/'r511-layer-composition-consumer-handoff.json'),GUI=0))
s=(P/'tools/build-r511-consumer.ps1').read_text('utf-8').replace('r511','r512').replace('layer-consumer','return-consumer');(P/'tools/build-r512-consumer.ps1').write_text(s,'utf-8')
root=P/'target/r512-player-entry-candidate';root.mkdir();main=root/'LiToModern-R512.rs';main.write_text((P/'target/r511-player-entry-candidate/LiToModern-R511.rs').read_text('utf-8').replace('r511','r512').replace('R511','R512'),'utf-8');v=read(L/'r511-entry-source-contract.json');v.pop('new_graphics',None);v['candidate_main']=pin(main);v['new_return']='AD368 receipt adapter + bounded return tokens; Native owner/GUI unqualified';save(L/'r512-entry-source-contract.json',v)
s=(P/'tools/build-r511-player.ps1').read_text('utf-8').replace('r511','r512').replace('R511','R512');(P/'tools/build-r512-player.ps1').write_text(s,'utf-8')
print('frozen',len(pins),'production files;',len(changes),'changes; new inputs prepared')
