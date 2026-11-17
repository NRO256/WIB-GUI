use crate::platform::win32::{
    BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, CF_UNICODETEXT, CloseClipboard,
    CoTaskMemFree, EmptyClipboard, GMEM_MOVEABLE, GetSaveFileNameW, GlobalAlloc, GlobalFree,
    GlobalLock, GlobalUnlock, HWND, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
    OPENFILENAMEW, OpenClipboard, SHBrowseForFolderW, SHGetPathFromIDListW, SetClipboardData,
    from_wide_slice, to_wide_null,
};
use std::path::PathBuf;

/// テキストを UTF-16LE でクリップボードにコピーする。
pub fn copy_to_clipboard(hwnd: HWND, text: &str) -> Result<(), String> {
    let wide = to_wide_null(text);
    let byte_len = wide.len() * std::mem::size_of::<u16>();

    unsafe {
        if OpenClipboard(hwnd) == 0 {
            return Err("クリップボードを開けませんでした。".to_string());
        }
        EmptyClipboard();
        let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if hmem.is_null() {
            CloseClipboard();
            return Err("クリップボード用のメモリ確保に失敗しました。".to_string());
        }
        let ptr = GlobalLock(hmem).cast::<u16>();
        if ptr.is_null() {
            GlobalFree(hmem);
            CloseClipboard();
            return Err("クリップボードメモリのロックに失敗しました。".to_string());
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(hmem);

        if SetClipboardData(CF_UNICODETEXT, hmem).is_null() {
            GlobalFree(hmem);
            CloseClipboard();
            return Err("クリップボードへのデータ設定に失敗しました。".to_string());
        }
        CloseClipboard();
    }
    Ok(())
}

/// 名前を付けて保存ダイアログを開き、UTF-8 BOM付きテキストとして保存する。
pub fn save_text_with_dialog(
    hwnd: HWND,
    default_filename: &str,
    content: &str,
) -> Result<Option<PathBuf>, String> {
    let mut file_buf = [0u16; 1024];
    for (i, ch) in default_filename.encode_utf16().enumerate() {
        if i + 1 < file_buf.len() {
            file_buf[i] = ch;
        }
    }

    let filter: Vec<u16> = "テキスト ファイル (*.txt)\0*.txt\0すべてのファイル (*.*)\0*.*\0\0"
        .encode_utf16()
        .collect();
    let def_ext = to_wide_null("txt");
    let title = to_wide_null("復元手順をテキストファイルに保存");

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: hwnd,
        lpstrFilter: filter.as_ptr(),
        lpstrFile: file_buf.as_mut_ptr(),
        nMaxFile: file_buf.len() as u32,
        lpstrDefExt: def_ext.as_ptr(),
        lpstrTitle: title.as_ptr(),
        Flags: OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
        ..Default::default()
    };

    let ok = unsafe { GetSaveFileNameW(&raw mut ofn) };
    if ok == 0 {
        return Ok(None);
    }

    let path = PathBuf::from(from_wide_slice(&file_buf));
    let mut bytes = Vec::with_capacity(3 + content.len());
    bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
    bytes.extend_from_slice(content.as_bytes());

    std::fs::write(&path, bytes).map_err(|e| format!("ファイルの保存に失敗しました: {e}"))?;
    Ok(Some(path))
}

/// フォルダ選択ダイアログを開く（UNC共有フォルダ参照用）。
pub fn browse_folder_dialog(hwnd: HWND, prompt: &str) -> Option<String> {
    let title_w = to_wide_null(prompt);
    let mut display_buf = [0u16; 260];
    let mut bi = BROWSEINFOW {
        hwndOwner: hwnd,
        pszDisplayName: display_buf.as_mut_ptr(),
        lpszTitle: title_w.as_ptr(),
        ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
        ..Default::default()
    };

    unsafe {
        let pidl = SHBrowseForFolderW(&raw mut bi);
        if pidl.is_null() {
            return None;
        }
        let mut path_buf = [0u16; 1024];
        let ok = SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr());
        CoTaskMemFree(pidl);
        if ok != 0 {
            let s = from_wide_slice(&path_buf);
            if s.is_empty() { None } else { Some(s) }
        } else {
            None
        }
    }
}
