use crate::domain::validation::DriveInfo;
use crate::domain::wbadmin::BackupVersion;
use crate::platform::process::{WorkerEvent, WorkerHandle};
use crate::platform::win32::{DeleteObject, HFONT, HWND};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub hwnd_main: HWND,
    pub dpi: u32,
    pub hfont_normal: HFONT,
    pub hfont_bold: HFONT,
    pub hfont_mono: HFONT,

    pub hwnd_tab: HWND,
    pub current_tab: usize,

    // タブ1: バックアップ作成
    pub t1_controls: Vec<HWND>,
    pub t1_guide: HWND,
    pub t1_grp_src: HWND,
    pub t1_src_desc: HWND,
    pub t1_src_lbl: HWND,
    pub t1_src_list: HWND,
    pub t1_btn_rescan: HWND,

    pub t1_grp_dst: HWND,
    pub t1_radio_local: HWND,
    pub t1_dst_local_lbl: HWND,
    pub t1_dst_combo: HWND,
    pub t1_radio_unc: HWND,
    pub t1_dst_unc_lbl: HWND,
    pub t1_dst_unc_edit: HWND,
    pub t1_btn_browse: HWND,
    pub t1_cred_user_lbl: HWND,
    pub t1_cred_user_edit: HWND,
    pub t1_cred_pass_lbl: HWND,
    pub t1_cred_pass_edit: HWND,
    pub t1_dst_note: HWND,
    pub t1_warn_lbl: HWND,

    pub t1_grp_cmd: HWND,
    pub t1_cmd_lbl: HWND,
    pub t1_cmd_edit: HWND,
    pub t1_btn_start: HWND,
    pub t1_btn_stop: HWND,
    pub t1_btn_copy_cmd: HWND,

    pub t1_grp_log: HWND,
    pub t1_status_lbl: HWND,
    pub t1_status_val: HWND,
    pub t1_progress: HWND,
    pub t1_log_lbl: HWND,
    pub t1_log_edit: HWND,
    pub t1_btn_clear_log: HWND,

    // タブ2: 既存バックアップ確認
    pub t2_controls: Vec<HWND>,
    pub t2_guide: HWND,
    pub t2_grp_dst: HWND,
    pub t2_radio_local: HWND,
    pub t2_dst_local_lbl: HWND,
    pub t2_dst_combo: HWND,
    pub t2_radio_unc: HWND,
    pub t2_dst_unc_lbl: HWND,
    pub t2_dst_unc_edit: HWND,
    pub t2_cred_user_lbl: HWND,
    pub t2_cred_user_edit: HWND,
    pub t2_cred_pass_lbl: HWND,
    pub t2_cred_pass_edit: HWND,
    pub t2_btn_fetch: HWND,
    pub t2_btn_sync_dst: HWND,

    pub t2_grp_list: HWND,
    pub t2_list_lbl: HWND,
    pub t2_ver_list: HWND,
    pub t2_btn_copy_ver: HWND,
    pub t2_btn_apply_guide: HWND,

    pub t2_grp_out: HWND,
    pub t2_out_lbl: HWND,
    pub t2_out_edit: HWND,

    // タブ3: 復元手順・印刷
    pub t3_controls: Vec<HWND>,
    pub t3_banner: HWND,
    pub t3_grp_info: HWND,
    pub t3_target_lbl: HWND,
    pub t3_target_edit: HWND,
    pub t3_ver_lbl: HWND,
    pub t3_ver_edit: HWND,
    pub t3_grp_guide: HWND,
    pub t3_guide_lbl: HWND,
    pub t3_guide_edit: HWND,
    pub t3_btn_save: HWND,
    pub t3_btn_print: HWND,
    pub t3_btn_copy: HWND,

    // データ
    pub all_drives: Vec<DriveInfo>,
    pub target_drives: Vec<DriveInfo>,
    pub versions: Vec<BackupVersion>,
    pub last_status_percent: Option<u8>,
    pub is_stopping: bool,

    pub worker: Option<WorkerHandle>,
    pub event_queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
}

impl AppState {
    pub fn release_fonts(&mut self) {
        unsafe {
            if !self.hfont_normal.is_null() {
                DeleteObject(self.hfont_normal);
                self.hfont_normal = std::ptr::null_mut();
            }
            if !self.hfont_bold.is_null() {
                DeleteObject(self.hfont_bold);
                self.hfont_bold = std::ptr::null_mut();
            }
            if !self.hfont_mono.is_null() {
                DeleteObject(self.hfont_mono);
                self.hfont_mono = std::ptr::null_mut();
            }
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.worker.take();
        self.release_fonts();
    }
}
