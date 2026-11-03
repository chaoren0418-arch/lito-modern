//! Small caller-owned shared-icon loader for the match-local shop. No UI or TLS
//! thread reads an archive. Missing art is explicit and does not alter authority.
use super::inventory_icons::{IconSource, SharedInventoryIcons, TbtImage};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::{self, JoinHandle};

pub const MAX_ICONS: usize = 9;
const MAX_PIXELS: usize = MAX_ICONS * 512 * 255;

#[derive(Clone, Debug)]
pub struct Icon {
    pub source: IconSource,
    pub width: u32,
    pub height: u32,
    /// Natural-size, opaque BGRA preview over the shop's dark background.
    /// RGB555-zero follows the existing qualified TBT reference preview policy.
    pub bgra: Vec<u8>,
}
impl Icon {
    fn from_image(source: IconSource, image: &TbtImage) -> Result<Self, &'static str> {
        let rgba = image.reference_rgba().map_err(|_| "圖示解碼失敗")?;
        let mut bgra = rgba;
        for pixel in bgra.chunks_exact_mut(4) {
            if pixel[3] == 0 {
                pixel.copy_from_slice(&[32, 32, 32, 255]);
            } else {
                pixel.swap(0, 2);
                pixel[3] = 255;
            }
        }
        Ok(Self {
            source,
            width: image.width() as u32,
            height: image.height() as u32,
            bgra,
        })
    }
}
pub type Bank = BTreeMap<i64, Result<Icon, &'static str>>;

fn keys(graphics: &[i64]) -> Result<Vec<i64>, &'static str> {
    if graphics.len() > MAX_ICONS || graphics.iter().any(|id| *id < 0) {
        return Err("圖示清單超出安全限制");
    }
    let mut result = graphics.to_vec();
    result.sort_unstable();
    result.dedup();
    Ok(result)
}
fn load(root: &Path, keys: &[i64], cancel: &AtomicBool) -> Bank {
    let mut bank = Bank::new();
    let Ok(mut archive) = SharedInventoryIcons::open_cancellable(root, cancel) else {
        for key in keys {
            bank.insert(*key, Err("共用圖示資源無法開啟"));
        }
        return bank;
    };
    let mut pixels = 0usize;
    for key in keys {
        if cancel.load(Ordering::Acquire) {
            break;
        }
        let value = archive
            .load(*key, cancel)
            .map_err(|_| "此商品的共用圖示無法載入")
            .and_then(|icon| {
                let count = icon
                    .image()
                    .width()
                    .checked_mul(icon.image().height())
                    .ok_or("圖示預算超限")?;
                pixels = pixels.checked_add(count).ok_or("圖示預算超限")?;
                if pixels > MAX_PIXELS {
                    return Err("圖示預算超限");
                }
                Icon::from_image(icon.source().clone(), icon.image())
            });
        bank.insert(*key, value);
    }
    bank
}
struct Job {
    keys: Vec<i64>,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Bank>,
    handle: Option<JoinHandle<()>>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
/// One job, one desired key-set; replacement cooperatively cancels but never
/// detaches the old job. Shutdown remains pending until its real thread ends.
pub struct Loader {
    root: PathBuf,
    desired: Vec<i64>,
    completed: Option<Vec<i64>>,
    bank: Arc<Bank>,
    job: Option<Job>,
    closing: bool,
}
impl Loader {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
            desired: Vec::new(),
            completed: None,
            bank: Arc::new(Bank::new()),
            job: None,
            closing: false,
        }
    }
    pub fn request(&mut self, graphics: &[i64]) -> Result<(), &'static str> {
        let wanted = keys(graphics)?;
        if self.closing {
            return Err("圖示工作正在結束");
        }
        if self.desired != wanted {
            self.desired = wanted;
            self.completed = None;
            self.bank = Arc::new(Bank::new());
            if let Some(job) = &self.job {
                job.cancel.store(true, Ordering::Release);
            }
        }
        Ok(())
    }
    pub fn poll(&mut self) {
        if self
            .job
            .as_ref()
            .is_some_and(|job| job.handle.as_ref().is_some_and(JoinHandle::is_finished))
        {
            let mut job = self.job.take().unwrap();
            let joined = job.handle.take().unwrap().join();
            if !self.closing && job.keys == self.desired && !job.cancel.load(Ordering::Acquire) {
                let bank = if joined.is_ok() {
                    job.receiver.try_recv().ok()
                } else {
                    None
                };
                self.bank = Arc::new(bank.unwrap_or_else(|| {
                    job.keys
                        .iter()
                        .map(|key| (*key, Err("圖示工作異常結束")))
                        .collect()
                }));
                self.completed = Some(job.keys.clone());
            }
        }
        if self.closing || self.job.is_some() || self.completed.as_ref() == Some(&self.desired) {
            return;
        }
        if self.desired.is_empty() {
            self.completed = Some(Vec::new());
            return;
        }
        let root = self.root.clone();
        let graphics = self.desired.clone();
        let worker_keys = graphics.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        match thread::Builder::new()
            .name("lito-shop-icons".into())
            .spawn(move || {
                let bank = load(&root, &worker_keys, &worker_cancel);
                let _ = sender.send(bank);
            }) {
            Ok(handle) => {
                self.job = Some(Job {
                    keys: graphics,
                    cancel,
                    receiver,
                    handle: Some(handle),
                })
            }
            Err(_) => {
                self.bank = Arc::new(
                    graphics
                        .iter()
                        .map(|key| (*key, Err("無法啟動圖示工作")))
                        .collect(),
                );
                self.completed = Some(graphics);
            }
        }
    }
    pub fn bank(&self) -> Arc<Bank> {
        self.bank.clone()
    }
    pub fn cancel(&mut self) {
        self.closing = true;
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Release);
        }
    }
    pub fn is_quiescent(&self) -> bool {
        self.job.is_none()
    }
}
impl Drop for Loader {
    fn drop(&mut self) {
        self.cancel();
        self.job.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icons_are_bounded_sorted_deduplicated_without_id_coercion() {
        assert_eq!(keys(&[8, 2, 8]).unwrap(), vec![2, 8]);
        assert!(keys(&[-1]).is_err());
        assert!(keys(&[1, 2, 3, 4, 5, 6, 7]).is_ok());
        assert_eq!(
            keys(&[8, 7, 6, 5, 4, 3, 2, 1]).unwrap(),
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(
            keys(&[9, 8, 7, 6, 5, 4, 3, 2, 1]).unwrap(),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert!(keys(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).is_err());
        assert_eq!(keys(&[i64::MAX]).unwrap(), vec![i64::MAX]);
    }
    #[test]
    fn natural_tbt_preview_uses_real_pixels_and_explicit_background() {
        let image = super::super::inventory_icons::decode_tbt(&[0, 0, 2, 1, 1, 0, 2, 0, 124, 0, 0])
            .unwrap();
        let source = IconSource {
            archive: super::super::resources::SharedArchiveKind::Sprite,
            name: "1.tbt".into(),
        };
        let icon = Icon::from_image(source.clone(), &image).unwrap();
        assert_eq!((icon.width, icon.height), (2, 1));
        assert_eq!(icon.source, source);
        assert_eq!(icon.bgra, vec![0, 0, 255, 255, 32, 32, 32, 255]);
    }
    #[test]
    fn closed_loader_never_starts_or_reuses_a_late_request() {
        let mut loader = Loader::new(Path::new("unused"));
        loader.request(&[3]).unwrap();
        loader.cancel();
        loader.poll();
        assert!(loader.is_quiescent());
        assert!(loader.request(&[3]).is_err());
        assert!(loader.bank().is_empty());
    }
    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; independent three-potion C# pixels and real owned icon worker, no UI"]
    fn actual_shop_potion_icons_match_reference_and_worker_retires() {
        use sha2::{Digest, Sha256};
        use std::time::{Duration, Instant};
        let root = PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root required"),
        );
        let cancel = AtomicBool::new(false);
        let mut source = SharedInventoryIcons::open(&root).unwrap();
        let mut expected = Bank::new();
        // Independently extracted in memory with the existing PakViewer
        // ImageConverter.LoadL1Image (not its older Raw decoder).
        for (id, hash) in [
            (
                78,
                "37CBCBC9B3B8E5009BD68E110EBC8F7911E5C03122089A54615C1EBF44CE105C",
            ),
            (
                80,
                "B244DFCC969A4FD818B0BF19E2B06B5BED0258084C90B2E748046FC654D1F917",
            ),
            (
                81,
                "F3C881B13070E7BB8A9CEC3AFB013ED3AC6BD93277187F4EF4A8192FF3A0F5BC",
            ),
        ] {
            let icon = source.load(id, &cancel).unwrap();
            assert_eq!(
                icon.source().archive,
                super::super::resources::SharedArchiveKind::Sprite
            );
            assert_eq!(icon.source().name.to_ascii_lowercase(), format!("{id}.tbt"));
            let image = icon.image();
            assert_eq!(image.raw_offset(), [8, 4]);
            assert_eq!((image.width(), image.height()), (16, 24));
            let rgba = image.reference_rgba().unwrap();
            assert_eq!(rgba.len(), 1536);
            assert_eq!(format!("{:X}", Sha256::digest(&rgba)), hash);
            expected.insert(
                id,
                Ok(Icon::from_image(icon.source().clone(), image).unwrap()),
            );
        }
        drop(source);
        let mut loader = Loader::new(&root);
        loader.request(&[80, 78, 81]).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while loader.completed.as_ref() != Some(&vec![78, 80, 81]) {
            assert!(Instant::now() < deadline, "real icon worker deadline");
            loader.poll();
            thread::sleep(Duration::from_millis(5));
        }
        assert!(loader.is_quiescent());
        let bank = loader.bank();
        assert_eq!(bank.len(), 3);
        for (id, expected) in expected {
            let actual = bank[&id].as_ref().unwrap();
            let expected = expected.unwrap();
            assert_eq!(actual.source, expected.source);
            assert_eq!((actual.width, actual.height), (16, 24));
            assert_eq!(actual.bgra, expected.bgra);
        }
        loader.cancel();
        loader.poll();
        assert!(loader.is_quiescent());
        assert!(loader.request(&[78]).is_err());
    }
}
