use crate::platform::win32::{
    HWND, IDYES, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONWARNING,
    MB_OK, MB_YESNO, MessageBoxW, to_wide_null,
};

fn message_box(hwnd: HWND, title: &str, message: &str, flags: u32) -> i32 {
    let t = to_wide_null(title);
    let m = to_wide_null(message);
    unsafe { MessageBoxW(hwnd, m.as_ptr(), t.as_ptr(), flags) }
}

pub fn show_info(hwnd: HWND, title: &str, message: &str) {
    message_box(hwnd, title, message, MB_OK | MB_ICONINFORMATION);
}

pub fn show_warning(hwnd: HWND, title: &str, message: &str) {
    message_box(hwnd, title, message, MB_OK | MB_ICONWARNING);
}

pub fn show_error(hwnd: HWND, title: &str, message: &str) {
    message_box(hwnd, title, message, MB_OK | MB_ICONERROR);
}

pub fn confirm_backup_start(
    hwnd: HWND,
    includes_disp: &str,
    target_disp: &str,
    command_preview: &str,
    low_space_warning: Option<&str>,
) -> bool {
    let warn_block = match low_space_warning {
        Some(w) => format!("【容量に関する警告】\n{w}\n\n"),
        None => String::new(),
    };
    let msg = format!(
        "{warn_block}以下の内容でシステムイメージのバックアップを開始します。\n\n\
         ・対象ドライブ: {includes_disp}\n\
         ・保存先: {target_disp}\n\
         ・実行コマンド: {command_preview}\n\n\
         【ご注意】\n\
         ・バックアップには数十分〜数時間かかる場合があります。\n\
         ・保存先直下に「WindowsImageBackup」フォルダが作成されます。空き容量に応じて古い世代が上書き・削除される場合があります。\n\
         ・実行中はPCの電源を切ったり、保存先ドライブを取り外したりしないでください。\n\n\
         バックアップを開始してもよろしいですか？"
    );
    message_box(
        hwnd,
        "バックアップの開始確認",
        &msg,
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    ) == IDYES
}

pub fn confirm_backup_stop(hwnd: HWND) -> bool {
    let msg = "現在実行中のバックアップを安全に中断しますか？\n\n\
               中断すると、作成途中のバックアップは不完全な状態になり復元に使用できません。";
    message_box(
        hwnd,
        "バックアップの中断確認",
        msg,
        MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
    ) == IDYES
}
