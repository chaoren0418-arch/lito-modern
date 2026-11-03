"""Stage the missing current S6B/S6C receiver without changing the frozen normal tree."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
candidate=ROOT/'維護工具/candidates/native44FE-ability-buff-requests';prep=read(candidate/'preparation.json')
codec=V/'native44FE-ability-buff-requests/codecs-20261018-185336-838208';cp=read(codec/'readback.json');assert sha(codec/'codec.stdout')==cp['raw_files']['codec.stdout']
data=read(codec/'codec.stdout');assert data['level_ability_enabled']is False and data['cases'][0]['hex']=='6cb0040f05'and data['cases'][3]['hex']=='6bb004123205'
original=P/'src/modern_lito/native_world.rs';raw=original.read_bytes();s=raw.decode('utf-8-sig');(candidate/'native_world.rs.original').write_bytes(raw)
old='    pub attribute_updates: [u64;4],';assert s.count(old)==1
s=s.replace(old,old+'\n    /// Last active server ability receipts for this local identity; no inferred expiry.\n    pub ability_buffs: BTreeMap<u8,super::native_ability_buffs::Update>,')
old='            0x50|0x51|0x52|0x56=>{';assert s.count(old)==1
s=s.replace(old,'''            0x6b|0x6c=>{
                let update=super::native_ability_buffs::parse(b)?;
                if let Some(status)=&mut self.status {
                    let count=self.attribute_updates[0].checked_add(1).context("ability receipt counter exhausted")?;
                    if op==0x6b {status.strength=update.score;status.weight_gauge=update.weight_gauge.unwrap();}
                    else {status.dexterity=update.score;}
                    self.attribute_updates[0]=count;
                    if update.kind==1&&update.received_seconds==0 {self.ability_buffs.remove(&op);}
                    else {self.ability_buffs.insert(op,update);}
                }else{self.orphan_updates+=1;}
            }
'''+old)
old='                    self.magic_modifiers=None;self.attribute_updates=[0;4];';assert s.count(old)==1
s=s.replace(old,old+'self.ability_buffs.clear();')
(candidate/'native_world.rs').write_text(s,encoding='utf-8',newline='\n')
prep.update(world_files={n:sha(candidate/n)for n in ['native_world.rs','native_ability_buffs.rs','ability_world_probe.rs']},base_world_sha256=sha(original),
    current_codec_evidence=str(codec),current_codec_readback_sha256=sha(codec/'readback.json'),default_schema_S50_ability_updates_enabled=False,
    world_integration_typecheck_verified=False,world_integration_executed=False,scope='Requests26/42 component passed; newly staged exact S6B/S6C authoritative scores/duration receipts, stop removal, orphan rejection and identity/world reset. Full library and resource/live probes not yet run.')
(candidate/'preparation.json').write_text(json.dumps(prep,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(candidate),missing_normal_opcodes=['0x6B','0x6C'],normal_modified=False,world_integration_executed=False),ensure_ascii=False))
