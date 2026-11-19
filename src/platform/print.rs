use crate::platform::win32::{
    AbortDoc, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DEFAULT_PITCH,
    DOCINFOW, DT_CALCRECT, DT_EDITCONTROL, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_WORDBREAK,
    DeleteDC, DeleteObject, DrawTextW, EndDoc, EndPage, FF_DONTCARE, FW_NORMAL, GetDeviceCaps,
    GlobalFree, HDC, HORZRES, HWND, LOGPIXELSY, MulDiv, OUT_DEFAULT_PRECIS, PD_NOPAGENUMS,
    PD_NOSELECTION, PD_RETURNDC, PD_USEDEVMODECOPIESANDCOLLATE, PRINTDLGW, PrintDlgW, RECT,
    SelectObject, StartDocW, StartPage, VERTRES, to_wide_null,
};

unsafe fn free_print_dlg_handles(pd: &mut PRINTDLGW) {
    unsafe {
        if !pd.hDevMode.is_null() {
            GlobalFree(pd.hDevMode);
            pd.hDevMode = std::ptr::null_mut();
        }
        if !pd.hDevNames.is_null() {
            GlobalFree(pd.hDevNames);
            pd.hDevNames = std::ptr::null_mut();
        }
    }
}

unsafe fn draw_page_footer(hdc: HDC, page_num: u32, left: i32, right: i32, top: i32, bottom: i32) {
    let footer_text = format!("- {page_num} -");
    let footer_w: Vec<u16> = footer_text.encode_utf16().collect();
    let mut rc = RECT {
        left,
        top,
        right,
        bottom,
    };
    unsafe {
        DrawTextW(
            hdc,
            footer_w.as_ptr(),
            footer_w.len() as i32,
            &raw mut rc,
            DT_RIGHT | DT_SINGLELINE,
        );
    }
}

/// Win32 GDI (PrintDlgW + StartDocW/StartPage/DrawTextW) で復元手順を印刷する。
/// ユーザーがキャンセルした場合は Ok(false) を返す。
pub fn print_text_document(hwnd: HWND, doc_title: &str, content: &str) -> Result<bool, String> {
    unsafe {
        let mut pd = PRINTDLGW {
            lStructSize: std::mem::size_of::<PRINTDLGW>() as u32,
            hwndOwner: hwnd,
            Flags: PD_RETURNDC | PD_NOSELECTION | PD_NOPAGENUMS | PD_USEDEVMODECOPIESANDCOLLATE,
            nCopies: 1,
            ..Default::default()
        };

        if PrintDlgW(&raw mut pd) == 0 {
            free_print_dlg_handles(&mut pd);
            return Ok(false);
        }

        let hdc = pd.hDC;
        if hdc.is_null() {
            free_print_dlg_handles(&mut pd);
            return Err("プリンタデバイスコンテキストの取得に失敗しました。".to_string());
        }

        let dpi_y = GetDeviceCaps(hdc, LOGPIXELSY).max(96);
        let page_w = GetDeviceCaps(hdc, HORZRES);
        let page_h = GetDeviceCaps(hdc, VERTRES);
        let margin_x = MulDiv(18, dpi_y, 25); // 約18mm相当
        let margin_y = MulDiv(18, dpi_y, 25);

        let font_height = -MulDiv(10, dpi_y, 72);
        let face = to_wide_null("BIZ UDゴシック");
        let hfont = CreateFontW(
            font_height,
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
        let old_font = if !hfont.is_null() {
            SelectObject(hdc, hfont)
        } else {
            std::ptr::null_mut()
        };

        let title_w = to_wide_null(doc_title);
        let di = DOCINFOW {
            cbSize: std::mem::size_of::<DOCINFOW>() as i32,
            lpszDocName: title_w.as_ptr(),
            ..Default::default()
        };

        let mut doc_started = false;
        let mut print_err: Option<String> = None;
        if StartDocW(hdc, &raw const di) <= 0 {
            print_err = Some("印刷ドキュメントの開始（StartDocW）に失敗しました。".to_string());
        } else {
            doc_started = true;
            if StartPage(hdc) <= 0 {
                print_err = Some("印刷ページの開始（StartPage）に失敗しました。".to_string());
            } else {
                let printable_left = margin_x;
                let printable_right = (page_w - margin_x).max(printable_left + 100);
                let footer_sample: Vec<u16> = "- 999 -".encode_utf16().collect();
                let mut footer_calc = RECT {
                    left: printable_left,
                    top: 0,
                    right: printable_right,
                    bottom: 0,
                };
                DrawTextW(
                    hdc,
                    footer_sample.as_ptr(),
                    footer_sample.len() as i32,
                    &raw mut footer_calc,
                    DT_RIGHT | DT_SINGLELINE | DT_CALCRECT,
                );
                let footer_h = (footer_calc.bottom - footer_calc.top).max(1);
                let footer_gap = (footer_h / 2).max(4);
                let footer_bottom = (page_h - margin_y).max(margin_y + footer_h + 100);
                let footer_top = footer_bottom - footer_h;
                let printable_bottom = (footer_top - footer_gap).max(margin_y + 100);
                let mut cur_y = margin_y;
                let mut page_num: u32 = 1;
                let mut line_w: Vec<u16> = Vec::with_capacity(128);

                for raw_line in content.split('\n') {
                    let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
                    let text_to_draw = if line.is_empty() { " " } else { line };
                    line_w.clear();
                    line_w.extend(text_to_draw.encode_utf16());

                    let mut calc_rc = RECT {
                        left: printable_left,
                        top: 0,
                        right: printable_right,
                        bottom: 0,
                    };
                    let flags = DT_LEFT | DT_WORDBREAK | DT_EDITCONTROL;
                    DrawTextW(
                        hdc,
                        line_w.as_ptr(),
                        line_w.len() as i32,
                        &raw mut calc_rc,
                        flags | DT_CALCRECT,
                    );
                    let line_h = (calc_rc.bottom - calc_rc.top).max(1);
                    let step_y = ((line_h * 5) / 4).max(line_h + 1);

                    if cur_y + line_h > printable_bottom && cur_y > margin_y {
                        draw_page_footer(
                            hdc,
                            page_num,
                            printable_left,
                            printable_right,
                            footer_top,
                            footer_bottom,
                        );
                        EndPage(hdc);
                        if StartPage(hdc) <= 0 {
                            print_err = Some("改ページ（StartPage）に失敗しました。".to_string());
                            break;
                        }
                        page_num += 1;
                        cur_y = margin_y;
                    }

                    let mut draw_rc = RECT {
                        left: printable_left,
                        top: cur_y,
                        right: printable_right,
                        bottom: (cur_y + line_h).min(printable_bottom),
                    };
                    DrawTextW(
                        hdc,
                        line_w.as_ptr(),
                        line_w.len() as i32,
                        &raw mut draw_rc,
                        flags,
                    );
                    cur_y += step_y;
                }

                if print_err.is_none() {
                    draw_page_footer(
                        hdc,
                        page_num,
                        printable_left,
                        printable_right,
                        footer_top,
                        footer_bottom,
                    );
                    EndPage(hdc);
                    EndDoc(hdc);
                }
            }
        }
        if doc_started && print_err.is_some() {
            AbortDoc(hdc);
        }

        if !old_font.is_null() {
            SelectObject(hdc, old_font);
        }
        if !hfont.is_null() {
            DeleteObject(hfont);
        }
        DeleteDC(hdc);
        free_print_dlg_handles(&mut pd);

        match print_err {
            Some(e) => Err(e),
            None => Ok(true),
        }
    }
}
