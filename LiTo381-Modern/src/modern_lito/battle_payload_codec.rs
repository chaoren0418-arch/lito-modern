//! Private, bounded schema primitives. No serde/native pointers or unchecked
//! enum conversions. Reads and count/write passes share the same exact layout.
use super::{invalid, ApplicationError, MAX_PRESENTATION_BYTES as MAX_BYTES};

pub(super) trait Wire: Sized {
    const LIMIT: usize = MAX_BYTES;
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError>;
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError>;
}
pub(super) struct Reader<'a> {
    pub schema: u16,
    bytes: &'a [u8],
    pub pos: usize,
    end: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            schema: super::SCHEMA,
            bytes,
            pos: 0,
            end: bytes.len(),
        }
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&v| v <= self.end)
            .ok_or_else(invalid)?;
        let value = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(value)
    }
    pub fn read<T: Wire>(&mut self) -> Result<T, ApplicationError> {
        let old = self.end;
        self.end = old.min(self.pos.saturating_add(T::LIMIT));
        let result = T::read(self);
        self.end = old;
        result
    }
    pub fn remaining(&self) -> usize {
        self.end - self.pos
    }
    pub fn finish(&self) -> Result<(), ApplicationError> {
        if self.pos == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid())
        }
    }
}
pub(super) struct Writer {
    pub schema: u16,
    pub bytes: Option<Vec<u8>>,
    pub pos: usize,
    end: usize,
}
impl Writer {
    pub fn count() -> Self {
        Self {
            schema: super::SCHEMA,
            bytes: None,
            pos: 0,
            end: MAX_BYTES,
        }
    }
    pub fn allocated(n: usize) -> Result<Self, ApplicationError> {
        if n > MAX_BYTES {
            return Err(invalid());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(n).map_err(|_| invalid())?;
        Ok(Self {
            schema: super::SCHEMA,
            bytes: Some(bytes),
            pos: 0,
            end: n,
        })
    }
    pub fn put(&mut self, bytes: &[u8]) -> Result<(), ApplicationError> {
        let end = self
            .pos
            .checked_add(bytes.len())
            .filter(|&v| v <= self.end)
            .ok_or_else(invalid)?;
        if let Some(out) = &mut self.bytes {
            out.extend_from_slice(bytes);
        }
        self.pos = end;
        Ok(())
    }
    pub fn write<T: Wire>(&mut self, v: &T) -> Result<(), ApplicationError> {
        let old = self.end;
        self.end = old.min(self.pos.saturating_add(T::LIMIT));
        let result = v.write(self);
        self.end = old;
        result
    }
    pub fn section<T: Wire>(&mut self, id: usize, value: &T) -> Result<(), ApplicationError> {
        self.put(&(id as u16).to_be_bytes())?;
        self.put(&0u16.to_be_bytes())?;
        let mark = self.pos;
        self.put(&0u32.to_be_bytes())?;
        let start = self.pos;
        let old = self.end;
        self.end = old.min(start.saturating_add(super::section_limit(id).ok_or_else(invalid)?));
        let result = self.write(value);
        self.end = old;
        result?;
        let len = u32::try_from(self.pos - start).map_err(|_| invalid())?;
        if let Some(bytes) = &mut self.bytes {
            bytes[mark..mark + 4].copy_from_slice(&len.to_be_bytes());
        }
        Ok(())
    }
}
macro_rules! scalar {
    ($ty:ty, $size:expr) => {
        impl Wire for $ty {
            fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
                Ok(Self::from_be_bytes(
                    r.take($size)?.try_into().map_err(|_| invalid())?,
                ))
            }
            fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
                w.put(&self.to_be_bytes())
            }
        }
    };
}
scalar!(i32, 4);
scalar!(i64, 8);
impl Wire for u8 {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        Ok(r.take(1)?[0])
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        w.put(&[*self])
    }
}
impl Wire for bool {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        match r.read::<u8>()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        w.write(&u8::from(*self))
    }
}
impl<T: Wire> Wire for Option<T> {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        if r.read::<bool>()? {
            Ok(Some(r.read()?))
        } else {
            Ok(None)
        }
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        w.write(&self.is_some())?;
        if let Some(v) = self {
            w.write(v)?;
        }
        Ok(())
    }
}
impl<const N: usize> Wire for [u8; N] {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        r.take(N)?.try_into().map_err(|_| invalid())
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        w.put(self)
    }
}

/// Layout declarations are intentionally explicit, ordered and local. They do
/// not derive admission, defaults, coercion, unknown-field skipping or Debug.
macro_rules! record {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => { record!($name [super::MAX_BYTES] { $($field:$ty),* }); };
    ($name:ident [$limit:expr] { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, PartialEq, Eq)]
        pub struct $name { $(pub $field: $ty),* }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(concat!(stringify!($name), "[redacted]")) }
        }
        impl Wire for $name {
            const LIMIT: usize = $limit;
            fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> { Ok(Self { $($field: r.read()?),* }) }
            fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> { $(w.write(&self.$field)?;)* Ok(()) }
        }
    };
}
pub(super) use record;
macro_rules! tags {
    ($name:ident { $($variant:ident = $tag:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(u8)]
        pub enum $name { $($variant = $tag),+ }
        impl Wire for $name {
            fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
                match r.read::<u8>()? { $($tag => Ok(Self::$variant),)+ _ => Err(invalid()) }
            }
            fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> { w.write(&(*self as u8)) }
        }
    };
}
pub(super) use tags;
