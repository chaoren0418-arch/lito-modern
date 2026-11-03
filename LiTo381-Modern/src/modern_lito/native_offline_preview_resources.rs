//! Modern-owned pinned input for the candidate's offline UI. No source inference.
use super::{native_resources as r,native_offline_map_flow::{BoundMap,Catalog}};
use serde_json::{Value,json};
use anyhow::{ensure,Context,Result};
use std::path::{Path,PathBuf};
#[path="../../tools/native-resource-adapter.rs"]mod adapter;
#[derive(Clone,Debug)]pub struct Inputs {pub project:PathBuf,pub manifest:PathBuf,pub manifest_pin:r::BytePin}
fn bp(v:&Value)->Result<r::BytePin>{let h=v["sha256"].as_str().context("SHA absent")?;ensure!(h.len()==64,"SHA length");let mut sha256=[0;32];for(i,b)in sha256.iter_mut().enumerate(){*b=u8::from_str_radix(&h[2*i..2*i+2],16)?;}Ok(r::BytePin{bytes:v["bytes"].as_u64().context("byte count absent")?,sha256})}
fn read(root:&Path,p:&Value)->Result<Vec<u8>>{let mut p=p.clone();p["namespace"]=json!("MODERN_OWNED_OFFLINE_INPUT");adapter::read_pinned(root,&p,"MODERN_OWNED_OFFLINE_INPUT",256<<20).map_err(|e|anyhow::anyhow!("{e}"))}
pub(super) fn load(inputs:&Inputs)->Result<(Catalog,BoundMap)>{
 ensure!(inputs.project.is_absolute()&&inputs.manifest.starts_with(&inputs.project),"explicit Modern input root required");
 let p=json!({"path":inputs.manifest,"bytes":inputs.manifest_pin.bytes,"sha256":adapter::hex(&inputs.manifest_pin.sha256)});
 let m:Value=serde_json::from_slice(&read(&inputs.project,&p)?)?;
 let index=read(&inputs.project,&m["LTtext_index"])?;let pak=read(&inputs.project,&m["LTtext_pak"])?;
 let catalog=Catalog::from_lttext(true,&index,&pak,bp(&m["LTtext_index"])?,bp(&m["LTtext_pak"])?,bp(&m["table_decoded"])?)?;
 let v:Value=serde_json::from_slice(&read(&inputs.project,&m["selected_provider"])?)?;ensure!(v["native_map"]["logical_map"]==564&&v["native_map"]["namespace"]=="NATIVE_LITO_LTMAP_V1"&&v["native_map"]["sector"]=="7fff8000","exact R474 native564 binding");
 let map=Path::new(v["native_map"]["path"].as_str().context("map path absent")?);let a=v["archive_pins"].as_array().context("archives absent")?;ensure!(a.len()==4,"four original archive roles");
 let mut canonical=v["native_map"].clone();canonical["sha256"]=v["native_map"]["canonical_sha256"].clone();
 let plan=r::NativeAssetPlan{namespace:r::ResourceNamespace::NativeLitoV1,map_id:564,canonical_sha256:bp(&canonical)?.sha256,sectors:vec![r::SectorPin{key:r::NativeSectorKey{map_id:564,x:0x7fff,y:0x8000},pin:bp(&v["native_map"])?}],archives:a.iter().zip([r::ArchiveRole::TileIndex,r::ArchiveRole::TilePak,r::ArchiveRole::SpriteIndex,r::ArchiveRole::SpritePak]).map(|(v,role)|Ok(r::ArchivePin{role,pin:bp(v)?})).collect::<Result<_>>()?};
 let sources=r::NativeSources{map_root:map.parent().and_then(Path::parent).context("map root absent")?.to_owned(),archive_root:Path::new(a[0]["path"].as_str().context("archive path absent")?).parent().context("archive root absent")?.to_owned()};
 ensure!(sources.map_root.starts_with(&inputs.project)&&sources.archive_root.starts_with(&inputs.project),"owned snapshot roots required");
 Ok((catalog,BoundMap{mode:1,sources,plan}))
}
