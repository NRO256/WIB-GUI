#![allow(clippy::upper_case_acronyms)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HMENU = *mut c_void;
pub type HFONT = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HDC = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HICON = *mut c_void;
pub type HGLOBAL = *mut c_void;
pub type HANDLE = *mut c_void;
pub type UINT_PTR = usize;
pub type DWORD_PTR = usize;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type BOOL = i32;
pub type DWORD = u32;
pub type WORD = u16;
pub type UINT = u32;
pub type LONG = i32;
pub type ATOM = u16;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// Window Messages
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_SETFONT: UINT = 0x0030;
pub const WM_NOTIFY: UINT = 0x004E;
pub const WM_NCDESTROY: UINT = 0x0082;
pub const WM_GETDLGCODE: UINT = 0x0087;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_CHAR: UINT = 0x0102;
pub const WM_DPICHANGED: UINT = 0x02E0;
pub const WM_APP: UINT = 0x8000;

// Window Styles
pub const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF_0000;
pub const WS_CHILD: DWORD = 0x4000_0000;
pub const WS_VISIBLE: DWORD = 0x1000_0000;
pub const WS_TABSTOP: DWORD = 0x0001_0000;
pub const WS_GROUP: DWORD = 0x0002_0000;
pub const WS_VSCROLL: DWORD = 0x0020_0000;
pub const WS_BORDER: DWORD = 0x0080_0000;
pub const WS_EX_CLIENTEDGE: DWORD = 0x0000_0200;

// Button / Static / Edit / ComboBox / ListView / Tab / Progress Styles & Messages
pub const BS_PUSHBUTTON: DWORD = 0x0000_0000;
pub const BS_DEFPUSHBUTTON: DWORD = 0x0000_0001;
pub const BS_GROUPBOX: DWORD = 0x0000_0007;
pub const BS_AUTORADIOBUTTON: DWORD = 0x0000_0009;
pub const BM_GETCHECK: UINT = 0x00F0;
pub const BM_SETCHECK: UINT = 0x00F1;
pub const BST_UNCHECKED: usize = 0;
pub const BST_CHECKED: usize = 1;

pub const SS_LEFT: DWORD = 0x0000_0000;
pub const SS_NOPREFIX: DWORD = 0x0000_0080;

pub const ES_LEFT: DWORD = 0x0000;
pub const ES_MULTILINE: DWORD = 0x0004;
pub const ES_PASSWORD: DWORD = 0x0020;
pub const ES_AUTOVSCROLL: DWORD = 0x0040;
pub const ES_AUTOHSCROLL: DWORD = 0x0080;
pub const ES_READONLY: DWORD = 0x0800;
pub const EM_GETSEL: UINT = 0x00B0;
pub const EM_SETSEL: UINT = 0x00B1;
pub const EM_REPLACESEL: UINT = 0x00C2;
pub const EM_SETLIMITTEXT: UINT = 0x00C5;
pub const EN_CHANGE: WORD = 0x0300;
pub const DLGC_WANTTAB: LRESULT = 0x0002;
pub const DLGC_WANTALLKEYS: LRESULT = 0x0004;

pub const CBS_DROPDOWNLIST: DWORD = 0x0003;
pub const CBS_HASSTRINGS: DWORD = 0x0200;
pub const CB_ADDSTRING: UINT = 0x0143;
pub const CB_GETCURSEL: UINT = 0x0147;
pub const CB_SETCURSEL: UINT = 0x014E;
pub const CB_RESETCONTENT: UINT = 0x014B;
pub const CBN_SELCHANGE: WORD = 1;

pub const LVS_REPORT: DWORD = 0x0001;
pub const LVS_SINGLESEL: DWORD = 0x0004;
pub const LVS_SHOWSELALWAYS: DWORD = 0x0008;
pub const LVS_EX_CHECKBOXES: DWORD = 0x0000_0004;
pub const LVS_EX_FULLROWSELECT: DWORD = 0x0000_0020;
pub const LVS_EX_GRIDLINES: DWORD = 0x0000_0001;
pub const LVM_FIRST: UINT = 0x1000;
pub const LVM_DELETEALLITEMS: UINT = LVM_FIRST + 9;
pub const LVM_GETNEXTITEM: UINT = LVM_FIRST + 12;
pub const LVM_GETITEMSTATE: UINT = LVM_FIRST + 44;
pub const LVM_SETITEMSTATE: UINT = LVM_FIRST + 43;
pub const LVM_SETEXTENDEDLISTVIEWSTYLE: UINT = LVM_FIRST + 54;
pub const LVM_INSERTITEMW: UINT = LVM_FIRST + 77;
pub const LVM_SETITEMTEXTW: UINT = LVM_FIRST + 116;
pub const LVM_INSERTCOLUMNW: UINT = LVM_FIRST + 97;
pub const LVIF_TEXT: UINT = 0x0001;
pub const LVIF_STATE: UINT = 0x0008;
pub const LVCF_FMT: UINT = 0x0001;
pub const LVCF_WIDTH: UINT = 0x0002;
pub const LVCF_TEXT: UINT = 0x0004;
pub const LVCFMT_LEFT: i32 = 0x0000;
pub const LVNI_SELECTED: usize = 0x0002;
pub const LVN_ITEMCHANGED: i32 = -101;

pub const TCM_FIRST: UINT = 0x1300;
pub const TCM_GETCURSEL: UINT = TCM_FIRST + 11;
pub const TCM_SETCURSEL: UINT = TCM_FIRST + 12;
pub const TCM_INSERTITEMW: UINT = TCM_FIRST + 62;
pub const TCIF_TEXT: UINT = 0x0001;
pub const TCN_SELCHANGE: i32 = -551;

pub const PBM_SETRANGE: UINT = 0x0401;
pub const PBM_SETPOS: UINT = 0x0402;

// Misc
pub const CW_USEDEFAULT: i32 = -0x8000_0000_i32;
pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;
pub const SWP_NOZORDER: UINT = 0x0004;
pub const SWP_NOACTIVATE: UINT = 0x0010;
pub const COLOR_BTNFACE: i32 = 15;
pub const IDC_ARROW: u16 = 32512;
pub const IDI_APPLICATION: u16 = 32512;

pub const MB_OK: UINT = 0x0000_0000;
pub const MB_YESNO: UINT = 0x0000_0004;
pub const MB_ICONERROR: UINT = 0x0000_0010;
pub const MB_ICONQUESTION: UINT = 0x0000_0020;
pub const MB_ICONWARNING: UINT = 0x0000_0030;
pub const MB_ICONINFORMATION: UINT = 0x0000_0040;
pub const MB_DEFBUTTON2: UINT = 0x0000_0100;
pub const IDYES: i32 = 6;

pub const VK_TAB: u16 = 0x09;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_A: u16 = 0x41;

pub const ICC_LISTVIEW_CLASSES: DWORD = 0x0000_0001;
pub const ICC_PROGRESS_CLASS: DWORD = 0x0000_0020;
pub const ICC_TAB_CLASSES: DWORD = 0x0000_0008;
pub const ICC_STANDARD_CLASSES: DWORD = 0x0000_4000;

pub const CF_UNICODETEXT: UINT = 13;
pub const GMEM_MOVEABLE: UINT = 0x0002;

// GDI & Print
pub const FW_NORMAL: i32 = 400;
pub const FW_BOLD: i32 = 700;
pub const DEFAULT_CHARSET: DWORD = 1;
pub const OUT_DEFAULT_PRECIS: DWORD = 0;
pub const CLIP_DEFAULT_PRECIS: DWORD = 0;
pub const CLEARTYPE_QUALITY: DWORD = 5;
pub const DEFAULT_PITCH: DWORD = 0;
pub const FF_DONTCARE: DWORD = 0;
pub const LOGPIXELSY: i32 = 90;
pub const HORZRES: i32 = 8;
pub const VERTRES: i32 = 10;
pub const DT_LEFT: UINT = 0x0000_0000;
pub const DT_WORDBREAK: UINT = 0x0000_0010;
pub const DT_CALCRECT: UINT = 0x0000_0400;
pub const DT_EDITCONTROL: UINT = 0x0000_2000;

pub const PD_RETURNDC: DWORD = 0x0000_0100;
pub const PD_NOSELECTION: DWORD = 0x0000_0004;
pub const PD_NOPAGENUMS: DWORD = 0x0000_0008;
pub const PD_USEDEVMODECOPIESANDCOLLATE: DWORD = 0x0004_0000;

pub const OFN_OVERWRITEPROMPT: DWORD = 0x0000_0002;
pub const OFN_PATHMUSTEXIST: DWORD = 0x0000_0800;
pub const OFN_NOCHANGEDIR: DWORD = 0x0000_0008;

pub const BIF_RETURNONLYFSDIRS: UINT = 0x0000_0001;
pub const BIF_NEWDIALOGSTYLE: UINT = 0x0000_0040;

pub const DRIVE_FIXED: UINT = 3;
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
pub const OPEN_EXISTING: DWORD = 3;
pub const FILE_SHARE_READ: DWORD = 0x0000_0001;
pub const FILE_SHARE_WRITE: DWORD = 0x0000_0002;
pub const IOCTL_STORAGE_GET_DEVICE_NUMBER: DWORD = 0x002D_1080;

pub const ES_CONTINUOUS: DWORD = 0x8000_0000;
pub const ES_SYSTEM_REQUIRED: DWORD = 0x0000_0001;
pub const INFINITE: DWORD = 0xFFFF_FFFF;

pub const CP_932: UINT = 932;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

impl Default for MSG {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HICON,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: DWORD,
    pub dwICC: DWORD,
}

#[repr(C)]
pub struct NMHDR {
    pub hwndFrom: HWND,
    pub idFrom: UINT_PTR,
    pub code: i32,
}

#[repr(C)]
pub struct TCITEMW {
    pub mask: UINT,
    pub dwState: DWORD,
    pub dwStateMask: DWORD,
    pub pszText: *mut u16,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: LPARAM,
}

impl Default for TCITEMW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct LVCOLUMNW {
    pub mask: UINT,
    pub fmt: i32,
    pub cx: i32,
    pub pszText: *mut u16,
    pub cchTextMax: i32,
    pub iSubItem: i32,
    pub iImage: i32,
    pub iOrder: i32,
    pub cxMin: i32,
    pub cxDefault: i32,
    pub cxIdeal: i32,
}

impl Default for LVCOLUMNW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct LVITEMW {
    pub mask: UINT,
    pub iItem: i32,
    pub iSubItem: i32,
    pub state: UINT,
    pub stateMask: UINT,
    pub pszText: *mut u16,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: LPARAM,
    pub iIndent: i32,
    pub iGroupId: i32,
    pub cColumns: UINT,
    pub puColumns: *mut UINT,
    pub piColFmt: *mut i32,
    pub iGroup: i32,
}

impl Default for LVITEMW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: DWORD,
    pub nFilterIndex: DWORD,
    pub lpstrFile: *mut u16,
    pub nMaxFile: DWORD,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: DWORD,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: DWORD,
    pub nFileOffset: WORD,
    pub nFileExtension: WORD,
    pub lpstrDefExt: *const u16,
    pub lCustData: LPARAM,
    pub lpfnHook: *const c_void,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: DWORD,
    pub FlagsEx: DWORD,
}

impl Default for OPENFILENAMEW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct BROWSEINFOW {
    pub hwndOwner: HWND,
    pub pidlRoot: *mut c_void,
    pub pszDisplayName: *mut u16,
    pub lpszTitle: *const u16,
    pub ulFlags: UINT,
    pub lpfn: *const c_void,
    pub lParam: LPARAM,
    pub iImage: i32,
}

impl Default for BROWSEINFOW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct PRINTDLGW {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hDevMode: HGLOBAL,
    pub hDevNames: HGLOBAL,
    pub hDC: HDC,
    pub Flags: DWORD,
    pub nFromPage: WORD,
    pub nToPage: WORD,
    pub nMinPage: WORD,
    pub nMaxPage: WORD,
    pub nCopies: WORD,
    pub hInstance: HINSTANCE,
    pub lCustData: LPARAM,
    pub lpfnPrintHook: *const c_void,
    pub lpfnSetupHook: *const c_void,
    pub lpPrintTemplateName: *const u16,
    pub lpSetupTemplateName: *const u16,
    pub hPrintTemplate: HGLOBAL,
    pub hSetupTemplate: HGLOBAL,
}

impl Default for PRINTDLGW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct DOCINFOW {
    pub cbSize: i32,
    pub lpszDocName: *const u16,
    pub lpszOutput: *const u16,
    pub lpszDatatype: *const u16,
    pub fwType: DWORD,
}

impl Default for DOCINFOW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct STORAGE_DEVICE_NUMBER {
    pub DeviceType: DWORD,
    pub DeviceNumber: DWORD,
    pub PartitionNumber: DWORD,
}

pub type SUBCLASSPROC =
    unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, UINT_PTR, DWORD_PTR) -> LRESULT;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    pub fn GetSystemDirectoryW(lpBuffer: *mut u16, uSize: UINT) -> UINT;
    pub fn GetWindowsDirectoryW(lpBuffer: *mut u16, uSize: UINT) -> UINT;
    pub fn GetLogicalDriveStringsW(nBufferLength: DWORD, lpBuffer: *mut u16) -> DWORD;
    pub fn GetDriveTypeW(lpRootPathName: *const u16) -> UINT;
    pub fn GetVolumeInformationW(
        lpRootPathName: *const u16,
        lpVolumeNameBuffer: *mut u16,
        nVolumeNameSize: DWORD,
        lpVolumeSerialNumber: *mut DWORD,
        lpMaximumComponentLength: *mut DWORD,
        lpFileSystemFlags: *mut DWORD,
        lpFileSystemNameBuffer: *mut u16,
        nFileSystemNameSize: DWORD,
    ) -> BOOL;
    pub fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailableToCaller: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> BOOL;
    pub fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *const c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn DeviceIoControl(
        hDevice: HANDLE,
        dwIoControlCode: DWORD,
        lpInBuffer: *const c_void,
        nInBufferSize: DWORD,
        lpOutBuffer: *mut c_void,
        nOutBufferSize: DWORD,
        lpBytesReturned: *mut DWORD,
        lpOverlapped: *mut c_void,
    ) -> BOOL;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: *const u8,
        cbMultiByte: i32,
        lpWideCharStr: *mut u16,
        cchWideChar: i32,
    ) -> i32;
    pub fn GlobalAlloc(uFlags: UINT, dwBytes: usize) -> HGLOBAL;
    pub fn GlobalLock(hMem: HGLOBAL) -> *mut c_void;
    pub fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    pub fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    pub fn SetThreadExecutionState(esFlags: DWORD) -> DWORD;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn MulDiv(nNumber: i32, nNumerator: i32, nDenominator: i32) -> i32;
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: DWORD,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *const c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
    ) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn IsDialogMessageW(hDlg: HWND, lpMsg: *const MSG) -> BOOL;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> BOOL;
    pub fn GetWindowTextLengthW(hWnd: HWND) -> i32;
    pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn GetFocus() -> HWND;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: UINT,
    ) -> BOOL;
    pub fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: UINT) -> i32;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HICON;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    pub fn GetKeyState(nVirtKey: i32) -> i16;
    pub fn GetDpiForWindow(hwnd: HWND) -> UINT;
    pub fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    pub fn EmptyClipboard() -> BOOL;
    pub fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE;
    pub fn CloseClipboard() -> BOOL;
    pub fn DrawTextW(
        hdc: HDC,
        lpchText: *const u16,
        cchText: i32,
        lprc: *mut RECT,
        format: UINT,
    ) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: *const u16,
    ) -> HFONT;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
    pub fn StartDocW(hdc: HDC, lpdi: *const DOCINFOW) -> i32;
    pub fn EndDoc(hdc: HDC) -> i32;
    pub fn AbortDoc(hdc: HDC) -> i32;
    pub fn StartPage(hdc: HDC) -> i32;
    pub fn EndPage(hdc: HDC) -> i32;
}

#[link(name = "comctl32")]
unsafe extern "system" {
    pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> BOOL;
    pub fn SetWindowSubclass(
        hWnd: HWND,
        pfnSubclass: SUBCLASSPROC,
        uIdSubclass: UINT_PTR,
        dwRefData: DWORD_PTR,
    ) -> BOOL;
    pub fn RemoveWindowSubclass(
        hWnd: HWND,
        pfnSubclass: SUBCLASSPROC,
        uIdSubclass: UINT_PTR,
    ) -> BOOL;
    pub fn DefSubclassProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    pub fn GetSaveFileNameW(arg1: *mut OPENFILENAMEW) -> BOOL;
    pub fn PrintDlgW(pPD: *mut PRINTDLGW) -> BOOL;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn SHBrowseForFolderW(lpbi: *mut BROWSEINFOW) -> *mut c_void;
    pub fn SHGetPathFromIDListW(pidl: *const c_void, pszPath: *mut u16) -> BOOL;
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn CoTaskMemFree(pv: *mut c_void);
    pub fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: DWORD) -> i32;
    pub fn CoUninitialize();
}

pub fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn from_wide_slice(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// 64bit符号拡張バグ回避のため必ず u16 -> usize -> *const u16 と変換する。
pub const fn make_int_resource_w(id: u16) -> *const u16 {
    id as usize as *const u16
}

/// Win32 MultiByteToWideChar(CP_932) を用いて Shift-JIS バイト列をデコードする。
pub fn decode_cp932(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let Ok(len_i32) = i32::try_from(bytes.len()) else {
        return String::from_utf8_lossy(bytes).into_owned();
    };
    unsafe {
        let needed =
            MultiByteToWideChar(CP_932, 0, bytes.as_ptr(), len_i32, std::ptr::null_mut(), 0);
        if needed <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut wide = vec![0u16; needed as usize];
        let written = MultiByteToWideChar(
            CP_932,
            0,
            bytes.as_ptr(),
            len_i32,
            wide.as_mut_ptr(),
            needed,
        );
        if written <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        String::from_utf16_lossy(&wide[..written as usize])
    }
}

pub fn get_window_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let needed = (len as usize) + 1;
        if needed <= 256 {
            let mut stack_buf = [0u16; 256];
            let read = GetWindowTextW(hwnd, stack_buf.as_mut_ptr(), stack_buf.len() as i32);
            if read <= 0 {
                return String::new();
            }
            return String::from_utf16_lossy(&stack_buf[..read as usize]);
        }
        let mut buf = vec![0u16; needed];
        let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if read <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..read as usize])
    }
}

pub fn set_window_text(hwnd: HWND, text: &str) {
    let wide = to_wide_null(text);
    unsafe {
        SetWindowTextW(hwnd, wide.as_ptr());
    }
}
