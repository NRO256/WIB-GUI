use crate::platform::win32::{
    GetClientRect, GetWindowRect, HWND, HWND_BOTTOM, LPARAM, LVM_SETCOLUMNWIDTH, RECT, SB_SETPARTS,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SendMessageW, SetWindowPos,
    TCM_ADJUSTRECT, WM_SIZE,
};
use crate::ui::state::AppState;
use crate::ui::theme::scale;

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

fn set_lv_col_width(lv: HWND, col: usize, width: i32) {
    unsafe {
        SendMessageW(lv, LVM_SETCOLUMNWIDTH, col, width.max(20) as LPARAM);
    }
}

fn update_listview_column_widths(st: &AppState, src_list_w: i32, ver_list_w: i32) {
    let dpi = st.dpi;
    let pad_scroll = scale(24, dpi);

    // ドライブ一覧 (t1_src_list)
    let s0 = scale(86, dpi);
    let s1 = scale(145, dpi);
    let s2 = scale(68, dpi);
    let s3 = scale(190, dpi);
    let s4 = (src_list_w - (s0 + s1 + s2 + s3) - pad_scroll).max(scale(140, dpi));
    set_lv_col_width(st.t1_src_list, 0, s0);
    set_lv_col_width(st.t1_src_list, 1, s1);
    set_lv_col_width(st.t1_src_list, 2, s2);
    set_lv_col_width(st.t1_src_list, 3, s3);
    set_lv_col_width(st.t1_src_list, 4, s4);

    // 既存バックアップ一覧 (t2_ver_list)
    let v0 = scale(165, dpi);
    let v1 = scale(155, dpi);
    let v2 = scale(195, dpi);
    let v3 = (ver_list_w - (v0 + v1 + v2) - pad_scroll).max(scale(180, dpi));
    set_lv_col_width(st.t2_ver_list, 0, v0);
    set_lv_col_width(st.t2_ver_list, 1, v1);
    set_lv_col_width(st.t2_ver_list, 2, v2);
    set_lv_col_width(st.t2_ver_list, 3, v3);
}

pub fn layout_controls(st: &AppState) {
    let mut rc = RECT::default();
    unsafe {
        GetClientRect(st.hwnd_main, &raw mut rc);
        SendMessageW(st.hwnd_status, WM_SIZE, 0, 0);
    }

    let mut sb_rc = RECT::default();
    unsafe {
        GetWindowRect(st.hwnd_status, &raw mut sb_rc);
    }
    let dpi = st.dpi;
    let sb_h = (sb_rc.bottom - sb_rc.top).max(scale(22, dpi));

    let cw = (rc.right - rc.left).max(scale(780, dpi));
    let ch = (rc.bottom - rc.top).max(scale(620, dpi));

    let left_part = (cw - scale(230, dpi)).max(cw / 2);
    let parts = [left_part, -1];
    unsafe {
        SendMessageW(st.hwnd_status, SB_SETPARTS, 2, parts.as_ptr() as LPARAM);
    }

    let pad = scale(10, dpi);
    let tab_w = cw - pad * 2;
    let tab_h = (ch - sb_h - pad * 2).max(scale(420, dpi));
    mv(st.hwnd_tab, pad, pad, tab_w, tab_h);
    unsafe {
        SetWindowPos(
            st.hwnd_tab,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE,
        );
    }

    let mut tab_rc = RECT {
        left: pad,
        top: pad,
        right: pad + tab_w,
        bottom: pad + tab_h,
    };
    unsafe {
        SendMessageW(st.hwnd_tab, TCM_ADJUSTRECT, 0, (&raw mut tab_rc) as LPARAM);
    }

    let in_pad = scale(12, dpi);
    let inner_x = tab_rc.left + in_pad;
    let inner_y = tab_rc.top + scale(8, dpi);
    let inner_w = (tab_rc.right - tab_rc.left - in_pad * 2).max(scale(500, dpi));
    let inner_bottom = tab_rc.bottom - in_pad;

    let gp = scale(12, dpi);
    let sx = inner_x + gp;
    let sw = inner_w - gp * 2;
    let row_h = scale(24, dpi);
    let btn_h = scale(28, dpi);
    let gap = scale(6, dpi);
    let lbl_w = scale(135, dpi);
    let cred_lbl_w = scale(136, dpi);
    let show_w = scale(72, dpi);

    let indent_x = sx + scale(18, dpi);
    let indent_w = sw - scale(18, dpi);
    let avail_cred = indent_w - show_w - gap * 2;
    let user_edit_w = ((avail_cred / 2) - cred_lbl_w).max(scale(75, dpi));
    let pass_x = indent_x + cred_lbl_w + user_edit_w + gap;
    let pass_edit_w = (sx + sw - show_w - gap - (pass_x + cred_lbl_w)).max(scale(75, dpi));

    // --- タブ1レイアウト ---
    mv(st.t1_guide, inner_x, inner_y, inner_w, scale(22, dpi));
    let src_y = inner_y + scale(26, dpi);
    let src_h = scale(152, dpi);
    mv(st.t1_grp_src, inner_x, src_y, inner_w, src_h);
    mv(
        st.t1_src_desc,
        sx,
        src_y + scale(18, dpi),
        sw,
        scale(18, dpi),
    );
    mv(
        st.t1_src_lbl,
        sx,
        src_y + scale(38, dpi),
        scale(200, dpi),
        scale(18, dpi),
    );
    let btn_rescan_w = scale(126, dpi);
    let src_list_w = sw - btn_rescan_w - gap;
    mv(
        st.t1_src_list,
        sx,
        src_y + scale(56, dpi),
        src_list_w,
        scale(86, dpi),
    );
    mv(
        st.t1_btn_rescan,
        sx + sw - btn_rescan_w,
        src_y + scale(56, dpi),
        btn_rescan_w,
        btn_h,
    );

    let dst_y = src_y + src_h + gap;
    let dst_h = scale(172, dpi);
    mv(st.t1_grp_dst, inner_x, dst_y, inner_w, dst_h);
    mv(
        st.t1_radio_local,
        sx,
        dst_y + scale(18, dpi),
        sw,
        scale(20, dpi),
    );
    mv(
        st.t1_dst_local_lbl,
        indent_x,
        dst_y + scale(42, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t1_dst_combo,
        indent_x + lbl_w,
        dst_y + scale(42, dpi),
        indent_w - lbl_w,
        scale(200, dpi),
    );
    mv(
        st.t1_radio_unc,
        sx,
        dst_y + scale(70, dpi),
        sw,
        scale(20, dpi),
    );
    let browse_w = scale(95, dpi);
    mv(
        st.t1_dst_unc_lbl,
        indent_x,
        dst_y + scale(94, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t1_dst_unc_edit,
        indent_x + lbl_w,
        dst_y + scale(94, dpi),
        indent_w - lbl_w - browse_w - gap,
        row_h,
    );
    mv(
        st.t1_btn_browse,
        sx + sw - browse_w,
        dst_y + scale(92, dpi),
        browse_w,
        btn_h,
    );

    let cred_y = dst_y + scale(122, dpi);
    mv(st.t1_cred_user_lbl, indent_x, cred_y, cred_lbl_w, row_h);
    mv(
        st.t1_cred_user_edit,
        indent_x + cred_lbl_w,
        cred_y,
        user_edit_w,
        row_h,
    );
    mv(st.t1_cred_pass_lbl, pass_x, cred_y, cred_lbl_w, row_h);
    mv(
        st.t1_cred_pass_edit,
        pass_x + cred_lbl_w,
        cred_y,
        pass_edit_w,
        row_h,
    );
    mv(st.t1_chk_show_pass, sx + sw - show_w, cred_y, show_w, row_h);
    mv(
        st.t1_dst_note,
        sx,
        dst_y + scale(148, dpi),
        sw,
        scale(18, dpi),
    );

    let warn_y = dst_y + dst_h + scale(4, dpi);
    let warn_h = scale(36, dpi);
    mv(st.t1_warn_lbl, inner_x, warn_y, inner_w, warn_h);

    let cmd_y = warn_y + warn_h + scale(4, dpi);
    let cmd_h = scale(84, dpi);
    mv(st.t1_grp_cmd, inner_x, cmd_y, inner_w, cmd_h);
    let cmd_lbl_w = scale(210, dpi);
    mv(st.t1_cmd_lbl, sx, cmd_y + scale(19, dpi), cmd_lbl_w, row_h);
    mv(
        st.t1_cmd_edit,
        sx + cmd_lbl_w,
        cmd_y + scale(19, dpi),
        sw - cmd_lbl_w,
        row_h,
    );
    let b_start_w = scale(180, dpi);
    let b_stop_w = scale(165, dpi);
    let b_copy_w = scale(160, dpi);
    mv(
        st.t1_btn_start,
        sx,
        cmd_y + scale(48, dpi),
        b_start_w,
        btn_h,
    );
    mv(
        st.t1_btn_stop,
        sx + b_start_w + gap,
        cmd_y + scale(48, dpi),
        b_stop_w,
        btn_h,
    );
    mv(
        st.t1_btn_copy_cmd,
        sx + b_start_w + b_stop_w + gap * 2,
        cmd_y + scale(48, dpi),
        b_copy_w,
        btn_h,
    );

    let log_y = cmd_y + cmd_h + gap;
    let log_h = (inner_bottom - log_y).max(scale(128, dpi));
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
        scale(290, dpi),
        row_h,
    );
    mv(
        st.t1_progress,
        sx + scale(344, dpi),
        log_y + scale(22, dpi),
        (sw - scale(344, dpi)).max(scale(120, dpi)),
        scale(20, dpi),
    );
    mv(
        st.t1_log_lbl,
        sx,
        log_y + scale(44, dpi),
        scale(140, dpi),
        scale(18, dpi),
    );
    let clear_w = scale(136, dpi);
    let log_btn_y = log_y + scale(62, dpi);
    let edit_h = (log_h - scale(70, dpi)).max(scale(58, dpi));
    mv(st.t1_log_edit, sx, log_btn_y, sw - clear_w - gap, edit_h);
    mv(
        st.t1_btn_clear_log,
        sx + sw - clear_w,
        log_btn_y,
        clear_w,
        btn_h,
    );
    mv(
        st.t1_btn_save_log,
        sx + sw - clear_w,
        log_btn_y + btn_h + scale(4, dpi),
        clear_w,
        btn_h,
    );

    // --- タブ2レイアウト ---
    mv(st.t2_guide, inner_x, inner_y, inner_w, scale(22, dpi));
    let t2_dst_y = inner_y + scale(26, dpi);
    let t2_dst_h = scale(186, dpi);
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
        indent_x,
        t2_dst_y + scale(45, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t2_dst_combo,
        indent_x + lbl_w,
        t2_dst_y + scale(45, dpi),
        indent_w - lbl_w,
        scale(200, dpi),
    );
    mv(
        st.t2_radio_unc,
        sx,
        t2_dst_y + scale(74, dpi),
        sw,
        scale(20, dpi),
    );
    mv(
        st.t2_dst_unc_lbl,
        indent_x,
        t2_dst_y + scale(99, dpi),
        lbl_w,
        row_h,
    );
    mv(
        st.t2_dst_unc_edit,
        indent_x + lbl_w,
        t2_dst_y + scale(99, dpi),
        indent_w - lbl_w - browse_w - gap,
        row_h,
    );
    mv(
        st.t2_btn_browse,
        sx + sw - browse_w,
        t2_dst_y + scale(97, dpi),
        browse_w,
        btn_h,
    );
    let t2_cred_y = t2_dst_y + scale(127, dpi);
    mv(st.t2_cred_user_lbl, indent_x, t2_cred_y, cred_lbl_w, row_h);
    mv(
        st.t2_cred_user_edit,
        indent_x + cred_lbl_w,
        t2_cred_y,
        user_edit_w,
        row_h,
    );
    mv(st.t2_cred_pass_lbl, pass_x, t2_cred_y, cred_lbl_w, row_h);
    mv(
        st.t2_cred_pass_edit,
        pass_x + cred_lbl_w,
        t2_cred_y,
        pass_edit_w,
        row_h,
    );
    mv(
        st.t2_chk_show_pass,
        sx + sw - show_w,
        t2_cred_y,
        show_w,
        row_h,
    );
    mv(
        st.t2_btn_fetch,
        sx,
        t2_dst_y + scale(154, dpi),
        scale(150, dpi),
        btn_h,
    );
    mv(
        st.t2_btn_sync_dst,
        sx + scale(156, dpi),
        t2_dst_y + scale(154, dpi),
        scale(250, dpi),
        btn_h,
    );

    let t2_list_y = t2_dst_y + t2_dst_h + gap;
    let t2_list_h = scale(224, dpi);
    mv(st.t2_grp_list, inner_x, t2_list_y, inner_w, t2_list_h);
    mv(
        st.t2_list_lbl,
        sx,
        t2_list_y + scale(20, dpi),
        scale(180, dpi),
        scale(18, dpi),
    );
    let ver_list_w = sw;
    mv(
        st.t2_ver_list,
        sx,
        t2_list_y + scale(40, dpi),
        ver_list_w,
        scale(140, dpi),
    );
    mv(
        st.t2_btn_copy_ver,
        sx,
        t2_list_y + scale(188, dpi),
        scale(215, dpi),
        btn_h,
    );
    mv(
        st.t2_btn_apply_guide,
        sx + scale(221, dpi),
        t2_list_y + scale(188, dpi),
        scale(235, dpi),
        btn_h,
    );

    let t2_out_y = t2_list_y + t2_list_h + gap;
    let t2_out_h = (inner_bottom - t2_out_y).max(scale(110, dpi));
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
        (t2_out_h - scale(50, dpi)).max(scale(45, dpi)),
    );

    // --- タブ3レイアウト ---
    mv(st.t3_banner, inner_x, inner_y, inner_w, scale(44, dpi));
    let t3_info_y = inner_y + scale(48, dpi);
    let t3_info_h = scale(62, dpi);
    mv(st.t3_grp_info, inner_x, t3_info_y, inner_w, t3_info_h);
    let t3_lbl1_w = scale(65, dpi);
    let t3_ed1_w = scale(185, dpi);
    let t3_lbl2_w = scale(130, dpi);
    let t3_sync_w = scale(140, dpi);
    let t3_info_ctrl_y = t3_info_y + scale(23, dpi);
    mv(st.t3_target_lbl, sx, t3_info_ctrl_y, t3_lbl1_w, row_h);
    mv(
        st.t3_target_edit,
        sx + t3_lbl1_w,
        t3_info_ctrl_y,
        t3_ed1_w,
        row_h,
    );
    let ver_x = sx + t3_lbl1_w + t3_ed1_w + pad;
    mv(st.t3_ver_lbl, ver_x, t3_info_ctrl_y, t3_lbl2_w, row_h);
    let ver_edit_w = (sw - (ver_x - sx) - t3_lbl2_w - t3_sync_w - gap).max(scale(110, dpi));
    mv(
        st.t3_ver_edit,
        ver_x + t3_lbl2_w,
        t3_info_ctrl_y,
        ver_edit_w,
        row_h,
    );
    mv(
        st.t3_btn_sync_dst,
        sx + sw - t3_sync_w,
        t3_info_y + scale(21, dpi),
        t3_sync_w,
        btn_h,
    );

    let t3_btn_y = inner_bottom - btn_h;
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
        (t3_guide_h - scale(50, dpi)).max(scale(100, dpi)),
    );

    mv(st.t3_btn_save, inner_x, t3_btn_y, scale(220, dpi), btn_h);
    mv(
        st.t3_btn_print,
        inner_x + scale(226, dpi),
        t3_btn_y,
        scale(125, dpi),
        btn_h,
    );
    mv(
        st.t3_btn_copy,
        inner_x + scale(357, dpi),
        t3_btn_y,
        scale(190, dpi),
        btn_h,
    );

    update_listview_column_widths(st, src_list_w, ver_list_w);
}
