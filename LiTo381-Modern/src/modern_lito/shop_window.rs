//! Same-thread owned shop. A submitted request is an intent, never a purchase.
//! Only the icon loader owns a worker; network work remains in SessionController.
use super::{shop_icons, shop_protocol as shop};
use anyhow::{ensure, Result};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    marker::PhantomData,
    path::Path,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateFontW, DeleteObject, EndPaint, GetSysColorBrush, StretchDIBits,
            BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
            COLOR_BTNFACE, DEFAULT_CHARSET, DIB_RGB_COLORS, HFONT, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
            SRCCOPY,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::EM_SETLIMITTEXT,
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            Input::KeyboardAndMouse::{EnableWindow, GetFocus, GetKeyState, SetFocus},
            WindowsAndMessaging::*,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub request: shop::Request,
    pub state: shop::State,
    pub pending: bool,
    pub status: String,
    pub labels: BTreeMap<i64, String>,
}
pub enum Outcome {
    Request(shop::Request),
    Failed(&'static str),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewUpdate {
    Current { clear_notice: bool },
    AwaitTransaction,
    Retire,
}
/// Retirement is not a bad server snapshot. In particular, an old icon worker
/// can outlive its TLS session without acquiring the next selection's view.
fn apply_view(
    current: &mut View,
    next: Option<View>,
    owner_closing: bool,
    closing: bool,
    cancelled: bool,
) -> ViewUpdate {
    let Some(next) = next else {
        return ViewUpdate::Retire;
    };
    if owner_closing
        || cancelled
        || current.request.context != next.request.context
        || current.request.channel_id != next.request.channel_id
    {
        return ViewUpdate::Retire;
    }
    if closing {
        // Normal Escape/Return still waits for an already submitted transaction.
        // Only its same-scope pending fence matters; never repaint new shop data.
        current.pending = next.pending;
        return ViewUpdate::AwaitTransaction;
    }
    let clear_notice = current.state != next.state
        || current.request != next.request
        || current.pending != next.pending;
    *current = next;
    ViewUpdate::Current { clear_notice }
}
const INFO: usize = 0;
const LIST: usize = 1;
const DETAIL: usize = 2;
const QUANTITY: usize = 3;
const BUY: usize = 4;
const SELL: usize = 5;
const READ: usize = 6;
const BACK: usize = 7;
const STATUS: usize = 8;
const COUNT: usize = 9;
const WIDTH: i32 = 760;
const HEIGHT: i32 = 660;
const FIRST: usize = 700;
const STYLE: WINDOW_STYLE = WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 | WS_CLIPCHILDREN.0);
static NEXT: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BoxRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}
fn px(v: i32, dpi: u32) -> i32 {
    ((i64::from(v) * i64::from(dpi.clamp(96, 768)) + 48) / 96) as i32
}
fn layout(w: i32, h: i32, dpi: u32) -> [BoxRect; COUNT] {
    let m = px(16, dpi);
    let full = (w - 2 * m).max(1);
    let mut r = [BoxRect::default(); COUNT];
    r[INFO] = BoxRect {
        x: m,
        y: m,
        w: full,
        h: px(112, dpi),
    };
    r[LIST] = BoxRect {
        x: m,
        y: px(136, dpi),
        w: full,
        h: (h - px(464, dpi)).max(px(160, dpi)),
    };
    let top = r[LIST].y + r[LIST].h + px(12, dpi);
    r[DETAIL] = BoxRect {
        x: px(128, dpi),
        y: top,
        w: (w - px(144, dpi)).max(1),
        h: px(140, dpi),
    };
    for (i, x, width) in [
        (QUANTITY, 16, 88),
        (BUY, 116, 100),
        (SELL, 228, 100),
        (READ, 340, 116),
    ] {
        r[i] = BoxRect {
            x: px(x, dpi),
            y: h - px(136, dpi),
            w: px(width, dpi),
            h: px(30, dpi),
        };
    }
    r[STATUS] = BoxRect {
        x: m,
        y: h - px(96, dpi),
        w: full,
        h: px(42, dpi),
    };
    r[BACK] = BoxRect {
        x: w - px(136, dpi),
        y: h - px(44, dpi),
        w: px(120, dpi),
        h: px(28, dpi),
    };
    r
}
fn quantity(s: &str) -> std::result::Result<i64, &'static str> {
    if s.is_empty() || s.len() > 7 || !s.bytes().all(|c| c.is_ascii_digit()) {
        return Err("數量請輸入 1–1000000 的整數。");
    }
    let n = s.parse::<i64>().map_err(|_| "數量超出限制。")?;
    if !(1..=shop::MAX_QUANTITY).contains(&n) {
        return Err("數量請輸入 1–1000000 的整數。");
    }
    Ok(n)
}
fn ids(view: &View) -> Vec<i64> {
    let mut result = view.state.snapshot().map_or_else(Vec::new, |s| {
        s.offers
            .iter()
            .map(|o| o.item_id)
            .chain(s.stacks.iter().map(|s| s.item_id))
            .collect()
    });
    result.sort_unstable();
    result.dedup();
    result
}
pub(crate) fn requested_icons(view: &View, selected: Option<i64>) -> Vec<i64> {
    let Some(s) = view.state.snapshot() else {
        return Vec::new();
    };
    // Selection has priority over speculative preloading. Count distinct art,
    // not offers; a shared graphic must not consume several bounded slots.
    let selected = selected
        .filter(|id| s.purchasable(*id))
        .and_then(|id| s.offer(id));
    let mut graphics = Vec::new();
    for offer in selected
        .into_iter()
        .chain(s.offers.iter().filter(|o| s.purchasable(o.item_id)))
    {
        if !graphics.contains(&offer.inventory_graphic) {
            graphics.push(offer.inventory_graphic);
            if graphics.len() == shop_icons::MAX_ICONS {
                break;
            }
        }
    }
    graphics.sort_unstable();
    graphics
}
fn next_focus(current: usize, back: bool, enabled: &[bool; COUNT]) -> Option<usize> {
    let order = [LIST, DETAIL, QUANTITY, BUY, SELL, READ, BACK];
    let pos =
        order
            .iter()
            .position(|i| *i == current)
            .unwrap_or(if back { 0 } else { order.len() - 1 });
    (1..=order.len())
        .map(|n| order[(pos + if back { order.len() - n } else { n }) % order.len()])
        .find(|i| enabled[*i])
}
pub(crate) fn selected_text(view: &View, id: Option<i64>) -> String {
    let Some(s) = view.state.snapshot() else {
        return "等待伺服器確認商品、餘額與本場補給。錯誤不代表零餘額。".into();
    };
    let Some(id) = id else {
        return "請選擇商品。來源背包及保存裝備不會在此更動。".into();
    };
    let own = s.stack(id);
    let name = view
        .labels
        .get(&id)
        .cloned()
        .unwrap_or_else(|| format!("商品 ID {id}"));
    let Some(o) = s.offer(id) else {
        return format!(
            "{name}\r\n本場持有 {}；目前目錄已無此商品，無法買入或賣回。",
            own.map_or(0, |s| s.quantity)
        );
    };
    let support = if o.support == shop::Support::Healing {
        "已資格化補血藥，可購買並在本場世界使用。".into()
    } else if o.support == shop::Support::Material && s.materials_enabled {
        "已資格化魔法寶石（施法材料）；可購買，使用時由伺服器隨需要材料的技能扣除，不能在背包直接使用。".into()
    } else if let Some(t) = s.timed_offer(id) {
        format!(
            "已資格化{}，{}秒；{}。可購買，使用時仍由伺服器確認職業與本場狀態。",
            t.kind.label(),
            t.kind.duration_millis() / 1000,
            if t.eligible {
                "目前職業可用"
            } else {
                "目前職業不可使用"
            }
        )
    } else if s
        .ammo
        .as_ref()
        .is_some_and(|facts| facts.iter().any(|f| f.item_id == id))
    {
        "已資格化箭矢；入場後按 F8／F9 選箭，普通弓命中或未命中均消耗一支。耗盡不會自動換箭。"
            .into()
    } else {
        "此商品尚未實作／未資格化，不能購買。".into()
    };
    format!(
        "{name}\r\n目錄原始名稱：{}\r\nID {} · 買入 {}／賣回 {} · 本場持有 {}\r\n{support}\r\n圖示 {} · 重量保存值 {}（千分單位）\r\n賣回每件不超過原支付金額；補給僅本場使用。",
        o.name,
        o.item_id,
        o.buy_price,
        o.sell_price,
        own.map_or(0, |s| s.quantity),
        o.inventory_graphic,
        o.weight
    )
}
struct State {
    hwnd: Cell<Option<HWND>>,
    controls: Cell<Option<[HWND; COUNT]>>,
    view: RefCell<View>,
    rows: RefCell<Vec<i64>>,
    selected: Cell<Option<i64>>,
    outcome: RefCell<Option<Outcome>>,
    local_notice: RefCell<Option<String>>,
    submitted: Cell<bool>,
    closing: Cell<bool>,
    force_close: Cell<bool>,
    rendering: Cell<bool>,
    icons: RefCell<shop_icons::Loader>,
    bank: RefCell<Arc<shop_icons::Bank>>,
    font: Cell<HFONT>,
    fault: Cell<bool>,
}
struct Class {
    name: Vec<u16>,
    instance: HINSTANCE,
}
impl Drop for Class {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterClassW(PCWSTR(self.name.as_ptr()), Some(self.instance));
        }
    }
}
pub struct ShopWindow {
    state: Box<State>,
    _class: Class,
    _thread: PhantomData<Rc<()>>,
}
impl ShopWindow {
    pub fn open(parent: HWND, root: &Path, view: View) -> Result<Self> {
        view.request.validate()?;
        ensure!(
            view.request.action == shop::Action::Read,
            "shop open requires read scope"
        );
        let id = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| anyhow::anyhow!("shop window ID exhausted"))?;
        let name = wide(&format!("ModernLiToShop{id}"));
        unsafe {
            ensure!(IsWindow(Some(parent)).as_bool(), "shop owner missing");
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            ensure!(
                RegisterClassW(&WNDCLASSW {
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    lpszClassName: PCWSTR(name.as_ptr()),
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                    ..Default::default()
                }) != 0,
                "register shop"
            );
            let mut result = Self {
                state: Box::new(State {
                    hwnd: Cell::new(None),
                    controls: Cell::new(None),
                    view: RefCell::new(view),
                    rows: RefCell::new(Vec::new()),
                    selected: Cell::new(None),
                    outcome: RefCell::new(None),
                    local_notice: RefCell::new(None),
                    submitted: Cell::new(false),
                    closing: Cell::new(false),
                    force_close: Cell::new(false),
                    rendering: Cell::new(true),
                    icons: RefCell::new(shop_icons::Loader::new(root)),
                    bank: RefCell::new(Arc::new(shop_icons::Bank::new())),
                    font: Cell::new(HFONT::default()),
                    fault: Cell::new(false),
                }),
                _class: Class { name, instance },
                _thread: PhantomData,
            };
            let hwnd = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                PCWSTR(result._class.name.as_ptr()),
                w!("LiTo — 本場補給商店"),
                STYLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WIDTH,
                HEIGHT,
                Some(parent),
                None,
                Some(instance),
                Some((&mut *result.state as *mut State).cast()),
            )?;
            result.state.hwnd.set(Some(hwnd));
            let controls = create_controls(hwnd, instance)?;
            result.state.controls.set(Some(controls));
            set_text(controls[QUANTITY], "1")?;
            replace_font(&result.state)?;
            let dpi = GetDpiForWindow(hwnd).clamp(96, 768);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: px(WIDTH, dpi),
                bottom: px(HEIGHT, dpi),
            };
            AdjustWindowRectExForDpi(&mut rect, STYLE, false, WS_EX_CONTROLPARENT, dpi)?;
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            resize(&result.state)?;
            result.state.rendering.set(false);
            result.update_inner()?;
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetFocus(Some(controls[LIST]));
            Ok(result)
        }
    }
    pub fn window_handle(&self) -> Option<HWND> {
        self.state.hwnd.get()
    }
    pub fn refresh(&mut self, view: Option<View>, owner_closing: bool) -> Result<()> {
        let update = {
            let mut current = self.state.view.borrow_mut();
            apply_view(
                &mut current,
                view,
                owner_closing,
                self.state.closing.get(),
                self.state.force_close.get(),
            )
        };
        match update {
            ViewUpdate::Retire => self.cancel(),
            ViewUpdate::Current { clear_notice: true } => {
                self.state.local_notice.borrow_mut().take();
            }
            _ => {}
        }
        // Poll/join the actual worker even for a retired scope. Real rendering
        // and native teardown failures remain errors for the parent to report.
        self.update_inner()
    }
    fn update_inner(&mut self) -> Result<()> {
        {
            let mut loader = self.state.icons.borrow_mut();
            loader.poll();
            *self.state.bank.borrow_mut() = loader.bank();
        }
        let pending = self.state.view.borrow().pending;
        if self.state.closing.get()
            && (!pending || self.state.force_close.get())
            && self.state.icons.borrow().is_quiescent()
        {
            if let Some(hwnd) = self.state.hwnd.get() {
                unsafe {
                    DestroyWindow(hwnd)?;
                }
            }
            return Ok(());
        }
        unsafe { render(&self.state) }
    }
    pub fn poll(&mut self) -> Option<Outcome> {
        self.state.outcome.borrow_mut().take()
    }
    /// Parent invokes this only after its controller has accepted/rejected the
    /// intent and it has obtained the new pending snapshot. No optimistic model.
    pub fn dispatched(&mut self) {
        self.state.submitted.set(false);
    }
    pub fn cancel(&mut self) {
        self.state.force_close.set(true);
        self.state.outcome.borrow_mut().take();
        close(&self.state);
    }
    pub fn is_quiescent(&self) -> bool {
        self.state.hwnd.get().is_none() && self.state.icons.borrow().is_quiescent()
    }
    pub fn pre_translate_window(hwnd: HWND, msg: &MSG) -> bool {
        unsafe {
            if !IsWindow(Some(hwnd)).as_bool()
                || !(msg.hwnd == hwnd || IsChild(hwnd, msg.hwnd).as_bool())
            {
                return false;
            }
            if msg.message == WM_KEYDOWN && msg.wParam.0 == 0x1b {
                let _ = SendMessageW(hwnd, WM_CLOSE, None, None);
                return true;
            }
            if msg.message == WM_KEYDOWN && msg.wParam.0 == 9 {
                let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
                if let Some(s) = p.as_ref() {
                    if let Some(c) = s.controls.get() {
                        let focused = GetFocus();
                        let current = c.iter().position(|h| *h == focused).unwrap_or(BACK);
                        let enabled = std::array::from_fn(|i| {
                            IsWindowVisible(c[i]).as_bool()
                                && windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(
                                    c[i],
                                )
                                .as_bool()
                        });
                        if let Some(i) = next_focus(current, GetKeyState(0x10) < 0, &enabled) {
                            let _ = SetFocus(Some(c[i]));
                            return true;
                        }
                    }
                }
            }
            IsDialogMessageW(hwnd, msg).as_bool()
        }
    }
}
impl Drop for ShopWindow {
    fn drop(&mut self) {
        self.cancel(); // Exceptional teardown joins the actual icon worker before state retirement.
        let replacement = shop_icons::Loader::new(Path::new("unused"));
        let mut old = self.state.icons.replace(replacement);
        old.cancel();
        drop(old);
        unsafe {
            if let Some(hwnd) = self.state.hwnd.get() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(hwnd);
            }
            let font = self.state.font.replace(HFONT::default());
            if !font.0.is_null() {
                let _ = DeleteObject(font.into());
            }
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
unsafe fn set_text(hwnd: HWND, s: &str) -> Result<()> {
    ensure!(!s.contains('\0'), "shop text NUL");
    let v = wide(s);
    ensure!(v.len() <= 65537, "shop text bound");
    let n = GetWindowTextLengthW(hwnd);
    let mut old = vec![0; (n.max(0) as usize).min(65536) + 1];
    if n >= 0 && n as usize <= 65536 && GetWindowTextW(hwnd, &mut old) == n && old == v {
        return Ok(());
    }
    SetWindowTextW(hwnd, PCWSTR(v.as_ptr()))?;
    ensure!(
        GetWindowTextLengthW(hwnd) as usize == v.len() - 1,
        "shop text truncated"
    );
    let mut check = vec![0; v.len()];
    ensure!(
        GetWindowTextW(hwnd, &mut check) as usize == v.len() - 1 && check == v,
        "shop text mismatch"
    );
    Ok(())
}
unsafe fn create_controls(parent: HWND, instance: HINSTANCE) -> Result<[HWND; COUNT]> {
    let mut c = [HWND::default(); COUNT];
    for i in 0..COUNT {
        let (class, text, style, ex) = match i {
            LIST => (
                w!("LISTBOX"),
                "",
                WS_TABSTOP.0 | WS_VSCROLL.0 | LBS_NOTIFY as u32 | LBS_NOINTEGRALHEIGHT as u32,
                WS_EX_CLIENTEDGE,
            ),
            DETAIL => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0
                    | WS_VSCROLL.0
                    | ES_MULTILINE as u32
                    | ES_READONLY as u32
                    | ES_AUTOVSCROLL as u32,
                WS_EX_CLIENTEDGE,
            ),
            QUANTITY => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0 | ES_NUMBER as u32 | ES_AUTOHSCROLL as u32,
                WS_EX_CLIENTEDGE,
            ),
            BUY => (w!("BUTTON"), "購買", WS_TABSTOP.0, WINDOW_EX_STYLE(0)),
            SELL => (w!("BUTTON"), "賣回", WS_TABSTOP.0, WINDOW_EX_STYLE(0)),
            READ => (w!("BUTTON"), "重新讀取", WS_TABSTOP.0, WINDOW_EX_STYLE(0)),
            BACK => (w!("BUTTON"), "返回", WS_TABSTOP.0, WINDOW_EX_STYLE(0)),
            _ => (w!("STATIC"), "", 0, WINDOW_EX_STYLE(0)),
        };
        let label = wide(text);
        c[i] = CreateWindowExW(
            ex,
            class,
            PCWSTR(label.as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style),
            0,
            0,
            1,
            1,
            Some(parent),
            Some(HMENU((FIRST + i) as *mut _)),
            Some(instance),
            None,
        )?;
    }
    let _ = SendMessageW(c[QUANTITY], EM_SETLIMITTEXT, Some(WPARAM(7)), None);
    let _ = SendMessageW(c[DETAIL], EM_SETLIMITTEXT, Some(WPARAM(65536)), None);
    Ok(c)
}
unsafe fn replace_font(s: &State) -> Result<()> {
    let Some(hwnd) = s.hwnd.get() else {
        return Ok(());
    };
    let f = CreateFontW(
        -px(16, GetDpiForWindow(hwnd)),
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        0,
        w!("Microsoft JhengHei UI"),
    );
    ensure!(!f.0.is_null(), "shop font");
    let old = s.font.replace(f);
    if let Some(c) = s.controls.get() {
        for h in c {
            let _ = SendMessageW(h, WM_SETFONT, Some(WPARAM(f.0 as usize)), Some(LPARAM(1)));
        }
    }
    if !old.0.is_null() {
        let _ = DeleteObject(old.into());
    }
    Ok(())
}
unsafe fn resize(s: &State) -> Result<()> {
    if let (Some(hwnd), Some(c)) = (s.hwnd.get(), s.controls.get()) {
        let mut r = RECT::default();
        GetClientRect(hwnd, &mut r)?;
        for (h, b) in c
            .iter()
            .zip(layout(r.right, r.bottom, GetDpiForWindow(hwnd)))
        {
            MoveWindow(*h, b.x, b.y, b.w, b.h, true)?;
        }
    }
    Ok(())
}
fn close(s: &State) {
    s.closing.set(true);
    s.icons.borrow_mut().cancel();
}
fn fail(s: &State) {
    s.fault.set(true);
    *s.outcome.borrow_mut() = Some(Outcome::Failed("商店視窗無法完整更新，已停止提交操作。"));
    close(s);
}
unsafe fn render(s: &State) -> Result<()> {
    if s.rendering.replace(true) {
        return Ok(());
    }
    struct Guard<'a>(&'a Cell<bool>);
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.set(false)
        }
    }
    let _g = Guard(&s.rendering);
    let Some(c) = s.controls.get() else {
        return Ok(());
    };
    let v = s.view.borrow().clone();
    let rows = ids(&v);
    let old = s.rows.borrow().clone();
    let id = s
        .selected
        .get()
        .filter(|id| rows.contains(id))
        .or_else(|| rows.first().copied());
    s.selected.set(id);
    // List text is compared separately so real price/name changes are visible,
    // while unchanged 30ms polls preserve list selection and scroll position.
    let labels = rows
        .iter()
        .map(|id| {
            let snap = v.state.snapshot().unwrap();
            let offer = snap.offer(*id);
            format!(
                "{} · 持有 {} · {}",
                v.labels
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| format!("商品 ID {id}")),
                snap.stack(*id).map_or(0, |x| x.quantity),
                offer.map_or_else(
                    || "目錄缺項".into(),
                    |o| format!(
                        "買 {}／賣 {}{}",
                        o.buy_price,
                        o.sell_price,
                        if snap.purchasable(o.item_id) {
                            ""
                        } else {
                            " · 未實作"
                        }
                    )
                )
            )
        })
        .collect::<Vec<_>>();
    let actual_count = SendMessageW(c[LIST], LB_GETCOUNT, None, None).0;
    let mut same = old == rows && actual_count == labels.len() as isize;
    if same {
        for (i, label) in labels.iter().enumerate() {
            let len = SendMessageW(c[LIST], LB_GETTEXTLEN, Some(WPARAM(i)), None).0;
            if !(0..=4096).contains(&len) {
                same = false;
                break;
            }
            let mut buf = vec![0u16; len as usize + 1];
            if SendMessageW(
                c[LIST],
                LB_GETTEXT,
                Some(WPARAM(i)),
                Some(LPARAM(buf.as_mut_ptr() as isize)),
            )
            .0 != len
                || buf != wide(label)
            {
                same = false;
                break;
            }
        }
    }
    if !same {
        let _ = SendMessageW(c[LIST], LB_RESETCONTENT, None, None);
        for (i, label) in labels.iter().enumerate() {
            let text = wide(label);
            ensure!(
                SendMessageW(
                    c[LIST],
                    LB_ADDSTRING,
                    None,
                    Some(LPARAM(text.as_ptr() as isize))
                )
                .0 == i as isize,
                "shop list failed"
            );
        }
        *s.rows.borrow_mut() = rows.clone();
        if let Some(index) = id.and_then(|id| rows.iter().position(|v| *v == id)) {
            ensure!(
                SendMessageW(c[LIST], LB_SETCURSEL, Some(WPARAM(index)), None).0 == index as isize,
                "shop selection failed"
            );
        }
    }
    let info=v.state.snapshot().map_or_else(||"尚未取得本場補給餘額；請等待或重新讀取。".into(),|x|format!("本場補給額度 {} · 餘額 {} · 回合 {}／版本 {}\r\n這不是來源金幣或背包；戰鬥結束後額度與補給會重設。",x.allowance,x.balance,x.round_id,x.revision));
    let weight_notice = v
        .state
        .snapshot()
        .and_then(|s| s.weight.as_ref())
        .map(|w| w.notice())
        .unwrap_or_else(|| "比賽負重／容量未確認，不推定為零。".into());
    set_text(c[INFO], &format!("{info}\r\n{weight_notice}"))?;
    let mut detail = selected_text(&v, id);
    let icon_id = id
        .and_then(|id| v.state.snapshot()?.offer(id))
        .map(|o| o.inventory_graphic);
    let bank = s.bank.borrow().clone();
    if let Some(icon) = icon_id {
        detail.push_str(match bank.get(&icon) {
            Some(Ok(_)) => "\r\n已載入真正共用圖示。",
            Some(Err(e)) => e,
            None if id.is_some_and(|id| v.state.snapshot().is_some_and(|s| s.purchasable(id))) => {
                "\r\n等待所選商品的共用圖示載入；不影響伺服器交易確認。"
            }
            None => "\r\n此商品未資格化，不預載圖示。",
        });
    }
    set_text(c[DETAIL], &detail)?;
    let blocked = v.pending || s.submitted.get() || s.closing.get() || s.fault.get();
    for i in [LIST, QUANTITY, READ] {
        let _ = EnableWindow(c[i], !blocked);
    }
    let snap = v.state.snapshot();
    let offer = id.and_then(|id| snap?.offer(id));
    let own = id.and_then(|id| snap?.stack(id));
    let _ = EnableWindow(
        c[BUY],
        !blocked && id.is_some_and(|id| snap.is_some_and(|s| s.purchasable(id))),
    );
    let _ = EnableWindow(c[SELL], !blocked && offer.is_some() && own.is_some());
    let local = s.local_notice.borrow().clone();
    set_text(
        c[STATUS],
        status_text(
            s.closing.get(),
            v.pending || s.submitted.get(),
            local.as_deref(),
            &v.status,
        ),
    )?;
    if !s.closing.get() {
        let graphics = requested_icons(&v, id);
        s.icons
            .borrow_mut()
            .request(&graphics)
            .map_err(anyhow::Error::msg)?;
    }
    if let Some(hwnd) = s.hwnd.get() {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, false);
    }
    Ok(())
}
fn status_text<'a>(
    closing: bool,
    pending: bool,
    local: Option<&'a str>,
    server: &'a str,
) -> &'a str {
    if closing && pending {
        "正在等待已提交交易的確認；返回不會重送或取消已提交的購買。"
    } else if pending {
        "操作已送出，等待伺服器確認；不會重複提交。"
    } else {
        local.unwrap_or(server)
    }
}
unsafe fn click(s: &State, index: usize) {
    if index == BACK {
        close(s);
        return;
    }
    if s.closing.get() || s.submitted.get() || s.rendering.get() {
        return;
    }
    let v = s.view.borrow().clone();
    if v.pending {
        return;
    }
    let request = if index == READ {
        Ok(v.request)
    } else if matches!(index, BUY | SELL) {
        (function_request)(s, &v, index)
    } else {
        return;
    };
    match request {
        Ok(r) => {
            s.local_notice.borrow_mut().take();
            s.submitted.set(true);
            *s.outcome.borrow_mut() = Some(Outcome::Request(r));
        }
        Err(e) => {
            *s.local_notice.borrow_mut() = Some(e.into());
            if let Some(c) = s.controls.get() {
                if set_text(c[STATUS], e).is_err() {
                    fail(s)
                }
            }
        }
    }
}
unsafe fn function_request(
    s: &State,
    v: &View,
    index: usize,
) -> std::result::Result<shop::Request, &'static str> {
    let c = s.controls.get().ok_or("商店正在結束。")?;
    let n = GetWindowTextLengthW(c[QUANTITY]);
    if !(1..=7).contains(&n) {
        return Err("數量請輸入 1–1000000 的整數。");
    }
    let mut buf = vec![0; n as usize + 1];
    if GetWindowTextW(c[QUANTITY], &mut buf) != n {
        return Err("無法讀取完整數量。");
    }
    let text = String::from_utf16(&buf[..n as usize]).map_err(|_| "數量格式錯誤。")?;
    transaction_request(
        v,
        s.selected.get(),
        if index == BUY {
            shop::Action::Buy
        } else {
            shop::Action::Sell
        },
        &text,
    )
}
/// The actual Buy/Sell button and controller integration checks share this
/// bounded intent builder. It neither predicts a wallet nor sends a packet.
pub(crate) fn transaction_request(
    view: &View,
    selected: Option<i64>,
    action: shop::Action,
    text: &str,
) -> std::result::Result<shop::Request, &'static str> {
    if view.pending {
        return Err("上一筆交易仍等待伺服器確認。");
    }
    if !matches!(action, shop::Action::Buy | shop::Action::Sell) {
        return Err("不是購買或賣回操作。");
    }
    let id = selected.ok_or("請選擇商品。")?;
    let snapshot = view.state.snapshot().ok_or("尚無伺服器確認的餘額。")?;
    snapshot
        .offer(id)
        .ok_or("目前目錄沒有此商品，請重新讀取。")?;
    if action == shop::Action::Buy && !snapshot.purchasable(id) {
        return Err("此商品尚未實作，不能購買。");
    }
    let r = view.request.mutation(snapshot, action, id, quantity(text)?);
    r.validate().map_err(|_| "交易參數格式錯誤。")?;
    Ok(r)
}
unsafe fn paint(s: &State, hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    if !hdc.0.is_null() {
        let view = s.view.borrow().clone();
        let icon = s
            .selected
            .get()
            .and_then(|id| view.state.snapshot()?.offer(id))
            .map(|o| o.inventory_graphic);
        let bank = s.bank.borrow().clone();
        if let Some(Ok(icon)) = icon.and_then(|id| bank.get(&id)) {
            let mut r = RECT::default();
            if GetClientRect(hwnd, &mut r).is_ok() {
                let dpi = GetDpiForWindow(hwnd);
                let b = layout(r.right, r.bottom, dpi)[DETAIL];
                let ratio = (px(96, dpi) as f64 / f64::from(icon.width.max(icon.height))).min(1.0);
                let w = (f64::from(icon.width) * ratio).max(1.0) as i32;
                let h = (f64::from(icon.height) * ratio).max(1.0) as i32;
                let info = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: icon.width as i32,
                        biHeight: -(icon.height as i32),
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let _ = StretchDIBits(
                    hdc,
                    px(16, dpi),
                    b.y,
                    w,
                    h,
                    0,
                    0,
                    icon.width as i32,
                    icon.height as i32,
                    Some(icon.bgra.as_ptr().cast()),
                    &info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
        }
    }
    let _ = EndPaint(hwnd, &ps);
}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lp.0 as *const CREATESTRUCTW);
        let ptr = create.lpCreateParams as *mut State;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as _);
        if let Some(s) = ptr.as_ref() {
            s.hwnd.set(Some(hwnd));
        }
        return LRESULT(1);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    let Some(s) = ptr.as_ref() else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_CLOSE => {
            close(s);
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = wp.0 & 0xffff;
            let event = (wp.0 >> 16) & 0xffff;
            if id >= FIRST && id < FIRST + COUNT {
                let i = id - FIRST;
                if i == LIST && event == LBN_SELCHANGE as usize {
                    if let Some(c) = s.controls.get() {
                        let row = SendMessageW(c[LIST], LB_GETCURSEL, None, None).0;
                        if row >= 0 {
                            s.selected.set(s.rows.borrow().get(row as usize).copied());
                        }
                    }
                    if render(s).is_err() {
                        fail(s)
                    }
                } else if event == BN_CLICKED as usize {
                    click(s, i);
                }
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SIZE => {
            if resize(s).is_err() {
                fail(s)
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let r = &*(lp.0 as *const RECT);
            if SetWindowPos(
                hwnd,
                None,
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .is_err()
                || replace_font(s).is_err()
                || resize(s).is_err()
            {
                fail(s)
            }
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lp.0 as *mut MINMAXINFO);
            let dpi = GetDpiForWindow(hwnd).clamp(96, 768);
            info.ptMinTrackSize.x = px(WIDTH, dpi);
            info.ptMinTrackSize.y = px(HEIGHT + 40, dpi);
            LRESULT(0)
        }
        WM_PAINT => {
            paint(s, hwnd);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            s.hwnd.set(None);
            s.controls.set(None);
            close(s);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_fixture(materials: bool) -> View {
        let mut view = view_fixture();
        view.pending = false;
        view.status = "目錄已確認".into();
        view.labels.insert(40318, "魔法寶石".into());
        let mut offers: Vec<_> = (0..9)
            .map(|n| shop::Offer {
                item_id: 40000 + n,
                sort: n as u16,
                support: shop::Support::Healing,
                buy_price: 100,
                sell_price: 50,
                inventory_graphic: 70 + n,
                ground_graphic: 1,
                weight: 1000,
                name: format!("Synthetic potion {n}"),
                healing: Some(shop::Healing {
                    minimum: 1,
                    maximum: 2,
                    effect: 1,
                    delay_group: 0,
                    delay_millis: 500,
                }),
            })
            .collect();
        offers.push(shop::Offer {
            item_id: 40318,
            sort: 9,
            support: if materials {
                shop::Support::Material
            } else {
                shop::Support::Unimplemented
            },
            buy_price: 100,
            sell_price: 50,
            inventory_graphic: 235,
            ground_graphic: 773,
            weight: 2000,
            name: "$508".into(),
            healing: None,
        });
        let snapshot = shop::Snapshot {
            round_id: 2,
            revision: 7,
            allowance: 20000,
            balance: 20000,
            token: [3; 32],
            offers,
            stacks: vec![],
            weight: Some(super::super::weight_protocol::Snapshot {
                source_gear_weight: 0,
                supplies_weight: 0,
                total: 0,
                maximum_bits: 10000_f64.to_bits(),
                stage: 0,
                hit_penalty: 0,
                units: vec![],
            }),
            timed: Some(shop::TimedCatalog {
                owner_class: 1,
                offers: vec![],
            }),
            ammo: None,
            materials_enabled: materials,
        };
        let reply = shop::Reply {
            echo: view.request,
            status: shop::Status::Ok,
            reason: shop::Reason::None,
            snapshot: Some(Arc::new(snapshot)),
            room: Some(super::super::rooms::tests::sample(
                view.request.context,
                false,
            )),
        };
        reply.validate().unwrap();
        view.state.receive(reply).unwrap();
        view
    }

    #[test]
    fn qualified_material_description_matches_buy_authority_and_explains_spell_consumption() {
        let view = catalog_fixture(true);
        assert!(view.state.snapshot().unwrap().purchasable(40318));
        let text = selected_text(&view, Some(40318));
        assert!(text.contains("施法材料"), "{text}");
        assert!(text.contains("不能在背包直接使用"), "{text}");
        assert!(!text.contains("尚未實作"), "{text}");
        let old = catalog_fixture(false);
        assert!(!old.state.snapshot().unwrap().purchasable(40318));
        assert!(selected_text(&old, Some(40318)).contains("尚未實作"));
    }

    #[test]
    fn selecting_a_late_qualified_offer_requests_its_icon_within_the_original_budget() {
        let view = catalog_fixture(true);
        let graphics = requested_icons(&view, Some(40318));
        assert!(
            graphics.contains(&235),
            "selected icon omitted: {graphics:?}"
        );
        assert_eq!(graphics.len(), shop_icons::MAX_ICONS);
        assert!(graphics.windows(2).all(|x| x[0] < x[1]));
        assert!(!requested_icons(&catalog_fixture(false), Some(40318)).contains(&235));
        assert_eq!(
            requested_icons(&view, Some(-1)),
            requested_icons(&view, None)
        );
    }

    #[test]
    fn icon_budget_counts_unique_graphics_and_stable_selection_does_not_churn_keys() {
        let mut view = catalog_fixture(true);
        let ordinary = requested_icons(&view, None);
        assert_eq!(requested_icons(&view, Some(40000)), ordinary);
        let selected = requested_icons(&view, Some(40318));
        for _ in 0..20 {
            assert_eq!(requested_icons(&view, Some(40318)), selected);
        }
        let shop::State::Ready { confirmed, .. } = &mut view.state else {
            unreachable!()
        };
        let snapshot = Arc::make_mut(Arc::make_mut(confirmed).snapshot.as_mut().unwrap());
        snapshot.offers[1].inventory_graphic = snapshot.offers[0].inventory_graphic;
        view.state.reply().unwrap().validate().unwrap();
        let deduplicated = requested_icons(&view, None);
        assert_eq!(deduplicated.len(), shop_icons::MAX_ICONS);
        assert!(
            deduplicated.contains(&235),
            "duplicate art cannot crowd out the last distinct icon"
        );
        assert!(requested_icons(&view_fixture(), Some(40318)).is_empty());
    }

    #[test]
    fn shop_button_intents_preserve_confirmed_scope_wallet_and_quantity_validation() {
        let view = catalog_fixture(true);
        let before = view.clone();
        for action in [shop::Action::Buy, shop::Action::Sell] {
            for (text, count) in [("1", 1), ("4", 4), ("1000000", 1000000)] {
                let q = transaction_request(&view, Some(40318), action, text).unwrap();
                assert_eq!(
                    q,
                    view.request
                        .mutation(view.state.snapshot().unwrap(), action, 40318, count)
                );
                q.validate().unwrap();
            }
        }
        for text in ["", "0", "-1", "1.0", " 1", "1 ", "1000001", "１２"] {
            assert!(transaction_request(&view, Some(40318), shop::Action::Buy, text).is_err());
        }
        assert!(transaction_request(&view, None, shop::Action::Buy, "1").is_err());
        assert!(transaction_request(&view, Some(99999), shop::Action::Buy, "1").is_err());
        assert!(transaction_request(&view, Some(40318), shop::Action::Read, "1").is_err());
        assert!(
            transaction_request(&catalog_fixture(false), Some(40318), shop::Action::Buy, "1")
                .is_err()
        );
        let mut pending = view.clone();
        pending.pending = true;
        assert!(transaction_request(&pending, Some(40318), shop::Action::Buy, "1").is_err());
        assert_eq!(
            view, before,
            "building an intent cannot spend, stock or update the confirmed wallet"
        );
    }

    #[test]
    #[ignore = "requires real shared resources; selection-driven bounded shop icon worker, not an HWND or independent decoder golden"]
    fn actual_shop_selected_material_icon_loads_after_catalog_preload_and_retires() {
        use sha2::{Digest, Sha256};
        use std::time::{Duration, Instant};
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root"),
        );
        assert!(root.is_absolute());
        let view = catalog_fixture(true);
        let mut loader = shop_icons::Loader::new(&root);
        for selection in [None, Some(40318), Some(40000)] {
            let wanted = requested_icons(&view, selection);
            assert_eq!(wanted.len(), shop_icons::MAX_ICONS);
            loader.request(&wanted).unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            loop {
                loader.poll();
                if loader.is_quiescent()
                    && loader.bank().keys().copied().collect::<Vec<_>>() == wanted
                {
                    break;
                }
                assert!(Instant::now() < deadline, "selection icon worker deadline");
                std::thread::park_timeout(Duration::from_millis(5));
            }
            let bank = loader.bank();
            if selection == Some(40318) {
                let gem = bank
                    .get(&235)
                    .unwrap()
                    .as_ref()
                    .expect("selected real gem icon");
                assert_eq!(gem.source.name.to_ascii_lowercase(), "235.tbt");
                assert!(gem.width > 0 && gem.height > 0);
                assert_eq!(gem.bgra.len(), gem.width as usize * gem.height as usize * 4);
                assert!(gem.bgra.chunks_exact(4).any(|p| p[..3] != [32, 32, 32]));
                eprintln!("SHOP_SELECTED_ICON graphic=235 source={:?} width={} height={} bgra_sha256={:X}", gem.source, gem.width, gem.height, Sha256::digest(&gem.bgra));
            } else {
                assert!(
                    !bank.contains_key(&235),
                    "selection replacement retires the prior bank"
                );
            }
        }
        loader.cancel();
        loader.poll();
        assert!(loader.is_quiescent());
        assert!(loader.request(&[235]).is_err());
    }

    fn view_fixture() -> View {
        use crate::modern_lito::application_protocol::{CharacterKey, SelectionContext};
        View {
            request: shop::Request::read(
                SelectionContext {
                    key: CharacterKey {
                        source_id: 7,
                        object_id: 3333,
                    },
                    epoch: 11,
                },
                0,
                None,
            ),
            state: shop::State::NotRequested,
            pending: true,
            status: "舊交易待確認".into(),
            labels: BTreeMap::from([(40010, "舊角色商品".into())]),
        }
    }

    // A controllable, genuinely owned thread represents icon IO which has not
    // returned yet. Assertions cannot strand it: Drop releases and joins it.
    struct HeldWorker {
        release: Option<std::sync::mpsc::Sender<()>>,
        handle: Option<std::thread::JoinHandle<()>>,
    }
    impl HeldWorker {
        fn new() -> Self {
            let (release, wait) = std::sync::mpsc::channel();
            let (started, ready) = std::sync::mpsc::channel();
            let handle = std::thread::spawn(move || {
                let _ = started.send(());
                let _ = wait.recv();
            });
            let result = Self {
                release: Some(release),
                handle: Some(handle),
            };
            ready
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
            result
        }
        fn is_quiescent(&self) -> bool {
            self.handle.is_none()
        }
        fn finish(&mut self) {
            self.release.take().unwrap().send(()).unwrap();
            self.handle.take().unwrap().join().unwrap();
        }
    }
    impl Drop for HeldWorker {
        fn drop(&mut self) {
            if let Some(release) = self.release.take() {
                let _ = release.send(());
            }
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }

    #[test]
    fn retiring_worker_does_not_accept_a_reconnected_selection_or_claim_early_retirement() {
        let mut worker = HeldWorker::new();
        let original = view_fixture();
        for change in 0..4 {
            let mut next = original.clone();
            match change {
                0 => next.request.context.key.source_id = 42,
                1 => next.request.context.key.object_id += 1,
                2 => next.request.context.epoch += 1,
                _ => next.request.channel_id = 1,
            }
            next.pending = false;
            next.status = "新角色商店".into();
            next.labels.clear();
            for cancelled in [false, true] {
                let mut current = original.clone();
                assert_eq!(
                    apply_view(
                        &mut current,
                        Some(next.clone()),
                        false,
                        cancelled,
                        cancelled
                    ),
                    ViewUpdate::Retire,
                );
                assert_eq!(current, original, "new scope must not overwrite old view");
                assert!(!worker.is_quiescent());
                assert!(!worker.handle.as_ref().unwrap().is_finished());
            }
        }
        worker.finish();
        assert!(worker.is_quiescent(), "retirement requires the real join");
    }

    #[test]
    fn cancelled_or_missing_scope_retires_without_overwriting_prior_failure() {
        let mut original = view_fixture();
        original.status = "商店視窗無法完整更新，已停止提交操作。".into();
        let mut next = original.clone();
        next.pending = false;
        next.status = "新連線已確認".into();
        for (view, owner_closing, cancelled) in [
            (Some(next.clone()), true, false),
            (Some(next), false, true),
            (None, false, false),
        ] {
            let mut current = original.clone();
            assert_eq!(
                apply_view(&mut current, view, owner_closing, true, cancelled),
                ViewUpdate::Retire,
            );
            assert_eq!(current, original);
        }
    }

    #[test]
    fn normal_return_waits_for_same_scope_transaction_ack_without_refreshing_data() {
        let original = view_fixture();
        let mut current = original.clone();
        let mut next = original.clone();
        next.status = "新的商店資料".into();
        next.labels.clear();
        assert_eq!(
            apply_view(&mut current, Some(next.clone()), false, true, false),
            ViewUpdate::AwaitTransaction,
        );
        assert_eq!(current, original);
        next.pending = false;
        assert_eq!(
            apply_view(&mut current, Some(next), false, true, false),
            ViewUpdate::AwaitTransaction,
        );
        let mut acknowledged = original;
        acknowledged.pending = false;
        assert_eq!(current, acknowledged);
    }

    #[test]
    fn current_scope_still_applies_confirmed_view_and_notice_reset() {
        let mut current = view_fixture();
        let mut next = current.clone();
        next.pending = false;
        next.status = "交易已確認".into();
        assert_eq!(
            apply_view(&mut current, Some(next.clone()), false, false, false),
            ViewUpdate::Current { clear_notice: true },
        );
        assert_eq!(current, next);
        assert_eq!(
            apply_view(&mut current, Some(next), false, false, false),
            ViewUpdate::Current {
                clear_notice: false
            },
        );
    }

    #[test]
    fn local_validation_notice_survives_unchanged_timer_and_pending_takes_priority() {
        for _ in 0..20 {
            assert_eq!(
                status_text(false, false, Some("數量錯誤"), "已連線"),
                "數量錯誤"
            );
        }
        assert!(status_text(false, true, Some("數量錯誤"), "舊資料").contains("等待伺服器確認"));
        assert!(status_text(true, true, None, "舊資料").contains("不會重送"));
        assert_eq!(status_text(false, false, None, "新ACK"), "新ACK");
    }
    #[test]
    fn quantities_reject_coercion_and_preserve_boundaries() {
        assert_eq!(quantity("1000000").unwrap(), 1_000_000);
        assert_eq!(quantity("1").unwrap(), 1);
        for s in ["0", "-1", "1.0", " 1", "1 ", "1000001", "１２"] {
            assert!(quantity(s).is_err(), "{s}");
        }
    }
    #[test]
    fn tab_routes_out_of_readonly_edit_and_skips_disabled_controls() {
        let all = [true; COUNT];
        assert_eq!(next_focus(DETAIL, false, &all), Some(QUANTITY));
        assert_eq!(next_focus(DETAIL, true, &all), Some(LIST));
        let mut busy = [false; COUNT];
        busy[DETAIL] = true;
        busy[BACK] = true;
        assert_eq!(next_focus(DETAIL, false, &busy), Some(BACK));
        assert_eq!(next_focus(BACK, false, &busy), Some(DETAIL));
        assert_eq!(next_focus(BACK, true, &busy), Some(DETAIL));
        assert_eq!(next_focus(LIST, false, &[false; COUNT]), None);
    }
    #[test]
    fn dpi_layout_keeps_controls_inside_and_transaction_row_disjoint() {
        for dpi in [96, 120, 144, 192, 384] {
            let w = px(WIDTH, dpi);
            let h = px(HEIGHT, dpi);
            let b = layout(w, h, dpi);
            for r in b {
                assert!(
                    r.x >= 0 && r.y >= 0 && r.w > 0 && r.h > 0 && r.x + r.w <= w && r.y + r.h <= h,
                    "{dpi}:{r:?}"
                );
            }
            assert!(b[LIST].y + b[LIST].h <= b[DETAIL].y);
            assert!(b[DETAIL].y + b[DETAIL].h <= b[BUY].y);
            for (a, c) in [(QUANTITY, BUY), (BUY, SELL), (SELL, READ)] {
                assert!(b[a].x + b[a].w <= b[c].x);
            }
        }
    }
}
