from r491_common import *
from PIL import Image
snapshot=read(L/'r510c-production-source-snapshot.json');results=read(L/'r510-new-consumer-results.json');adoption=read(L/'r510-input-adoption.json');consumer=read(L/'r510c-consumer-candidate.json');player=read(L/'r510c-player-candidate.json')
assert results['new_comparisons']==42 and not results['native_skill_owner_installed'] and results['GUI']==0
assert consumer['production_library']['sha256']==player['production_library']['sha256']
checked={}
def check(e):
 key=(e['path'],e['sha256']);actual=checked.get(key)
 if not actual:actual=pin(e['path']);assert actual=={k:e[k] for k in ['path','bytes','sha256']},(e,actual);checked[key]=actual
for e in snapshot['source_pins']:
 check(e);assert pin(e['snapshot'])['sha256']==e['sha256'];checked[(e['snapshot'],e['sha256'])]=pin(e['snapshot'])
for e in adoption['verified_payload_pins']+adoption['contracts']:check(e)
for v in [consumer,player]:
 for e in v.values():
  if isinstance(e,dict) and all(k in e for k in ['path','bytes','sha256']):check(e)
build=[json.loads(s) for s in (L/'r510c-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert any(e.get('reason')=='build-finished' and e['success'] for e in build)
assert (L/'r510c-consumer.stderr.log').stat().st_size==0
# Decode the emitted LE16 CPU result for a reference PNG. No image edits,
# native capture or liveGUIoracle are involved.
previews=[]
for name,shifts,bits in [('555',[10,5,0],[5,5,5]),('565',[11,5,0],[5,6,5])]:
 raw=(L/f'r510-cpu/grid-{name}.le16').read_bytes();rgba=bytearray()
 for i in range(0,len(raw),2):
  w=int.from_bytes(raw[i:i+2],'little')
  for s,b in zip(shifts,bits):
   v=(w>>s)&((1<<b)-1);rgba.append((v<<(8-b))|(v>>(2*b-8)))
  rgba.append(255)
 path=L/f'r510-cpu/grid-{name}-reference.png';Image.frombytes('RGBA',(640,480),bytes(rgba)).save(path);previews.append(pin(path))
out=[pin(p) for p in sorted((L/'r510-cpu').glob('*')) if p.is_file()]
boundaries=dict(
 API='Production native_skill_graphics module registered in modern_lito, explicit conditional CPU inspection only. Default player/network/worldWindow behavior remains unchanged; no skills learned/owner/registry readiness comes from graphics.',
 icon='Namespace LtIcon versus SharedTbt retained. Missing-LTicon fallback is shared Sprite/91.tbt, not LTicon/91.lbt; no runtime fallback selection installed here. Five real samples independently match10sealed normal word destinations including coveredzero; raw555 immutable andconverted565 cache distinct; second conversion and555 destination for565 cache refuse.',
 blit='Normal signedi8 offsets, skip2*floor(skip/2),rowpitch2*floor(pitch/2); timed exact byte skip/pitch and color0 halfchannels. Both copy every coveredword includingblack; no percell42clip, scaling oralpha/colorkey. Destination safety validates beforemutation; malformed/negativeheight/payloadbudget/outofbounds cases are Modern policy, not originalunsafe behavior claims.',
 initialization='Complete6x1024timedtable words and2x512selectiontable words matchsealed LE16 outputs. Format immutableperTables; no native641298/global initialization reach/order guarantee. Syntheticoddskip/pitch uses source cache in declaredpixel format rather than raw555 loader input.',
 grid='34px4columns6rows max24, positionrelative startslot, placeholder consumesposition; normal/timedthenwhole34x34selectedrectangle at(x-1,y-1) includingbackground. No choice_none availability dim orfixedborder inferred. New2-format composite checkedagainst independentPython usingsealedexternal spanwords andLUT, not Rustderivedoutput.',
 geometry='ExactLTsurf934 nativecandidate SHA1A3691...,207x289,width207 yieldschild433,0/grid470,43/owner470,43,607,248 underdrag/G0. Explicitdrag/inputowner coordinates anddrawGseparate; rawinputhelperdoesnot subtractG. Bottominputpixel can mapindex24 despiteonly24renderedslots; rawhelpernotclickauthority. Scroll4/page36/slider/directcallback plusregistrydraw3to0/input0to3 qualifiedstatic only.',
 frame='Frame934 SHAcheckedbounds andreferenceRGBA SHA matchsealed policy; noNativeIMGblitter equivalence or934 substitute forconditional933offset8,10/935 layer. Existingworld_skill_panel shared2057/193x282/5over4 UI unchanged; different resourcebinding remains explicit until liveowner/input/layers qualified.',
 sources='Fourtopcontracts pinned;237currentdirect/batchpayloadpins and4165uniquePErawinstructionclaims verified. Earliercontracts/manifests onlydirectpins; initialoverbroad archival recursionfoundmutableglobalcompleteness-audit/handoff changes andnonabsolutehistoricalreference. Failuremetadata preserved and excluded; nochangedassetpin silentlyaccepted.',
 runtime='42newModernCPUcomparisons withexistingindependentcodevectors, not replayedBIN producer PASS/nativeGUI orR509wirecases. NoJavaSQL started thismilestone, wireframes0, playerexecutions0,GUI0,16of40unchanged.',
 remaining='R509firstopcode0C/body37owner; actor205/world/skills andinitialS1E readiness; live641298/pitch/surfacevmethods/initializationorder; full933/935framebinding; dynamicG/OSinputnormalization; actualpane registrymembership/privateerrorpaths andNativeGUI allremainopen. Fullreturn/reconnect remainsfuture finite work.'
)
save(L/'r510-contract-boundaries.json',boundaries)
refs=[pin(L/n) for n in ['r510-input-adoption.json','r510c-production-source-snapshot.json','r510c-source-difference.patch','r510c-library-build.jsonl','r510c-library-build.stderr.log','r510c-player-candidate.json','r510c-consumer-candidate.json','r510c-consumer-build.stderr.log','r510c-consumer.stdout.log','r510c-consumer.stderr.log','r510-new-consumer-results.json','r510-independent-grid-inputs.json','r510-independent-grid-555.le16','r510-independent-grid-565.le16','r510-contract-boundaries.json','r510-library-build.jsonl','r510-library-build.stderr.log','r510-transitive-pin-failures.json']]
for e in refs+out:check(e)
toolpins=[pin(p) for p in sorted((P/'tools').glob('*r510*')) if p.is_file()]
for e in toolpins:check(e)
save(L/'r510-sealed-link-verification.json',dict(verified_unique_file_pins=len(checked),current_source_pairs=len(snapshot['source_pins']),files=list(checked.values()),GUI=0,recursive_scope=adoption['recursive_scope']))
handoff=dict(schema='modern.R510.finite-BIN-batches5to8-graphics-consumer.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),state='FINITE42_NEW_CPU_COMPARISONS_PASS_NATIVE_GUI_AND_SKILL_OWNER_OPEN',new_Modern_comparisons=42,production_source_count=len(snapshot['source_pins']),changed_source_files=snapshot['changes'],same_player_consumer_library_SHA=consumer['production_library']['sha256'],player=player,consumer=consumer,contracts=adoption['contracts'],references=refs,link_verification=pin(L/'r510-sealed-link-verification.json'),outputs=out,tools=toolpins,boundaries=boundaries,previous_R509=pin(L/'r509-character-selection-channel-handoff.json'),wire_frames=0,Java_SQL_runs=0,GUI=0,player_executions=0,historical_PASS_reruns=0,overall_maps='16/40 unchanged')
save(L/'r510-bin-graphics-consumer-handoff.json',handoff)
report=P/'project/Native-R510_BIN圖像批次採用交接.txt';report.write_text('R510：BIN batch5..8確定部分採入production native_skill_graphics CPU檢視API，42項新的Modern比較PASS。\n五真實樣本／兩格式完整word目的緩衝區、完整8份色彩表、限時oddskip/pitch、34px格線與scroll/input邊界、934原始SHA/207x289/433origin、原始555與565cache、placeholder/wholecell selection及原子拒絕均有限核對。\n主框referenceRGBA及格線word輸出附CPU預覽；沒有Native GUI像素oracle，尚未接上技能owner／live registry／933/935其餘框層／OS G normalization。R509成功receipt後首0C/body37仍開放。\n624來源中2檔變更；玩家候選與consumer同rlib、玩家執行0、GUI0、JavaSQL0、wire0、16/40不變。\n詳見'+str(L/'r510-bin-graphics-consumer-handoff.json')+'\n',encoding='utf-8')
q=P.parent/'監製進度.json';v=read(q);v.update(updated_at=handoff['updated_at'],phase='R510_finite42_BIN_graphics_CPU_adoption_sealed_GUI0',current_task='R5099項及R51042項新驗證已封存；BIN5..8採入production CPU檢視API，GUI0／16of40。0C37技能owner、返回重連、live框層與輸入仍開放。');v['r510']=dict(state=handoff['state'],handoff=pin(L/'r510-bin-graphics-consumer-handoff.json'),new_comparisons=42,source_files=624,GUI=0,Java_SQL_runs=0,wire_frames=0);save(q,v)
print(json.dumps(dict(handoff=pin(L/'r510-bin-graphics-consumer-handoff.json'),new_comparisons=42,unique_pins=len(checked),source_files=624,GUI=0),ensure_ascii=False))
