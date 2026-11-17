pub mod dialogs;
pub mod state;

use crate::domain::guide::generate_restore_guide;
use crate::domain::validation::{
    BackupTarget, DriveInfo, validate_backup_plan, validate_credentials, validate_unc_path,
};
use crate::domain::wbadmin::{
    build_command_preview, build_get_versions_args, build_start_backup_args,
    parse_get_versions_output,
};
use crate::platform::drive::enumerate_fixed_drives;
use crate::platform::io::{browse_folder_dialog, copy_to_clipboard, save_text_with_dialog};
use crate::platform::print::print_text_document;
use crate::platform::process::{WM_APP_WORKER_EVENT, WorkerEvent, WorkerKind, spawn_wbadmin};
use crate::platform::win32::{
    ATOM, BM_GETCHECK, BM_SETCHECK, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON, BS_GROUPBOX,
    BS_PUSHBUTTON, BST_CHECKED, BST_UNCHECKED, CB_ADDSTRING, CB_GETCURSEL, CB_RESETCONTENT,
    CB_SETCURSEL, CBN_SELCHANGE, CBS_DROPDOWNLIST, CBS_HASSTRINGS, CLEARTYPE_QUALITY,
    CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, CoInitializeEx, CoUninitialize, CreateFontW,
    CreateWindowExW, DEFAULT_CHARSET, DEFAULT_PITCH, DLGC_WANTALLKEYS, DLGC_WANTTAB, DWORD_PTR,
    DefSubclassProc, DefWindowProcW, DispatchMessageW, EM_GETSEL, EM_REPLACESEL, EM_SETLIMITTEXT,
    EM_SETSEL, EN_CHANGE, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE, ES_PASSWORD,
    ES_READONLY, EnableWindow, FALSE, FF_DONTCARE, FW_BOLD, FW_NORMAL, GetClientRect,
    GetDpiForWindow, GetFocus, GetKeyState, GetMessageW, GetModuleHandleW, HFONT, HINSTANCE, HMENU,
    HWND, ICC_LISTVIEW_CLASSES, ICC_PROGRESS_CLASS, ICC_STANDARD_CLASSES, ICC_TAB_CLASSES,
    IDC_ARROW, IDI_APPLICATION, INITCOMMONCONTROLSEX, InitCommonControlsEx, IsDialogMessageW,
    LPARAM, LRESULT, LVCF_FMT, LVCF_TEXT, LVCF_WIDTH, LVCFMT_LEFT, LVCOLUMNW, LVIF_STATE,
    LVIF_TEXT, LVITEMW, LVM_DELETEALLITEMS, LVM_GETITEMSTATE, LVM_GETNEXTITEM, LVM_INSERTCOLUMNW,
    LVM_INSERTITEMW, LVM_SETEXTENDEDLISTVIEWSTYLE, LVM_SETITEMSTATE, LVM_SETITEMTEXTW,
    LVN_ITEMCHANGED, LVNI_SELECTED, LVS_EX_CHECKBOXES, LVS_EX_FULLROWSELECT, LVS_EX_GRIDLINES,
    LVS_REPORT, LVS_SHOWSELALWAYS, LVS_SINGLESEL, LoadCursorW, LoadIconW, MSG, MulDiv, NMHDR,
    OUT_DEFAULT_PRECIS, PBM_SETPOS, PBM_SETRANGE, PostQuitMessage, RECT, RegisterClassExW,
    RemoveWindowSubclass, SS_LEFT, SS_NOPREFIX, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER,
    SendMessageW, SetFocus, SetWindowPos, SetWindowSubclass, ShowWindow, TCIF_TEXT, TCITEMW,
    TCM_GETCURSEL, TCM_INSERTITEMW, TCM_SETCURSEL, TCN_SELCHANGE, TRUE, TranslateMessage, UINT,
    UINT_PTR, UpdateWindow, VK_A, VK_CONTROL, VK_SHIFT, VK_TAB, WM_CHAR, WM_CLOSE, WM_COMMAND,
    WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_GETDLGCODE, WM_KEYDOWN, WM_NCDESTROY, WM_NOTIFY,
    WM_SETFONT, WM_SIZE, WNDCLASSEXW, WPARAM, WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE, WS_GROUP,
    WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL, get_window_text, make_int_resource_w,
    set_window_text, to_wide_null,
};
use state::AppState;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const ID_TAB: usize = 100;
const ID_T1_SRC_LIST: usize = 101;
const ID_T1_BTN_RESCAN: usize = 102;
const ID_T1_RADIO_LOCAL: usize = 103;
const ID_T1_DST_COMBO: usize = 104;
const ID_T1_RADIO_UNC: usize = 105;
const ID_T1_DST_UNC_EDIT: usize = 106;
const ID_T1_BTN_BROWSE: usize = 107;
const ID_T1_CRED_USER_EDIT: usize = 108;
const ID_T1_CRED_PASS_EDIT: usize = 109;
const ID_T1_CMD_EDIT: usize = 110;
const ID_T1_BTN_START: usize = 111;
const ID_T1_BTN_STOP: usize = 112;
const ID_T1_BTN_COPY_CMD: usize = 113;
const ID_T1_LOG_EDIT: usize = 114;
const ID_T1_BTN_CLEAR_LOG: usize = 115;

const ID_T2_RADIO_LOCAL: usize = 201;
const ID_T2_DST_COMBO: usize = 202;
const ID_T2_RADIO_UNC: usize = 203;
const ID_T2_DST_UNC_EDIT: usize = 204;
const ID_T2_CRED_USER_EDIT: usize = 205;
const ID_T2_CRED_PASS_EDIT: usize = 206;
const ID_T2_BTN_FETCH: usize = 207;
const ID_T2_BTN_SYNC_DST: usize = 208;
const ID_T2_VER_LIST: usize = 209;
const ID_T2_BTN_COPY_VER: usize = 210;
const ID_T2_BTN_APPLY_GUIDE: usize = 211;
const ID_T2_OUT_EDIT: usize = 212;

const ID_T3_TARGET_EDIT: usize = 301;
const ID_T3_VER_EDIT: usize = 302;
const ID_T3_GUIDE_EDIT: usize = 303;
const ID_T3_BTN_SAVE: usize = 304;
const ID_T3_BTN_PRINT: usize = 305;
const ID_T3_BTN_COPY: usize = 306;

thread_local! {
    static APP_STATE: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

fn with_state<R>(f: impl FnOnce(&AppState) -> R) -> Option<R> {
    APP_STATE.with(|cell| {
        cell.try_borrow()
            .ok()
            .and_then(|guard| guard.as_ref().map(f))
    })
}

fn with_state_mut<R>(f: impl FnOnce(&mut AppState) -> R) -> Option<R> {
    APP_STATE.with(|cell| {
        cell.try_borrow_mut()
            .ok()
            .and_then(|mut guard| guard.as_mut().map(f))
    })
}

pub fn run_gui() -> i32 {
    unsafe {
        let co_hr = CoInitializeEx(std::ptr::null_mut(), 0x2); // COINIT_APARTMENTTHREADED
        let icce = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_LISTVIEW_CLASSES
                | ICC_PROGRESS_CLASS
                | ICC_TAB_CLASSES
                | ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&raw const icce);

        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide_null("WibGuiMainWindowClass");
        let title = to_wide_null(concat!(
            "WindowsImageBackup GUI (WIB-GUI) v",
            env!("CARGO_PKG_VERSION"),
            " [管理者]"
        ));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(std::ptr::null_mut(), make_int_resource_w(IDI_APPLICATION)),
            hCursor: LoadCursorW(std::ptr::null_mut(), make_int_resource_w(IDC_ARROW)),
            hbrBackground: (COLOR_BTNFACE + 1) as isize as crate::platform::win32::HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        let atom: ATOM = RegisterClassExW(&raw const wc);
        if atom == 0 {
            return 1;
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            crate::platform::win32::CW_USEDEFAULT,
            crate::platform::win32::CW_USEDEFAULT,
            900,
            740,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return 1;
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        with_state(|st| {
            SetFocus(st.t1_src_list);
        });

        let mut msg = MSG::default();
        while GetMessageW(&raw mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_KEYDOWN
                && msg.wParam == VK_TAB as usize
                && (GetKeyState(VK_CONTROL) as u16 & 0x8000) != 0
            {
                let reverse = (GetKeyState(VK_SHIFT) as u16 & 0x8000) != 0;
                with_state_mut(|st| {
                    let next = if reverse {
                        (st.current_tab + 2) % 3
                    } else {
                        (st.current_tab + 1) % 3
                    };
                    SendMessageW(st.hwnd_tab, TCM_SETCURSEL, next, 0);
                    switch_tab(st, next);
                });
                continue;
            }

            if IsDialogMessageW(hwnd, &raw const msg) == 0 {
                TranslateMessage(&raw const msg);
                DispatchMessageW(&raw const msg);
            }
        }
        if co_hr >= 0 {
            CoUninitialize();
        }
        msg.wParam as i32
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let res = std::panic::catch_unwind(|| unsafe { wnd_proc_inner(hwnd, msg, wparam, lparam) });
    res.unwrap_or(0)
}

unsafe fn wnd_proc_inner(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let state = create_initial_state(hwnd);
            APP_STATE.with(|cell| {
                *cell.borrow_mut() = Some(state);
            });
            with_state_mut(|st| {
                refresh_drives(st);
                update_tab1_preview_and_validation(st);
                update_tab3_guide(st);
                layout_controls(st);
                switch_tab(st, 0);
            });
            0
        }
        WM_SIZE => {
            with_state_mut(|st| {
                layout_controls(st);
            });
            0
        }
        WM_DPICHANGED => {
            let new_dpi = (wparam & 0xFFFF) as u32;
            let rect_ptr = lparam as *const RECT;
            unsafe {
                if !rect_ptr.is_null() {
                    let rc = *rect_ptr;
                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        rc.left,
                        rc.top,
                        rc.right - rc.left,
                        rc.bottom - rc.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
            with_state_mut(|st| {
                st.dpi = new_dpi.max(96);
                rebuild_fonts(st);
                layout_controls(st);
            });
            0
        }
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            let code = ((wparam >> 16) & 0xFFFF) as u16;
            with_state_mut(|st| {
                handle_command(st, id, code);
                // モーダルダイアログ表示中に到着したワーカイベントの取りこぼしを回収
                handle_worker_events(st);
            });
            0
        }
        WM_NOTIFY => {
            with_state_mut(|st| {
                handle_notify(st, lparam);
            });
            0
        }
        WM_APP_WORKER_EVENT => {
            with_state_mut(|st| {
                handle_worker_events(st);
            });
            0
        }
        WM_CLOSE => {
            let is_backup_running = with_state(|s| {
                s.worker
                    .as_ref()
                    .is_some_and(|w| w.kind() == WorkerKind::Backup)
            })
            .unwrap_or(false);
            if is_backup_running {
                dialogs::show_warning(
                    hwnd,
                    "バックアップ実行中",
                    "現在バックアップを実行中です。安全のため、先に「バックアップ中断」ボタンで停止してからウィンドウを閉じてください。",
                );
                return 0;
            }
            unsafe {
                crate::platform::win32::DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            APP_STATE.with(|cell| {
                if let Ok(mut guard) = cell.try_borrow_mut() {
                    *guard = None;
                }
            });
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn edit_ctrl_a_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
    uid: UINT_PTR,
    _ref_data: DWORD_PTR,
) -> LRESULT {
    unsafe {
        match msg {
            WM_GETDLGCODE => {
                // 複数行READONLY EDITから Tab / Shift+Tab で抜けるため DLGC_WANTTAB / DLGC_WANTALLKEYS を除外
                let code = DefSubclassProc(hwnd, msg, wparam, lparam);
                code & !(DLGC_WANTALLKEYS | DLGC_WANTTAB)
            }
            WM_KEYDOWN => {
                if wparam == VK_A as usize && (GetKeyState(VK_CONTROL) as u16 & 0x8000) != 0 {
                    SendMessageW(hwnd, EM_SETSEL, 0, -1);
                    return 0;
                }
                DefSubclassProc(hwnd, msg, wparam, lparam)
            }
            WM_CHAR => {
                // Ctrl+A の制御文字 (0x01) による警告音を抑制する
                if wparam == 0x01 {
                    return 0;
                }
                DefSubclassProc(hwnd, msg, wparam, lparam)
            }
            WM_NCDESTROY => {
                RemoveWindowSubclass(hwnd, edit_ctrl_a_proc, uid);
                DefSubclassProc(hwnd, msg, wparam, lparam)
            }
            _ => DefSubclassProc(hwnd, msg, wparam, lparam),
        }
    }
}

fn create_fonts(dpi: u32) -> (HFONT, HFONT, HFONT) {
    let h_normal = -unsafe { MulDiv(10, dpi as i32, 72) };
    let h_mono = -unsafe { MulDiv(9, dpi as i32, 72) };
    let face = to_wide_null("BIZ UDゴシック");
    unsafe {
        let normal = CreateFontW(
            h_normal,
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
            face.as_ptr(),
        );
        let bold = CreateFontW(
            h_normal,
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
            face.as_ptr(),
        );
        let mono = CreateFontW(
            h_mono,
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
            face.as_ptr(),
        );
        (normal, bold, mono)
    }
}

fn apply_font(hwnd: HWND, hfont: HFONT) {
    unsafe {
        SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
    }
}

fn rebuild_fonts(st: &mut AppState) {
    let (normal, bold, mono) = create_fonts(st.dpi);

    apply_font(st.hwnd_tab, normal);
    for &h in &st.t1_controls {
        apply_font(h, normal);
    }
    for &h in &st.t2_controls {
        apply_font(h, normal);
    }
    for &h in &st.t3_controls {
        apply_font(h, normal);
    }
    apply_font(st.t3_banner, bold);
    apply_font(st.t1_cmd_edit, mono);
    apply_font(st.t1_log_edit, mono);
    apply_font(st.t2_out_edit, mono);
    apply_font(st.t3_guide_edit, mono);

    st.release_fonts();
    st.hfont_normal = normal;
    st.hfont_bold = bold;
    st.hfont_mono = mono;
}

#[allow(clippy::too_many_arguments)]
fn make_ctrl(
    hinst: HINSTANCE,
    parent: HWND,
    class: &str,
    text: &str,
    style: u32,
    ex_style: u32,
    id: usize,
    font: HFONT,
) -> HWND {
    let cls_w = to_wide_null(class);
    let txt_w = to_wide_null(text);
    unsafe {
        let h = CreateWindowExW(
            ex_style,
            cls_w.as_ptr(),
            txt_w.as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            0,
            0,
            10,
            10,
            parent,
            id as HMENU,
            hinst,
            std::ptr::null(),
        );
        apply_font(h, font);
        h
    }
}

fn make_readonly_edit(
    hinst: HINSTANCE,
    parent: HWND,
    id: usize,
    multiline: bool,
    font: HFONT,
) -> HWND {
    let style = if multiline {
        WS_TABSTOP | WS_VSCROLL | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY
    } else {
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL | ES_READONLY
    };
    let h = make_ctrl(hinst, parent, "EDIT", "", style, WS_EX_CLIENTEDGE, id, font);
    unsafe {
        if multiline {
            SendMessageW(h, EM_SETLIMITTEXT, 0x7FFF_FFFE, 0);
        }
        SetWindowSubclass(h, edit_ctrl_a_proc, 1, 0);
    }
    h
}

fn create_initial_state(hwnd_main: HWND) -> AppState {
    let dpi = unsafe { GetDpiForWindow(hwnd_main) }.max(96);
    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    let (hfont_normal, hfont_bold, hfont_mono) = create_fonts(dpi);

    let hwnd_tab = make_ctrl(
        hinst,
        hwnd_main,
        "SysTabControl32",
        "",
        WS_TABSTOP,
        0,
        ID_TAB,
        hfont_normal,
    );
    for (idx, label) in [
        "バックアップ作成(&1)",
        "既存バックアップ確認(&2)",
        "復元手順・印刷(&3)",
    ]
    .iter()
    .enumerate()
    {
        let mut lbl_w = to_wide_null(label);
        let item = TCITEMW {
            mask: TCIF_TEXT,
            pszText: lbl_w.as_mut_ptr(),
            ..Default::default()
        };
        unsafe {
            SendMessageW(hwnd_tab, TCM_INSERTITEMW, idx, (&raw const item) as LPARAM);
        }
    }

    // --- タブ1 ---
    let t1_guide = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "Windows全体のシステムイメージを作成します。対象ドライブと保存先を選び、内容を確認してから実行してください。",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_normal,
    );
    let t1_grp_src = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "1. バックアップ対象のドライブ（複数選択可）",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t1_src_desc = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "システムドライブ（C: など）は復旧に必要なため、既定で選択されています。",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_normal,
    );
    let t1_src_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "対象ドライブ一覧(&D):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_src_list = make_ctrl(
        hinst,
        hwnd_main,
        "SysListView32",
        "",
        WS_TABSTOP | WS_BORDER | LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS,
        WS_EX_CLIENTEDGE,
        ID_T1_SRC_LIST,
        hfont_normal,
    );
    unsafe {
        SendMessageW(
            t1_src_list,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            0,
            (LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES) as LPARAM,
        );
    }
    init_drive_list_columns(t1_src_list);
    let t1_btn_rescan = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "再スキャン(&R)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T1_BTN_RESCAN,
        hfont_normal,
    );

    let t1_grp_dst = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "2. バックアップの保存先",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t1_radio_local = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "ローカルドライブ（外付けHDD/SSD等、NTFS必須）(&L)",
        WS_TABSTOP | WS_GROUP | BS_AUTORADIOBUTTON,
        0,
        ID_T1_RADIO_LOCAL,
        hfont_normal,
    );
    unsafe {
        SendMessageW(t1_radio_local, BM_SETCHECK, BST_CHECKED, 0);
    }
    let t1_dst_local_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "保存先ドライブ(&T):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_dst_combo = make_ctrl(
        hinst,
        hwnd_main,
        "COMBOBOX",
        "",
        WS_TABSTOP | CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_VSCROLL,
        0,
        ID_T1_DST_COMBO,
        hfont_normal,
    );
    let t1_radio_unc = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "ネットワーク共有フォルダ（例: \\\\Server\\Share）(&N)",
        WS_TABSTOP | BS_AUTORADIOBUTTON,
        0,
        ID_T1_RADIO_UNC,
        hfont_normal,
    );
    let t1_dst_unc_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "共有パス(&P):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_dst_unc_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_T1_DST_UNC_EDIT,
        hfont_normal,
    );
    let t1_btn_browse = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "参照(&B)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T1_BTN_BROWSE,
        hfont_normal,
    );
    let t1_cred_user_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "ユーザー名(任意)(&U):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_cred_user_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_T1_CRED_USER_EDIT,
        hfont_normal,
    );
    let t1_cred_pass_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "パスワード(任意)(&W):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_cred_pass_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL | ES_PASSWORD,
        WS_EX_CLIENTEDGE,
        ID_T1_CRED_PASS_EDIT,
        hfont_normal,
    );
    let t1_dst_note = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "※ ネットワーク共有は1世代上書き保存となり、必要時は資格情報をご入力ください。",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_normal,
    );
    let t1_warn_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_bold,
    );

    let t1_grp_cmd = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "3. 実行コマンドの確認と操作",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t1_cmd_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "実行コマンド（プレビュー）(&M):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_cmd_edit = make_readonly_edit(hinst, hwnd_main, ID_T1_CMD_EDIT, false, hfont_mono);
    let t1_btn_start = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "バックアップ開始(&S)...",
        WS_TABSTOP | BS_DEFPUSHBUTTON,
        0,
        ID_T1_BTN_START,
        hfont_normal,
    );
    let t1_btn_stop = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "バックアップ中断(&C)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T1_BTN_STOP,
        hfont_normal,
    );
    let t1_btn_copy_cmd = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "コマンドをコピー(&K)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T1_BTN_COPY_CMD,
        hfont_normal,
    );

    let t1_grp_log = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "4. 実行状況とログ",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t1_status_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "状態:",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_status_val = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "待機中",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_bold,
    );
    let t1_progress = make_ctrl(
        hinst,
        hwnd_main,
        "msctls_progress32",
        "",
        0,
        0,
        0,
        hfont_normal,
    );
    unsafe {
        SendMessageW(t1_progress, PBM_SETRANGE, 0, (100 << 16) as LPARAM);
        SendMessageW(t1_progress, PBM_SETPOS, 0, 0);
    }
    let t1_log_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "実行ログ(&G):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t1_log_edit = make_readonly_edit(hinst, hwnd_main, ID_T1_LOG_EDIT, true, hfont_mono);
    let t1_btn_clear_log = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "ログをクリア(&E)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T1_BTN_CLEAR_LOG,
        hfont_normal,
    );

    unsafe {
        EnableWindow(t1_dst_unc_edit, FALSE);
        EnableWindow(t1_btn_browse, FALSE);
        EnableWindow(t1_cred_user_edit, FALSE);
        EnableWindow(t1_cred_pass_edit, FALSE);
        EnableWindow(t1_btn_stop, FALSE);
    }

    let t1_controls = vec![
        t1_guide,
        t1_grp_src,
        t1_src_desc,
        t1_src_lbl,
        t1_src_list,
        t1_btn_rescan,
        t1_grp_dst,
        t1_radio_local,
        t1_dst_local_lbl,
        t1_dst_combo,
        t1_radio_unc,
        t1_dst_unc_lbl,
        t1_dst_unc_edit,
        t1_btn_browse,
        t1_cred_user_lbl,
        t1_cred_user_edit,
        t1_cred_pass_lbl,
        t1_cred_pass_edit,
        t1_dst_note,
        t1_warn_lbl,
        t1_grp_cmd,
        t1_cmd_lbl,
        t1_cmd_edit,
        t1_btn_start,
        t1_btn_stop,
        t1_btn_copy_cmd,
        t1_grp_log,
        t1_status_lbl,
        t1_status_val,
        t1_progress,
        t1_log_lbl,
        t1_log_edit,
        t1_btn_clear_log,
    ];

    // --- タブ2 ---
    let t2_guide = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "指定した保存先に作成済みのシステムイメージ一覧を確認します。バージョン識別子をコピーして復元手順に反映できます。",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_normal,
    );
    let t2_grp_dst = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "確認する保存先",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t2_radio_local = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "ローカルドライブ(&L)",
        WS_TABSTOP | WS_GROUP | BS_AUTORADIOBUTTON,
        0,
        ID_T2_RADIO_LOCAL,
        hfont_normal,
    );
    unsafe {
        SendMessageW(t2_radio_local, BM_SETCHECK, BST_CHECKED, 0);
    }
    let t2_dst_local_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "保存先ドライブ(&T):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t2_dst_combo = make_ctrl(
        hinst,
        hwnd_main,
        "COMBOBOX",
        "",
        WS_TABSTOP | CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_VSCROLL,
        0,
        ID_T2_DST_COMBO,
        hfont_normal,
    );
    let t2_radio_unc = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "ネットワーク共有フォルダ(&N)",
        WS_TABSTOP | BS_AUTORADIOBUTTON,
        0,
        ID_T2_RADIO_UNC,
        hfont_normal,
    );
    let t2_dst_unc_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "共有パス(&P):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t2_dst_unc_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_T2_DST_UNC_EDIT,
        hfont_normal,
    );
    let t2_cred_user_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "ユーザー名(任意)(&U):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t2_cred_user_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_T2_CRED_USER_EDIT,
        hfont_normal,
    );
    let t2_cred_pass_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "パスワード(任意)(&W):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t2_cred_pass_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL | ES_PASSWORD,
        WS_EX_CLIENTEDGE,
        ID_T2_CRED_PASS_EDIT,
        hfont_normal,
    );
    let t2_btn_fetch = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "一覧を取得(&F)",
        WS_TABSTOP | BS_DEFPUSHBUTTON,
        0,
        ID_T2_BTN_FETCH,
        hfont_normal,
    );
    let t2_btn_sync_dst = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "バックアップタブの保存先を反映(&M)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T2_BTN_SYNC_DST,
        hfont_normal,
    );

    let t2_grp_list = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "保存済みバックアップ一覧",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t2_list_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "バックアップ一覧(&V):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t2_ver_list = make_ctrl(
        hinst,
        hwnd_main,
        "SysListView32",
        "",
        WS_TABSTOP | WS_BORDER | LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS,
        WS_EX_CLIENTEDGE,
        ID_T2_VER_LIST,
        hfont_normal,
    );
    unsafe {
        SendMessageW(
            t2_ver_list,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            0,
            (LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES) as LPARAM,
        );
    }
    init_version_list_columns(t2_ver_list);
    let t2_btn_copy_ver = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "バージョン識別子をコピー(&C)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T2_BTN_COPY_VER,
        hfont_normal,
    );
    let t2_btn_apply_guide = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "選択項目を復元手順に反映(&A)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T2_BTN_APPLY_GUIDE,
        hfont_normal,
    );

    let t2_grp_out = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "取得結果の詳細出力",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t2_out_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "コマンド出力(&D):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t2_out_edit = make_readonly_edit(hinst, hwnd_main, ID_T2_OUT_EDIT, true, hfont_mono);

    unsafe {
        EnableWindow(t2_dst_unc_edit, FALSE);
        EnableWindow(t2_cred_user_edit, FALSE);
        EnableWindow(t2_cred_pass_edit, FALSE);
        EnableWindow(t2_btn_copy_ver, FALSE);
        EnableWindow(t2_btn_apply_guide, FALSE);
    }

    let t2_controls = vec![
        t2_guide,
        t2_grp_dst,
        t2_radio_local,
        t2_dst_local_lbl,
        t2_dst_combo,
        t2_radio_unc,
        t2_dst_unc_lbl,
        t2_dst_unc_edit,
        t2_cred_user_lbl,
        t2_cred_user_edit,
        t2_cred_pass_lbl,
        t2_cred_pass_edit,
        t2_btn_fetch,
        t2_btn_sync_dst,
        t2_grp_list,
        t2_list_lbl,
        t2_ver_list,
        t2_btn_copy_ver,
        t2_btn_apply_guide,
        t2_grp_out,
        t2_out_lbl,
        t2_out_edit,
    ];

    // --- タブ3 ---
    let t3_banner = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "【重要】システムイメージからの復元は、現在起動しているWindows上からは実行できません。必ずPCを再起動し、「Windows回復環境（WinRE）」から実行してください。",
        SS_LEFT | SS_NOPREFIX,
        0,
        0,
        hfont_bold,
    );
    let t3_grp_info = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "手順に反映する情報",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t3_target_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "保存先(&T):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t3_target_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "D:",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_T3_TARGET_EDIT,
        hfont_normal,
    );
    let t3_ver_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "バージョン識別子(&I):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t3_ver_edit = make_ctrl(
        hinst,
        hwnd_main,
        "EDIT",
        "",
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_T3_VER_EDIT,
        hfont_normal,
    );
    let t3_grp_guide = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "復元手順ガイド",
        BS_GROUPBOX,
        0,
        0,
        hfont_normal,
    );
    let t3_guide_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "手順本文(&G):",
        SS_LEFT,
        0,
        0,
        hfont_normal,
    );
    let t3_guide_edit = make_readonly_edit(hinst, hwnd_main, ID_T3_GUIDE_EDIT, true, hfont_mono);
    let t3_btn_save = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "テキストファイルに保存(&S)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T3_BTN_SAVE,
        hfont_normal,
    );
    let t3_btn_print = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "印刷(&P)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T3_BTN_PRINT,
        hfont_normal,
    );
    let t3_btn_copy = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "手順をすべてコピー(&C)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T3_BTN_COPY,
        hfont_normal,
    );

    let t3_controls = vec![
        t3_banner,
        t3_grp_info,
        t3_target_lbl,
        t3_target_edit,
        t3_ver_lbl,
        t3_ver_edit,
        t3_grp_guide,
        t3_guide_lbl,
        t3_guide_edit,
        t3_btn_save,
        t3_btn_print,
        t3_btn_copy,
    ];

    AppState {
        hwnd_main,
        dpi,
        hfont_normal,
        hfont_bold,
        hfont_mono,
        hwnd_tab,
        current_tab: 0,
        t1_controls,
        t1_guide,
        t1_grp_src,
        t1_src_desc,
        t1_src_lbl,
        t1_src_list,
        t1_btn_rescan,
        t1_grp_dst,
        t1_radio_local,
        t1_dst_local_lbl,
        t1_dst_combo,
        t1_radio_unc,
        t1_dst_unc_lbl,
        t1_dst_unc_edit,
        t1_btn_browse,
        t1_cred_user_lbl,
        t1_cred_user_edit,
        t1_cred_pass_lbl,
        t1_cred_pass_edit,
        t1_dst_note,
        t1_warn_lbl,
        t1_grp_cmd,
        t1_cmd_lbl,
        t1_cmd_edit,
        t1_btn_start,
        t1_btn_stop,
        t1_btn_copy_cmd,
        t1_grp_log,
        t1_status_lbl,
        t1_status_val,
        t1_progress,
        t1_log_lbl,
        t1_log_edit,
        t1_btn_clear_log,
        t2_controls,
        t2_guide,
        t2_grp_dst,
        t2_radio_local,
        t2_dst_local_lbl,
        t2_dst_combo,
        t2_radio_unc,
        t2_dst_unc_lbl,
        t2_dst_unc_edit,
        t2_cred_user_lbl,
        t2_cred_user_edit,
        t2_cred_pass_lbl,
        t2_cred_pass_edit,
        t2_btn_fetch,
        t2_btn_sync_dst,
        t2_grp_list,
        t2_list_lbl,
        t2_ver_list,
        t2_btn_copy_ver,
        t2_btn_apply_guide,
        t2_grp_out,
        t2_out_lbl,
        t2_out_edit,
        t3_controls,
        t3_banner,
        t3_grp_info,
        t3_target_lbl,
        t3_target_edit,
        t3_ver_lbl,
        t3_ver_edit,
        t3_grp_guide,
        t3_guide_lbl,
        t3_guide_edit,
        t3_btn_save,
        t3_btn_print,
        t3_btn_copy,
        all_drives: Vec::new(),
        target_drives: Vec::new(),
        versions: Vec::new(),
        last_status_percent: None,
        is_stopping: false,
        worker: None,
        event_queue: Arc::new(Mutex::new(VecDeque::new())),
    }
}

fn insert_lv_col(lv: HWND, idx: i32, text: &str, width: i32) {
    let mut txt_w = to_wide_null(text);
    let col = LVCOLUMNW {
        mask: LVCF_FMT | LVCF_WIDTH | LVCF_TEXT,
        fmt: LVCFMT_LEFT,
        cx: width,
        pszText: txt_w.as_mut_ptr(),
        ..Default::default()
    };
    unsafe {
        SendMessageW(
            lv,
            LVM_INSERTCOLUMNW,
            idx as usize,
            (&raw const col) as LPARAM,
        );
    }
}

fn init_drive_list_columns(lv: HWND) {
    insert_lv_col(lv, 0, "ドライブ", 80);
    insert_lv_col(lv, 1, "ボリューム名", 140);
    insert_lv_col(lv, 2, "形式", 75);
    insert_lv_col(lv, 3, "空き / 合計", 190);
    insert_lv_col(lv, 4, "種別", 150);
}

fn init_version_list_columns(lv: HWND) {
    insert_lv_col(lv, 0, "バックアップ日時", 170);
    insert_lv_col(lv, 1, "バージョン識別子", 160);
    insert_lv_col(lv, 2, "保存先", 200);
    insert_lv_col(lv, 3, "復元可能項目", 240);
}

fn scale(v: i32, dpi: u32) -> i32 {
    unsafe { MulDiv(v, dpi as i32, 96) }
}

fn mv(hwnd: HWND, x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            w.max(10),
            h.max(10),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

fn layout_controls(st: &AppState) {
    let mut rc = RECT::default();
    unsafe {
        GetClientRect(st.hwnd_main, &raw mut rc);
    }
    let dpi = st.dpi;
    let cw = (rc.right - rc.left).max(scale(720, dpi));
    let ch = (rc.bottom - rc.top).max(scale(580, dpi));
    let pad = scale(12, dpi);
    let inner_x = pad * 2;
    let inner_w = cw - pad * 4;
    let tab_top = pad;
    let tab_h = ch - pad * 2;
    mv(st.hwnd_tab, pad, tab_top, cw - pad * 2, tab_h);

    let y0 = tab_top + scale(32, dpi);
    let row_h = scale(24, dpi);
    let btn_h = scale(28, dpi);
    let gap = scale(6, dpi);

    // --- タブ1レイアウト ---
    mv(st.t1_guide, inner_x, y0, inner_w, scale(22, dpi));
    let src_y = y0 + scale(26, dpi);
    let src_h = scale(146, dpi);
    mv(st.t1_grp_src, inner_x, src_y, inner_w, src_h);
    let sx = inner_x + pad;
    let sw = inner_w - pad * 2;
    mv(
        st.t1_src_desc,
        sx,
        src_y + scale(20, dpi),
        sw,
        scale(18, dpi),
    );
    mv(
        st.t1_src_lbl,
        sx,
        src_y + scale(40, dpi),
        scale(180, dpi),
        scale(18, dpi),
    );
    let btn_rescan_w = scale(120, dpi);
    mv(
        st.t1_src_list,
        sx,
        src_y + scale(60, dpi),
        sw - btn_rescan_w - gap,
        scale(76, dpi),
    );
    mv(
        st.t1_btn_rescan,
        sx + sw - btn_rescan_w,
        src_y + scale(60, dpi),
        btn_rescan_w,
        btn_h,
    );

    let dst_y = src_y + src_h + gap;
    let dst_h = scale(178, dpi);
    mv(st.t1_grp_dst, inner_x, dst_y, inner_w, dst_h);
    mv(
        st.t1_radio_local,
        sx,
        dst_y + scale(20, dpi),
        sw,
        scale(20, dpi),
    );
    let lbl_w = scale(135, dpi);
    mv(
        st.t1_dst_local_lbl,
        sx + scale(18, dpi),
        dst_y + scale(46, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t1_dst_combo,
        sx + scale(18, dpi) + lbl_w,
        dst_y + scale(43, dpi),
        sw - scale(18, dpi) - lbl_w,
        scale(200, dpi),
    );
    mv(
        st.t1_radio_unc,
        sx,
        dst_y + scale(72, dpi),
        sw,
        scale(20, dpi),
    );
    let browse_w = scale(95, dpi);
    mv(
        st.t1_dst_unc_lbl,
        sx + scale(18, dpi),
        dst_y + scale(98, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t1_dst_unc_edit,
        sx + scale(18, dpi) + lbl_w,
        dst_y + scale(95, dpi),
        sw - scale(18, dpi) - lbl_w - browse_w - gap,
        row_h,
    );
    mv(
        st.t1_btn_browse,
        sx + sw - browse_w,
        dst_y + scale(93, dpi),
        browse_w,
        btn_h,
    );

    let cred_y = dst_y + scale(125, dpi);
    let half_w = (sw - scale(18, dpi) - gap * 2) / 2;
    let cred_lbl_w = scale(140, dpi);
    mv(
        st.t1_cred_user_lbl,
        sx + scale(18, dpi),
        cred_y + 2,
        cred_lbl_w,
        row_h,
    );
    mv(
        st.t1_cred_user_edit,
        sx + scale(18, dpi) + cred_lbl_w,
        cred_y,
        (half_w - cred_lbl_w).max(80),
        row_h,
    );
    let pass_x = sx + scale(18, dpi) + half_w + gap;
    mv(st.t1_cred_pass_lbl, pass_x, cred_y + 2, cred_lbl_w, row_h);
    mv(
        st.t1_cred_pass_edit,
        pass_x + cred_lbl_w,
        cred_y,
        (sw - (pass_x - sx) - cred_lbl_w).max(80),
        row_h,
    );
    mv(
        st.t1_dst_note,
        sx,
        dst_y + scale(152, dpi),
        sw,
        scale(18, dpi),
    );
    // 警告ラベルはコマンドグループ直上・またはログのスペースを圧迫せずdst内に収めるため位置調整
    // dst_noteの下ではなく、コマンドグループの上に配置するか、dst_hを少し広げて配置
    let warn_y = dst_y + dst_h + scale(2, dpi);
    mv(st.t1_warn_lbl, inner_x, warn_y, inner_w, scale(20, dpi));

    let cmd_y = warn_y + scale(22, dpi);
    let cmd_h = scale(86, dpi);
    mv(st.t1_grp_cmd, inner_x, cmd_y, inner_w, cmd_h);
    let cmd_lbl_w = scale(210, dpi);
    mv(st.t1_cmd_lbl, sx, cmd_y + scale(22, dpi), cmd_lbl_w, row_h);
    mv(
        st.t1_cmd_edit,
        sx + cmd_lbl_w,
        cmd_y + scale(19, dpi),
        sw - cmd_lbl_w,
        row_h,
    );
    let b_start_w = scale(175, dpi);
    let b_stop_w = scale(165, dpi);
    let b_copy_w = scale(155, dpi);
    mv(
        st.t1_btn_start,
        sx,
        cmd_y + scale(49, dpi),
        b_start_w,
        btn_h,
    );
    mv(
        st.t1_btn_stop,
        sx + b_start_w + gap,
        cmd_y + scale(49, dpi),
        b_stop_w,
        btn_h,
    );
    mv(
        st.t1_btn_copy_cmd,
        sx + b_start_w + b_stop_w + gap * 2,
        cmd_y + scale(49, dpi),
        b_copy_w,
        btn_h,
    );

    let log_y = cmd_y + cmd_h + gap;
    let log_h = (ch - pad * 2 - log_y).max(scale(130, dpi));
    mv(st.t1_grp_log, inner_x, log_y, inner_w, log_h);
    mv(
        st.t1_status_lbl,
        sx,
        log_y + scale(20, dpi),
        scale(45, dpi),
        row_h,
    );
    mv(
        st.t1_status_val,
        sx + scale(48, dpi),
        log_y + scale(20, dpi),
        scale(260, dpi),
        row_h,
    );
    mv(
        st.t1_progress,
        sx + scale(315, dpi),
        log_y + scale(20, dpi),
        sw - scale(315, dpi),
        scale(20, dpi),
    );
    mv(
        st.t1_log_lbl,
        sx,
        log_y + scale(45, dpi),
        scale(140, dpi),
        scale(18, dpi),
    );
    let clear_w = scale(130, dpi);
    let edit_h = (log_h - scale(72, dpi)).max(scale(45, dpi));
    mv(
        st.t1_log_edit,
        sx,
        log_y + scale(64, dpi),
        sw - clear_w - gap,
        edit_h,
    );
    mv(
        st.t1_btn_clear_log,
        sx + sw - clear_w,
        log_y + scale(64, dpi),
        clear_w,
        btn_h,
    );

    // --- タブ2レイアウト ---
    mv(st.t2_guide, inner_x, y0, inner_w, scale(22, dpi));
    let t2_dst_y = y0 + scale(26, dpi);
    let t2_dst_h = scale(184, dpi);
    mv(st.t2_grp_dst, inner_x, t2_dst_y, inner_w, t2_dst_h);
    mv(
        st.t2_radio_local,
        sx,
        t2_dst_y + scale(20, dpi),
        sw,
        scale(20, dpi),
    );
    mv(
        st.t2_dst_local_lbl,
        sx + scale(18, dpi),
        t2_dst_y + scale(46, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t2_dst_combo,
        sx + scale(18, dpi) + lbl_w,
        t2_dst_y + scale(43, dpi),
        sw - scale(18, dpi) - lbl_w,
        scale(200, dpi),
    );
    mv(
        st.t2_radio_unc,
        sx,
        t2_dst_y + scale(72, dpi),
        sw,
        scale(20, dpi),
    );
    mv(
        st.t2_dst_unc_lbl,
        sx + scale(18, dpi),
        t2_dst_y + scale(98, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t2_dst_unc_edit,
        sx + scale(18, dpi) + lbl_w,
        t2_dst_y + scale(95, dpi),
        sw - scale(18, dpi) - lbl_w,
        row_h,
    );
    let t2_cred_y = t2_dst_y + scale(124, dpi);
    mv(
        st.t2_cred_user_lbl,
        sx + scale(18, dpi),
        t2_cred_y + 2,
        cred_lbl_w,
        row_h,
    );
    mv(
        st.t2_cred_user_edit,
        sx + scale(18, dpi) + cred_lbl_w,
        t2_cred_y,
        (half_w - cred_lbl_w).max(80),
        row_h,
    );
    mv(
        st.t2_cred_pass_lbl,
        pass_x,
        t2_cred_y + 2,
        cred_lbl_w,
        row_h,
    );
    mv(
        st.t2_cred_pass_edit,
        pass_x + cred_lbl_w,
        t2_cred_y,
        (sw - (pass_x - sx) - cred_lbl_w).max(80),
        row_h,
    );
    mv(
        st.t2_btn_fetch,
        sx,
        t2_dst_y + scale(152, dpi),
        scale(150, dpi),
        btn_h,
    );
    mv(
        st.t2_btn_sync_dst,
        sx + scale(156, dpi),
        t2_dst_y + scale(152, dpi),
        scale(245, dpi),
        btn_h,
    );

    let t2_list_y = t2_dst_y + t2_dst_h + gap;
    let t2_list_h = scale(210, dpi);
    mv(st.t2_grp_list, inner_x, t2_list_y, inner_w, t2_list_h);
    mv(
        st.t2_list_lbl,
        sx,
        t2_list_y + scale(20, dpi),
        scale(180, dpi),
        scale(18, dpi),
    );
    mv(
        st.t2_ver_list,
        sx,
        t2_list_y + scale(40, dpi),
        sw,
        scale(128, dpi),
    );
    mv(
        st.t2_btn_copy_ver,
        sx,
        t2_list_y + scale(174, dpi),
        scale(210, dpi),
        btn_h,
    );
    mv(
        st.t2_btn_apply_guide,
        sx + scale(216, dpi),
        t2_list_y + scale(174, dpi),
        scale(225, dpi),
        btn_h,
    );

    let t2_out_y = t2_list_y + t2_list_h + gap;
    let t2_out_h = (ch - pad * 2 - t2_out_y).max(scale(110, dpi));
    mv(st.t2_grp_out, inner_x, t2_out_y, inner_w, t2_out_h);
    mv(
        st.t2_out_lbl,
        sx,
        t2_out_y + scale(20, dpi),
        scale(160, dpi),
        scale(18, dpi),
    );
    mv(
        st.t2_out_edit,
        sx,
        t2_out_y + scale(40, dpi),
        sw,
        (t2_out_h - scale(48, dpi)).max(scale(45, dpi)),
    );

    // --- タブ3レイアウト ---
    mv(st.t3_banner, inner_x, y0, inner_w, scale(38, dpi));
    let t3_info_y = y0 + scale(42, dpi);
    let t3_info_h = scale(58, dpi);
    mv(st.t3_grp_info, inner_x, t3_info_y, inner_w, t3_info_h);
    let t3_lbl1_w = scale(85, dpi);
    let t3_ed1_w = scale(190, dpi);
    let t3_lbl2_w = scale(145, dpi);
    mv(
        st.t3_target_lbl,
        sx,
        t3_info_y + scale(24, dpi),
        t3_lbl1_w,
        row_h,
    );
    mv(
        st.t3_target_edit,
        sx + t3_lbl1_w,
        t3_info_y + scale(21, dpi),
        t3_ed1_w,
        row_h,
    );
    let ver_x = sx + t3_lbl1_w + t3_ed1_w + pad;
    mv(
        st.t3_ver_lbl,
        ver_x,
        t3_info_y + scale(24, dpi),
        t3_lbl2_w,
        row_h,
    );
    mv(
        st.t3_ver_edit,
        ver_x + t3_lbl2_w,
        t3_info_y + scale(21, dpi),
        (sw - (ver_x - sx) - t3_lbl2_w).max(120),
        row_h,
    );

    let t3_btn_y = ch - pad * 2 - btn_h;
    let t3_guide_y = t3_info_y + t3_info_h + gap;
    let t3_guide_h = (t3_btn_y - gap - t3_guide_y).max(scale(180, dpi));
    mv(st.t3_grp_guide, inner_x, t3_guide_y, inner_w, t3_guide_h);
    mv(
        st.t3_guide_lbl,
        sx,
        t3_guide_y + scale(20, dpi),
        scale(150, dpi),
        scale(18, dpi),
    );
    mv(
        st.t3_guide_edit,
        sx,
        t3_guide_y + scale(40, dpi),
        sw,
        (t3_guide_h - scale(48, dpi)).max(scale(100, dpi)),
    );

    mv(st.t3_btn_save, inner_x, t3_btn_y, scale(215, dpi), btn_h);
    mv(
        st.t3_btn_print,
        inner_x + scale(221, dpi),
        t3_btn_y,
        scale(120, dpi),
        btn_h,
    );
    mv(
        st.t3_btn_copy,
        inner_x + scale(347, dpi),
        t3_btn_y,
        scale(185, dpi),
        btn_h,
    );
}

fn switch_tab(st: &mut AppState, new_tab: usize) {
    st.current_tab = new_tab;
    let show_flag = |b: bool| if b { SW_SHOW } else { SW_HIDE };
    unsafe {
        for &h in &st.t1_controls {
            ShowWindow(h, show_flag(new_tab == 0));
        }
        for &h in &st.t2_controls {
            ShowWindow(h, show_flag(new_tab == 1));
        }
        for &h in &st.t3_controls {
            ShowWindow(h, show_flag(new_tab == 2));
        }
        let target_focus = match new_tab {
            0 => st.t1_src_list,
            1 => st.t2_radio_local,
            _ => st.t3_target_edit,
        };
        if GetFocus() != st.hwnd_tab {
            SetFocus(target_focus);
        }
    }
}

fn set_lv_checkbox(lv: HWND, item: i32, checked: bool) {
    let state_val = if checked { 2u32 << 12 } else { 1u32 << 12 };
    let lvi = LVITEMW {
        mask: LVIF_STATE,
        iItem: item,
        state: state_val,
        stateMask: 0xF000,
        ..Default::default()
    };
    unsafe {
        SendMessageW(
            lv,
            LVM_SETITEMSTATE,
            item as usize,
            (&raw const lvi) as LPARAM,
        );
    }
}

fn is_lv_checked(lv: HWND, item: i32) -> bool {
    let state = unsafe { SendMessageW(lv, LVM_GETITEMSTATE, item as usize, 0xF000) } as u32;
    (state >> 12) == 2
}

fn set_lv_text(lv: HWND, item: i32, subitem: i32, text: &str) {
    let mut txt_w = to_wide_null(text);
    let lvi = LVITEMW {
        mask: LVIF_TEXT,
        iItem: item,
        iSubItem: subitem,
        pszText: txt_w.as_mut_ptr(),
        ..Default::default()
    };
    unsafe {
        SendMessageW(
            lv,
            LVM_SETITEMTEXTW,
            item as usize,
            (&raw const lvi) as LPARAM,
        );
    }
}

fn refresh_drives(st: &mut AppState) {
    st.all_drives = enumerate_fixed_drives();
    st.target_drives = st
        .all_drives
        .iter()
        .filter(|d| !d.is_system)
        .cloned()
        .collect();

    unsafe {
        SendMessageW(st.t1_src_list, LVM_DELETEALLITEMS, 0, 0);
        SendMessageW(st.t1_dst_combo, CB_RESETCONTENT, 0, 0);
        SendMessageW(st.t2_dst_combo, CB_RESETCONTENT, 0, 0);
    }

    for (idx, d) in st.all_drives.iter().enumerate() {
        let mut drive_w = to_wide_null(&format!("{}:\\", d.letter));
        let lvi = LVITEMW {
            mask: LVIF_TEXT,
            iItem: idx as i32,
            iSubItem: 0,
            pszText: drive_w.as_mut_ptr(),
            ..Default::default()
        };
        unsafe {
            SendMessageW(
                st.t1_src_list,
                LVM_INSERTITEMW,
                0,
                (&raw const lvi) as LPARAM,
            );
        }
        let label = if d.label.is_empty() {
            "ローカル ディスク"
        } else {
            &d.label
        };
        let free_gb = d.free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = d.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let kind_str = if d.is_system {
            "システムドライブ"
        } else {
            "固定ディスク"
        };
        set_lv_text(st.t1_src_list, idx as i32, 1, label);
        set_lv_text(st.t1_src_list, idx as i32, 2, &d.fs_name);
        set_lv_text(
            st.t1_src_list,
            idx as i32,
            3,
            &format!("{free_gb:.1} GB / {total_gb:.1} GB"),
        );
        set_lv_text(st.t1_src_list, idx as i32, 4, kind_str);
        set_lv_checkbox(st.t1_src_list, idx as i32, d.is_system);
    }

    for d in &st.target_drives {
        let item_w = to_wide_null(&d.format_display());
        unsafe {
            SendMessageW(st.t1_dst_combo, CB_ADDSTRING, 0, item_w.as_ptr() as LPARAM);
            SendMessageW(st.t2_dst_combo, CB_ADDSTRING, 0, item_w.as_ptr() as LPARAM);
        }
    }

    if !st.target_drives.is_empty() {
        let prefer_ntfs = st
            .target_drives
            .iter()
            .position(DriveInfo::is_ntfs)
            .unwrap_or(0);
        unsafe {
            SendMessageW(st.t1_dst_combo, CB_SETCURSEL, prefer_ntfs, 0);
            SendMessageW(st.t2_dst_combo, CB_SETCURSEL, prefer_ntfs, 0);
        }
    }
}

fn selected_includes(st: &AppState) -> Vec<char> {
    let mut out = Vec::new();
    for (idx, d) in st.all_drives.iter().enumerate() {
        if is_lv_checked(st.t1_src_list, idx as i32) {
            out.push(d.letter);
        }
    }
    out
}

fn is_checked_radio(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, 0, 0) as usize == BST_CHECKED }
}

#[allow(clippy::too_many_arguments)]
fn read_target_controls(
    st: &AppState,
    radio_local: HWND,
    dst_combo: HWND,
    dst_unc_edit: HWND,
    cred_user_edit: HWND,
    cred_pass_edit: HWND,
    empty_drive_msg: &str,
) -> Result<BackupTarget, String> {
    if is_checked_radio(radio_local) {
        let sel = unsafe { SendMessageW(dst_combo, CB_GETCURSEL, 0, 0) };
        if sel < 0 || (sel as usize) >= st.target_drives.len() {
            return Err(empty_drive_msg.to_string());
        }
        Ok(BackupTarget::LocalDrive(
            st.target_drives[sel as usize].letter,
        ))
    } else {
        let raw_path = get_window_text(dst_unc_edit);
        let path = validate_unc_path(&raw_path).map_err(|e| e.to_string())?;
        let raw_user = get_window_text(cred_user_edit);
        let raw_pass = get_window_text(cred_pass_edit);
        let (user, password) = validate_credentials(
            if raw_user.is_empty() {
                None
            } else {
                Some(&raw_user)
            },
            if raw_pass.is_empty() {
                None
            } else {
                Some(&raw_pass)
            },
        )
        .map_err(|e| e.to_string())?;
        Ok(BackupTarget::NetworkUnc {
            path,
            user,
            password,
        })
    }
}

fn read_tab1_target(st: &AppState) -> Result<BackupTarget, String> {
    read_target_controls(
        st,
        st.t1_radio_local,
        st.t1_dst_combo,
        st.t1_dst_unc_edit,
        st.t1_cred_user_edit,
        st.t1_cred_pass_edit,
        "保存先ドライブを選択してください。",
    )
}

fn read_tab2_target(st: &AppState) -> Result<BackupTarget, String> {
    read_target_controls(
        st,
        st.t2_radio_local,
        st.t2_dst_combo,
        st.t2_dst_unc_edit,
        st.t2_cred_user_edit,
        st.t2_cred_pass_edit,
        "確認する保存先ドライブを選択してください。",
    )
}

fn apply_tab1_target_mode(st: &AppState) {
    let local_mode = is_checked_radio(st.t1_radio_local);
    let local_flag = if local_mode { TRUE } else { FALSE };
    let unc_flag = if local_mode { FALSE } else { TRUE };
    unsafe {
        EnableWindow(st.t1_dst_combo, local_flag);
        EnableWindow(st.t1_dst_unc_edit, unc_flag);
        EnableWindow(st.t1_btn_browse, unc_flag);
        EnableWindow(st.t1_cred_user_edit, unc_flag);
        EnableWindow(st.t1_cred_pass_edit, unc_flag);
    }
}

fn apply_tab2_target_mode(st: &AppState) {
    let local_mode = is_checked_radio(st.t2_radio_local);
    let local_flag = if local_mode { TRUE } else { FALSE };
    let unc_flag = if local_mode { FALSE } else { TRUE };
    unsafe {
        EnableWindow(st.t2_dst_combo, local_flag);
        EnableWindow(st.t2_dst_unc_edit, unc_flag);
        EnableWindow(st.t2_cred_user_edit, unc_flag);
        EnableWindow(st.t2_cred_pass_edit, unc_flag);
    }
}

fn selected_version_index(st: &AppState) -> Option<usize> {
    let sel = unsafe {
        SendMessageW(
            st.t2_ver_list,
            LVM_GETNEXTITEM,
            usize::MAX,
            LVNI_SELECTED as LPARAM,
        )
    };
    if sel >= 0 && (sel as usize) < st.versions.len() {
        Some(sel as usize)
    } else {
        None
    }
}

fn update_tab1_preview_and_validation(st: &AppState) {
    let includes = selected_includes(st);
    match read_tab1_target(st) {
        Ok(target) => {
            if includes.is_empty() {
                set_window_text(st.t1_cmd_edit, "");
                set_window_text(
                    st.t1_warn_lbl,
                    "注意: バックアップ対象のドライブを1つ以上選択してください。",
                );
                return;
            }
            let preview = build_command_preview(&includes, &target);
            set_window_text(st.t1_cmd_edit, &preview);

            match validate_backup_plan(&includes, &target, &st.all_drives) {
                Ok(()) => set_window_text(st.t1_warn_lbl, "事前チェック: 問題ありません。"),
                Err(e) => set_window_text(st.t1_warn_lbl, &format!("【注意】{e}")),
            }
        }
        Err(msg) => {
            set_window_text(st.t1_cmd_edit, "");
            set_window_text(st.t1_warn_lbl, &format!("【注意】{msg}"));
        }
    }
}

fn update_tab3_guide(st: &AppState) {
    let target = get_window_text(st.t3_target_edit);
    let version = get_window_text(st.t3_ver_edit);
    let guide = generate_restore_guide(&target, &version);
    set_window_text(st.t3_guide_edit, &guide);
}

fn append_edit_line(hwnd_edit: HWND, line: &str) {
    let mut formatted = String::with_capacity(line.len() + 2);
    formatted.push_str(line);
    formatted.push_str("\r\n");
    let w = to_wide_null(&formatted);
    unsafe {
        let focused = GetFocus() == hwnd_edit;
        let mut sel_start: u32 = 0;
        let mut sel_end: u32 = 0;
        if focused {
            SendMessageW(
                hwnd_edit,
                EM_GETSEL,
                (&raw mut sel_start) as WPARAM,
                (&raw mut sel_end) as LPARAM,
            );
        }
        let len = crate::platform::win32::GetWindowTextLengthW(hwnd_edit);
        SendMessageW(hwnd_edit, EM_SETSEL, len as usize, len as isize);
        SendMessageW(hwnd_edit, EM_REPLACESEL, 0, w.as_ptr() as LPARAM);
        if focused && (sel_end as i32) < len {
            SendMessageW(hwnd_edit, EM_SETSEL, sel_start as usize, sel_end as isize);
        }
    }
}

fn set_backup_running_ui(st: &mut AppState, running: bool) {
    unsafe {
        if running {
            EnableWindow(st.t1_btn_stop, TRUE);
            let cur_focus = GetFocus();
            let will_disable = [
                st.t1_btn_start,
                st.t1_btn_rescan,
                st.t1_src_list,
                st.t1_radio_local,
                st.t1_dst_combo,
                st.t1_radio_unc,
                st.t1_dst_unc_edit,
                st.t1_btn_browse,
                st.t1_cred_user_edit,
                st.t1_cred_pass_edit,
            ];
            if will_disable.contains(&cur_focus) {
                SetFocus(st.t1_btn_stop);
            }
            EnableWindow(st.t1_btn_start, FALSE);
            EnableWindow(st.t1_btn_rescan, FALSE);
            EnableWindow(st.t1_src_list, FALSE);
            EnableWindow(st.t1_radio_local, FALSE);
            EnableWindow(st.t1_dst_combo, FALSE);
            EnableWindow(st.t1_radio_unc, FALSE);
            EnableWindow(st.t1_dst_unc_edit, FALSE);
            EnableWindow(st.t1_btn_browse, FALSE);
            EnableWindow(st.t1_cred_user_edit, FALSE);
            EnableWindow(st.t1_cred_pass_edit, FALSE);
            EnableWindow(st.t2_btn_fetch, FALSE);
        } else {
            EnableWindow(st.t1_btn_start, TRUE);
            EnableWindow(st.t1_btn_rescan, TRUE);
            EnableWindow(st.t1_src_list, TRUE);
            EnableWindow(st.t1_radio_local, TRUE);
            EnableWindow(st.t1_radio_unc, TRUE);
            EnableWindow(st.t2_btn_fetch, TRUE);
            apply_tab1_target_mode(st);
            if GetFocus() == st.t1_btn_stop {
                SetFocus(st.t1_btn_start);
            }
            EnableWindow(st.t1_btn_stop, FALSE);
        }
    }
}

fn handle_command(st: &mut AppState, id: usize, code: u16) {
    match id {
        ID_T1_BTN_RESCAN => {
            refresh_drives(st);
            update_tab1_preview_and_validation(st);
        }
        ID_T1_RADIO_LOCAL | ID_T1_RADIO_UNC => {
            apply_tab1_target_mode(st);
            update_tab1_preview_and_validation(st);
        }
        ID_T1_DST_COMBO if code == CBN_SELCHANGE => {
            update_tab1_preview_and_validation(st);
        }
        ID_T1_DST_UNC_EDIT | ID_T1_CRED_USER_EDIT | ID_T1_CRED_PASS_EDIT if code == EN_CHANGE => {
            update_tab1_preview_and_validation(st);
        }
        ID_T1_BTN_BROWSE => {
            if let Some(path) =
                browse_folder_dialog(st.hwnd_main, "バックアップ保存先の共有フォルダを選択")
            {
                set_window_text(st.t1_dst_unc_edit, &path);
                update_tab1_preview_and_validation(st);
            }
        }
        ID_T1_BTN_COPY_CMD => {
            let cmd = get_window_text(st.t1_cmd_edit);
            if cmd.is_empty() {
                dialogs::show_warning(
                    st.hwnd_main,
                    "コピー不可",
                    "コピーできるコマンドがありません。",
                );
            } else if let Err(e) = copy_to_clipboard(st.hwnd_main, &cmd) {
                dialogs::show_error(st.hwnd_main, "エラー", &e);
            } else {
                dialogs::show_info(
                    st.hwnd_main,
                    "コピー完了",
                    "実行コマンドをクリップボードにコピーしました。",
                );
            }
        }
        ID_T1_BTN_CLEAR_LOG => {
            set_window_text(st.t1_log_edit, "");
        }
        ID_T1_BTN_START => {
            if st.worker.is_some() {
                return;
            }
            let includes = selected_includes(st);
            let target = match read_tab1_target(st) {
                Ok(t) => t,
                Err(e) => {
                    dialogs::show_error(st.hwnd_main, "入力エラー", &e);
                    return;
                }
            };
            if let Err(e) = validate_backup_plan(&includes, &target, &st.all_drives) {
                dialogs::show_error(st.hwnd_main, "設定エラー", &e.to_string());
                return;
            }
            let inc_disp: Vec<String> = includes.iter().map(|c| format!("{c}:")).collect();
            let preview = build_command_preview(&includes, &target);
            if !dialogs::confirm_backup_start(
                st.hwnd_main,
                &inc_disp.join(", "),
                &target.as_wbadmin_target(),
                &preview,
            ) {
                return;
            }

            set_window_text(st.t1_log_edit, "");
            set_window_text(st.t1_status_val, "準備中...");
            unsafe {
                SendMessageW(st.t1_progress, PBM_SETPOS, 0, 0);
            }
            st.last_status_percent = None;
            st.is_stopping = false;
            append_edit_line(st.t1_log_edit, &format!("実行: {preview}"));

            let args = build_start_backup_args(&includes, &target);
            st.worker = Some(spawn_wbadmin(
                st.hwnd_main,
                WorkerKind::Backup,
                args,
                Arc::clone(&st.event_queue),
            ));
            set_backup_running_ui(st, true);
        }
        ID_T1_BTN_STOP => {
            if let Some(ref w) = st.worker
                && w.kind() == WorkerKind::Backup
                && dialogs::confirm_backup_stop(st.hwnd_main)
            {
                st.is_stopping = true;
                set_window_text(st.t1_status_val, "中断処理中...");
                append_edit_line(
                    st.t1_log_edit,
                    "安全のため wbadmin stop job -quiet で中断を要求しています...",
                );
                w.request_stop();
                unsafe {
                    SetFocus(st.t1_log_edit);
                    EnableWindow(st.t1_btn_stop, FALSE);
                }
            }
        }
        ID_T2_RADIO_LOCAL | ID_T2_RADIO_UNC => {
            apply_tab2_target_mode(st);
        }
        ID_T2_BTN_SYNC_DST => {
            let local_mode = is_checked_radio(st.t1_radio_local);
            unsafe {
                SendMessageW(
                    st.t2_radio_local,
                    BM_SETCHECK,
                    if local_mode {
                        BST_CHECKED
                    } else {
                        BST_UNCHECKED
                    },
                    0,
                );
                SendMessageW(
                    st.t2_radio_unc,
                    BM_SETCHECK,
                    if local_mode {
                        BST_UNCHECKED
                    } else {
                        BST_CHECKED
                    },
                    0,
                );
                let sel = SendMessageW(st.t1_dst_combo, CB_GETCURSEL, 0, 0);
                if sel >= 0 {
                    SendMessageW(st.t2_dst_combo, CB_SETCURSEL, sel as usize, 0);
                }
            }
            apply_tab2_target_mode(st);
            set_window_text(st.t2_dst_unc_edit, &get_window_text(st.t1_dst_unc_edit));
            set_window_text(st.t2_cred_user_edit, &get_window_text(st.t1_cred_user_edit));
            set_window_text(st.t2_cred_pass_edit, &get_window_text(st.t1_cred_pass_edit));
        }
        ID_T2_BTN_FETCH => {
            if st.worker.is_some() {
                dialogs::show_warning(
                    st.hwnd_main,
                    "実行中",
                    "別の処理が実行中です。完了までお待ちください。",
                );
                return;
            }
            let target = match read_tab2_target(st) {
                Ok(t) => t,
                Err(e) => {
                    dialogs::show_error(st.hwnd_main, "入力エラー", &e);
                    return;
                }
            };
            unsafe {
                SendMessageW(st.t2_ver_list, LVM_DELETEALLITEMS, 0, 0);
                let cur_focus = GetFocus();
                if cur_focus == st.t2_btn_copy_ver
                    || cur_focus == st.t2_btn_apply_guide
                    || cur_focus == st.t2_btn_fetch
                {
                    SetFocus(st.t2_out_edit);
                }
                EnableWindow(st.t2_btn_copy_ver, FALSE);
                EnableWindow(st.t2_btn_apply_guide, FALSE);
                EnableWindow(st.t2_btn_fetch, FALSE);
            }
            st.versions.clear();
            set_window_text(st.t2_out_edit, "既存バックアップを検索しています...\r\n");

            let args = build_get_versions_args(&target);
            st.worker = Some(spawn_wbadmin(
                st.hwnd_main,
                WorkerKind::GetVersions,
                args,
                Arc::clone(&st.event_queue),
            ));
        }
        ID_T2_BTN_COPY_VER => {
            if let Some(idx) = selected_version_index(st) {
                let vid = &st.versions[idx].version_id;
                if let Err(e) = copy_to_clipboard(st.hwnd_main, vid) {
                    dialogs::show_error(st.hwnd_main, "エラー", &e);
                } else {
                    dialogs::show_info(
                        st.hwnd_main,
                        "コピー完了",
                        &format!("バージョン識別子「{vid}」をコピーしました。"),
                    );
                }
            }
        }
        ID_T2_BTN_APPLY_GUIDE => {
            if let Some(idx) = selected_version_index(st) {
                let vid = st.versions[idx].version_id.clone();
                if let Ok(target) = read_tab2_target(st) {
                    set_window_text(st.t3_target_edit, &target.as_wbadmin_target());
                }
                set_window_text(st.t3_ver_edit, &vid);
                update_tab3_guide(st);
                unsafe {
                    SendMessageW(st.hwnd_tab, TCM_SETCURSEL, 2, 0);
                }
                switch_tab(st, 2);
                dialogs::show_info(
                    st.hwnd_main,
                    "反映完了",
                    "選択した保存先とバージョン識別子を「復元手順・印刷」タブに反映しました。",
                );
            }
        }
        ID_T3_TARGET_EDIT | ID_T3_VER_EDIT if code == EN_CHANGE => {
            update_tab3_guide(st);
        }
        ID_T3_BTN_SAVE => {
            let content = get_window_text(st.t3_guide_edit);
            match save_text_with_dialog(st.hwnd_main, "WindowsImageBackup_復元手順.txt", &content)
            {
                Ok(Some(path)) => {
                    dialogs::show_info(
                        st.hwnd_main,
                        "保存完了",
                        &format!("復元手順を保存しました:\n{}", path.display()),
                    );
                }
                Ok(None) => {}
                Err(e) => dialogs::show_error(st.hwnd_main, "保存エラー", &e),
            }
        }
        ID_T3_BTN_PRINT => {
            let content = get_window_text(st.t3_guide_edit);
            match print_text_document(st.hwnd_main, "WindowsImageBackup 復元手順ガイド", &content)
            {
                Ok(true) => {
                    dialogs::show_info(
                        st.hwnd_main,
                        "印刷完了",
                        "復元手順をプリンタへ送信しました。",
                    );
                }
                Ok(false) => {}
                Err(e) => dialogs::show_error(st.hwnd_main, "印刷エラー", &e),
            }
        }
        ID_T3_BTN_COPY => {
            let content = get_window_text(st.t3_guide_edit);
            if let Err(e) = copy_to_clipboard(st.hwnd_main, &content) {
                dialogs::show_error(st.hwnd_main, "エラー", &e);
            } else {
                dialogs::show_info(
                    st.hwnd_main,
                    "コピー完了",
                    "復元手順の全文をクリップボードにコピーしました。",
                );
            }
        }
        _ => {}
    }
}

fn handle_notify(st: &mut AppState, lparam: LPARAM) {
    if lparam == 0 {
        return;
    }
    let nmhdr = unsafe { &*(lparam as *const NMHDR) };
    if nmhdr.hwndFrom == st.hwnd_tab && nmhdr.code == TCN_SELCHANGE {
        let sel = unsafe { SendMessageW(st.hwnd_tab, TCM_GETCURSEL, 0, 0) };
        if sel >= 0 {
            switch_tab(st, sel as usize);
        }
    } else if nmhdr.hwndFrom == st.t1_src_list && nmhdr.code == LVN_ITEMCHANGED {
        update_tab1_preview_and_validation(st);
    } else if nmhdr.hwndFrom == st.t2_ver_list && nmhdr.code == LVN_ITEMCHANGED {
        let has_sel = selected_version_index(st).is_some();
        unsafe {
            if !has_sel && (GetFocus() == st.t2_btn_copy_ver || GetFocus() == st.t2_btn_apply_guide)
            {
                SetFocus(st.t2_ver_list);
            }
            EnableWindow(st.t2_btn_copy_ver, if has_sel { TRUE } else { FALSE });
            EnableWindow(st.t2_btn_apply_guide, if has_sel { TRUE } else { FALSE });
        }
    }
}

fn handle_worker_events(st: &mut AppState) {
    let events: Vec<WorkerEvent> = {
        let Ok(mut q) = st.event_queue.lock() else {
            return;
        };
        q.drain(..).collect()
    };

    for ev in events {
        match ev {
            WorkerEvent::LogLine { kind, text } => match kind {
                WorkerKind::Backup => append_edit_line(st.t1_log_edit, &text),
                WorkerKind::GetVersions => append_edit_line(st.t2_out_edit, &text),
            },
            WorkerEvent::Progress { percent } => {
                unsafe {
                    SendMessageW(st.t1_progress, PBM_SETPOS, percent as usize, 0);
                }
                if st.last_status_percent != Some(percent) {
                    st.last_status_percent = Some(percent);
                    set_window_text(st.t1_status_val, &format!("処理中: {percent}% 完了"));
                }
            }
            WorkerEvent::Error { kind, message } => {
                st.worker.take();
                match kind {
                    WorkerKind::Backup => {
                        set_backup_running_ui(st, false);
                        set_window_text(st.t1_status_val, "起動エラー");
                        append_edit_line(st.t1_log_edit, &message);
                        dialogs::show_error(st.hwnd_main, "実行エラー", &message);
                    }
                    WorkerKind::GetVersions => {
                        unsafe {
                            EnableWindow(st.t2_btn_fetch, TRUE);
                        }
                        append_edit_line(st.t2_out_edit, &message);
                        dialogs::show_error(st.hwnd_main, "取得エラー", &message);
                    }
                }
            }
            WorkerEvent::Finished {
                kind,
                exit_code,
                full_output,
            } => {
                st.worker.take();
                match kind {
                    WorkerKind::Backup => {
                        set_backup_running_ui(st, false);
                        let code_val = exit_code.unwrap_or(-1);
                        if st.is_stopping {
                            set_window_text(st.t1_status_val, "中断されました");
                            append_edit_line(
                                st.t1_log_edit,
                                &format!(
                                    "バックアップが中断されました（終了コード: {code_val}）。"
                                ),
                            );
                        } else if code_val == 0 {
                            unsafe {
                                SendMessageW(st.t1_progress, PBM_SETPOS, 100, 0);
                            }
                            set_window_text(st.t1_status_val, "完了しました");
                            append_edit_line(
                                st.t1_log_edit,
                                "システムイメージの作成が正常に完了しました。",
                            );
                            if let Ok(target) = read_tab1_target(st) {
                                set_window_text(st.t3_target_edit, &target.as_wbadmin_target());
                                update_tab3_guide(st);
                            }
                            dialogs::show_info(
                                st.hwnd_main,
                                "バックアップ完了",
                                "システムイメージの作成が完了しました。\n「復元手順・印刷」タブから復元手順を印刷またはテキスト保存しておくことをおすすめします。",
                            );
                        } else {
                            set_window_text(
                                st.t1_status_val,
                                &format!("エラー終了 (コード: {code_val})"),
                            );
                            append_edit_line(
                                st.t1_log_edit,
                                &format!("処理がエラーで終了しました（終了コード: {code_val}）。"),
                            );
                            dialogs::show_error(
                                st.hwnd_main,
                                "バックアップエラー",
                                &format!(
                                    "バックアップが正常に完了しませんでした（終了コード: {code_val}）。\n実行ログをご確認ください。"
                                ),
                            );
                        }
                        st.is_stopping = false;
                    }
                    WorkerKind::GetVersions => {
                        unsafe {
                            EnableWindow(st.t2_btn_fetch, TRUE);
                        }
                        st.versions = parse_get_versions_output(&full_output);
                        unsafe {
                            SendMessageW(st.t2_ver_list, LVM_DELETEALLITEMS, 0, 0);
                        }
                        for (idx, v) in st.versions.iter().enumerate() {
                            let mut time_w = to_wide_null(&v.backup_time);
                            let lvi = LVITEMW {
                                mask: LVIF_TEXT,
                                iItem: idx as i32,
                                iSubItem: 0,
                                pszText: time_w.as_mut_ptr(),
                                ..Default::default()
                            };
                            unsafe {
                                SendMessageW(
                                    st.t2_ver_list,
                                    LVM_INSERTITEMW,
                                    0,
                                    (&raw const lvi) as LPARAM,
                                );
                            }
                            set_lv_text(st.t2_ver_list, idx as i32, 1, &v.version_id);
                            set_lv_text(st.t2_ver_list, idx as i32, 2, &v.target_location);
                            set_lv_text(st.t2_ver_list, idx as i32, 3, &v.recoverable_items);
                        }
                    }
                }
            }
        }
    }
}
