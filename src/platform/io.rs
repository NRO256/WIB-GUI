use crate::platform::win32::{
    BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, CF_UNICODETEXT, CLSCTX_INPROC_SERVER,
    CLSID_FILE_OPEN_DIALOG, CloseClipboard, CoCreateInstance, CoTaskMemFree, DWORD, EmptyClipboard,
    FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, GMEM_MOVEABLE, GetSaveFileNameW,
    GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, HRESULT_CANCELLED, HWND, IFileOpenDialog,
    IID_IFILE_OPEN_DIALOG, IShellItem, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
    OPENFILENAMEW, OpenClipboard, SHBrowseForFolderW, SHGetPathFromIDListW, SIGDN_FILESYSPATH,
    SetClipboardData, from_wide_ptr, from_wide_slice, to_wide_null,
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
    dialog_title: &str,
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
    let title = to_wide_null(dialog_title);

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

struct FileOpenDialogGuard(*mut IFileOpenDialog);

impl Drop for FileOpenDialogGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ((*(*self.0).lpVtbl).Release)(self.0.cast());
            }
        }
    }
}

struct ShellItemGuard(*mut IShellItem);

impl Drop for ShellItemGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ((*(*self.0).lpVtbl).Release)(self.0.cast());
            }
        }
    }
}

/// フォルダ選択ダイアログを開く（IFileOpenDialog 優先、失敗時は SHBrowseForFolderW フォールバック）。
pub fn browse_folder_dialog(hwnd: HWND, prompt: &str) -> Option<String> {
    let clsid = CLSID_FILE_OPEN_DIALOG;
    let iid = IID_IFILE_OPEN_DIALOG;
    let mut raw_dialog: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr_create = unsafe {
        CoCreateInstance(
            &raw const clsid,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &raw const iid,
            &raw mut raw_dialog,
        )
    };
    if hr_create < 0 || raw_dialog.is_null() {
        return browse_folder_dialog_legacy(hwnd, prompt);
    }

    let dialog = FileOpenDialogGuard(raw_dialog.cast::<IFileOpenDialog>());
    unsafe {
        let mut opts: DWORD = 0;
        let hr_get = ((*(*dialog.0).lpVtbl).GetOptions)(dialog.0.cast(), &raw mut opts);
        if hr_get >= 0 {
            let new_opts = opts | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
            ((*(*dialog.0).lpVtbl).SetOptions)(dialog.0.cast(), new_opts);
        }
        let title_w = to_wide_null(prompt);
        ((*(*dialog.0).lpVtbl).SetTitle)(dialog.0.cast(), title_w.as_ptr());

        let hr_show = ((*(*dialog.0).lpVtbl).Show)(dialog.0.cast(), hwnd);
        if hr_show == HRESULT_CANCELLED || hr_show < 0 {
            return None;
        }

        let mut raw_item: *mut IShellItem = std::ptr::null_mut();
        let hr_res = ((*(*dialog.0).lpVtbl).GetResult)(dialog.0.cast(), &raw mut raw_item);
        if hr_res < 0 || raw_item.is_null() {
            return None;
        }
        let item = ShellItemGuard(raw_item);

        let mut path_ptr: *mut u16 = std::ptr::null_mut();
        let hr_name = ((*(*item.0).lpVtbl).GetDisplayName)(
            item.0.cast(),
            SIGDN_FILESYSPATH,
            &raw mut path_ptr,
        );
        if hr_name < 0 || path_ptr.is_null() {
            return None;
        }
        let path = from_wide_ptr(path_ptr);
        CoTaskMemFree(path_ptr.cast());
        if path.is_empty() { None } else { Some(path) }
    }
}

fn browse_folder_dialog_legacy(hwnd: HWND, prompt: &str) -> Option<String> {
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
