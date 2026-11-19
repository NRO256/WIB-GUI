pub mod dialogs;
pub mod layout;
pub mod state;
pub mod theme;

use crate::domain::guide::generate_restore_guide;
use crate::domain::validation::{
    BackupTarget, DriveInfo, PlanCheckOutcome, ValidationError, evaluate_backup_plan,
    format_elapsed, format_gib, validate_credentials, validate_unc_path,
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
    ATOM, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON,
    BS_GROUPBOX, BS_PUSHBUTTON, BST_CHECKED, BST_UNCHECKED, CB_ADDSTRING, CB_GETCURSEL,
    CB_RESETCONTENT, CB_SETCURSEL, CBN_SELCHANGE, CBS_DROPDOWNLIST, CBS_HASSTRINGS, COLOR_BTNFACE,
    CoInitializeEx, CoUninitialize, CreateWindowExW, DLGC_WANTALLKEYS, DLGC_WANTTAB, DWORD_PTR,
    DefSubclassProc, DefWindowProcW, DispatchMessageW, EC_LEFTMARGIN, EC_RIGHTMARGIN, EM_GETSEL,
    EM_REPLACESEL, EM_SETCUEBANNER, EM_SETLIMITTEXT, EM_SETMARGINS, EM_SETPASSWORDCHAR, EM_SETSEL,
    EN_CHANGE, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE, ES_PASSWORD, ES_READONLY,
    EnableWindow, FALSE, GWL_STYLE, GetDpiForSystem, GetDpiForWindow, GetFocus, GetKeyState,
    GetMessageW, GetModuleHandleW, GetWindowLongPtrW, HFONT, HINSTANCE, HMENU, HWND,
    ICC_BAR_CLASSES, ICC_LISTVIEW_CLASSES, ICC_PROGRESS_CLASS, ICC_STANDARD_CLASSES,
    ICC_TAB_CLASSES, ICON_BIG, ICON_SMALL, IDC_ARROW, IDI_APPLICATION, INITCOMMONCONTROLSEX,
    InitCommonControlsEx, InvalidateRect, IsDialogMessageW, KillTimer, LPARAM, LRESULT, LVCF_FMT,
    LVCF_TEXT, LVCF_WIDTH, LVCFMT_LEFT, LVCFMT_RIGHT, LVCOLUMNW, LVIF_STATE, LVIF_TEXT,
    LVIS_FOCUSED, LVIS_SELECTED, LVITEMW, LVM_DELETEALLITEMS, LVM_GETITEMSTATE, LVM_GETNEXTITEM,
    LVM_INSERTCOLUMNW, LVM_INSERTITEMW, LVM_SETEXTENDEDLISTVIEWSTYLE, LVM_SETITEMSTATE,
    LVM_SETITEMTEXTW, LVN_ITEMCHANGED, LVN_KEYDOWN, LVNI_SELECTED, LVS_EX_CHECKBOXES,
    LVS_EX_DOUBLEBUFFER, LVS_EX_FULLROWSELECT, LVS_REPORT, LVS_SHOWSELALWAYS, LVS_SINGLESEL,
    LoadCursorW, LoadIconW, MINMAXINFO, MSG, NM_DBLCLK, NMHDR, NMLVKEYDOWN, OPAQUE, PBM_SETMARQUEE,
    PBM_SETPOS, PBM_SETRANGE, PBM_SETSTATE, PBS_MARQUEE, PBST_ERROR, PBST_NORMAL, PBST_PAUSED,
    PostQuitMessage, RECT, RegisterClassExW, RemoveWindowSubclass, SB_SETTEXTW, SBARS_SIZEGRIP,
    SPI_GETWORKAREA, SS_CENTERIMAGE, SS_LEFT, SS_NOPREFIX, SW_HIDE, SW_SHOW, SWP_NOACTIVATE,
    SWP_NOZORDER, SendMessageW, SetBkColor, SetBkMode, SetFocus, SetTextColor, SetTimer,
    SetWindowLongPtrW, SetWindowPos, SetWindowSubclass, SetWindowTheme, ShowWindow,
    SystemParametersInfoW, TCIF_TEXT, TCITEMW, TCM_GETCURSEL, TCM_INSERTITEMW, TCM_SETCURSEL,
    TCN_SELCHANGE, TRANSPARENT, TRUE, TranslateMessage, UINT, UINT_PTR, UpdateWindow, VK_1, VK_2,
    VK_3, VK_A, VK_CONTROL, VK_F5, VK_MENU, VK_P, VK_RETURN, VK_S, VK_SHIFT, VK_TAB, WM_CHAR,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC,
    WM_DESTROY, WM_DPICHANGED, WM_GETDLGCODE, WM_GETMINMAXINFO, WM_KEYDOWN, WM_NCDESTROY,
    WM_NOTIFY, WM_SETFONT, WM_SETICON, WM_SETTINGCHANGE, WM_SIZE, WM_SYSCOLORCHANGE, WM_SYSKEYDOWN,
    WM_TIMER, WNDCLASSEXW, WPARAM, WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE, WS_GROUP,
    WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL, get_window_text, make_int_resource_w,
    set_window_text, to_wide_null,
};
use layout::layout_controls;
use state::AppState;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use theme::{
    COLOR_CODE_BG, COLOR_DANGER_BG, COLOR_OK_BG, COLOR_TAB_BG, COLOR_TEXT_DANGER, COLOR_TEXT_INFO,
    COLOR_TEXT_OK, COLOR_TEXT_PRIMARY, COLOR_TEXT_SECONDARY, COLOR_TEXT_WARN, COLOR_WARN_BG,
    StatusTone, Theme, ValidationLevel, extract_app_icons, scale,
};

const APP_TITLE_BASE: &str = concat!(
    "WindowsImageBackup GUI (WIB-GUI) v",
    env!("CARGO_PKG_VERSION"),
    " [管理者]"
);

const TIMER_ID_BACKUP: UINT_PTR = 1;
const ID_STATUS_BAR: usize = 99;
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
const ID_T1_CHK_SHOW_PASS: usize = 116;
const ID_T1_BTN_SAVE_LOG: usize = 117;

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
const ID_T2_BTN_BROWSE: usize = 213;
const ID_T2_CHK_SHOW_PASS: usize = 214;

const ID_T3_TARGET_EDIT: usize = 301;
const ID_T3_VER_EDIT: usize = 302;
const ID_T3_GUIDE_EDIT: usize = 303;
const ID_T3_BTN_SAVE: usize = 304;
const ID_T3_BTN_PRINT: usize = 305;
const ID_T3_BTN_COPY: usize = 306;
const ID_T3_BTN_SYNC_DST: usize = 307;

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

fn key_down(vk: i32) -> bool {
    (unsafe { GetKeyState(vk) } as u16 & 0x8000) != 0
}

fn set_status_bar(hwnd_status: HWND, left: Option<&str>, right: Option<&str>) {
    if let Some(txt) = left {
        let w = to_wide_null(txt);
        unsafe {
            SendMessageW(hwnd_status, SB_SETTEXTW, 0, w.as_ptr() as LPARAM);
        }
    }
    if let Some(txt) = right {
        let w = to_wide_null(txt);
        unsafe {
            SendMessageW(hwnd_status, SB_SETTEXTW, 1, w.as_ptr() as LPARAM);
        }
    }
}

pub fn run_gui() -> i32 {
    unsafe {
        let co_hr = CoInitializeEx(std::ptr::null_mut(), 0x2); // COINIT_APARTMENTTHREADED
        let icce = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_LISTVIEW_CLASSES
                | ICC_PROGRESS_CLASS
                | ICC_TAB_CLASSES
                | ICC_STANDARD_CLASSES
                | ICC_BAR_CLASSES,
        };
        InitCommonControlsEx(&raw const icce);

        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide_null("WibGuiMainWindowClass");
        let title = to_wide_null(APP_TITLE_BASE);

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

        let sys_dpi = GetDpiForSystem().max(96);
        let mut work = RECT::default();
        let has_work = SystemParametersInfoW(SPI_GETWORKAREA, 0, (&raw mut work).cast(), 0) != 0;
        let (win_x, win_y, win_w, win_h) = if has_work {
            let avail_w = (work.right - work.left).max(400);
            let avail_h = (work.bottom - work.top).max(400);
            let w = scale(960, sys_dpi).min((avail_w - 40).max(avail_w.min(scale(820, sys_dpi))));
            let h = scale(810, sys_dpi).min((avail_h - 40).max(avail_h.min(scale(710, sys_dpi))));
            let x = work.left + (avail_w - w) / 2;
            let y = work.top + (avail_h - h) / 2;
            (x, y, w, h)
        } else {
            (
                crate::platform::win32::CW_USEDEFAULT,
                crate::platform::win32::CW_USEDEFAULT,
                scale(960, sys_dpi),
                scale(810, sys_dpi),
            )
        };

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            win_x,
            win_y,
            win_w,
            win_h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return 1;
        }

        with_state(|st| {
            if !st.icon_large.0.is_null() {
                SendMessageW(hwnd, WM_SETICON, ICON_BIG, st.icon_large.0 as LPARAM);
            }
            if !st.icon_small.0.is_null() {
                SendMessageW(hwnd, WM_SETICON, ICON_SMALL, st.icon_small.0 as LPARAM);
            }
        });

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        with_state(|st| {
            SetFocus(st.t1_src_list);
        });

        let mut msg = MSG::default();
        while GetMessageW(&raw mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_KEYDOWN && msg.wParam == VK_TAB as usize && key_down(VK_CONTROL) {
                let reverse = key_down(VK_SHIFT);
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

            if msg.message == WM_SYSKEYDOWN && key_down(VK_MENU) && !key_down(VK_CONTROL) {
                let target_tab = match msg.wParam as u16 {
                    VK_1 => Some(0),
                    VK_2 => Some(1),
                    VK_3 => Some(2),
                    _ => None,
                };
                if let Some(tab) = target_tab {
                    with_state_mut(|st| {
                        SendMessageW(st.hwnd_tab, TCM_SETCURSEL, tab, 0);
                        switch_tab(st, tab);
                    });
                    continue;
                }
            }

            if msg.message == WM_KEYDOWN
                && msg.wParam == VK_F5 as usize
                && !key_down(VK_CONTROL)
                && !key_down(VK_MENU)
                && !key_down(VK_SHIFT)
            {
                let handled = with_state_mut(|st| match st.current_tab {
                    0 => {
                        if st.worker.is_none() {
                            refresh_drives(st);
                            update_tab1_preview_and_validation(st);
                        }
                        true
                    }
                    1 => {
                        if st.worker.is_none() {
                            start_fetch_versions(st);
                        }
                        true
                    }
                    _ => false,
                })
                .unwrap_or(false);
                if handled {
                    continue;
                }
            }

            if msg.message == WM_KEYDOWN
                && key_down(VK_CONTROL)
                && !key_down(VK_MENU)
                && !key_down(VK_SHIFT)
                && (msg.wParam == VK_S as usize || msg.wParam == VK_P as usize)
            {
                let wparam = msg.wParam;
                let handled = with_state_mut(|st| {
                    if st.current_tab == 2 {
                        if wparam == VK_S as usize {
                            save_restore_guide(st);
                        } else {
                            print_restore_guide(st);
                        }
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
                if handled {
                    continue;
                }
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
                if let Ok(target) = read_tab1_target(st) {
                    set_window_text(st.t3_target_edit, &target.as_wbadmin_target());
                }
                update_tab1_preview_and_validation(st);
                update_tab3_guide(st);
                layout_controls(st);
                switch_tab(st, 0);
                set_status_bar(st.hwnd_status, None, Some("状態: 待機中"));
            });
            0
        }
        WM_GETMINMAXINFO => {
            if lparam != 0 {
                let dpi = with_state(|s| s.dpi)
                    .unwrap_or_else(|| unsafe { GetDpiForWindow(hwnd) }.max(96));
                let mmi = unsafe { &mut *(lparam as *mut MINMAXINFO) };
                mmi.ptMinTrackSize.x = scale(820, dpi);
                mmi.ptMinTrackSize.y = scale(710, dpi);
            }
            0
        }
        WM_SIZE => {
            with_state(|st| {
                layout_controls(st);
            });
            0
        }
        WM_SYSCOLORCHANGE | WM_SETTINGCHANGE => {
            with_state_mut(|st| {
                st.theme.refresh_system_mode();
            });
            unsafe {
                InvalidateRect(hwnd, std::ptr::null(), TRUE);
            }
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
                st.theme = Theme::for_dpi(st.dpi);
                apply_all_fonts_and_margins(st);
                layout_controls(st);
            });
            unsafe {
                InvalidateRect(hwnd, std::ptr::null(), TRUE);
            }
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as crate::platform::win32::HDC;
            let ctrl = lparam as HWND;
            let brush = with_state(|st| unsafe {
                if st.theme.high_contrast {
                    return None;
                }
                if st.is_readonly_code_edit(ctrl) {
                    SetBkMode(hdc, OPAQUE);
                    SetBkColor(hdc, COLOR_CODE_BG);
                    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
                    return Some(st.theme.brush_code_bg.0);
                }
                if st.is_editable_input(ctrl) {
                    return None;
                }
                if ctrl == st.t1_warn_lbl {
                    let (bg, fg, br) = match st.validation_level {
                        ValidationLevel::Ok => (COLOR_OK_BG, COLOR_TEXT_OK, st.theme.brush_ok_bg.0),
                        ValidationLevel::Warn => {
                            (COLOR_WARN_BG, COLOR_TEXT_WARN, st.theme.brush_warn_bg.0)
                        }
                        ValidationLevel::Error => (
                            COLOR_DANGER_BG,
                            COLOR_TEXT_DANGER,
                            st.theme.brush_danger_bg.0,
                        ),
                    };
                    SetBkMode(hdc, OPAQUE);
                    SetBkColor(hdc, bg);
                    SetTextColor(hdc, fg);
                    return Some(br);
                }
                if ctrl == st.t3_banner {
                    SetBkMode(hdc, OPAQUE);
                    SetBkColor(hdc, COLOR_DANGER_BG);
                    SetTextColor(hdc, COLOR_TEXT_DANGER);
                    return Some(st.theme.brush_danger_bg.0);
                }
                if ctrl == st.t1_status_val {
                    let fg = match st.status_tone {
                        StatusTone::Idle => COLOR_TEXT_SECONDARY,
                        StatusTone::Running => COLOR_TEXT_INFO,
                        StatusTone::Ok => COLOR_TEXT_OK,
                        StatusTone::Warn => COLOR_TEXT_WARN,
                        StatusTone::Danger => COLOR_TEXT_DANGER,
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    SetBkColor(hdc, COLOR_TAB_BG);
                    SetTextColor(hdc, fg);
                    return Some(st.theme.brush_tab_bg.0);
                }
                if st.is_secondary_label(ctrl) {
                    SetBkMode(hdc, TRANSPARENT);
                    SetBkColor(hdc, COLOR_TAB_BG);
                    SetTextColor(hdc, COLOR_TEXT_SECONDARY);
                    return Some(st.theme.brush_tab_bg.0);
                }
                if st.is_tab_child(ctrl) {
                    SetBkMode(hdc, TRANSPARENT);
                    SetBkColor(hdc, COLOR_TAB_BG);
                    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
                    return Some(st.theme.brush_tab_bg.0);
                }
                None
            })
            .flatten();
            brush.map_or_else(
                || unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
                |b| b as LRESULT,
            )
        }
        WM_CTLCOLORBTN => {
            let hdc = wparam as crate::platform::win32::HDC;
            let ctrl = lparam as HWND;
            let brush = with_state(|st| unsafe {
                if !st.theme.high_contrast && st.is_tab_child(ctrl) {
                    SetBkMode(hdc, TRANSPARENT);
                    SetBkColor(hdc, COLOR_TAB_BG);
                    Some(st.theme.brush_tab_bg.0)
                } else {
                    None
                }
            })
            .flatten();
            brush.map_or_else(
                || unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
                |b| b as LRESULT,
            )
        }
        WM_CTLCOLOREDIT => {
            let hdc = wparam as crate::platform::win32::HDC;
            let ctrl = lparam as HWND;
            let brush = with_state(|st| unsafe {
                if !st.theme.high_contrast && st.is_readonly_code_edit(ctrl) {
                    SetBkMode(hdc, OPAQUE);
                    SetBkColor(hdc, COLOR_CODE_BG);
                    SetTextColor(hdc, COLOR_TEXT_PRIMARY);
                    Some(st.theme.brush_code_bg.0)
                } else {
                    None
                }
            })
            .flatten();
            brush.map_or_else(
                || unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
                |b| b as LRESULT,
            )
        }
        WM_TIMER => {
            if wparam == TIMER_ID_BACKUP {
                with_state(|st| {
                    update_running_backup_indicators(st);
                });
            }
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
            unsafe {
                KillTimer(hwnd, TIMER_ID_BACKUP);
            }
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

fn apply_font(hwnd: HWND, hfont: HFONT) {
    unsafe {
        SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
    }
}

fn set_edit_margins(hwnd: HWND, dpi: u32) {
    let m = scale(6, dpi).max(2) as usize;
    let lparam = ((m << 16) | m) as LPARAM;
    unsafe {
        SendMessageW(hwnd, EM_SETMARGINS, EC_LEFTMARGIN | EC_RIGHTMARGIN, lparam);
    }
}

fn set_cue_banner(hwnd: HWND, text: &str) {
    let w = to_wide_null(text);
    unsafe {
        SendMessageW(hwnd, EM_SETCUEBANNER, 1, w.as_ptr() as LPARAM);
    }
}

fn set_explorer_theme(lv: HWND) {
    let explorer = to_wide_null("Explorer");
    unsafe {
        SetWindowTheme(lv, explorer.as_ptr(), std::ptr::null());
    }
}

fn apply_all_fonts_and_margins(st: &AppState) {
    let normal = st.theme.font_normal.0;
    let bold = st.theme.font_bold.0;
    let mono = st.theme.font_mono.0;

    apply_font(st.hwnd_tab, normal);
    apply_font(st.hwnd_status, normal);
    for &h in &st.t1_controls {
        apply_font(h, normal);
    }
    for &h in &st.t2_controls {
        apply_font(h, normal);
    }
    for &h in &st.t3_controls {
        apply_font(h, normal);
    }

    for &h in &[
        st.t1_grp_src,
        st.t1_grp_dst,
        st.t1_grp_cmd,
        st.t1_grp_log,
        st.t2_grp_dst,
        st.t2_grp_list,
        st.t2_grp_out,
        st.t3_grp_info,
        st.t3_grp_guide,
        st.t1_warn_lbl,
        st.t1_status_val,
        st.t3_banner,
    ] {
        apply_font(h, bold);
    }

    for &h in &[
        st.t1_cmd_edit,
        st.t1_log_edit,
        st.t2_out_edit,
        st.t3_guide_edit,
    ] {
        apply_font(h, mono);
    }

    for &h in &[
        st.t1_dst_unc_edit,
        st.t1_cred_user_edit,
        st.t1_cred_pass_edit,
        st.t1_cmd_edit,
        st.t1_log_edit,
        st.t2_dst_unc_edit,
        st.t2_cred_user_edit,
        st.t2_cred_pass_edit,
        st.t2_out_edit,
        st.t3_target_edit,
        st.t3_ver_edit,
        st.t3_guide_edit,
    ] {
        set_edit_margins(h, st.dpi);
    }
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
    dpi: u32,
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
    set_edit_margins(h, dpi);
    h
}

fn make_input_edit(
    hinst: HINSTANCE,
    parent: HWND,
    id: usize,
    is_password: bool,
    font: HFONT,
    dpi: u32,
    cue: &str,
) -> HWND {
    let style = if is_password {
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL | ES_PASSWORD
    } else {
        WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL
    };
    let h = make_ctrl(hinst, parent, "EDIT", "", style, WS_EX_CLIENTEDGE, id, font);
    set_edit_margins(h, dpi);
    set_cue_banner(h, cue);
    h
}

fn create_initial_state(hwnd_main: HWND) -> AppState {
    let dpi = unsafe { GetDpiForWindow(hwnd_main) }.max(96);
    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    let theme = Theme::for_dpi(dpi);
    let (icon_large, icon_small) = extract_app_icons();
    let hfont_normal = theme.font_normal.0;
    let hfont_bold = theme.font_bold.0;
    let hfont_mono = theme.font_mono.0;

    let hwnd_status = make_ctrl(
        hinst,
        hwnd_main,
        "msctls_statusbar32",
        "",
        SBARS_SIZEGRIP,
        0,
        ID_STATUS_BAR,
        hfont_normal,
    );

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
        hfont_bold,
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
    set_explorer_theme(t1_src_list);
    unsafe {
        SendMessageW(
            t1_src_list,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            0,
            (LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER) as LPARAM,
        );
    }
    init_drive_list_columns(t1_src_list, dpi);
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
        hfont_bold,
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
        SS_LEFT | SS_CENTERIMAGE,
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
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t1_dst_unc_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T1_DST_UNC_EDIT,
        false,
        hfont_normal,
        dpi,
        "例: \\\\Server\\Share",
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
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t1_cred_user_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T1_CRED_USER_EDIT,
        false,
        hfont_normal,
        dpi,
        "例: PC名\\ユーザー名（省略可）",
    );
    let t1_cred_pass_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "パスワード(任意)(&W):",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t1_cred_pass_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T1_CRED_PASS_EDIT,
        true,
        hfont_normal,
        dpi,
        "必要な場合のみ入力",
    );
    let t1_chk_show_pass = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "表示(&H)",
        WS_TABSTOP | BS_AUTOCHECKBOX,
        0,
        ID_T1_CHK_SHOW_PASS,
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
        hfont_bold,
    );
    let t1_cmd_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "実行コマンド（プレビュー）(&M):",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t1_cmd_edit = make_readonly_edit(hinst, hwnd_main, ID_T1_CMD_EDIT, false, hfont_mono, dpi);
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
        hfont_bold,
    );
    let t1_status_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "状態:",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t1_status_val = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "待機中",
        SS_LEFT | SS_NOPREFIX | SS_CENTERIMAGE,
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
    let t1_log_edit = make_readonly_edit(hinst, hwnd_main, ID_T1_LOG_EDIT, true, hfont_mono, dpi);
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
    let t1_btn_save_log = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "ログを保存(&V)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T1_BTN_SAVE_LOG,
        hfont_normal,
    );

    unsafe {
        EnableWindow(t1_dst_unc_edit, FALSE);
        EnableWindow(t1_btn_browse, FALSE);
        EnableWindow(t1_cred_user_edit, FALSE);
        EnableWindow(t1_cred_pass_edit, FALSE);
        EnableWindow(t1_chk_show_pass, FALSE);
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
        t1_chk_show_pass,
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
        t1_btn_save_log,
    ];

    // --- タブ2 ---
    let t2_guide = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "指定した保存先に作成済みのシステムイメージ一覧を確認します。選択行をダブルクリックまたはEnterで復元手順へ反映できます。",
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
        hfont_bold,
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
        SS_LEFT | SS_CENTERIMAGE,
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
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t2_dst_unc_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T2_DST_UNC_EDIT,
        false,
        hfont_normal,
        dpi,
        "例: \\\\Server\\Share",
    );
    let t2_btn_browse = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "参照(&B)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T2_BTN_BROWSE,
        hfont_normal,
    );
    let t2_cred_user_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "ユーザー名(任意)(&U):",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t2_cred_user_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T2_CRED_USER_EDIT,
        false,
        hfont_normal,
        dpi,
        "例: PC名\\ユーザー名（省略可）",
    );
    let t2_cred_pass_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "パスワード(任意)(&W):",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t2_cred_pass_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T2_CRED_PASS_EDIT,
        true,
        hfont_normal,
        dpi,
        "必要な場合のみ入力",
    );
    let t2_chk_show_pass = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "表示(&H)",
        WS_TABSTOP | BS_AUTOCHECKBOX,
        0,
        ID_T2_CHK_SHOW_PASS,
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
        hfont_bold,
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
    set_explorer_theme(t2_ver_list);
    unsafe {
        SendMessageW(
            t2_ver_list,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            0,
            (LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER) as LPARAM,
        );
    }
    init_version_list_columns(t2_ver_list, dpi);
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
        hfont_bold,
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
    let t2_out_edit = make_readonly_edit(hinst, hwnd_main, ID_T2_OUT_EDIT, true, hfont_mono, dpi);

    unsafe {
        EnableWindow(t2_dst_unc_edit, FALSE);
        EnableWindow(t2_btn_browse, FALSE);
        EnableWindow(t2_cred_user_edit, FALSE);
        EnableWindow(t2_cred_pass_edit, FALSE);
        EnableWindow(t2_chk_show_pass, FALSE);
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
        t2_btn_browse,
        t2_cred_user_lbl,
        t2_cred_user_edit,
        t2_cred_pass_lbl,
        t2_cred_pass_edit,
        t2_chk_show_pass,
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
        " 【重要】システムイメージからの復元は、現在起動しているWindows上からは実行できません。\n 必ずPCを再起動し、「Windows回復環境（WinRE）」から実行してください。",
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
        hfont_bold,
    );
    let t3_target_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "保存先(&T):",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t3_target_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T3_TARGET_EDIT,
        false,
        hfont_normal,
        dpi,
        "例: D: または \\\\Server\\Share",
    );
    set_window_text(t3_target_edit, "D:");
    let t3_ver_lbl = make_ctrl(
        hinst,
        hwnd_main,
        "STATIC",
        "バージョン識別子(&I):",
        SS_LEFT | SS_CENTERIMAGE,
        0,
        0,
        hfont_normal,
    );
    let t3_ver_edit = make_input_edit(
        hinst,
        hwnd_main,
        ID_T3_VER_EDIT,
        false,
        hfont_normal,
        dpi,
        "例: 11/17/2026-00:30",
    );
    let t3_btn_sync_dst = make_ctrl(
        hinst,
        hwnd_main,
        "BUTTON",
        "保存先を反映(&M)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_T3_BTN_SYNC_DST,
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
        hfont_bold,
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
    let t3_guide_edit =
        make_readonly_edit(hinst, hwnd_main, ID_T3_GUIDE_EDIT, true, hfont_mono, dpi);
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
        t3_btn_sync_dst,
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
        theme,
        icon_large,
        icon_small,
        hwnd_tab,
        hwnd_status,
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
        t1_chk_show_pass,
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
        t1_btn_save_log,
        t2_controls,
        t2_guide,
        t2_grp_dst,
        t2_radio_local,
        t2_dst_local_lbl,
        t2_dst_combo,
        t2_radio_unc,
        t2_dst_unc_lbl,
        t2_dst_unc_edit,
        t2_btn_browse,
        t2_cred_user_lbl,
        t2_cred_user_edit,
        t2_cred_pass_lbl,
        t2_cred_pass_edit,
        t2_chk_show_pass,
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
        t3_btn_sync_dst,
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
        validation_level: ValidationLevel::Ok,
        status_tone: StatusTone::Idle,
        backup_started_at: None,
        is_marquee: false,
        worker: None,
        event_queue: Arc::new(Mutex::new(VecDeque::new())),
    }
}

fn insert_lv_col(lv: HWND, idx: i32, text: &str, width: i32, fmt: i32) {
    let mut txt_w = to_wide_null(text);
    let col = LVCOLUMNW {
        mask: LVCF_FMT | LVCF_WIDTH | LVCF_TEXT,
        fmt,
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

fn init_drive_list_columns(lv: HWND, dpi: u32) {
    insert_lv_col(lv, 0, "ドライブ", scale(86, dpi), LVCFMT_LEFT);
    insert_lv_col(lv, 1, "ボリューム名", scale(145, dpi), LVCFMT_LEFT);
    insert_lv_col(lv, 2, "形式", scale(68, dpi), LVCFMT_LEFT);
    insert_lv_col(lv, 3, "空き / 合計", scale(190, dpi), LVCFMT_RIGHT);
    insert_lv_col(lv, 4, "種別", scale(160, dpi), LVCFMT_LEFT);
}

fn init_version_list_columns(lv: HWND, dpi: u32) {
    insert_lv_col(lv, 0, "バックアップ日時", scale(165, dpi), LVCFMT_LEFT);
    insert_lv_col(lv, 1, "バージョン識別子", scale(155, dpi), LVCFMT_LEFT);
    insert_lv_col(lv, 2, "保存先", scale(195, dpi), LVCFMT_LEFT);
    insert_lv_col(lv, 3, "復元可能項目", scale(220, dpi), LVCFMT_LEFT);
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
        let backup_running = st
            .worker
            .as_ref()
            .is_some_and(|w| w.kind() == WorkerKind::Backup);
        let target_focus = match new_tab {
            0 => {
                if backup_running {
                    st.t1_btn_stop
                } else {
                    st.t1_src_list
                }
            }
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

fn set_lv_selected(lv: HWND, item: i32) {
    let lvi = LVITEMW {
        mask: LVIF_STATE,
        iItem: item,
        state: LVIS_FOCUSED | LVIS_SELECTED,
        stateMask: LVIS_FOCUSED | LVIS_SELECTED,
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

fn selected_combo_drive_letter(combo: HWND, drives: &[DriveInfo]) -> Option<char> {
    let sel = unsafe { SendMessageW(combo, CB_GETCURSEL, 0, 0) };
    if sel >= 0 && (sel as usize) < drives.len() {
        Some(drives[sel as usize].letter)
    } else {
        None
    }
}

fn select_combo_by_letter(combo: HWND, drives: &[DriveInfo], letter: Option<char>) {
    if drives.is_empty() {
        return;
    }
    let idx = letter
        .and_then(|c| {
            drives
                .iter()
                .position(|d| d.letter.eq_ignore_ascii_case(&c))
        })
        .or_else(|| drives.iter().position(DriveInfo::is_ntfs))
        .unwrap_or(0);
    unsafe {
        SendMessageW(combo, CB_SETCURSEL, idx, 0);
    }
}

fn refresh_drives(st: &mut AppState) {
    let is_rescan = !st.all_drives.is_empty();
    let prev_checked = if is_rescan {
        selected_includes(st)
    } else {
        Vec::new()
    };
    let prev_t1_dst = if is_rescan {
        selected_combo_drive_letter(st.t1_dst_combo, &st.target_drives)
    } else {
        None
    };
    let prev_t2_dst = if is_rescan {
        selected_combo_drive_letter(st.t2_dst_combo, &st.target_drives)
    } else {
        None
    };

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
        let free_str = format_gib(d.free_bytes);
        let total_str = format_gib(d.total_bytes);
        let kind_str = d.kind_label();
        set_lv_text(st.t1_src_list, idx as i32, 1, label);
        set_lv_text(st.t1_src_list, idx as i32, 2, &d.fs_name);
        set_lv_text(
            st.t1_src_list,
            idx as i32,
            3,
            &format!("{free_str} / {total_str}"),
        );
        set_lv_text(st.t1_src_list, idx as i32, 4, &kind_str);

        let should_check = if is_rescan {
            prev_checked
                .iter()
                .any(|c| c.eq_ignore_ascii_case(&d.letter))
        } else {
            d.is_system
        };
        set_lv_checkbox(st.t1_src_list, idx as i32, should_check);
    }

    if !st.all_drives.is_empty() {
        set_lv_selected(st.t1_src_list, 0);
    }

    for d in &st.target_drives {
        let item_w = to_wide_null(&d.format_display());
        unsafe {
            SendMessageW(st.t1_dst_combo, CB_ADDSTRING, 0, item_w.as_ptr() as LPARAM);
            SendMessageW(st.t2_dst_combo, CB_ADDSTRING, 0, item_w.as_ptr() as LPARAM);
        }
    }

    select_combo_by_letter(st.t1_dst_combo, &st.target_drives, prev_t1_dst);
    select_combo_by_letter(st.t2_dst_combo, &st.target_drives, prev_t2_dst);

    let count = st.all_drives.len();
    let msg = if is_rescan {
        format!("ドライブ一覧を再スキャンしました（{count} 台検出）")
    } else {
        format!("固定ドライブを {count} 台検出しました")
    };
    set_status_bar(st.hwnd_status, Some(&msg), None);
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
) -> Result<BackupTarget, ValidationError> {
    if is_checked_radio(radio_local) {
        let sel = unsafe { SendMessageW(dst_combo, CB_GETCURSEL, 0, 0) };
        if sel < 0 || (sel as usize) >= st.target_drives.len() {
            return Err(ValidationError::EmptyTarget);
        }
        Ok(BackupTarget::LocalDrive(
            st.target_drives[sel as usize].letter,
        ))
    } else {
        let raw_path = get_window_text(dst_unc_edit);
        let path = validate_unc_path(&raw_path)?;
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
        )?;
        Ok(BackupTarget::NetworkUnc {
            path,
            user,
            password,
        })
    }
}

fn read_tab1_target(st: &AppState) -> Result<BackupTarget, ValidationError> {
    read_target_controls(
        st,
        st.t1_radio_local,
        st.t1_dst_combo,
        st.t1_dst_unc_edit,
        st.t1_cred_user_edit,
        st.t1_cred_pass_edit,
    )
}

fn read_tab2_target(st: &AppState) -> Result<BackupTarget, ValidationError> {
    read_target_controls(
        st,
        st.t2_radio_local,
        st.t2_dst_combo,
        st.t2_dst_unc_edit,
        st.t2_cred_user_edit,
        st.t2_cred_pass_edit,
    )
}

fn focus_target_for_error(st: &AppState, err: &ValidationError) {
    let target_hwnd = match err {
        ValidationError::NoIncludeDrives | ValidationError::InvalidDriveLetter(_) => st.t1_src_list,
        ValidationError::SameDrive(_)
        | ValidationError::SamePhysicalDisk { .. }
        | ValidationError::NotNtfs { .. } => st.t1_dst_combo,
        ValidationError::EmptyTarget => {
            if is_checked_radio(st.t1_radio_local) {
                st.t1_dst_combo
            } else {
                st.t1_dst_unc_edit
            }
        }
        ValidationError::InvalidUncPath(_) => st.t1_dst_unc_edit,
        ValidationError::InvalidCredentials(_) => st.t1_cred_user_edit,
    };
    unsafe {
        SetFocus(target_hwnd);
    }
}

fn toggle_password_visibility(chk: HWND, pass_edit: HWND) {
    let show = is_checked_radio(chk);
    let ch: WPARAM = if show { 0 } else { 0x25CF };
    unsafe {
        SendMessageW(pass_edit, EM_SETPASSWORDCHAR, ch, 0);
        InvalidateRect(pass_edit, std::ptr::null(), TRUE);
    }
}

fn apply_tab1_target_mode(st: &AppState) {
    let local_mode = is_checked_radio(st.t1_radio_local);
    let local_flag = if local_mode { TRUE } else { FALSE };
    let unc_flag = if local_mode { FALSE } else { TRUE };
    unsafe {
        let cur = GetFocus();
        if local_mode
            && (cur == st.t1_dst_unc_edit
                || cur == st.t1_btn_browse
                || cur == st.t1_cred_user_edit
                || cur == st.t1_cred_pass_edit
                || cur == st.t1_chk_show_pass)
        {
            SetFocus(st.t1_radio_local);
        } else if !local_mode && cur == st.t1_dst_combo {
            SetFocus(st.t1_radio_unc);
        }
        EnableWindow(st.t1_dst_combo, local_flag);
        EnableWindow(st.t1_dst_unc_edit, unc_flag);
        EnableWindow(st.t1_btn_browse, unc_flag);
        EnableWindow(st.t1_cred_user_edit, unc_flag);
        EnableWindow(st.t1_cred_pass_edit, unc_flag);
        EnableWindow(st.t1_chk_show_pass, unc_flag);
    }
}

fn apply_tab2_target_mode(st: &AppState) {
    let local_mode = is_checked_radio(st.t2_radio_local);
    let local_flag = if local_mode { TRUE } else { FALSE };
    let unc_flag = if local_mode { FALSE } else { TRUE };
    unsafe {
        let cur = GetFocus();
        if local_mode
            && (cur == st.t2_dst_unc_edit
                || cur == st.t2_btn_browse
                || cur == st.t2_cred_user_edit
                || cur == st.t2_cred_pass_edit
                || cur == st.t2_chk_show_pass)
        {
            SetFocus(st.t2_radio_local);
        } else if !local_mode && cur == st.t2_dst_combo {
            SetFocus(st.t2_radio_unc);
        }
        EnableWindow(st.t2_dst_combo, local_flag);
        EnableWindow(st.t2_dst_unc_edit, unc_flag);
        EnableWindow(st.t2_btn_browse, unc_flag);
        EnableWindow(st.t2_cred_user_edit, unc_flag);
        EnableWindow(st.t2_cred_pass_edit, unc_flag);
        EnableWindow(st.t2_chk_show_pass, unc_flag);
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

fn update_tab1_preview_and_validation(st: &mut AppState) {
    let includes = selected_includes(st);
    match read_tab1_target(st) {
        Ok(target) => {
            if includes.is_empty() {
                set_window_text(st.t1_cmd_edit, "");
                st.validation_level = ValidationLevel::Error;
                set_window_text(
                    st.t1_warn_lbl,
                    " 【エラー】バックアップ対象のドライブを1つ以上選択してください。",
                );
                unsafe {
                    InvalidateRect(st.t1_warn_lbl, std::ptr::null(), TRUE);
                }
                return;
            }
            let preview = build_command_preview(&includes, &target);
            set_window_text(st.t1_cmd_edit, &preview);

            match evaluate_backup_plan(&includes, &target, &st.all_drives) {
                Ok(PlanCheckOutcome::Ok {
                    estimated_used_bytes,
                    target_free_bytes,
                }) => {
                    st.validation_level = ValidationLevel::Ok;
                    let used_s = format_gib(estimated_used_bytes);
                    let msg = match target_free_bytes {
                        Some(free) => format!(
                            " 事前チェック: 問題ありません。（対象使用量目安: {used_s} / 保存先空き: {}）",
                            format_gib(free)
                        ),
                        None => {
                            format!(" 事前チェック: 問題ありません。（対象使用量目安: {used_s}）")
                        }
                    };
                    set_window_text(st.t1_warn_lbl, &msg);
                }
                Ok(PlanCheckOutcome::LowSpaceWarning {
                    estimated_used_bytes,
                    target_free_bytes,
                    target_drive,
                }) => {
                    st.validation_level = ValidationLevel::Warn;
                    let msg = format!(
                        " 【注意】保存先({target_drive}:)の空き容量（{}）が対象ドライブの使用量目安（{}）を下回っています。\n 空き容量不足でバックアップに失敗するおそれがあります。",
                        format_gib(target_free_bytes),
                        format_gib(estimated_used_bytes)
                    );
                    set_window_text(st.t1_warn_lbl, &msg);
                }
                Err(e) => {
                    st.validation_level = ValidationLevel::Error;
                    set_window_text(st.t1_warn_lbl, &format!(" 【エラー】{e}"));
                }
            }
        }
        Err(err) => {
            set_window_text(st.t1_cmd_edit, "");
            st.validation_level = ValidationLevel::Error;
            set_window_text(st.t1_warn_lbl, &format!(" 【エラー】{err}"));
        }
    }
    unsafe {
        InvalidateRect(st.t1_warn_lbl, std::ptr::null(), TRUE);
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

fn set_progress_marquee(st: &mut AppState, enable: bool) {
    if st.is_marquee == enable {
        return;
    }
    unsafe {
        let style = GetWindowLongPtrW(st.t1_progress, GWL_STYLE) as u32;
        if enable {
            SetWindowLongPtrW(st.t1_progress, GWL_STYLE, (style | PBS_MARQUEE) as isize);
            SendMessageW(st.t1_progress, PBM_SETSTATE, PBST_NORMAL, 0);
            SendMessageW(st.t1_progress, PBM_SETMARQUEE, 1, 40);
        } else {
            SendMessageW(st.t1_progress, PBM_SETMARQUEE, 0, 0);
            SetWindowLongPtrW(st.t1_progress, GWL_STYLE, (style & !PBS_MARQUEE) as isize);
        }
    }
    st.is_marquee = enable;
}

fn set_status_text_and_tone(st: &mut AppState, val_text: &str, tone: StatusTone, sb_right: &str) {
    st.status_tone = tone;
    set_window_text(st.t1_status_val, val_text);
    set_status_bar(st.hwnd_status, None, Some(sb_right));
    unsafe {
        InvalidateRect(st.t1_status_val, std::ptr::null(), TRUE);
    }
}

fn update_running_backup_indicators(st: &AppState) {
    let Some(started) = st.backup_started_at else {
        return;
    };
    if !st
        .worker
        .as_ref()
        .is_some_and(|w| w.kind() == WorkerKind::Backup)
    {
        return;
    }
    let elapsed = format_elapsed(started.elapsed().as_secs());
    let (status_txt, title_prefix) = if st.is_stopping {
        (
            format!("中断処理中... (経過 {elapsed})"),
            "[中断中]".to_string(),
        )
    } else if let Some(pct) = st.last_status_percent {
        (
            format!("処理中: {pct}% 完了 (経過 {elapsed})"),
            format!("[{pct}%]"),
        )
    } else {
        (
            format!("準備中... (経過 {elapsed})"),
            "[準備中]".to_string(),
        )
    };
    set_window_text(st.t1_status_val, &status_txt);
    set_status_bar(st.hwnd_status, None, Some(&status_txt));
    set_window_text(st.hwnd_main, &format!("{title_prefix} {APP_TITLE_BASE}"));
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
                st.t1_chk_show_pass,
            ];
            if will_disable.contains(&cur_focus) {
                SetFocus(st.t1_btn_stop);
            }
            for h in will_disable {
                EnableWindow(h, FALSE);
            }
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

fn apply_selected_version_to_guide(st: &mut AppState) {
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
        set_status_bar(
            st.hwnd_status,
            Some(&format!("バージョン「{vid}」を復元手順へ反映しました")),
            None,
        );
        dialogs::show_info(
            st.hwnd_main,
            "反映完了",
            "選択した保存先とバージョン識別子を「復元手順・印刷」タブに反映しました。",
        );
    }
}

fn start_fetch_versions(st: &mut AppState) {
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
            dialogs::show_error(st.hwnd_main, "入力エラー", &e.to_string());
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
        EnableWindow(st.t1_btn_start, FALSE);
    }
    set_window_text(st.t2_btn_fetch, "取得中...");
    st.versions.clear();
    set_window_text(st.t2_out_edit, "既存バックアップを検索しています...\r\n");
    set_status_bar(st.hwnd_status, Some("既存バックアップを検索中..."), None);

    let args = build_get_versions_args(&target);
    st.worker = Some(spawn_wbadmin(
        st.hwnd_main,
        WorkerKind::GetVersions,
        args,
        Arc::clone(&st.event_queue),
    ));
}

fn save_restore_guide(st: &AppState) {
    let content = get_window_text(st.t3_guide_edit);
    match save_text_with_dialog(
        st.hwnd_main,
        "復元手順をテキストファイルに保存",
        "WindowsImageBackup_復元手順.txt",
        &content,
    ) {
        Ok(Some(path)) => {
            set_status_bar(
                st.hwnd_status,
                Some(&format!("復元手順を保存しました: {}", path.display())),
                None,
            );
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

fn print_restore_guide(st: &AppState) {
    let content = get_window_text(st.t3_guide_edit);
    match print_text_document(st.hwnd_main, "WindowsImageBackup 復元手順ガイド", &content) {
        Ok(true) => {
            set_status_bar(
                st.hwnd_status,
                Some("復元手順をプリンタへ送信しました"),
                None,
            );
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

fn sync_tab1_target_to_tab3(st: &AppState) {
    let target_text = match read_tab1_target(st) {
        Ok(t) => t.as_wbadmin_target(),
        Err(_) => {
            if is_checked_radio(st.t1_radio_local) {
                selected_combo_drive_letter(st.t1_dst_combo, &st.target_drives)
                    .map(|c| format!("{c}:"))
                    .unwrap_or_default()
            } else {
                get_window_text(st.t1_dst_unc_edit).trim().to_string()
            }
        }
    };
    if !target_text.is_empty() {
        set_window_text(st.t3_target_edit, &target_text);
        update_tab3_guide(st);
        set_status_bar(
            st.hwnd_status,
            Some(&format!(
                "バックアップタブの保存先（{target_text}）を反映しました"
            )),
            None,
        );
    } else {
        dialogs::show_warning(
            st.hwnd_main,
            "反映不可",
            "バックアップ作成タブで保存先が指定されていません。",
        );
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
        ID_T1_CHK_SHOW_PASS => {
            toggle_password_visibility(st.t1_chk_show_pass, st.t1_cred_pass_edit);
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
                set_status_bar(st.hwnd_status, Some("実行コマンドをコピーしました"), None);
                dialogs::show_info(
                    st.hwnd_main,
                    "コピー完了",
                    "実行コマンドをクリップボードにコピーしました。",
                );
            }
        }
        ID_T1_BTN_CLEAR_LOG => {
            set_window_text(st.t1_log_edit, "");
            set_status_bar(st.hwnd_status, Some("実行ログをクリアしました"), None);
        }
        ID_T1_BTN_SAVE_LOG => {
            let content = get_window_text(st.t1_log_edit);
            if content.trim().is_empty() {
                set_status_bar(st.hwnd_status, Some("保存できるログがありません"), None);
                dialogs::show_warning(st.hwnd_main, "保存不可", "保存できる実行ログがありません。");
                return;
            }
            match save_text_with_dialog(
                st.hwnd_main,
                "実行ログをテキストファイルに保存",
                "WIB-GUI_実行ログ.txt",
                &content,
            ) {
                Ok(Some(path)) => {
                    set_status_bar(
                        st.hwnd_status,
                        Some(&format!("実行ログを保存しました: {}", path.display())),
                        None,
                    );
                    dialogs::show_info(
                        st.hwnd_main,
                        "保存完了",
                        &format!("実行ログを保存しました:\n{}", path.display()),
                    );
                }
                Ok(None) => {}
                Err(e) => dialogs::show_error(st.hwnd_main, "保存エラー", &e),
            }
        }
        ID_T1_BTN_START => {
            if st.worker.is_some() {
                return;
            }
            let includes = selected_includes(st);
            let target = match read_tab1_target(st) {
                Ok(t) => t,
                Err(err) => {
                    dialogs::show_error(st.hwnd_main, "入力エラー", &err.to_string());
                    focus_target_for_error(st, &err);
                    return;
                }
            };
            let outcome = match evaluate_backup_plan(&includes, &target, &st.all_drives) {
                Ok(o) => o,
                Err(err) => {
                    dialogs::show_error(st.hwnd_main, "設定エラー", &err.to_string());
                    focus_target_for_error(st, &err);
                    return;
                }
            };
            let low_space_msg = match outcome {
                PlanCheckOutcome::LowSpaceWarning {
                    estimated_used_bytes,
                    target_free_bytes,
                    target_drive,
                } => Some(format!(
                    "保存先ドライブ({target_drive}:)の空き容量（{}）が対象ドライブの使用量目安（{}）を下回っています。\n空き容量不足でバックアップに失敗するおそれがあります。",
                    format_gib(target_free_bytes),
                    format_gib(estimated_used_bytes)
                )),
                PlanCheckOutcome::Ok { .. } => None,
            };

            let inc_disp: Vec<String> = includes.iter().map(|c| format!("{c}:")).collect();
            let preview = build_command_preview(&includes, &target);
            if !dialogs::confirm_backup_start(
                st.hwnd_main,
                &inc_disp.join(", "),
                &target.as_wbadmin_target(),
                &preview,
                low_space_msg.as_deref(),
            ) {
                return;
            }

            set_window_text(st.t1_log_edit, "");
            st.last_status_percent = None;
            st.is_stopping = false;
            st.backup_started_at = Some(Instant::now());
            unsafe {
                SendMessageW(st.t1_progress, PBM_SETPOS, 0, 0);
            }
            set_progress_marquee(st, true);
            set_status_text_and_tone(
                st,
                "準備中... (経過 00:00)",
                StatusTone::Running,
                "準備中... (経過 00:00)",
            );
            set_status_bar(st.hwnd_status, Some("バックアップを実行中..."), None);
            set_window_text(st.hwnd_main, &format!("[準備中] {APP_TITLE_BASE}"));
            append_edit_line(st.t1_log_edit, &format!("実行: {preview}"));

            let args = build_start_backup_args(&includes, &target);
            st.worker = Some(spawn_wbadmin(
                st.hwnd_main,
                WorkerKind::Backup,
                args,
                Arc::clone(&st.event_queue),
            ));
            unsafe {
                SetTimer(st.hwnd_main, TIMER_ID_BACKUP, 1000, std::ptr::null());
            }
            set_backup_running_ui(st, true);
        }
        ID_T1_BTN_STOP => {
            if st
                .worker
                .as_ref()
                .is_some_and(|w| w.kind() == WorkerKind::Backup)
                && dialogs::confirm_backup_stop(st.hwnd_main)
            {
                st.is_stopping = true;
                let elapsed = st
                    .backup_started_at
                    .map(|t| format_elapsed(t.elapsed().as_secs()))
                    .unwrap_or_else(|| "00:00".to_string());
                let txt = format!("中断処理中... (経過 {elapsed})");
                set_status_text_and_tone(st, &txt, StatusTone::Warn, &txt);
                set_status_bar(st.hwnd_status, Some("バックアップの中断を要求中..."), None);
                unsafe {
                    SendMessageW(st.t1_progress, PBM_SETSTATE, PBST_PAUSED, 0);
                }
                append_edit_line(
                    st.t1_log_edit,
                    "安全のため wbadmin stop job -quiet で中断を要求しています...",
                );
                if let Some(ref w) = st.worker {
                    w.request_stop();
                }
                unsafe {
                    SetFocus(st.t1_log_edit);
                    EnableWindow(st.t1_btn_stop, FALSE);
                }
            }
        }
        ID_T2_RADIO_LOCAL | ID_T2_RADIO_UNC => {
            apply_tab2_target_mode(st);
        }
        ID_T2_CHK_SHOW_PASS => {
            toggle_password_visibility(st.t2_chk_show_pass, st.t2_cred_pass_edit);
        }
        ID_T2_BTN_BROWSE => {
            if let Some(path) =
                browse_folder_dialog(st.hwnd_main, "確認するネットワーク共有フォルダを選択")
            {
                set_window_text(st.t2_dst_unc_edit, &path);
            }
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
            set_status_bar(
                st.hwnd_status,
                Some("バックアップタブの保存先設定を反映しました"),
                None,
            );
        }
        ID_T2_BTN_FETCH => {
            start_fetch_versions(st);
        }
        ID_T2_BTN_COPY_VER => {
            if let Some(idx) = selected_version_index(st) {
                let vid = &st.versions[idx].version_id;
                if let Err(e) = copy_to_clipboard(st.hwnd_main, vid) {
                    dialogs::show_error(st.hwnd_main, "エラー", &e);
                } else {
                    set_status_bar(
                        st.hwnd_status,
                        Some(&format!("バージョン識別子「{vid}」をコピーしました")),
                        None,
                    );
                    dialogs::show_info(
                        st.hwnd_main,
                        "コピー完了",
                        &format!("バージョン識別子「{vid}」をコピーしました。"),
                    );
                }
            }
        }
        ID_T2_BTN_APPLY_GUIDE => {
            apply_selected_version_to_guide(st);
        }
        ID_T3_TARGET_EDIT | ID_T3_VER_EDIT if code == EN_CHANGE => {
            update_tab3_guide(st);
        }
        ID_T3_BTN_SYNC_DST => {
            sync_tab1_target_to_tab3(st);
        }
        ID_T3_BTN_SAVE => {
            save_restore_guide(st);
        }
        ID_T3_BTN_PRINT => {
            print_restore_guide(st);
        }
        ID_T3_BTN_COPY => {
            let content = get_window_text(st.t3_guide_edit);
            if let Err(e) = copy_to_clipboard(st.hwnd_main, &content) {
                dialogs::show_error(st.hwnd_main, "エラー", &e);
            } else {
                set_status_bar(st.hwnd_status, Some("復元手順の全文をコピーしました"), None);
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
    } else if nmhdr.hwndFrom == st.t2_ver_list {
        if nmhdr.code == LVN_ITEMCHANGED {
            let has_sel = selected_version_index(st).is_some();
            unsafe {
                if !has_sel
                    && (GetFocus() == st.t2_btn_copy_ver || GetFocus() == st.t2_btn_apply_guide)
                {
                    SetFocus(st.t2_ver_list);
                }
                EnableWindow(st.t2_btn_copy_ver, if has_sel { TRUE } else { FALSE });
                EnableWindow(st.t2_btn_apply_guide, if has_sel { TRUE } else { FALSE });
            }
        } else if nmhdr.code == NM_DBLCLK {
            apply_selected_version_to_guide(st);
        } else if nmhdr.code == LVN_KEYDOWN {
            let kd_ptr = lparam as *const NMLVKEYDOWN;
            let vkey = unsafe { std::ptr::addr_of!((*kd_ptr).wVKey).read_unaligned() };
            if vkey == VK_RETURN {
                apply_selected_version_to_guide(st);
            }
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
                set_progress_marquee(st, false);
                unsafe {
                    SendMessageW(st.t1_progress, PBM_SETPOS, percent as usize, 0);
                }
                if st.last_status_percent != Some(percent) {
                    st.last_status_percent = Some(percent);
                    if !st.is_stopping {
                        let elapsed = st
                            .backup_started_at
                            .map(|t| format_elapsed(t.elapsed().as_secs()))
                            .unwrap_or_else(|| "00:00".to_string());
                        let txt = format!("処理中: {percent}% 完了 (経過 {elapsed})");
                        set_status_text_and_tone(st, &txt, StatusTone::Running, &txt);
                        set_window_text(st.hwnd_main, &format!("[{percent}%] {APP_TITLE_BASE}"));
                    }
                }
            }
            WorkerEvent::Error { kind, message } => {
                st.worker.take();
                match kind {
                    WorkerKind::Backup => {
                        unsafe {
                            KillTimer(st.hwnd_main, TIMER_ID_BACKUP);
                        }
                        st.backup_started_at = None;
                        set_progress_marquee(st, false);
                        unsafe {
                            SendMessageW(st.t1_progress, PBM_SETSTATE, PBST_ERROR, 0);
                        }
                        set_backup_running_ui(st, false);
                        set_window_text(st.hwnd_main, APP_TITLE_BASE);
                        set_status_text_and_tone(
                            st,
                            "起動エラー",
                            StatusTone::Danger,
                            "状態: 起動エラー",
                        );
                        set_status_bar(
                            st.hwnd_status,
                            Some("wbadmin.exe の起動に失敗しました"),
                            None,
                        );
                        append_edit_line(st.t1_log_edit, &message);
                        dialogs::show_error(st.hwnd_main, "実行エラー", &message);
                    }
                    WorkerKind::GetVersions => {
                        set_window_text(st.t2_btn_fetch, "一覧を取得(&F)");
                        unsafe {
                            EnableWindow(st.t2_btn_fetch, TRUE);
                            EnableWindow(st.t1_btn_start, TRUE);
                        }
                        set_status_bar(
                            st.hwnd_status,
                            Some("既存バックアップの取得に失敗しました"),
                            None,
                        );
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
                        unsafe {
                            KillTimer(st.hwnd_main, TIMER_ID_BACKUP);
                        }
                        let elapsed = st
                            .backup_started_at
                            .take()
                            .map(|t| format_elapsed(t.elapsed().as_secs()))
                            .unwrap_or_else(|| "00:00".to_string());
                        set_progress_marquee(st, false);
                        set_backup_running_ui(st, false);
                        set_window_text(st.hwnd_main, APP_TITLE_BASE);

                        let code_val = exit_code.unwrap_or(-1);
                        if st.is_stopping {
                            unsafe {
                                SendMessageW(st.t1_progress, PBM_SETSTATE, PBST_PAUSED, 0);
                            }
                            let txt = format!("中断されました (経過 {elapsed})");
                            set_status_text_and_tone(st, &txt, StatusTone::Warn, &txt);
                            set_status_bar(
                                st.hwnd_status,
                                Some("バックアップが中断されました"),
                                None,
                            );
                            append_edit_line(
                                st.t1_log_edit,
                                &format!(
                                    "バックアップが中断されました（終了コード: {code_val}）。"
                                ),
                            );
                        } else if code_val == 0 {
                            unsafe {
                                SendMessageW(st.t1_progress, PBM_SETSTATE, PBST_NORMAL, 0);
                                SendMessageW(st.t1_progress, PBM_SETPOS, 100, 0);
                            }
                            let txt = format!("完了しました (所要 {elapsed})");
                            set_status_text_and_tone(st, &txt, StatusTone::Ok, &txt);
                            set_status_bar(
                                st.hwnd_status,
                                Some("バックアップが正常に完了しました"),
                                None,
                            );
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
                                &format!(
                                    "システムイメージの作成が完了しました（所要時間: {elapsed}）。\n「復元手順・印刷」タブから復元手順を印刷またはテキスト保存しておくことをおすすめします。"
                                ),
                            );
                        } else {
                            unsafe {
                                SendMessageW(st.t1_progress, PBM_SETSTATE, PBST_ERROR, 0);
                            }
                            let txt = format!("エラー終了 (コード: {code_val})");
                            set_status_text_and_tone(st, &txt, StatusTone::Danger, &txt);
                            set_status_bar(
                                st.hwnd_status,
                                Some("バックアップがエラーで終了しました"),
                                None,
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
                        set_window_text(st.t2_btn_fetch, "一覧を取得(&F)");
                        unsafe {
                            EnableWindow(st.t2_btn_fetch, TRUE);
                            EnableWindow(st.t1_btn_start, TRUE);
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
                        let count = st.versions.len();
                        if count > 0 {
                            set_lv_selected(st.t2_ver_list, 0);
                            unsafe {
                                EnableWindow(st.t2_btn_copy_ver, TRUE);
                                EnableWindow(st.t2_btn_apply_guide, TRUE);
                                if st.current_tab == 1 {
                                    SetFocus(st.t2_ver_list);
                                }
                            }
                            set_status_bar(
                                st.hwnd_status,
                                Some(&format!("既存バックアップを {count} 件取得しました")),
                                None,
                            );
                        } else {
                            unsafe {
                                if st.current_tab == 1 {
                                    SetFocus(st.t2_btn_fetch);
                                }
                            }
                            set_status_bar(
                                st.hwnd_status,
                                Some("指定された保存先に既存バックアップは見つかりませんでした"),
                                None,
                            );
                        }
                    }
                }
            }
        }
    }
}
