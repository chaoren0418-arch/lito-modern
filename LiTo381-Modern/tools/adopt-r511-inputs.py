from r491_common import *
import shutil
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源');p=B/'skill-pane-layer-composition-contract.json';e=pin(p);assert e['sha256']=='E288BB0EA62C8C014E50BB87542B3F800E30B18B10D96D073DFCF27EC7FF81C6'
pe=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe').read_bytes();pins={};seen=set();instructions=set()
def walk(v,scope):
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   q=Path(v['path']);key=(str(q),v['sha256'].upper());assert q.is_absolute()
   if key not in pins:a=pin(q);assert a['bytes']==v['bytes'] and a['sha256']==key[1],(v,a);pins[key]=a
   if q.suffix=='.json' and scope in q.parents and key not in seen:seen.add(key);walk(read(q),scope)
  if v.get('raw_backed',True) and 'va' in v and 'raw_hex' in v:
   code=bytes.fromhex(v['raw_hex']);at=int(v['va'],16)-0x400000;assert pe[at:at+len(code)]==code,(v['va'],v['raw_hex']);instructions.add((v['va'],v['raw_hex']))
  for x in v.values():walk(x,scope)
 elif isinstance(v,list):
  for x in v:walk(x,scope)
walk(read(p),B/'skill-pane-layer-composition');layerpins=list(pins.values());codecount=len(instructions)
for asset in read(p)['qualified_assets']:
 m=asset['member'];src=Path(m['source_pak']['path'])
 with src.open('rb')as f:f.seek(m['offset']);raw=f.read(m['bytes'])
 assert raw==Path(m['raw']['path']).read_bytes() and hashlib.sha256(raw).hexdigest().upper()==m['raw']['sha256']
q=Path(r'D:/lineage開發/3-lito開發/lito伺服器/server-channel-return-network-lifecycle.json');qe=pin(q);assert qe['sha256']=='A6FEADD3AC90D4DB53887C5299BDCA5F1D2A6F6D0747A7D6C342045D7E66CA0F'
sv=read(q);pins={};walk(sv,Path(sv['runtime_root']));networkpins=list(pins.values())
dest=L/'r511-inputs';dest.mkdir();shutil.copyfile(p,dest/p.name);shutil.copyfile(q,dest/q.name)
save(L/'r511-input-adoption.json',dict(layer_contract=e,layer_payloads=layerpins,verified_unique_fixed_PE_instruction_claims=codecount,qualified933_935_exact_pak_slices=True,network_contract=qe,network_payloads=networkpins,network_scope='Saved for later return/reconnect acceptance only; existing4epochs/58frames evidence, not new Modern execution; preserve masked4C_LOGIN and R5090C37/S0 boundary',previous_R510=pin(L/'r510-bin-graphics-consumer-handoff.json'),GUI=0,wire_runs=0))
v=read(P.parent/'監製進度.json');v.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R511_finite_batch9_layer_composition_in_progress',current_task='核對batch9圖層與IMG word規則；933/934已證部分接續CPU API，935外部舍入維持unsupported；保存返回重連網路證據，不重跑R509。');v['r511']=dict(state='input_pins_adopted',input_adoption=pin(L/'r511-input-adoption.json'),GUI=0);save(P.parent/'監製進度.json',v)
print('Layer pins',len(layerpins),'instruction claims',codecount,'network pins saved',len(networkpins))
