#![cfg_attr(not(windows), allow(dead_code))]
use patapim::Document;
#[cfg(windows)]
use patapim::{Color, Paint};
use std::path::Path;

const WELCOME: &str = include_str!("../assets/welcome.html");
struct Tab {
    address: String,
    page: Document,
}
struct App {
    tabs: Vec<Tab>,
    active: usize,
    input: String,
    editing: bool,
    width: u32,
    height: u32,
    status: String,
}
impl App {
    fn new() -> Self {
        let page = Document::from_local_html(WELCOME, 1000, 540);
        Self {
            tabs: vec![Tab {
                address: "akateku:welcome".into(),
                page,
            }],
            active: 0,
            input: "akateku:welcome".into(),
            editing: false,
            width: 1100,
            height: 760,
            status: "Local documents only".into(),
        }
    }
    fn new_tab(&mut self) {
        self.tabs.push(Tab {
            address: "akateku:welcome".into(),
            page: Document::from_local_html(WELCOME, self.width, self.height.saturating_sub(220)),
        });
        self.active = self.tabs.len() - 1;
        self.input = self.tabs[self.active].address.clone();
        self.editing = false;
    }
    fn navigate(&mut self) {
        let address = self.input.trim().to_string();
        if address == "akateku:welcome" {
            self.tabs[self.active] = Tab {
                address: address.clone(),
                page: Document::from_local_html(
                    WELCOME,
                    self.width,
                    self.height.saturating_sub(220),
                ),
            };
            self.status = "Welcome page".into();
            return;
        }
        if address.starts_with("http:")
            || address.starts_with("https:")
            || address.starts_with("javascript:")
        {
            self.status = "Network and scripts are disabled in M0".into();
            return;
        }
        let path = address
            .strip_prefix("file:///")
            .unwrap_or(&address)
            .replace('/', "\\");
        let Ok(canonical) = Path::new(&path).canonicalize() else {
            self.status = "File not found".into();
            return;
        };
        if canonical
            .extension()
            .and_then(|e| e.to_str())
            .is_none_or(|e| !e.eq_ignore_ascii_case("html"))
        {
            self.status = "Only .html files can be opened".into();
            return;
        }
        let Ok(meta) = std::fs::metadata(&canonical) else {
            self.status = "Cannot read file".into();
            return;
        };
        if meta.len() > 1024 * 1024 {
            self.status = "File exceeds 1 MiB limit".into();
            return;
        }
        let Ok(html) = std::fs::read_to_string(&canonical) else {
            self.status = "File must be UTF-8".into();
            return;
        };
        let page = Document::from_local_html(&html, self.width, self.height.saturating_sub(220));
        self.tabs[self.active] = Tab {
            address: canonical.display().to_string(),
            page,
        };
        self.input = self.tabs[self.active].address.clone();
        self.editing = false;
        self.status = "Local file loaded".into();
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Akateku M0 GUI currently supports Windows x64 only.");
}
#[cfg(windows)]
fn main() {
    win::run();
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::{
        ffi::c_void,
        ptr::null,
        sync::{Mutex, OnceLock},
    };
    type Handle = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct PaintStruct {
        hdc: Handle,
        erase: i32,
        paint: Rect,
        restore: i32,
        incupdate: i32,
        reserved: [u8; 32],
    }
    #[repr(C)]
    struct Msg {
        hwnd: Handle,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        point: Point,
        private: u32,
    }
    #[repr(C)]
    struct WndClass {
        style: u32,
        proc: Option<unsafe extern "system" fn(Handle, u32, usize, isize) -> isize>,
        cls_extra: i32,
        win_extra: i32,
        instance: Handle,
        icon: Handle,
        cursor: Handle,
        background: Handle,
        menu: *const u16,
        name: *const u16,
    }
    #[repr(C)]
    struct GdiPlusStartupInput {
        version: u32,
        callback: Handle,
        suppress_background_thread: i32,
        suppress_external_codecs: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn RegisterClassW(c: *const WndClass) -> u16;
        fn CreateWindowExW(
            ex: u32,
            class: *const u16,
            title: *const u16,
            style: u32,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            parent: Handle,
            menu: Handle,
            instance: Handle,
            param: Handle,
        ) -> Handle;
        fn DefWindowProcW(w: Handle, m: u32, a: usize, b: isize) -> isize;
        fn ShowWindow(w: Handle, n: i32) -> i32;
        fn UpdateWindow(w: Handle) -> i32;
        fn GetMessageW(m: *mut Msg, w: Handle, min: u32, max: u32) -> i32;
        fn TranslateMessage(m: *const Msg) -> i32;
        fn DispatchMessageW(m: *const Msg) -> isize;
        fn PostQuitMessage(n: i32);
        fn BeginPaint(w: Handle, p: *mut PaintStruct) -> Handle;
        fn EndPaint(w: Handle, p: *const PaintStruct) -> i32;
        fn InvalidateRect(w: Handle, r: *const Rect, erase: i32) -> i32;
        fn GetKeyState(key: i32) -> i16;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreateSolidBrush(color: u32) -> Handle;
        fn SaveDC(h: Handle) -> i32;
        fn RestoreDC(h: Handle, saved: i32) -> i32;
        fn IntersectClipRect(h: Handle, left: i32, top: i32, right: i32, bottom: i32) -> i32;
        fn DeleteObject(o: Handle) -> i32;
        fn FillRect(h: Handle, r: *const Rect, brush: Handle) -> i32;
        fn SetBkMode(h: Handle, mode: i32) -> i32;
        fn SetTextColor(h: Handle, color: u32) -> u32;
        fn TextOutW(h: Handle, x: i32, y: i32, text: *const u16, len: i32) -> i32;
        fn CreateFontW(
            height: i32,
            width: i32,
            esc: i32,
            orient: i32,
            weight: i32,
            italic: u32,
            underline: u32,
            strike: u32,
            charset: u32,
            out: u32,
            clip: u32,
            quality: u32,
            pitch: u32,
            face: *const u16,
        ) -> Handle;
        fn SelectObject(h: Handle, o: Handle) -> Handle;
    }
    #[link(name = "gdiplus")]
    unsafe extern "system" {
        fn GdiplusStartup(
            token: *mut usize,
            input: *const GdiPlusStartupInput,
            output: Handle,
        ) -> i32;
        fn GdiplusShutdown(token: usize);
        fn GdipCreateFromHDC(hdc: Handle, graphics: *mut Handle) -> i32;
        fn GdipDeleteGraphics(graphics: Handle) -> i32;
        fn GdipSetSmoothingMode(graphics: Handle, mode: i32) -> i32;
        fn GdipCreatePath(fill_mode: i32, path: *mut Handle) -> i32;
        fn GdipAddPathArcI(
            path: Handle,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            start: f32,
            sweep: f32,
        ) -> i32;
        fn GdipClosePathFigure(path: Handle) -> i32;
        fn GdipDeletePath(path: Handle) -> i32;
        fn GdipCreateSolidFill(argb: u32, brush: *mut Handle) -> i32;
        fn GdipDeleteBrush(brush: Handle) -> i32;
        fn GdipFillPath(graphics: Handle, brush: Handle, path: Handle) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> Handle;
    }
    static STATE: OnceLock<Mutex<App>> = OnceLock::new();
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    fn rgb(c: Color) -> u32 {
        c.0 as u32 | (c.1 as u32) << 8 | (c.2 as u32) << 16
    }
    unsafe fn rect(h: Handle, x: i32, y: i32, w: i32, height: i32, c: Color) {
        let r = Rect {
            left: x,
            top: y,
            right: x + w,
            bottom: y + height,
        };
        let b = CreateSolidBrush(rgb(c));
        FillRect(h, &r, b);
        DeleteObject(b);
    }
    unsafe fn round_rect(h: Handle, x: i32, y: i32, w: i32, height: i32, radius: i32, c: Color) {
        if w <= 0 || height <= 0 {
            return;
        }
        let mut graphics = null::<c_void>() as Handle;
        if GdipCreateFromHDC(h, &mut graphics) != 0 {
            rect(h, x, y, w, height, c);
            return;
        }
        GdipSetSmoothingMode(graphics, 4); // SmoothingModeAntiAlias
        let mut path = null::<c_void>() as Handle;
        let mut brush = null::<c_void>() as Handle;
        if GdipCreatePath(0, &mut path) == 0 {
            let diameter = (radius * 2).min(w).min(height).max(1);
            let right = x + w - diameter;
            let bottom = y + height - diameter;
            GdipAddPathArcI(path, x, y, diameter, diameter, 180.0, 90.0);
            GdipAddPathArcI(path, right, y, diameter, diameter, 270.0, 90.0);
            GdipAddPathArcI(path, right, bottom, diameter, diameter, 0.0, 90.0);
            GdipAddPathArcI(path, x, bottom, diameter, diameter, 90.0, 90.0);
            GdipClosePathFigure(path);
            let argb = 0xff00_0000 | ((c.0 as u32) << 16) | ((c.1 as u32) << 8) | c.2 as u32;
            if GdipCreateSolidFill(argb, &mut brush) == 0 {
                GdipFillPath(graphics, brush, path);
                GdipDeleteBrush(brush);
            }
            GdipDeletePath(path);
        }
        GdipDeleteGraphics(graphics);
    }
    unsafe fn text(h: Handle, x: i32, y: i32, s: &str, size: u32, c: Color) {
        let face = wide("Segoe UI");
        let font = CreateFontW(
            -(size as i32),
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            1,
            0,
            0,
            5, // CLEARTYPE_QUALITY
            0,
            face.as_ptr(),
        );
        let old = SelectObject(h, font);
        SetBkMode(h, 1);
        SetTextColor(h, rgb(c));
        let utf: Vec<u16> = s.encode_utf16().collect();
        TextOutW(h, x, y, utf.as_ptr(), utf.len() as i32);
        SelectObject(h, old);
        DeleteObject(font);
    }
    unsafe fn draw(h: Handle, a: &App) {
        let width = a.width as i32;
        let height = a.height as i32;
        let graphite = Color(29, 34, 42);
        let raised = Color(43, 50, 60);
        let muted = Color(161, 174, 190);
        let mint = Color(105, 218, 188);
        rect(h, 0, 0, width, height, Color(231, 235, 240));
        rect(h, 0, 0, width, 160, graphite);
        round_rect(h, 16, 11, 34, 32, 10, mint);
        text(h, 26, 17, "A", 18, graphite);
        text(h, 60, 15, "AKATEKU", 18, Color(247, 249, 250));
        text(h, 169, 19, "BROWSER", 11, muted);
        text(h, width - 168, 19, "PATAPIM  /  M0", 11, mint);
        rect(h, 0, 47, width, 1, Color(61, 70, 82));
        for (i, t) in a.tabs.iter().enumerate().take(6) {
            let x = 16 + i as i32 * 184;
            round_rect(
                h,
                x,
                55,
                176,
                37,
                11,
                if i == a.active {
                    Color(234, 239, 243)
                } else {
                    raised
                },
            );
            round_rect(
                h,
                x + 11,
                66,
                7,
                7,
                3,
                if i == a.active {
                    Color(40, 159, 137)
                } else {
                    mint
                },
            );
            text(
                h,
                x + 26,
                64,
                if t.address == "akateku:welcome" {
                    "New tab"
                } else {
                    "Local page"
                },
                14,
                if i == a.active {
                    graphite
                } else {
                    Color(221, 229, 234)
                },
            );
        }
        let plus_x = 16 + a.tabs.len().min(6) as i32 * 184;
        if plus_x + 38 < width {
            round_rect(h, plus_x, 55, 37, 37, 11, raised);
            text(h, plus_x + 11, 58, "+", 23, mint);
        }
        round_rect(h, 16, 105, 52, 42, 13, raised);
        text(h, 23, 118, "Home", 12, Color(213, 223, 230));
        round_rect(
            h,
            78,
            104,
            (width - 172).max(20),
            44,
            14,
            Color(239, 243, 246),
        );
        round_rect(h, 91, 118, 15, 15, 7, Color(66, 163, 141));
        round_rect(h, 95, 122, 7, 7, 3, Color(239, 243, 246));
        let shown = if a.editing {
            &a.input
        } else {
            &a.tabs[a.active].address
        };
        text(h, 116, 115, shown, 15, Color(37, 48, 59));
        round_rect(h, width - 82, 104, 66, 44, 14, mint);
        text(h, width - 63, 115, "Go", 15, graphite);
        round_rect(
            h,
            12,
            172,
            (width - 24).max(1),
            (height - 220).max(1),
            18,
            Color(208, 215, 223),
        );
        round_rect(
            h,
            13,
            171,
            (width - 26).max(1),
            (height - 220).max(1),
            18,
            Color(255, 255, 255),
        );
        let saved = SaveDC(h);
        IntersectClipRect(h, 22, 177, width - 22, height - 55);
        for p in a.tabs[a.active].page.display_list.iter().skip(1) {
            match p {
                Paint::Rect {
                    x,
                    y,
                    w,
                    h: height,
                    color,
                } => rect(h, *x, *y + 171, *w as i32, *height as i32, *color),
                Paint::Text {
                    x,
                    y,
                    text: s,
                    size,
                    color,
                } => text(h, *x, *y + 171, s, *size, *color),
            }
        }
        RestoreDC(h, saved);
        rect(h, 0, height - 38, width, 38, Color(222, 227, 234));
        round_rect(h, 16, height - 27, 7, 7, 3, Color(53, 165, 139));
        let m = &a.tabs[a.active].page.metrics;
        text(
            h,
            32,
            height - 28,
            &format!(
                "{} | nodes {} | commands {} | parse {:.2} ms | layout {:.2} ms",
                a.status,
                m.nodes,
                m.commands,
                m.parse.as_secs_f64() * 1000.0,
                m.layout.as_secs_f64() * 1000.0
            ),
            12,
            Color(75, 87, 99),
        );
    }
    unsafe extern "system" fn proc(w: Handle, m: u32, a: usize, b: isize) -> isize {
        match m {
            2 => {
                PostQuitMessage(0);
                0
            }
            5 => {
                if let Some(state) = STATE.get() {
                    let mut s = state.lock().unwrap();
                    s.width = (b as u32 & 0xffff).max(1);
                    s.height = ((b as u32 >> 16) & 0xffff).max(1);
                    let width = s.width;
                    let height = s.height.saturating_sub(220);
                    for t in &mut s.tabs {
                        t.page.resize(width, height)
                    }
                }
                InvalidateRect(w, null(), 0);
                0
            }
            15 => {
                let mut ps = PaintStruct {
                    hdc: null::<c_void>() as Handle,
                    erase: 0,
                    paint: Rect::default(),
                    restore: 0,
                    incupdate: 0,
                    reserved: [0; 32],
                };
                let h = BeginPaint(w, &mut ps);
                if let Some(state) = STATE.get() {
                    draw(h, &state.lock().unwrap())
                }
                EndPaint(w, &ps);
                0
            }
            256 => {
                if let Some(state) = STATE.get() {
                    let mut s = state.lock().unwrap();
                    let ctrl = GetKeyState(17) < 0;
                    match a {
                        84 if ctrl => s.new_tab(),
                        76 if ctrl => {
                            s.editing = true;
                            s.input = s.tabs[s.active].address.clone()
                        }
                        13 => s.navigate(),
                        8 if s.editing => {
                            s.input.pop();
                        }
                        _ => {}
                    }
                }
                InvalidateRect(w, null(), 0);
                0
            }
            258 => {
                if let Some(state) = STATE.get() {
                    let mut s = state.lock().unwrap();
                    if s.editing && a >= 32 && a != 127 {
                        if let Some(ch) = char::from_u32(a as u32) {
                            s.input.push(ch)
                        }
                    }
                }
                InvalidateRect(w, null(), 0);
                0
            }
            513 => {
                if let Some(state) = STATE.get() {
                    let mut s = state.lock().unwrap();
                    let x = (b as u32 & 0xffff) as i32;
                    let y = ((b as u32 >> 16) & 0xffff) as i32;
                    if (55..92).contains(&y) && x >= 16 {
                        let i = ((x - 16) / 184) as usize;
                        if i < s.tabs.len() {
                            s.active = i;
                            s.input = s.tabs[i].address.clone();
                            s.editing = false
                        } else if i == s.tabs.len() && x < 16 + (i as i32) * 184 + 37 {
                            s.new_tab();
                        }
                    } else if (104..148).contains(&y) {
                        if x >= s.width as i32 - 82 {
                            s.navigate()
                        } else if x >= 78 {
                            s.editing = true;
                            s.input = s.tabs[s.active].address.clone()
                        } else if x >= 16 {
                            s.input = "akateku:welcome".into();
                            s.navigate();
                        }
                    }
                }
                InvalidateRect(w, null(), 0);
                0
            }
            _ => DefWindowProcW(w, m, a, b),
        }
    }
    pub fn run() {
        unsafe {
            STATE.set(Mutex::new(App::new())).ok();
            let mut gdip_token = 0usize;
            let gdip_input = GdiPlusStartupInput {
                version: 1,
                callback: null::<c_void>() as Handle,
                suppress_background_thread: 0,
                suppress_external_codecs: 0,
            };
            if GdiplusStartup(&mut gdip_token, &gdip_input, null::<c_void>() as Handle) != 0 {
                eprintln!("GDI+ startup failed");
                return;
            }
            let name = wide("AkatekuGraphite");
            let title = wide("AkatekuBrowser — Patapim M0");
            let instance = GetModuleHandleW(null());
            let class = WndClass {
                style: 0,
                proc: Some(proc),
                cls_extra: 0,
                win_extra: 0,
                instance,
                icon: null::<c_void>() as Handle,
                cursor: null::<c_void>() as Handle,
                background: null::<c_void>() as Handle,
                menu: null(),
                name: name.as_ptr(),
            };
            if RegisterClassW(&class) == 0 {
                eprintln!("RegisterClassW failed");
                GdiplusShutdown(gdip_token);
                return;
            }
            let w = CreateWindowExW(
                0,
                name.as_ptr(),
                title.as_ptr(),
                0x10cf0000,
                100,
                100,
                1100,
                760,
                null::<c_void>() as Handle,
                null::<c_void>() as Handle,
                instance,
                null::<c_void>() as Handle,
            );
            if w.is_null() {
                eprintln!("CreateWindowExW failed");
                GdiplusShutdown(gdip_token);
                return;
            }
            ShowWindow(w, 1);
            UpdateWindow(w);
            let mut msg = Msg {
                hwnd: null::<c_void>() as Handle,
                message: 0,
                wparam: 0,
                lparam: 0,
                time: 0,
                point: Point::default(),
                private: 0,
            };
            while GetMessageW(&mut msg, null::<c_void>() as Handle, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            GdiplusShutdown(gdip_token);
        }
    }
}
