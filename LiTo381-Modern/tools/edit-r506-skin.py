from r491_common import *
p=P/'src/modern_lito/native_mode_menu_skin.rs';s=p.read_text(encoding='utf-8')
s=s.replace('pub character_buttons:Vec<Img>,','pub character_buttons:Vec<Img>,pub channel_buttons:Vec<Img>,pub channel_background:Option<Img>,pub channel_back:Vec<Img>,')
marker='  ensure!(table.len()==1623,'
a=s.index(marker);s=s[:a]+'''  let load_channel=|row:&Value,id:u32,geometry:(u32,u32),mode:CopyMode|->Result<Img>{ensure!(row["id"]==id,"channel image identity");let range=table.get(&format!("{id}.img")).context("channel image absent")?;let raw=&pak[range.clone()];ensure!(raw.len()as u64==row["bytes"].as_u64().context("channel size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("channel SHA")?,"channel image pin drift");let image=decode_img(raw,mode)?;ensure!((image.width,image.height)==geometry,"channel geometry");Ok(image)};
  let channel_background=m.get("channel_background").map(|r|load_channel(r,7580,(640,480),CopyMode::Opaque)).transpose()?;
  let mut channel_buttons=Vec::new();if let Some(rows)=m.get("channel_buttons"){let rows=rows.as_array().context("channel buttons")?;ensure!(rows.len()==8,"four channel normal/pressed pairs");for(row,id)in rows.iter().zip(7581..7589){channel_buttons.push(load_channel(row,id,(72,69),CopyMode::ColorKey)?);}}
  let mut channel_back=Vec::new();if let Some(rows)=m.get("channel_back"){let rows=rows.as_array().context("channel Back")?;ensure!(rows.len()==2,"channel Back normal/pressed pair");for(row,id)in rows.iter().zip([6205,6206]){channel_back.push(load_channel(row,id,(51,23),CopyMode::ColorKey)?);}}
'''+s[a:]
s=s.replace('entry_art,character_buttons,thumb,','entry_art,character_buttons,channel_buttons,channel_background,channel_back,thumb,')
p.write_text(s,encoding='utf-8')
p=P/'src/modern_lito/native_entry_transfer.rs';s=p.read_text(encoding='utf-8')
s=s.replace('pub port_word:u16,pub at_110','pub port_word:u16,pub at_10c:u32,pub at_110')
s=s.replace('at_110:u32::from_le_bytes(fields[6..10].try_into().unwrap()),at_114:u32::from_le_bytes(fields[10..14].try_into().unwrap())','at_10c:if i==1{u32::from_le_bytes(fields[6..10].try_into().unwrap())}else{0},at_110:u32::from_le_bytes(fields[if i==1{10}else{6}..if i==1{14}else{10}].try_into().unwrap()),at_114:if i==1{0}else{u32::from_le_bytes(fields[10..14].try_into().unwrap())}')
s=s.replace('self.key.primary_return=true;self.key.alternate_return=false;','self.key.primary_return=true;')
p.write_text(s,encoding='utf-8')
print('R506 channel images loaded with exact pins and geometry; first/second row layouts separated')
