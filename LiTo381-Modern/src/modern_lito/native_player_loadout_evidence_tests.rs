//! Offline LPL1 evidence only. No wire selection, authentication, navigation,
//! Physical.Source admission or renderer capability follows from these bytes.
use super::character_data::{self,ItemKind};
use anyhow::{ensure,Context,Result};
use serde_json::{Value,json,Map};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs,path::Path};
const DOMAIN:&[u8]=b"LITO-NATIVE-PLAYER-LOADOUT-PROJECTION\0";
const MAX:usize=2*1024*1024;
fn num(v:&Value)->Result<i64>{v.as_i64().context("loadout signed integer absent")}
fn text(v:&Value)->Result<&str>{v.as_str().context("loadout text absent")}
fn array(v:&Value)->Result<&Vec<Value>>{v.as_array().context("loadout collection absent")}
fn hex(b:&[u8])->String{b.iter().map(|x|format!("{x:02x}")).collect()}
fn sha(b:&[u8])->String{hex(&Sha256::digest(b))}
fn digest(b:&[u8])->[u8;32]{let mut h=Sha256::new();h.update(DOMAIN);h.update(b);h.finalize().into()}
fn signed_object(m:&Map<String,Value>)->Result<()>{for v in m.values(){num(v)?;}Ok(())}
struct Reader<'a>{bytes:&'a [u8],at:usize}
impl<'a> Reader<'a>{
    fn take(&mut self,n:usize)->Result<&'a [u8]>{ensure!(n<=self.bytes.len().saturating_sub(self.at),"Truncated loadout");let b=&self.bytes[self.at..self.at+n];self.at+=n;Ok(b)}
    fn u16(&mut self)->Result<usize>{Ok(u16::from_be_bytes(self.take(2)?.try_into()?) as usize)}
    fn u32(&mut self)->Result<u32>{Ok(u32::from_be_bytes(self.take(4)?.try_into()?))}
    fn i64(&mut self)->Result<i64>{Ok(i64::from_be_bytes(self.take(8)?.try_into()?))}
    fn boolean(&mut self)->Result<bool>{let b=self.take(1)?[0];ensure!(b<=1,"Nonboolean loadout flag");Ok(b==1)}
    fn name(&mut self)->Result<String>{let n=self.u16()?;ensure!((1..=128).contains(&n),"Loadout name bound");let b=self.take(n)?;ensure!(b.iter().all(|b|b.is_ascii_graphic()),"Noncanonical ASCII loadout name");Ok(std::str::from_utf8(b)?.to_owned())}
    fn count(&mut self,max:usize)->Result<usize>{let n=self.u16()?;ensure!(n<=max,"Loadout collection bound");Ok(n)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct ExpectedOwner{pub(super) source:u32,pub(super) object:i64,pub(super) epoch:i64,pub(super) channel:u32}
fn baseline(b:&[u8],j:&Value,o:ExpectedOwner)->Result<()> {
    ensure!((16..=character_data::MAX_FRAME_BYTES).contains(&b.len()),"Baseline length");let mut r=Reader{bytes:b,at:0};
    ensure!(r.take(4)?==b"LTM1"&&r.u32()? as usize==b.len()&&r.u32()?==1&&r.u16()?==15&&r.u16()?==0,"Baseline envelope");
    let reply=character_data::decode_body(&b[16..])?;
    ensure!(reply.context.key.source_id==o.source&&reply.context.key.object_id==o.object as u64&&reply.context.epoch==o.epoch as u64,"Nested baseline owner");
    let d=reply.result.map_err(|_|anyhow::anyhow!("Baseline unavailable"))?;let c=&d.character;let s=&d.stats;
    let character=json!({"objectId":c.key.object_id,"name":c.name,"characterClass":c.class,"sex":c.sex,"level":c.level});
    let names=["maxHp","maxMp","savedHp","savedMp","armorClass","strength","dexterity","constitution","intelligence","wisdom","charisma","experience","lawful","originalStrength","originalDexterity","originalConstitution","originalIntelligence","originalWisdom","food225"];
    let values=[s.max_hp,s.max_mp,s.saved_hp,s.saved_mp,s.armor_class,s.strength,s.dexterity,s.constitution,s.intelligence,s.wisdom,s.charisma,s.experience,s.lawful,s.original_strength,s.original_dexterity,s.original_constitution,s.original_intelligence,s.original_wisdom,s.food225];
    let persisted:Map<_,_>=names.into_iter().zip(values).map(|(k,v)|(k.to_owned(),json!(v))).collect();
    let items:Vec<_>=d.items.iter().map(|i|json!({"objectId":i.object_id,"template":{"catalogId":i.catalog_id,"kind":match i.kind{ItemKind::Weapon=>"WEAPON",ItemKind::Armor=>"ARMOR",ItemKind::Etc=>"ETC"},"nameToken":i.name_token,"inventoryGraphic":i.inventory_graphic,"groundGraphic":i.ground_graphic,"unitWeight":i.unit_weight},"quantity":i.quantity,"equipped":i.equipped,"enchantment":i.enchantment,"durability":i.durability,"identified":i.identified,"blessing":i.blessing,"sealed":i.sealed,"chargeCount":i.charge_count})).collect();
    ensure!(json!({"sourceId":o.source,"character":character,"persisted":persisted,"inventory":items})==*j,"Complete nested15 baseline differs from pinned source JSON");Ok(())
}
/// Naive DB-local Gregorian milliseconds; used only as a difference. No
/// timezone conversion or query epoch inferred from a local timestamp.
fn local_millis(s:&str)->Result<i64>{
    let b=s.as_bytes();ensure!(b.len()>=19&&b.len()<=26&&b[4]==b'-'&&b[7]==b'-'&&matches!(b[10],b' '|b'T')&&b[13]==b':'&&b[16]==b':',"DB local datetime syntax");
    let field=|a,z|->Result<i64>{let bytes=&b[a..z];ensure!(bytes.iter().all(u8::is_ascii_digit),"Datetime digits");Ok(std::str::from_utf8(bytes)?.parse()?) };
    let (y,m,d,h,min,sec)=(field(0,4)?,field(5,7)?,field(8,10)?,field(11,13)?,field(14,16)?,field(17,19)?);
    ensure!((1900..=2200).contains(&y)&&(1..=12).contains(&m)&&(0..24).contains(&h)&&(0..60).contains(&min)&&(0..60).contains(&sec),"Datetime range");
    let leap=|y:i64|y%4==0&&(y%100!=0||y%400==0);let lengths=[31,if leap(y){29}else{28},31,30,31,30,31,31,30,31,30,31];ensure!(d>=1&&d<=lengths[(m-1) as usize],"Gregorian date");
    let days:i64=(1900..y).map(|n|if leap(n){366}else{365}).sum::<i64>()+lengths[..(m-1) as usize].iter().sum::<i64>()+d-1;
    let frac=if b.len()==19{0}else{ensure!(b[19]==b'.'&&b.len()>20,"Datetime fraction");let digits=&b[20..];ensure!(digits.iter().all(u8::is_ascii_digit),"Datetime fraction digits");let micro=std::str::from_utf8(digits)?.parse::<i64>()?*10i64.pow((6-digits.len()) as u32);micro/1000};
    Ok((((days*24+h)*60+min)*60+sec)*1000+frac)
}
fn lifetime(f:&Value,ob:&Value,now:i64)->Result<(&'static str,i64)> {
    let start=num(&ob["queryStartedEpochMillis"])?;let uncertainty=num(&ob["dbClockUncertaintyMillis"])?;
    ensure!(start>=0&&(0..=60000).contains(&uncertainty)&&num(&ob["queryStartedNanos"])? >=0,"Observation clock facts");
    if now<start{return Ok(("UNQUALIFIED",-1));}
    let lighting=f["lighting"].as_bool().context("Lighting fact absent")?;if lighting{return Ok(("LIGHTING_UNQUALIFIED",-1));}
    if !f["absoluteLocalTime"].is_null(){
        let delta=local_millis(text(&f["absoluteLocalTime"])?)?.checked_sub(local_millis(text(&ob["dbNowLocal"])?)?).context("Datetime delta overflow")?;
        let cutoff=start.checked_add(delta).and_then(|x|x.checked_sub(uncertainty)).context("Cutoff overflow")?.max(0);
        return Ok((if now<cutoff{"ACTIVE"}else{"EXPIRED"},cutoff));
    }
    let rule=f["timeRulePresent"].as_bool().context("Time rule flag absent")?;
    if f["remainingSeconds"].is_null()||(rule&&f["timeRuleHours"].is_null()){return Ok(("UNQUALIFIED",-1));}
    if num(&f["remainingSeconds"])? >0{return Ok(("RELATIVE_UNQUALIFIED",-1));}
    if rule||num(&f["templateMaxUseSeconds"])? >0{return Ok(("UNSTARTED_OR_MISSING",-1));}
    Ok(("PERMANENT",-1))
}
fn verify_raw(j:&Value,o:ExpectedOwner)->Result<usize>{
    let inv=array(&j["source"]["inventory"])?;let flags=array(&j["equipped"])?;let lives=array(&j["lifetimes"])?;let raw=array(&j["source_equipment_instances"])?;
    ensure!(inv.len()==flags.len()&&inv.len()==lives.len()&&inv.len()==raw.len()&&inv.len()<=512,"Raw inventory coverage");
    let ob=&j["source_lifetime_observation"];ensure!(num(&ob["sourceId"])?==i64::from(o.source)&&num(&ob["objectId"])?==o.object&&ob["items"].as_object().context("Raw lifetime map absent")?.len()==inv.len(),"Raw lifetime owner/coverage");
    ensure!(text(&j["catalog_generation_hex"])?.len()==64&&text(&j["catalog_generation_hex"])?.bytes().all(|b|b.is_ascii_hexdigit()),"Catalog generation digest");
    let mut current=BTreeMap::<String,i64>::new();let mut source_ac=0i64;let mut cutoff=-1;let now=num(&j["evaluated_at_millis"])?;let mut changed_flags=0;
    for (i,item) in inv.iter().enumerate(){
        ensure!(raw[i]["source"]==*item&&raw[i]["template"]["metadata"]==item["template"],"Raw instance/template metadata drift");
        let id=num(&item["objectId"])?;ensure!(flags[i]["objectId"]==item["objectId"]&&lives[i]["itemObjectId"]==item["objectId"],"Current/source item identity");
        let f=&ob["items"][id.to_string()];ensure!(f["itemObjectId"]==item["objectId"]&&f["catalogId"]==item["template"]["catalogId"]&&f["kind"]==item["template"]["kind"]&&f["sourceEquipped"]==item["equipped"],"Raw lifetime instance mapping");
        let (state,end)=lifetime(f,ob,now)?;ensure!(lives[i]==json!({"itemObjectId":id,"state":state,"expiresAtMillis":end}),"Raw lifetime view not reproducible");
        let equipped=flags[i]["equipped"].as_bool().context("Current equipped flag absent")?;let baseline=item["equipped"].as_bool().context("Source equipped flag absent")?;changed_flags+=usize::from(equipped!=baseline);
        if equipped&&state=="ACTIVE"{cutoff=if cutoff<0{end}else{cutoff.min(end)};}
        // Narrow numerical cross-check of these fixed neutral-addon fixtures.
        // Never generalize to unrepresented card/dark/greater/stone/set rules.
        if equipped||baseline{
            let v=&raw[i];ensure!(num(&v["greater"])?==0&&v["dark"]["present"]==false&&array(&v["cards"])?.iter().all(|x|x==0)&&num(&v["stone"]["itemId"])?==0&&num(&v["stone"]["tier"])?==0&&array(&v["template"]["setIds"])?.is_empty(),"Nonneutral extra rules unqualified by this fixture checker");
            if item["template"]["kind"]=="ARMOR"{ensure!(num(&item["enchantment"])?==0,"Armor enchant formula outside neutral fixture study");}
            let values=v["template"]["base"]["values"].as_object().context("Raw template base absent")?;signed_object(values)?;
            if baseline{source_ac=source_ac.checked_add(num(&values["AC"])?).context("Source AC overflow")?;}
            if equipped{for (name,value) in values{let old=*current.get(name).unwrap_or(&0);current.insert(name.clone(),old.checked_add(num(value)?).context("Modifier overflow")?);}}
        }
    }
    let mods=j["modifiers"].as_object().context("Modifiers absent")?;
    for (name,value) in mods{ensure!(num(value)?==*current.get(name).unwrap_or(&0),"Raw current modifier differs from projection");}
    ensure!(current.keys().all(|k|mods.contains_key(k)),"Unrepresented raw modifier");
    ensure!(num(&j["effective"]["armorClass"])?==num(&j["source"]["persisted"]["armorClass"])?.checked_add(num(&mods["AC"])?).and_then(|x|x.checked_sub(source_ac)).context("Effective AC overflow")?,"Saved-total AC baseline correction");
    ensure!(num(&j["valid_until_millis"])?==cutoff,"Numeric equipped lifetime minimum");Ok(changed_flags)
}
fn numeric_current_at(j:&Value,now:i64)->Result<bool>{let start=num(&j["evaluated_at_millis"])?;let end=num(&j["valid_until_millis"])?;Ok(now>=start&&(end<0||now<end))}
pub(super) fn verify(b:&[u8],j:&Value,o:ExpectedOwner)->Result<Value>{
    ensure!((128..=MAX).contains(&b.len()),"LPL1 byte bound");let end=b.len()-32;ensure!(digest(&b[..end]).as_slice()==&b[end..],"LPL1 checksum");let mut r=Reader{bytes:&b[..end],at:0};
    ensure!(r.take(4)?==b"LPL1"&&r.u32()? as usize==b.len()&&r.u16()?==1,"LPL1 header");let supported=r.u16()?;ensure!(supported<=1&&j["fighter_supported"]==json!(supported==1),"Support flags");
    let actual=ExpectedOwner{source:r.u32()?,object:r.i64()?,epoch:r.i64()?,channel:r.u32()?};ensure!(actual==o&&o.source>0&&o.object>0&&o.epoch!=0,"Externally pinned owner differs");
    ensure!(j["owner"]==json!({"sourceId":o.source,"objectId":o.object,"epoch":o.epoch,"channelId":o.channel})&&text(&j["epoch_u64_hex"])?==format!("{:016x}",o.epoch as u64),"Opaque epoch/JSON owner");
    for field in ["equipment_revision","evaluated_at_millis","valid_until_millis"]{ensure!(r.i64()?==num(&j[field])?,"Revision/time mismatch");}ensure!(num(&j["equipment_revision"])? >=0,"Negative revision");
    ensure!(hex(r.take(32)?)==text(&j["owner_binding_hex"])?,"Owner binding bytes");let n=r.u32()? as usize;ensure!(n<=character_data::MAX_FRAME_BYTES,"Baseline overbound");baseline(r.take(n)?,&j["source"],o)?;
    for field in ["maximumHp","maximumMp","initialHp","initialMp","armorClass","strength","dexterity","constitution","intelligence","wisdom","charisma"]{ensure!(r.i64()?==num(&j["effective"][field])?,"Effective field mismatch");}
    let n=r.count(64)?;let mut mods=Map::new();for _ in 0..n{let name=r.name()?;ensure!(!mods.contains_key(&name),"Duplicate modifier");mods.insert(name,json!(r.i64()?));}ensure!(Value::Object(mods)==j["modifiers"],"Complete modifiers differ");
    let n=r.count(512)?;let mut flags=Vec::new();let mut lives=Vec::new();for _ in 0..n{let id=r.i64()?;flags.push(json!({"objectId":id,"equipped":r.boolean()?}));let state=r.name()?;ensure!(["UNQUALIFIED","PERMANENT","ACTIVE","EXPIRED","UNSTARTED_OR_MISSING","RELATIVE_UNQUALIFIED","LIGHTING_UNQUALIFIED"].contains(&state.as_str()),"Lifetime enum");lives.push(json!({"itemObjectId":id,"state":state,"expiresAtMillis":r.i64()?}));}ensure!(json!(flags)==j["equipped"]&&json!(lives)==j["lifetimes"],"Complete equipment/lifetimes differ");
    let n=r.count(512)?;let mut weapons=Vec::new();for _ in 0..n{let mut w=Map::new();for key in ["objectId","catalogId","retainedType","damageSmall","damageLarge","range","material","templateBlessing","maximumDamageChance","criticalChance"]{w.insert(key.to_owned(),json!(r.i64()?));}let f=f64::from_bits(u64::from_be_bytes(r.take(8)?.try_into()?));ensure!(f.is_finite(),"Nonfinite weapon multiplier");w.insert("criticalMultiplier".into(),json!(f));for key in ["attributeKind","attributeLevel"]{w.insert(key.to_owned(),json!(r.i64()?));}weapons.push(Value::Object(w));}ensure!(json!(weapons)==j["weapons"],"All carried weapon facts differ");
    ensure!(r.name()?==text(&j["weapon_state"])?&&r.i64()?==num(&j["weapon_object"])?&&r.name()?==text(&j["unsupported_reason"])?,"Weapon selection/reason differs");
    for name in ["strength","dexterity"]{ensure!(r.count(127)?==127,"Incomplete growth");let mut rows=Vec::new();for expected in 1..=127{let id=r.i64()?;ensure!(id==expected,"Growth ID order");rows.push(json!({"id":id,"hit":r.i64()?,"damage":r.i64()?}));}ensure!(json!(rows)==j["growth"][name],"Complete growth differs");}ensure!(r.at==end,"LPL1 trailing payload");
    let inv=array(&j["source"]["inventory"])?;let expected:Vec<_>=inv.iter().filter(|i|i["template"]["kind"]=="WEAPON").map(|i|i["objectId"].clone()).collect();ensure!(weapons.iter().map(|w|w["objectId"].clone()).collect::<Vec<_>>()==expected,"Weapon fact coverage/order");
    for (out,base,modifier) in [("maximumHp","maxHp","MAX_HP"),("maximumMp","maxMp","MAX_MP"),("strength","strength","STR"),("dexterity","dexterity","DEX"),("constitution","constitution","CON"),("intelligence","intelligence","INT"),("wisdom","wisdom","WIS"),("charisma","charisma","CHA")]{ensure!(num(&j["effective"][out])?==num(&j["source"]["persisted"][base])?.checked_add(num(&j["modifiers"][modifier])?).context("Base modifier overflow")?,"Effective base+modifier");}ensure!(j["effective"]["initialHp"]==j["effective"]["maximumHp"]&&j["effective"]["initialMp"]==j["effective"]["maximumMp"],"New-match maxima policy");
    let changed=verify_raw(j,o)?;
    Ok(json!({"source":o.source,"object":o.object,"epoch_u64_hex":format!("{:016x}",o.epoch as u64),"bytes":b.len(),"revision":j["equipment_revision"],"inventory":inv.len(),"weapons":weapons.len(),"growth_rows":254,"source_vs_current_flag_changes":changed,"effective_AC":j["effective"]["armorClass"],"MR":j["modifiers"]["MR"],"numeric_current_at_evaluation":numeric_current_at(j,num(&j["evaluated_at_millis"])?)?,"publication_intent_or_authentication_qualified":false}))
}
pub(super) fn expected(name:&str)->ExpectedOwner{ExpectedOwner{source:if name.starts_with("source42"){42}else{7},object:if name.contains("reselected"){10163}else{10275},epoch:if name.contains("reselected"){i64::MIN+1}else{i64::MIN},channel:0}}
fn fixture()->Value{let p=Path::new(env!("CARGO_MANIFEST_DIR"));serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r436-loadout-snapshot.json")).unwrap()).unwrap()}
fn pair()->(Vec<u8>,Value){let ptr=fixture();let root=Path::new(ptr["replay"]["root"].as_str().unwrap());(fs::read(root.join("source7-initial.lpl")).unwrap(),serde_json::from_slice(&fs::read(root.join("source7-initial.json")).unwrap()).unwrap())}
#[test]fn fixed_full_projection_rawfacts_and_source_isolation(){
    let ptr=fixture();let mut rows=Vec::new();for family in ["candidate","replay"]{let root=Path::new(ptr[family]["root"].as_str().unwrap());let manifest:Value=serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();let input=if family=="candidate"{root.join("evidence")}else{root.to_owned()};let mut cases=BTreeMap::new();
        for entry in fs::read_dir(&input).unwrap(){let p=entry.unwrap().path();if p.extension().is_none_or(|e|e!="lpl"){continue;}let name=p.file_stem().unwrap().to_str().unwrap();let b=fs::read(&p).unwrap();let json_bytes=fs::read(p.with_extension("json")).unwrap();for (path,bytes) in [(&p,&b),(&p.with_extension("json"),&json_bytes)]{let rel=path.strip_prefix(root).unwrap().to_str().unwrap().replace('\\',"/");let pin=manifest["files"].as_array().unwrap().iter().find(|e|e["path"]==rel).unwrap();assert_eq!(sha(bytes),pin["sha256"].as_str().unwrap().to_ascii_lowercase());assert_eq!(bytes.len(),pin["bytes"].as_u64().unwrap() as usize);}let j:Value=serde_json::from_slice(&json_bytes).unwrap();let mut row=verify(&b,&j,expected(name)).unwrap();row["family"]=json!(family);row["case"]=json!(name);row["projection_sha256"]=json!(sha(&b));row["full_source_facts_sha256"]=json!(sha(&json_bytes));rows.push(row);cases.insert(name.to_owned(),j);
        }
        assert_eq!(cases.len(),8);let a=&cases["source7-initial"];let cas=&cases["source7-after-CAS"];let other=&cases["source42-initial"];assert_ne!(a["owner_binding_hex"],other["owner_binding_hex"]);assert_eq!(cas["weapon_state"],"UNARMED");assert_eq!(cas["equipment_revision"],1);for name in ["source42-after-other-CAS","source42-after-other-armor-CAS"]{assert_eq!(cases[name]["equipped"],other["equipped"]);assert_eq!(cases[name]["effective"],other["effective"]);assert_eq!(cases[name]["equipment_revision"],0);}
        let armor=&cases["source7-after-real-armor-CAS"];assert_eq!(armor["equipment_revision"],2);assert_eq!(armor["effective"]["armorClass"],-3);assert_eq!(armor["modifiers"]["MR"],4);let timed=&cases["source7-actual-SQL-timed-fixture"];let cutoff=num(&timed["valid_until_millis"]).unwrap();assert!(numeric_current_at(timed,cutoff-1).unwrap());assert!(!numeric_current_at(timed,cutoff).unwrap());assert!(!numeric_current_at(timed,cutoff+1).unwrap());assert_eq!(timed["equipment_revision"],4);
    }
    assert_eq!(rows.len(),16);fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("logs/continuation-20261003/r436-loadout-consumer-details.json"),serde_json::to_vec_pretty(&json!({"state":"FIXED_LPL1_FULL_PROJECTION_AND_RAW_FACTS_VERIFIED_OFFLINE_ONLY","rows":rows,"projections":16,"raw_lifetimes_recomputed":true,"signed_AC_and_MR_source_CAS_isolation":true,"growth_rows":4064,"exclusive_numeric_deadline":true,"query_valid_then_publish_expired_intent_race_covered":false,"UNARMED_never_proves_original_equipment_intent":true,"source_and_current_flags_retained_separately":true,"neutral_addon_fixture_numerical_scope_only":true,"Rust_live_SQL_TLS":false,"authentication":false,"game_wire":false,"complete_Physical_Source":false,"LOADED":false,"GUI":0})).unwrap()).unwrap();
}
#[test]fn truncation_checksum_and_recomputed_owner_revision_rejections(){let (b,j)=pair();let owner=expected("source7-initial");for n in [0,16,100,127,b.len()-33,b.len()-1]{assert!(verify(&b[..n],&j,owner).is_err());}for (offset,bytes) in [(12,42u32.to_be_bytes().to_vec()),(16,10163i64.to_be_bytes().to_vec()),(24,(i64::MIN+1).to_be_bytes().to_vec()),(32,1u32.to_be_bytes().to_vec()),(36,99i64.to_be_bytes().to_vec())]{let mut v=b.clone();v[offset..offset+bytes.len()].copy_from_slice(&bytes);let end=v.len()-32;let hash=digest(&v[..end]);v[end..].copy_from_slice(&hash);assert!(verify(&v,&j,owner).is_err());}let mut bad=b.clone();bad[100]^=1;assert!(verify(&bad,&j,owner).is_err());let mut bad=b.clone();bad[10]=0x80;let end=bad.len()-32;let hash=digest(&bad[..end]);bad[end..].copy_from_slice(&hash);assert!(verify(&bad,&j,owner).is_err());}
#[test]fn raw_lifetime_calendar_order_and_precision_boundaries(){let ob=json!({"queryStartedEpochMillis":10000,"queryStartedNanos":3,"dbClockUncertaintyMillis":1000,"dbNowLocal":"2024-02-28 23:59:59.900000"});let f=json!({"lighting":false,"absoluteLocalTime":"2024-02-29 00:00:01.100000","remainingSeconds":0,"timeRulePresent":false,"timeRuleHours":null,"templateMaxUseSeconds":0});assert_eq!(lifetime(&f,&ob,9999).unwrap(),("UNQUALIFIED",-1));assert_eq!(lifetime(&f,&ob,10199).unwrap(),("ACTIVE",10200));assert_eq!(lifetime(&f,&ob,10200).unwrap(),("EXPIRED",10200));assert!(local_millis("2023-02-29 00:00:00").is_err());assert!(local_millis("2024-01-01 24:00:00").is_err());let mut f=f;f["absoluteLocalTime"]=Value::Null;f["remainingSeconds"]=json!(30);assert_eq!(lifetime(&f,&ob,10000).unwrap().0,"RELATIVE_UNQUALIFIED");f["remainingSeconds"]=json!(0);f["timeRulePresent"]=json!(true);assert_eq!(lifetime(&f,&ob,10000).unwrap().0,"UNQUALIFIED");f["timeRuleHours"]=json!(1);assert_eq!(lifetime(&f,&ob,10000).unwrap().0,"UNSTARTED_OR_MISSING");f["lighting"]=json!(true);assert_eq!(lifetime(&f,&ob,10000).unwrap().0,"LIGHTING_UNQUALIFIED");}
#[test]fn raw_instance_lifetime_and_filtered_unarmed_intent_are_separate(){let (_,j)=pair();let o=expected("source7-initial");let mut v=j.clone();v["source_lifetime_observation"]["sourceId"]=json!(42);assert!(verify_raw(&v,o).is_err());let mut v=j.clone();v["source_equipment_instances"][0]["template"]["metadata"]["catalogId"]=json!(999);assert!(verify_raw(&v,o).is_err());let mut v=j.clone();v["source_lifetime_observation"]["items"]["10304"]["absoluteLocalTime"]=json!("2026-10-04 07:00:00");assert!(verify_raw(&v,o).is_err());let mut filtered=j;filtered["weapon_state"]=json!("UNARMED");filtered["valid_until_millis"]=json!(-1);let now=num(&filtered["evaluated_at_millis"]).unwrap()+1;assert!(numeric_current_at(&filtered,now).unwrap());assert_eq!(filtered["source_lifetime_observation"]["items"]["10304"]["sourceEquipped"],true);/* Numeric freshness cannot mint publication-intent or auth authority. */}
