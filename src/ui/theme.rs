use crate::platform::win32::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLORREF, CreateFontW, CreateSolidBrush,
    DEFAULT_CHARSET, DEFAULT_PITCH, DeleteObject, DestroyIcon, ExtractIconExW, FF_DONTCARE,
    FF_MODERN, FIXED_PITCH, FW_BOLD, FW_NORMAL, GetSystemDirectoryW, HBRUSH, HCF_HIGHCONTRASTON,
    HFONT, HICON, HIGHCONTRASTW, MulDiv, OUT_DEFAULT_PRECIS, SPI_GETHIGHCONTRAST,
    SystemParametersInfoW, to_wide_null,
};
use std::path::Path;

pub const COLOR_TAB_BG: COLORREF = 0x00FF_FFFF;
pub const COLOR_CODE_BG: COLORREF = 0x00FC_FAF8;
pub const COLOR_OK_BG: COLORREF = 0x00F5_FDEC;
pub const COLOR_WARN_BG: COLORREF = 0x00EB_FBFF;
pub const COLOR_DANGER_BG: COLORREF = 0x00F2_F2FE;

pub const COLOR_TEXT_PRIMARY: COLORREF = 0x002A_170F;
pub const COLOR_TEXT_SECONDARY: COLORREF = 0x0069_5547;
pub const COLOR_TEXT_INFO: COLORREF = 0x0078_3A0F;
pub const COLOR_TEXT_OK: COLORREF = 0x003D_6B15;
pub const COLOR_TEXT_WARN: COLORREF = 0x000E_4092;
pub const COLOR_TEXT_DANGER: COLORREF = 0x001C_1CB9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTone {
    Idle,
    Running,
    Ok,
    Warn,
    Danger,
}

pub struct FontGuard(pub HFONT);

impl Drop for FontGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DeleteObject(self.0);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}

pub struct BrushGuard(pub HBRUSH);

impl Drop for BrushGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DeleteObject(self.0);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}

pub struct IconGuard(pub HICON);

impl Drop for IconGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DestroyIcon(self.0);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}

pub fn scale(v: i32, dpi: u32) -> i32 {
    unsafe { MulDiv(v, dpi as i32, 96) }
}

pub fn is_high_contrast() -> bool {
    let mut hc = HIGHCONTRASTW::default();
    let ok =
        unsafe { SystemParametersInfoW(SPI_GETHIGHCONTRAST, hc.cbSize, (&raw mut hc).cast(), 0) };
    ok != 0 && (hc.dwFlags & HCF_HIGHCONTRASTON) != 0
}

pub struct Theme {
    pub high_contrast: bool,
    pub font_normal: FontGuard,
    pub font_bold: FontGuard,
    pub font_mono: FontGuard,
    pub brush_tab_bg: BrushGuard,
    pub brush_code_bg: BrushGuard,
    pub brush_ok_bg: BrushGuard,
    pub brush_warn_bg: BrushGuard,
    pub brush_danger_bg: BrushGuard,
}

impl Theme {
    pub fn for_dpi(dpi: u32) -> Self {
        let dpi = dpi.max(96);
        let h_ui = -unsafe { MulDiv(19, dpi as i32, 144) };
        let face_ui = to_wide_null("BIZ UDPゴシック");
        let face_mono = to_wide_null("BIZ UDゴシック");

        unsafe {
            let font_normal = CreateFontW(
                h_ui,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                DEFAULT_PITCH | FF_DONTCARE,
                face_ui.as_ptr(),
            );
            let font_bold = CreateFontW(
                h_ui,
                0,
                0,
                0,
                FW_BOLD,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                DEFAULT_PITCH | FF_DONTCARE,
                face_ui.as_ptr(),
            );
            let font_mono = CreateFontW(
                h_ui,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                FIXED_PITCH | FF_MODERN,
                face_mono.as_ptr(),
            );

            Self {
                high_contrast: is_high_contrast(),
                font_normal: FontGuard(font_normal),
                font_bold: FontGuard(font_bold),
                font_mono: FontGuard(font_mono),
                brush_tab_bg: BrushGuard(CreateSolidBrush(COLOR_TAB_BG)),
                brush_code_bg: BrushGuard(CreateSolidBrush(COLOR_CODE_BG)),
                brush_ok_bg: BrushGuard(CreateSolidBrush(COLOR_OK_BG)),
                brush_warn_bg: BrushGuard(CreateSolidBrush(COLOR_WARN_BG)),
                brush_danger_bg: BrushGuard(CreateSolidBrush(COLOR_DANGER_BG)),
            }
        }
    }

    pub fn refresh_system_mode(&mut self) {
        self.high_contrast = is_high_contrast();
    }
}

fn try_extract_icon(dll_path: &Path, index: i32) -> Option<(IconGuard, IconGuard)> {
    let path_str = dll_path.to_string_lossy();
    let path_w = to_wide_null(&path_str);
    let mut large: HICON = std::ptr::null_mut();
    let mut small: HICON = std::ptr::null_mut();
    let count =
        unsafe { ExtractIconExW(path_w.as_ptr(), index, &raw mut large, &raw mut small, 1) };
    if count > 0 && (!large.is_null() || !small.is_null()) {
        Some((IconGuard(large), IconGuard(small)))
    } else {
        if !large.is_null() {
            unsafe {
                DestroyIcon(large);
            }
        }
        if !small.is_null() {
            unsafe {
                DestroyIcon(small);
            }
        }
        None
    }
}

pub fn extract_app_icons() -> (IconGuard, IconGuard) {
    let mut buf = [0u16; 260];
    let len = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) };
    if len > 0 && (len as usize) < buf.len() {
        let sys_dir = String::from_utf16_lossy(&buf[..len as usize]);
        let sys_path = Path::new(&sys_dir);
        if let Some(pair) = try_extract_icon(&sys_path.join("imageres.dll"), 31) {
            return pair;
        }
        if let Some(pair) = try_extract_icon(&sys_path.join("shell32.dll"), 8) {
            return pair;
        }
    }
    (
        IconGuard(std::ptr::null_mut()),
        IconGuard(std::ptr::null_mut()),
    )
}
