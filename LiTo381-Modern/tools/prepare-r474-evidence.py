"""Read-only bounded original evidence; write only this Modern project's R474 files."""
import hashlib,json,re
from pathlib import Path
P=Path(__file__).resolve().parent.parent
L=P/'logs/continuation-20261003'; O=P/'target/r474-offline-map-flow'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def write(p,v):
 assert p.resolve().is_relative_to(P.resolve());assert not p.exists();p.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8')
s=json.loads((L/'r473-resource-snapshot.json').read_text(encoding='utf-8'))['pairs'][2]
idx,pak=[a['snapshot'] for a in s['archives']]
for a in s['archives']:
 assert pin(a['snapshot']['path'])==a['snapshot'];assert pin(a['original']['path'])==a['original']
decoded=O/'LTmapstr-k.tbl.decoded';assert pin(decoded)['sha256']=='E7A5D41C7CC3678E5038121DF5AF98F5544DC418AD0C4DF2EEC88F06981114FD'
lines=decoded.read_text(encoding='ascii').splitlines();assert int(lines[0])==67
mode={'petrace':0,'deathmatch':1,'questbattle':2,'siegewar':3,'timeattack':4,'honormatch':5,'town':6};choices=[]
for i,line in enumerate(lines[1:68]):
 if line.startswith((';','no exist#')):continue
 fields=line.split('#');choices.append(dict(mode=mode[fields[1].lower()],map_id=int(fields[0]),name=fields[2],source_row=i,raw_line=line))
coverage=json.loads((L/'mode-resource-coverage.json').read_text(encoding='utf-8'));assert len(coverage['rows'])==40
native_root=Path('D:/lineage開發/0-客戶端/LiTo/LTmap'); audit=[]
for r in coverage['rows']:
 found=[c for c in choices if c['mode']==r['mode'] and c['map_id']==r['logical_map']];assert len(found)==1
 files=sorted((native_root/str(r['logical_map'])).glob('*.s32'))
 audit.append(dict(mode=r['mode'],logical_map=r['logical_map'],original_table=found[0],v47_completed=r.get('v47_program_completed',False),v47_physical_binding=r['explicit_v47_fixture_physical_map'],native_directory=str(native_root/str(r['logical_map'])),native_sector_files=[pin(f) for f in files],modern_offline_preview_selected=r['mode']==1 and r['logical_map']==564,full_playable_mode_newly_accepted=False))
asm=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm');text=asm.read_text(encoding='utf-8');asm_pin=pin(asm);assert asm_pin['sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
source=[]
for address in ('46F9C0','46B7D0','464AD0','4D4450'):
 body=re.search(r'(?ms)^sub_'+address+r' proc near.*?^sub_'+address+r' endp[^\r\n]*',text).group();p=O/(address+'.asm');p.write_bytes(body.encode('utf-8'));source.append(pin(p))
v=json.loads((P/'target/r425-native-dm564-fixture/consumer-input.json').read_text(encoding='utf-8'));map_pin={k:v['native_map'][k] for k in ('path','bytes','sha256')};assert pin(map_pin['path'])==map_pin
original=native_root/'564/7fff8000.s32';assert pin(original)['sha256']==map_pin['sha256'];assert len(list((native_root/'564').glob('*.s32')))==1
for a in v['archive_pins']:
 assert pin(a['path'])=={k:a[k] for k in ('path','bytes','sha256')}
write(L/'r474-mode-map-binding.json',dict(state='ALL_40_ORIGINAL_TABLE_PAIRS_CROSS_CHECKED_NATIVE_DM564_OFFLINE_PREVIEW_SELECTED',coverage=pin(L/'mode-resource-coverage.json'),LTtext_index=idx,LTtext_pak=pak,LTtext_originals=[a['original'] for a in s['archives']],table_raw=pin(O/'LTmapstr-k.tbl.raw'),table_decoded=pin(decoded),record_index=245,member_offset=190888,declared_rows=67,active_rows=len(choices),choices=choices,original_disassembly=asm_pin,source_bodies=source,rows=audit,pending=24,completed=16,selected_provider=pin(P/'target/r425-native-dm564-fixture/consumer-input.json'),selected_native_map=map_pin,selected_original_map=pin(original),selected_archives=v['archive_pins'],common_full_mode_gaps=['All 24 lack explicit v47 Shared physical binding in preserved coverage','Original catalog plus Native sectors establishes resource identity, not complete Rust mode rules/spawn/network admission','Native full actor/UI/clock/lifetime/loading admission remains unqualified; R472 callbacks at 5000 unchanged'],offline_scope='Mode1/map564 single actual Native sector, static CPU preview, Modern inspection camera, entry/back/reentry owned lifecycle; no invented Shared alias',native_menu_source='46F9C0 category at record+4/mapID+20h/stride24h; 46B7D0 matches mode, emits mapID-500 to menu; 464AD0 selected map writes/request branch retained as source only',runtime_executed=False,GUI=0,renderer_LOADED=False))
# Extend exact source allowlists and freeze revisions for the two genuinely new files.
p=P/'tools/audit-continuation-source-scope.ps1';t=p.read_text(encoding='utf-8');assert '$newFiles.Count -ne 87' in t
t=t.replace('$newFiles.Count -ne 87','$newFiles.Count -ne 89').replace("'src/modern_lito/native_campaign_resource_study_tests.rs')","'src/modern_lito/native_campaign_resource_study_tests.rs','src/modern_lito/native_offline_map_flow.rs','src/modern_lito/native_offline_map_flow_tests.rs')").replace('source_files=566;','source_files=568;').replace('eighty-seven new Native sources.','eighty-nine new Native sources.');p.write_text(t,encoding='utf-8')
p=P/'tools/freeze-finite-appearance-study.ps1';t=p.read_text(encoding='utf-8');assert '|72|73)' in t;p.write_text(t.replace('|72|73)','|72|73|74)'),encoding='utf-8')
print(json.dumps(dict(active_rows=len(choices),pairs_checked=40,pending=24,native_pending_sectors=sum(len(r['native_sector_files']) for r in audit if not r['v47_completed']),selected_564_sectors=1)))
