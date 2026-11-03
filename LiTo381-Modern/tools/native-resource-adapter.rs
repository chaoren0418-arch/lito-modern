//! Read-only diagnostic adapter, deliberately separate from Shared loaders.
use serde_json::Value;
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs,io::Read,ops::Range,path::{Component,Path,PathBuf}};
pub type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
pub fn hex(bytes:&[u8])->String {bytes.iter().map(|b|format!("{b:02x}")).collect()}
fn safe_path(root:&Path,path:&Path)->Result<()> {
    if !path.is_absolute() || path.components().any(|c|matches!(c,Component::ParentDir|Component::CurDir)) {return Err("Native path must be explicit absolute confined path".into());}
    if !path.starts_with(root) {return Err("Native snapshot path escapes explicit source root".into());}
    for ancestor in path.ancestors() {
        let metadata=fs::symlink_metadata(ancestor)?;
        #[cfg(windows)] {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {return Err("Native reparse path rejected".into());}
        }
        if metadata.file_type().is_symlink() {return Err("Native symlink rejected".into());}
    }
    if !path.canonicalize()?.starts_with(root.canonicalize()?) {return Err("Native resolved source escapes root".into());}
    Ok(())
}
pub fn read_pinned(root:&Path,pin:&Value,namespace:&str,limit:u64)->Result<Vec<u8>> {
    if pin["namespace"].as_str()!=Some(namespace) {return Err("explicit Native namespace mismatch".into());}
    let path=PathBuf::from(pin["path"].as_str().ok_or("missing pinned Native path")?);
    safe_path(root,&path)?;
    let size=pin["bytes"].as_u64().ok_or("missing Native size pin")?;
    if size==0 || size>limit {return Err("Native input budget exceeded".into());}
    let mut options=fs::OpenOptions::new();options.read(true);
    #[cfg(windows)] {use std::os::windows::fs::OpenOptionsExt;options.share_mode(1);}
    let mut file=options.open(&path)?;
    if file.metadata()?.len()!=size {return Err("Native opened size differs".into());}
    let mut bytes=Vec::new();bytes.try_reserve_exact(usize::try_from(size)?)?;
    file.by_ref().take(size+1).read_to_end(&mut bytes)?;
    if bytes.len() as u64!=size || !hex(&Sha256::digest(&bytes)).eq_ignore_ascii_case(pin["sha256"].as_str().ok_or("missing Native SHA pin")?) {return Err("Native opened bytes differ from pin".into());}
    Ok(bytes)
}
pub struct NativeArchive {
    pub namespace:&'static str,
    bytes:Vec<u8>,
    pub entries:BTreeMap<String,Range<usize>>,
}
pub fn parse_index(index:&[u8],pak_size:usize)->Result<BTreeMap<String,Range<usize>>> {
    if index.len()<4 {return Err("Native index truncated".into());}
    let count=u32::from_le_bytes(index[..4].try_into()?) as usize;
    if count==0 || count>250_000 || index.len()!=4+count*28 {return Err("Native index is not exact bounded old28".into());}
    let mut entries=BTreeMap::new();
    for record in index[4..].chunks_exact(28) {
        let raw=&record[4..24];let end=raw.iter().position(|b|*b==0).unwrap_or(20);
        if raw[end..].iter().any(|b|*b!=0) {return Err("Native archive name padding invalid".into());}
        let name=std::str::from_utf8(&raw[..end])?.to_ascii_lowercase();
        if name.is_empty() || !name.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'.'|b'_'|b'-')) {return Err("Native archive entry name invalid".into());}
        let offset=u32::from_le_bytes(record[..4].try_into()?) as usize;
        let size=u32::from_le_bytes(record[24..].try_into()?) as usize;
        let limit=offset.checked_add(size).ok_or("Native entry overflow")?;
        if size>8*1024*1024 || limit>pak_size || entries.insert(name,offset..limit).is_some() {return Err("Native entry bounds/duplicate rejected".into());}
    }
    Ok(entries)
}
impl NativeArchive {
    pub fn open(root:&Path,pins:&[Value],sprite:bool)->Result<Self> {
        let (namespace,stem)=if sprite {("NATIVE_LITO_LTSPRITE_V1","LTsprite")}else{("NATIVE_LITO_LTTILE_V1","LTtile")};
        let find=|extension:&str|->Result<&Value>{
            let name=format!("{stem}.{extension}");
            let candidates:Vec<_>=pins.iter().filter(|pin|pin["namespace"].as_str()==Some(namespace)&&pin["path"].as_str().is_some_and(|p|Path::new(p).file_name()==Some(std::ffi::OsStr::new(&name)))).collect();
            if candidates.len()!=1 {return Err("Native archive source must select exactly one pinned kind".into());} Ok(candidates[0])
        };
        let index=read_pinned(root,find("idx")?,namespace,8*1024*1024)?;
        let bytes=read_pinned(root,find("pak")?,namespace,256*1024*1024)?;
        let entries=parse_index(&index,bytes.len())?;
        Ok(Self{namespace,bytes,entries})
    }
    pub fn entry(&self,name:&str)->Result<&[u8]> {
        let range=self.entries.get(name).ok_or("required Native archive entry absent")?;
        if range.is_empty() {return Err("required Native archive entry empty".into());}
        Ok(&self.bytes[range.clone()])
    }
}
