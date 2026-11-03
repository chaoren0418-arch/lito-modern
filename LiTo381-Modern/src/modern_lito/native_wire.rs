//! LiTo TCP framing and cipher primitives, independent of UI/gameplay owners.
//! Protocol.CipherState: unsigned 64-bit state, little endian key, chained XOR.
use anyhow::{ensure, Context, Result};

pub const KEY_HIGH: u32 = 0x8754_6ca1;
pub const MAX_FRAME: usize = u16::MAX as usize;
pub const MAX_BODY: usize = MAX_FRAME - 2;

pub fn initial_counter(seed: u32) -> u64 {
    (u64::from(KEY_HIGH) << 32) | u64::from(seed)
}

/// Transform only; the caller commits the counter exactly once per body.
pub fn encrypt_in_place(counter: u64, body: &mut [u8]) {
    let key = counter.to_le_bytes();
    let mut previous = 0;
    for (i, byte) in body.iter_mut().enumerate() {
        *byte ^= key[i & 7] ^ previous;
        previous = *byte;
    }
}
pub fn decrypt_in_place(counter: u64, body: &mut [u8]) {
    let key = counter.to_le_bytes();
    let mut previous = 0;
    for (i, byte) in body.iter_mut().enumerate() {
        let cipher = *byte;
        *byte ^= key[i & 7] ^ previous;
        previous = cipher;
    }
}

#[derive(Clone, Debug)]
pub struct CipherState {
    counter: u64,
}
impl CipherState {
    pub fn new(seed: u32) -> Self {
        Self {
            counter: initial_counter(seed),
        }
    }
    pub fn counter(&self) -> u64 {
        self.counter
    }
    pub fn encrypt(&mut self, body: &mut [u8]) {
        encrypt_in_place(self.counter, body);
        self.counter = self.counter.wrapping_add(body.len() as u64);
    }
    pub fn decrypt(&mut self, body: &mut [u8]) {
        decrypt_in_place(self.counter, body);
        self.counter = self.counter.wrapping_add(body.len() as u64);
    }
}

pub fn frame(body: &[u8]) -> Result<Vec<u8>> {
    ensure!(body.len() <= MAX_BODY, "native body exceeds u16 frame");
    let mut out = Vec::new();
    out.try_reserve_exact(body.len() + 2)
        .context("native frame allocation")?;
    out.extend_from_slice(&((body.len() + 2) as u16).to_le_bytes());
    out.extend_from_slice(body);
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EofState {
    Clean,
    PartialHeader,
    PartialBody,
}

/// One frame per feed result. Retains only one bounded payload; callers resume
/// at consumed bytes for coalesced frames. Invalid framing permanently retires it.
pub struct FrameDecoder {
    header: [u8; 2],
    header_len: usize,
    expected: Option<usize>,
    body: Vec<u8>,
    max_frame: usize,
    retired: bool,
}
impl FrameDecoder {
    pub fn new(max_frame: usize) -> Result<Self> {
        ensure!(
            (2..=MAX_FRAME).contains(&max_frame),
            "invalid native frame limit"
        );
        Ok(Self {
            header: [0; 2],
            header_len: 0,
            expected: None,
            body: Vec::new(),
            max_frame,
            retired: false,
        })
    }
    pub fn eof_state(&self) -> EofState {
        if self.expected.is_some() {
            EofState::PartialBody
        } else if self.header_len != 0 {
            EofState::PartialHeader
        } else {
            EofState::Clean
        }
    }
    pub fn feed(&mut self, input: &[u8]) -> Result<(usize, Option<Vec<u8>>)> {
        ensure!(!self.retired, "retired native decoder");
        let result = self.feed_inner(input);
        if result.is_err() {
            self.retired = true;
        }
        result
    }
    fn feed_inner(&mut self, input: &[u8]) -> Result<(usize, Option<Vec<u8>>)> {
        let mut used = 0;
        if self.expected.is_none() {
            let n = (2 - self.header_len).min(input.len());
            self.header[self.header_len..self.header_len + n].copy_from_slice(&input[..n]);
            self.header_len += n;
            used += n;
            if self.header_len < 2 {
                return Ok((used, None));
            }
            let total = usize::from(u16::from_le_bytes(self.header));
            ensure!(
                (2..=self.max_frame).contains(&total),
                "invalid native frame length {total}"
            );
            self.expected = Some(total - 2);
            self.body
                .try_reserve_exact(total - 2)
                .context("native receive allocation")?;
        }
        let expected = self.expected.unwrap();
        let n = (expected - self.body.len()).min(input.len() - used);
        self.body.extend_from_slice(&input[used..used + n]);
        used += n;
        if self.body.len() < expected {
            return Ok((used, None));
        }
        self.header_len = 0;
        self.expected = None;
        Ok((used, Some(std::mem::take(&mut self.body))))
    }
}
