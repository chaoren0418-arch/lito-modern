"""Read original healing input provenance and native clothes105 submission; no client runs."""
from pathlib import Path
import capstone,hashlib,json,pefile,struct,wave,zipfile
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-shared-healing-family-source'
assert not OUT.exists();OUT.mkdir()
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
pe_path=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
pe_sha=sha(pe_path);assert pe_sha=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(str(pe_path),fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase;cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges={}
for name,start,end in [('parse105',0x43e4be,0x43e52e),('init_clothes',0x4e6ea0,0x4e6f6e),('submit_body_clothes_shadow',0x4e0200,0x4e025e),('submit_clothes',0x4e7000,0x4e70a2),('clothes_from_current_image',0x4e5efd,0x4e5f21),('store_current_subframe',0x4e604a,0x4e6059),('SPR_frame_submission',0x4cd9d0,0x4cdae4)]:
 b=pe.get_data(start-base,end-start);rows=list(cs.disasm(b,start));assert sum(x.size for x in rows)==len(b)
 ranges[name]={'start':hex(start),'end_exclusive':hex(end),'bytes':b.hex(),'sha256':hashlib.sha256(b).hexdigest().upper(),'instructions':[{'va':hex(x.address),'bytes':x.bytes.hex(),'mnemonic':x.mnemonic,'operands':x.op_str} for x in rows]}
by_addr={int(row['va'],16):(row['mnemonic'],row['operands']) for chunk in ranges.values() for row in chunk['instructions']}
anchors={0x43e4be:('cmp','edi, 0x69'),0x43e4e2:('mov','dword ptr [edx + esi*8], edi'),0x43e524:('mov','word ptr [edx], ax'),0x4e5f0d:('push','edi'),0x4e5f10:('call','0x4e6ea0'),0x4e6efa:('cmp','dword ptr [ecx + edx*4], 0'),0x4e6f25:('mov','dword ptr [ebx], edx'),0x4e6f2d:('mov','dx, word ptr [ecx + esi*2]'),0x4e0210:('movsx','eax, byte ptr [esi + 0x19]'),0x4e021d:('call','0x4cd9d0'),0x4e022e:('call','0x4e7000'),0x4e7077:('movsx','edx, byte ptr [esi + 0x19]'),0x4e707f:('call','0x4cd9d0'),0x4e6056:('mov','byte ptr [esi + 0x19], dl'),0x4cda01:('mov','esi, dword ptr [ebp + 8]'),0x4cda06:('jge','0x4cdad1'),0x4cda22:('mov','cx, word ptr [eax + 0x20]'),0x4cda28:('jge','0x4cdaca')}
for addr,expect in anchors.items():assert by_addr[addr]==expect,(hex(addr),by_addr[addr])
clocks=[]
for skill,expected in [(1,10),(19,100),(35,200),(57,500)]:
 va=0x635774+(skill-1)*2;b=pe.get_data(va-base,2);assert struct.unpack('<H',b)[0]==expected
 clocks.append({'skill':skill,'va':hex(va),'bytes':b.hex(),'client_clock_ms':expected})
old=ROOT/'驗證紀錄/current/native-effect-native-clock/common-finite-followup/protocol-effect-id-audit';defs=read(old/'TW-selected-definitions.json');selected={x['graphics']:x for x in defs['definitions']};assert '105.clothes(1 11726)' in selected[11725]['definition'];assert '0.effect' not in selected[11726]['definition'];assert '102.type(0)' in selected[11726]['definition'] and '104.attr(8)' in selected[11726]['definition']
assert sha(defs['pack'])==defs['pack_sha256']=='B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218'
resources=[];wanted={f'{i}-0.spr'.encode() for i in range(11722,11727)}
for root in [Path(r'D:\lineage開發\0-客戶端'),Path(r'D:\lineage開發\0-客戶端-公版260707修正門跟icon')]:
 entries=[]
 for idx in sorted(root.glob('Sprite*.idx')):
  raw=idx.read_bytes();count=struct.unpack_from('<I',raw)[0];assert len(raw)==4+28*count
  for i in range(count):
   offset,name,size=struct.unpack_from('<I20sI',raw,4+i*28);name=name.split(b'\0')[0].lower()
   if name not in wanted:continue
   pak=idx.with_suffix('.pak');assert 0<size<1024*1024 and offset+size<=pak.stat().st_size
   with pak.open('rb') as f:f.seek(offset);payload=f.read(size)
   assert len(payload)==size
   entries.append(dict(member=name.decode(),index=idx.name,index_sha256=hashlib.sha256(raw).hexdigest().upper(),offset=offset,bytes=size,sha256=hashlib.sha256(payload).hexdigest().upper()))
 assert {x['member'].encode() for x in entries}==wanted and len(entries)==5
 layer=next(x for x in entries if x['member']=='11726-0.spr');assert layer['bytes']==44329 and layer['sha256']=='EFC16BA8E0778312AB474AA269915D3B6D3551B2475C02D8A3FAE26FFFFDE4AA'
 resources.append(dict(root=str(root),entries=entries))
assert {x['member']:x['sha256'] for x in resources[0]['entries']}=={x['member']:x['sha256'] for x in resources[1]['entries']}
sounds=[]
for sid in [109,114,115,126]:
 p=Path(r'D:\lineage開發\0-客戶端\Sound')/(str(sid)+'.wav')
 with wave.open(str(p),'rb') as w:
  assert (w.getnchannels(),w.getsampwidth(),w.getframerate(),w.getcomptype())==(1,2,22050,'NONE');frames=w.getnframes();pcm=w.readframes(frames);assert len(pcm)==frames*2
 sounds.append(dict(sound=sid,path=str(p),bytes=p.stat().st_size,sha256=sha(p),channels=1,rate=22050,bits=16,PCM_frames=frames,PCM_sha256=hashlib.sha256(pcm).hexdigest().upper()))
inputs=ROOT/'驗證紀錄/current/native-shared-basic-heal/final-source-inputs.json';pins=read(inputs);assert len(pins)==968
for n,h in pins.items():assert sha(ROOT/n)==h
raw=dict(scope='Static original-PE instructions and raw asset provenance only; no original/native client execution',original_PE_path=str(pe_path),original_PE_sha256=pe_sha,ranges=ranges,anchors_checked=len(anchors),client_clocks=clocks,selected_healing_definitions=[selected[i] for i in range(11722,11727)],SPR_sources=resources,WAV_PCM_sources=sounds,current_compilation_inputs_unchanged=968,current_source_manifest_sha256=sha(inputs),actual_game_sessions=0,SPR_decode_verified=False,clothes_runtime_playback_verified=False,physical_audio_verified=False,whole_goal_complete=False)
(OUT/'raw.json').write_text(json.dumps(raw,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files={'auditor.py':Path(__file__),'raw.json':OUT/'raw.json','selected-definitions.json':old/'TW-selected-definitions.json','prior-codec-qualification.json':old/'qualified.json'};fp={n:sha(p) for n,p in files.items()};archive=OUT/'raw-source-audit.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
 for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
 for n,h in fp.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
proof=dict(status='verified_original_healing_clothes105_submission_and_all_five_asset_inputs_only',original_PE_sha256=pe_sha,raw_sha256=sha(OUT/'raw.json'),client_clocks=clocks,clothes105_parser_and_same_subframe_submission_verified=True,full_heal_layer11726_exists_in_both_readonly_roots=True,layer11726_bytes=44329,layer11726_sha256='EFC16BA8E0778312AB474AA269915D3B6D3551B2475C02D8A3FAE26FFFFDE4AA',all_five_SPR_inputs_match_between_roots=True,four_original_WAV_PCM_inputs_verified=True,game_sessions=0,SPR_decode_verified=False,clothes_runtime_playback_verified=False,global_cross_object_draw_order_verified=False,physical_audio_verified=False,whole_goal_complete=False,archive_file_pins=fp,archive_sha256=sha(archive),archive_readback_verified=True)
(OUT/'qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:proof[k] for k in ['status','full_heal_layer11726_exists_in_both_readonly_roots','game_sessions','SPR_decode_verified']}))
