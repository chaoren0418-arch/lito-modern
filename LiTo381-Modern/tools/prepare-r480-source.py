from pathlib import Path
P=Path(__file__).resolve().parent.parent; S=P/'src/modern_lito'
copies=[('native_multi_map_resources','native_multisector_resources'),('native_offline_map_flow','native_multisector_flow'),('native_mode_menu_runtime','native_multisector_runtime'),('native_map_batch_window','native_multisector_window')]
for old,new in copies:
 t=(S/(old+'.rs')).read_text(encoding='utf-8')
 if new=='native_multisector_resources':
  t=t.replace('native_offline_map_flow','native_multisector_flow').replace('ensure!(s.len()==1,"current preview requires one complete actual sector");','ensure!(!s.is_empty()&&s.len()<=64,"complete Native sector count");').replace('#[derive(Clone,Debug)]pub struct Entry{pub choice:MapChoice,pub available:bool}','pub use super::native_multi_map_resources::Entry;')
  marker='  let a=v["archives"].as_array().context("archives")?;'
  validation='''  let authority=original["rows"].as_array().unwrap().iter().find(|r|r["logical_map"]==id&&r["mode"]==mode).context("whole-map authority absent")?;
  let exact=authority["native_sector_files"].as_array().context("complete sector authority")?;
  ensure!(exact.len()==s.len(),"incomplete Native map sector plan; no partial map preview");
  for actual in s{let name=Path::new(actual["pin"]["path"].as_str().unwrap()).file_name().context("sector name")?;let expected=exact.iter().find(|p|Path::new(p["path"].as_str().unwrap()).file_name()==Some(name)).context("sector outside full original map")?;ensure!(actual["pin"]["bytes"]==expected["bytes"]&&actual["pin"]["sha256"]==expected["sha256"],"sector differs from full original map authority");}
  ensure!(sectors.windows(2).all(|p|p[0].key<p[1].key),"canonical sector anchor/order required");
'''
  assert marker in t;t=t.replace(marker,validation+marker)
 elif new=='native_multisector_flow':
  a=t.index('#[derive(Clone, Debug, PartialEq, Eq)]');z=t.index('#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum Page')
  t=t[:a]+'pub use super::native_offline_map_flow::{MapChoice,Catalog,BoundMap};\n'+t[z:]
  t=t.replace('catalog.choices.iter()','catalog.choices().iter()').replace('self.catalog.choices.iter()','self.catalog.choices().iter()')
  t=t.replace('use anyhow::{','use super::native_multisector_scene::Scene;\nuse anyhow::{',1).replace('assets:Option<r::NativeStaticAssets>','assets:Option<Scene>').replace('assets:r::NativeStaticAssets','assets:Scene')
  t=t.replace('let canvas=assets.render_static(initial_camera(),cancel)?;','let assets=Scene::new(assets)?;\n        let canvas=assets.render(assets.initial_camera(),cancel)?;')
  t=t.replace('m.plan.sectors.len()==1 && m.plan.sectors[0].key.map_id==m.plan.map_id','!m.plan.sectors.is_empty() && m.plan.sectors.iter().all(|s|s.key.map_id==m.plan.map_id)').replace('explicit single Native sector required','explicit complete Native sector plan required')
  t=t.replace('ready.assets.verified_plan()','ready.assets.assets().verified_plan()')
  t=t.replace('self.assets=Some(ready.assets);self.camera=initial_camera();','self.camera=ready.assets.initial_camera();self.assets=Some(ready.assets);')
  start=t.index('        camera.x=(i64::from(camera.x)');end=t.index('        let revision=',start)
  t=t[:start]+'        camera=self.assets.as_ref().context("preview assets absent")?.pan_camera(camera,dx,dy)?;\n'+t[end:]
  t=t.replace('.render_static(camera,cancel)?','.render(camera,cancel)?')
  t=t[:t.index('#[cfg(test)]#[path="native_offline_map_flow_tests.rs"]')]
 elif new=='native_multisector_runtime':t=t.replace('native_multi_map_resources','native_multisector_resources').replace('native_offline_map_flow','native_multisector_flow')
 elif new=='native_multisector_window':t=t.replace('native_mode_menu_runtime','native_multisector_runtime').replace('R479','R480').replace('0x479','0x480')
 dest=S/(new+'.rs');assert not dest.exists();dest.write_text(t,encoding='utf-8')
mod=S/'mod.rs';t=mod.read_bytes();addition=b'\r\npub mod native_multisector_resources;\r\npub mod native_multisector_flow;\r\npub mod native_multisector_scene;\r\npub mod native_multisector_runtime;\r\npub mod native_multisector_window;\r\n#[cfg(test)]\r\nmod native_multisector_tests;\r\n';assert b'pub mod native_multisector_resources' not in t;mod.write_bytes(t+addition)
print('R480 separate resource/flow/runtime/window modules generated; old source modules unchanged')
