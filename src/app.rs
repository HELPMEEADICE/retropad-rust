use std::cell::RefCell;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use crate::file_io::{self, TextEncoding};
use crate::resource_ids::*;
use crate::search;
use crate::win32::*;

const APP_TITLE: &str = "retropad";
const UNTITLED_NAME: &str = "Untitled";
const MAX_PATH_BUFFER: usize = 1024;
const DEFAULT_WIDTH: i32 = 640;
const DEFAULT_HEIGHT: i32 = 480;

pub struct FindReplaceData {
    find_text: Vec<u16>,
    replace_text: Vec<u16>,
}

pub struct AppState {
    pub hwnd_main: HWND,
    pub hwnd_edit: HWND,
    pub hwnd_status: HWND,
    pub hfont: HFONT,
    pub current_path: String,
    pub word_wrap: bool,
    pub status_visible: bool,
    pub status_before_wrap: bool,
    pub modified: bool,
    pub encoding: TextEncoding,
    pub find_flags: u32,
    pub hfind_dlg: HWND,
    pub hreplace_dlg: HWND,
    pub find_replace_data: Option<FindReplaceData>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            hwnd_main: 0,
            hwnd_edit: 0,
            hwnd_status: 0,
            hfont: 0,
            current_path: String::new(),
            word_wrap: false,
            status_visible: true,
            status_before_wrap: true,
            modified: false,
            encoding: TextEncoding::Utf8,
            find_flags: FR_DOWN,
            hfind_dlg: 0,
            hreplace_dlg: 0,
            find_replace_data: None,
        }
    }
}

thread_local! {
    static APP_STATE: RefCell<AppState> = RefCell::new(AppState::default());
    static APP_HINST: RefCell<HINSTANCE> = const { RefCell::new(0) };
    static APP_FIND_MSG: RefCell<u32> = const { RefCell::new(0) };
}

fn wstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn info_msg(owner: HWND, msg: &str) {
    let t = wstr(APP_TITLE);
    let m = wstr(msg);
    unsafe { MessageBoxW(owner, m.as_ptr(), t.as_ptr(), MB_ICONINFORMATION); }
}

fn warning_msg(owner: HWND, msg: &str) {
    let t = wstr(APP_TITLE);
    let m = wstr(msg);
    unsafe { MessageBoxW(owner, m.as_ptr(), t.as_ptr(), MB_ICONWARNING); }
}

fn error_msg(owner: HWND, msg: &str) {
    let t = wstr(APP_TITLE);
    let m = wstr(msg);
    unsafe { MessageBoxW(owner, m.as_ptr(), t.as_ptr(), MB_ICONERROR); }
}

fn window_title() -> String {
    APP_STATE.with(|s| {
        let state = s.borrow();
        let name = if state.current_path.is_empty() {
            UNTITLED_NAME.to_string()
        } else {
            std::path::Path::new(&state.current_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&state.current_path)
                .to_string()
        };
        format!(
            "{}{} - {}",
            if state.modified { "*" } else { "" },
            name,
            APP_TITLE
        )
    })
}

fn update_title(hwnd: HWND) {
    let title = window_title();
    let w = wstr(&title);
    unsafe { SetWindowTextW(hwnd, w.as_ptr()); }
}

fn create_edit_control(hwnd: HWND) {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.hwnd_edit != 0 {
            let old = state.hwnd_edit;
            drop(state);
            unsafe { DestroyWindow(old); }
            state = s.borrow_mut();
        }

        let mut style = WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE
            | ES_AUTOVSCROLL | ES_WANTRETURN | ES_NOHIDESEL;

        if !state.word_wrap {
            style |= WS_HSCROLL | ES_AUTOHSCROLL;
        }

        let hfont = state.hfont;
        let hinst = APP_HINST.with(|h| *h.borrow());

        let edit = unsafe {
            CreateWindowExW(
                WS_EX_CLIENTEDGE,
                wstr("EDIT").as_ptr(),
                std::ptr::null(),
                style,
                0, 0, 0, 0,
                hwnd,
                1, // ID = 1
                hinst,
                std::ptr::null(),
            )
        };

        if edit != 0 {
            state.hwnd_edit = edit;
            if hfont != 0 {
                unsafe { SendMessageW(edit, WM_SETFONT, hfont as usize, 1); }
            }
            unsafe {
                SendMessageW(edit, EM_SETMODIFY, 0, 0);
                SendMessageW(edit, EM_SETLIMITTEXT, 0, 0);
            }
        }
    });
    update_layout(hwnd);
}

fn toggle_status_bar(hwnd: HWND, visible: bool) {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.status_visible = visible;
        if visible {
            if state.hwnd_status == 0 {
                let status = unsafe {
                    CreateStatusWindowW(
                        (WS_CHILD | SBARS_SIZEGRIP) as i32,
                        std::ptr::null(),
                        hwnd,
                        2,
                    )
                };
                if status != 0 {
                    state.hwnd_status = status;
                }
            }
            if state.hwnd_status != 0 {
                unsafe { ShowWindow(state.hwnd_status, SW_SHOW); }
            }
        } else if state.hwnd_status != 0 {
            unsafe { ShowWindow(state.hwnd_status, SW_HIDE); }
        }
    });
    update_layout(hwnd);
}

fn update_layout(hwnd: HWND) {
    APP_STATE.with(|s| {
        let state = s.borrow();
        let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        unsafe { GetClientRect(hwnd, &mut rc); }

        let mut status_height: i32 = 0;
        if state.status_visible && state.hwnd_status != 0 {
            unsafe { SendMessageW(state.hwnd_status, WM_SIZE, 0, 0); }
            let mut sbrc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            unsafe { GetWindowRect(state.hwnd_status, &mut sbrc); }
            status_height = sbrc.bottom - sbrc.top;
            unsafe {
                MoveWindow(
                    state.hwnd_status,
                    0,
                    rc.bottom - status_height,
                    rc.right,
                    status_height,
                    TRUE,
                );
            }
        }

        if state.hwnd_edit != 0 {
            unsafe {
                MoveWindow(
                    state.hwnd_edit,
                    0,
                    0,
                    rc.right,
                    rc.bottom - status_height,
                    TRUE,
                );
            }
        }
    });
}

fn prompt_save_changes(hwnd: HWND) -> bool {
    let (modified, name) = APP_STATE.with(|s| {
        let state = s.borrow();
        (
            state.modified,
            if state.current_path.is_empty() {
                UNTITLED_NAME.to_string()
            } else {
                state.current_path.clone()
            },
        )
    });

    if !modified {
        return true;
    }

    let prompt = format!("Do you want to save changes to {}?", name);
    let p = wstr(&prompt);
    let t = wstr(APP_TITLE);

    let res = unsafe {
        MessageBoxW(hwnd, p.as_ptr(), t.as_ptr(), MB_ICONQUESTION | MB_YESNOCANCEL)
    };

    match res {
        IDYES => do_file_save(hwnd, false),
        IDNO => true,
        _ => false,
    }
}

fn load_document_from_path(hwnd: HWND, path: &str) -> bool {
    let (text, enc) = match file_io::load_text_file(hwnd, path) {
        Some(r) => r,
        None => return false,
    };

    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let w = wstr(&text);
        unsafe { SetWindowTextW(state.hwnd_edit, w.as_ptr()); }
        state.current_path = path.to_string();
        state.encoding = enc;
        unsafe { SendMessageW(state.hwnd_edit, EM_SETMODIFY, 0, 0); }
        state.modified = false;
    });

    update_title(hwnd);
    true
}

fn do_file_open(hwnd: HWND) -> bool {
    if !prompt_save_changes(hwnd) {
        return false;
    }
    match file_io::open_file_dialog(hwnd) {
        Some(path) => load_document_from_path(hwnd, &path),
        None => false,
    }
}

fn do_file_save(hwnd: HWND, save_as: bool) -> bool {
    let (path, text, enc) = APP_STATE.with(|s| {
        let state = s.borrow();
        let path = state.current_path.clone();
        let enc = state.encoding;
        let text = search::get_edit_text(state.hwnd_edit);
        (path, text, enc)
    });

    let text = match text {
        Some(t) => t,
        None => return false,
    };

    let path = if save_as || path.is_empty() {
        match file_io::save_file_dialog(hwnd, &path) {
            Some(p) => p,
            None => return false,
        }
    } else {
        path
    };

    let ok = file_io::save_text_file(hwnd, &path, &text, enc);
    if ok {
        APP_STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.current_path = path;
            unsafe { SendMessageW(state.hwnd_edit, EM_SETMODIFY, 0, 0); }
            state.modified = false;
        });
        update_title(hwnd);
    }
    ok
}

fn do_file_new(hwnd: HWND) {
    if !prompt_save_changes(hwnd) {
        return;
    }
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        unsafe { SetWindowTextW(state.hwnd_edit, wstr("").as_ptr()); }
        state.current_path.clear();
        state.encoding = TextEncoding::Utf8;
        unsafe { SendMessageW(state.hwnd_edit, EM_SETMODIFY, 0, 0); }
        state.modified = false;
    });
    update_title(hwnd);
}

fn set_word_wrap(hwnd: HWND, enabled: bool) {
    let (already, edit, status_before) = APP_STATE.with(|s| {
        let state = s.borrow();
        (state.word_wrap == enabled, state.hwnd_edit, state.status_before_wrap)
    });

    if already {
        return;
    }

    let text = search::get_edit_text(edit).unwrap_or_default();
    let (sel_start, sel_end) = search::get_edit_sel(edit);

    APP_STATE.with(|s| s.borrow_mut().word_wrap = enabled);

    create_edit_control(hwnd);

    APP_STATE.with(|s| {
        let state = s.borrow();
        let w = wstr(&text);
        unsafe { SetWindowTextW(state.hwnd_edit, w.as_ptr()); }
        unsafe { SendMessageW(state.hwnd_edit, EM_SETSEL, sel_start, sel_end as isize); }
    });

    if enabled {
        APP_STATE.with(|s| s.borrow_mut().status_before_wrap = true);
        toggle_status_bar(hwnd, false);
        let menu = unsafe { GetMenu(hwnd) };
        unsafe {
            EnableMenuItem(menu, IDM_VIEW_STATUS_BAR as u32, MF_BYCOMMAND | MF_GRAYED);
            EnableMenuItem(menu, IDM_EDIT_GOTO as u32, MF_BYCOMMAND | MF_GRAYED);
        }
    } else {
        toggle_status_bar(hwnd, status_before);
        let menu = unsafe { GetMenu(hwnd) };
        unsafe {
            EnableMenuItem(menu, IDM_VIEW_STATUS_BAR as u32, MF_BYCOMMAND | MF_ENABLED);
            EnableMenuItem(menu, IDM_EDIT_GOTO as u32, MF_BYCOMMAND | MF_ENABLED);
        }
    }
    update_title(hwnd);
}

fn update_status_bar() {
    APP_STATE.with(|s| {
        let state = s.borrow();
        if !state.status_visible || state.hwnd_status == 0 {
            return;
        }

        let (sel_start, _) = search::get_edit_sel(state.hwnd_edit);
        let line = unsafe {
            SendMessageW(state.hwnd_edit, EM_LINEFROMCHAR, sel_start, 0)
        } as i32 + 1;
        let line_idx = unsafe {
            SendMessageW(state.hwnd_edit, EM_LINEINDEX, (line - 1) as usize, 0)
        } as i32;
        let col = sel_start as i32 - line_idx + 1;
        let lines = unsafe {
            SendMessageW(state.hwnd_edit, EM_GETLINECOUNT, 0, 0)
        } as i32;

        let text = format!("Ln {}, Col {}    Lines: {}", line, col, lines);
        let w = wstr(&text);
        unsafe {
            SendMessageW(state.hwnd_status, SB_SETTEXT, 0, w.as_ptr() as isize);
        }
    });
}

fn show_find_dialog(hwnd: HWND) {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.hfind_dlg != 0 {
            let dlg = state.hfind_dlg;
            drop(state);
            unsafe { SetForegroundWindow(dlg); }
            return;
        }

        let find_text = state.find_replace_data.as_ref()
            .and_then(|d| String::from_utf16(&d.find_text).ok())
            .unwrap_or_default();
        let find_text = find_text.trim_end_matches('\0').to_string();
        let flags = state.find_flags;

        let mut ft: Vec<u16> = wstr(&find_text);
        ft.resize(128, 0);

        let mut data = FindReplaceData {
            find_text: ft,
            replace_text: vec![0u16; 128],
        };

        let mut fr = FINDREPLACEW {
            lStructSize: std::mem::size_of::<FINDREPLACEW>() as u32,
            hwndOwner: hwnd,
            hInstance: 0,
            Flags: flags,
            lpstrFindWhat: data.find_text.as_mut_ptr(),
            lpstrReplaceWith: std::ptr::null_mut(),
            wFindWhatLen: 128,
            wReplaceWithLen: 0,
            lCustData: 0,
            lpfnHook: None,
            lpTemplateName: std::ptr::null(),
        };

        let dlg = unsafe { FindTextW(&mut fr) };
        state.hfind_dlg = dlg;
        state.find_replace_data = Some(data);
    });
}

fn show_replace_dialog(hwnd: HWND) {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.hreplace_dlg != 0 {
            let dlg = state.hreplace_dlg;
            drop(state);
            unsafe { SetForegroundWindow(dlg); }
            return;
        }

        let find_text = state.find_replace_data.as_ref()
            .and_then(|d| String::from_utf16(&d.find_text).ok())
            .unwrap_or_default();
        let find_text = find_text.trim_end_matches('\0').to_string();
        let replace_text = state.find_replace_data.as_ref()
            .and_then(|d| String::from_utf16(&d.replace_text).ok())
            .unwrap_or_default();
        let replace_text = replace_text.trim_end_matches('\0').to_string();
        let flags = state.find_flags;

        let mut ft: Vec<u16> = wstr(&find_text);
        ft.resize(128, 0);
        let mut rt: Vec<u16> = wstr(&replace_text);
        rt.resize(128, 0);

        let mut data = FindReplaceData {
            find_text: ft,
            replace_text: rt,
        };

        let mut fr = FINDREPLACEW {
            lStructSize: std::mem::size_of::<FINDREPLACEW>() as u32,
            hwndOwner: hwnd,
            hInstance: 0,
            Flags: flags,
            lpstrFindWhat: data.find_text.as_mut_ptr(),
            lpstrReplaceWith: data.replace_text.as_mut_ptr(),
            wFindWhatLen: 128,
            wReplaceWithLen: 128,
            lCustData: 0,
            lpfnHook: None,
            lpTemplateName: std::ptr::null(),
        };

        let dlg = unsafe { ReplaceTextW(&mut fr) };
        state.hreplace_dlg = dlg;
        state.find_replace_data = Some(data);
    });
}

fn do_find_next(reverse: bool) {
    let (find_text, flags, edit, main) = APP_STATE.with(|s| {
        let state = s.borrow();
        let ft = state.find_replace_data.as_ref()
            .map(|d| String::from_utf16_lossy(&d.find_text))
            .unwrap_or_default();
        let ft = ft.trim_end_matches('\0').to_string();
        (ft, state.find_flags, state.hwnd_edit, state.hwnd_main)
    });

    if find_text.is_empty() {
        show_find_dialog(main);
        return;
    }

    let match_case = (flags & FR_MATCHCASE) != 0;
    let down = if reverse {
        (flags & FR_DOWN) == 0
    } else {
        (flags & FR_DOWN) != 0
    };

    let (sel_start, sel_end) = search::get_edit_sel(edit);
    let search_start = if down { sel_end } else { sel_start };

    match search::find_in_edit(edit, &find_text, match_case, down, search_start) {
        Some((start, end)) => search::select_match(edit, start, end),
        None => info_msg(main, "Cannot find the text."),
    }
}

fn do_select_font(hwnd: HWND) {
    APP_STATE.with(|s| {
        let mut state = s.borrow_mut();
        let mut lf = LOGFONTW {
            lfHeight: 0, lfWidth: 0, lfEscapement: 0, lfOrientation: 0,
            lfWeight: 0, lfItalic: 0, lfUnderline: 0, lfStrikeOut: 0,
            lfCharSet: 0, lfOutPrecision: 0, lfClipPrecision: 0,
            lfQuality: 0, lfPitchAndFamily: 0, lfFaceName: [0u16; 32],
        };

        if state.hfont != 0 {
            unsafe {
                GetObjectW(
                    state.hfont as HGDIOBJ,
                    std::mem::size_of::<LOGFONTW>() as i32,
                    &mut lf as *mut _ as *mut std::ffi::c_void,
                );
            }
        } else {
            unsafe {
                SystemParametersInfoW(
                    SPI_GETICONTITLELOGFONT,
                    std::mem::size_of::<LOGFONTW>() as u32,
                    &mut lf as *mut _ as *mut std::ffi::c_void,
                    0,
                );
            }
        }

        let mut cf = CHOOSEFONTW {
            lStructSize: std::mem::size_of::<CHOOSEFONTW>() as u32,
            hwndOwner: hwnd,
            hDC: 0,
            lpLogFont: &mut lf,
            iPointSize: 0,
            Flags: CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT,
            rgbColors: 0,
            lCustData: 0,
            lpfnHook: None,
            lpTemplateName: std::ptr::null(),
            hInstance: 0,
            lpszStyle: std::ptr::null_mut(),
            nFontType: 0,
            ___MISSING_ALIGNMENT__: 0,
            nSizeMin: 0,
            nSizeMax: 0,
        };

        let result = unsafe { ChooseFontW(&mut cf) };
        if result != 0 {
            let new_font = unsafe { CreateFontIndirectW(&lf) };
            if new_font != 0 {
                if state.hfont != 0 {
                    unsafe { DeleteObject(state.hfont as HGDIOBJ); }
                }
                state.hfont = new_font;
                unsafe {
                    SendMessageW(state.hwnd_edit, WM_SETFONT, new_font as usize, 1);
                }
                update_layout(hwnd);
            }
        }
    });
}

fn insert_time_date(_hwnd: HWND) {
    APP_STATE.with(|s| {
        let state = s.borrow();
        let mut st = SYSTEMTIME {
            wYear: 0, wMonth: 0, wDayOfWeek: 0, wDay: 0,
            wHour: 0, wMinute: 0, wSecond: 0, wMilliseconds: 0,
        };
        unsafe { GetLocalTime(&mut st); }

        let mut date_buf = [0u16; 64];
        let mut time_buf = [0u16; 64];

        unsafe {
            GetDateFormatW(
                LOCALE_USER_DEFAULT,
                DATE_SHORTDATE,
                &st,
                std::ptr::null(),
                date_buf.as_mut_ptr(),
                64,
            );
            GetTimeFormatW(
                LOCALE_USER_DEFAULT,
                0,
                &st,
                std::ptr::null(),
                time_buf.as_mut_ptr(),
                64,
            );
        }

        let date_end = date_buf.iter().position(|&c| c == 0).unwrap_or(64);
        let time_end = time_buf.iter().position(|&c| c == 0).unwrap_or(64);
        let date_str = String::from_utf16_lossy(&date_buf[..date_end]);
        let time_str = String::from_utf16_lossy(&time_buf[..time_end]);

        let stamp = format!("{} {}", time_str, date_str);
        let w = wstr(&stamp);
        unsafe {
            SendMessageW(
                state.hwnd_edit,
                EM_REPLACESEL,
                TRUE as usize,
                w.as_ptr() as isize,
            );
        }
    });
}

fn unsigned_sub(a: usize, b: usize) -> usize {
    a.saturating_sub(b)
}

fn handle_find_replace(lpfr: isize) {
    if lpfr == 0 {
        return;
    }
    let fr = unsafe { &*(lpfr as *const FINDREPLACEW) };

    if (fr.Flags & FR_DIALOGTERM) != 0 {
        APP_STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.hfind_dlg = 0;
            state.hreplace_dlg = 0;
            state.find_replace_data = None;
        });
        return;
    }

    let flags = fr.Flags;
    let match_case = (flags & FR_MATCHCASE) != 0;
    let down = (flags & FR_DOWN) != 0;
    let find_text = if fr.lpstrFindWhat.is_null() {
        String::new()
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts(fr.lpstrFindWhat, 128);
            let len = slice.iter().position(|&c| c == 0).unwrap_or(128);
            String::from_utf16_lossy(&slice[..len])
        }
    };

    APP_STATE.with(|s| s.borrow_mut().find_flags = flags);

    let (edit, main) = APP_STATE.with(|s| {
        let state = s.borrow();
        (state.hwnd_edit, state.hwnd_main)
    });

    if (flags & FR_FINDNEXT) != 0 {
        if find_text.is_empty() { return; }
        let (sel_start, sel_end) = search::get_edit_sel(edit);
        let search_start = if down { sel_end } else { sel_start };
        match search::find_in_edit(edit, &find_text, match_case, down, search_start) {
            Some((start, end)) => search::select_match(edit, start, end),
            None => info_msg(main, "Cannot find the text."),
        }
    } else if (flags & FR_REPLACE) != 0 {
        if find_text.is_empty() { return; }
        let replace_text = if fr.lpstrReplaceWith.is_null() {
            String::new()
        } else {
            unsafe {
                let slice = std::slice::from_raw_parts(fr.lpstrReplaceWith, 128);
                let len = slice.iter().position(|&c| c == 0).unwrap_or(128);
                String::from_utf16_lossy(&slice[..len])
            }
        };
        let (sel_start, _) = search::get_edit_sel(edit);
        match search::find_in_edit(edit, &find_text, match_case, down, sel_start) {
            Some((start, end)) => {
                search::select_match(edit, start, end);
                let w = wstr(&replace_text);
                unsafe {
                    SendMessageW(edit, EM_REPLACESEL, TRUE as usize, w.as_ptr() as isize);
                    SendMessageW(edit, EM_SCROLLCARET, 0, 0);
                }
                APP_STATE.with(|s| s.borrow_mut().modified = true);
                update_title(main);
            }
            None => info_msg(main, "Cannot find the text."),
        }
    } else if (flags & FR_REPLACEALL) != 0 {
        if find_text.is_empty() { return; }
        let replace_text = if fr.lpstrReplaceWith.is_null() {
            String::new()
        } else {
            unsafe {
                let slice = std::slice::from_raw_parts(fr.lpstrReplaceWith, 128);
                let len = slice.iter().position(|&c| c == 0).unwrap_or(128);
                String::from_utf16_lossy(&slice[..len])
            }
        };
        let count = search::replace_all_occurrences(edit, &find_text, &replace_text, match_case);
        let msg = format!("Replaced {} occurrence{}.", count, if count == 1 { "" } else { "s" });
        info_msg(main, &msg);
        APP_STATE.with(|s| s.borrow_mut().modified = true);
        update_title(main);
    }
}

fn update_menu_states(hwnd: HWND) {
    let (word_wrap, status_visible, edit) = APP_STATE.with(|s| {
        let state = s.borrow();
        (state.word_wrap, state.status_visible, state.hwnd_edit)
    });

    let menu = unsafe { GetMenu(hwnd) };
    if menu == 0 {
        return;
    }

    let wrap_state = if word_wrap { MF_CHECKED } else { MF_UNCHECKED };
    let status_state = if status_visible { MF_CHECKED } else { MF_UNCHECKED };

    unsafe {
        CheckMenuItem(menu, IDM_FORMAT_WORD_WRAP as u32, MF_BYCOMMAND | wrap_state);
        CheckMenuItem(menu, IDM_VIEW_STATUS_BAR as u32, MF_BYCOMMAND | status_state);
    }

    let can_goto = !word_wrap;
    unsafe {
        EnableMenuItem(
            menu,
            IDM_EDIT_GOTO as u32,
            MF_BYCOMMAND | if can_goto { MF_ENABLED } else { MF_GRAYED },
        );
    }

    if word_wrap {
        unsafe {
            EnableMenuItem(menu, IDM_VIEW_STATUS_BAR as u32, MF_BYCOMMAND | MF_GRAYED);
        }
    } else {
        unsafe {
            EnableMenuItem(menu, IDM_VIEW_STATUS_BAR as u32, MF_BYCOMMAND | MF_ENABLED);
        }
    }

    let modified = unsafe {
        SendMessageW(edit, EM_GETMODIFY, 0, 0) != 0
    };
    unsafe {
        EnableMenuItem(
            menu,
            IDM_FILE_SAVE as u32,
            MF_BYCOMMAND | if modified { MF_ENABLED } else { MF_GRAYED },
        );
    }
}

fn handle_command(hwnd: HWND, wparam: WPARAM, _lparam: LPARAM) {
    let cmd = (wparam as u32 & 0xFFFF) as u16;
    match cmd {
        IDM_FILE_NEW => do_file_new(hwnd),
        IDM_FILE_OPEN => { do_file_open(hwnd); }
        IDM_FILE_SAVE => { do_file_save(hwnd, false); }
        IDM_FILE_SAVE_AS => { do_file_save(hwnd, true); }
        IDM_FILE_PAGE_SETUP | IDM_FILE_PRINT => {
            info_msg(hwnd, "Printing is not implemented in retropad.");
        }
        IDM_FILE_EXIT => {
            unsafe { PostQuitMessage(0); }
        }

        IDM_EDIT_UNDO => {
            APP_STATE.with(|s| {
                let edit = s.borrow().hwnd_edit;
                unsafe { SendMessageW(edit, EM_UNDO, 0, 0); }
            });
        }
        IDM_EDIT_CUT => {
            APP_STATE.with(|s| {
                let edit = s.borrow().hwnd_edit;
                unsafe { SendMessageW(edit, WM_CUT, 0, 0); }
            });
        }
        IDM_EDIT_COPY => {
            APP_STATE.with(|s| {
                let edit = s.borrow().hwnd_edit;
                unsafe { SendMessageW(edit, WM_COPY, 0, 0); }
            });
        }
        IDM_EDIT_PASTE => {
            APP_STATE.with(|s| {
                let edit = s.borrow().hwnd_edit;
                unsafe { SendMessageW(edit, WM_PASTE, 0, 0); }
            });
        }
        IDM_EDIT_DELETE => {
            APP_STATE.with(|s| {
                let edit = s.borrow().hwnd_edit;
                unsafe { SendMessageW(edit, WM_CLEAR, 0, 0); }
            });
        }
        IDM_EDIT_FIND => show_find_dialog(hwnd),
        IDM_EDIT_FIND_NEXT => do_find_next(false),
        IDM_EDIT_REPLACE => show_replace_dialog(hwnd),
        IDM_EDIT_GOTO => {
            let word_wrap = APP_STATE.with(|s| s.borrow().word_wrap);
            if word_wrap {
                info_msg(hwnd, "Go To is unavailable when Word Wrap is on.");
            } else {
                let hinst = APP_HINST.with(|h| *h.borrow());
                unsafe {
                    DialogBoxParamW(
                        hinst,
                        make_int_resource(IDD_GOTO),
                        hwnd,
                        Some(goto_dlg_proc),
                        0,
                    );
                }
            }
        }
        IDM_EDIT_SELECT_ALL => {
            APP_STATE.with(|s| {
                let edit = s.borrow().hwnd_edit;
                unsafe { SendMessageW(edit, EM_SETSEL, 0, -1isize as isize); }
            });
        }
        IDM_EDIT_TIME_DATE => insert_time_date(hwnd),

        IDM_FORMAT_WORD_WRAP => {
            let word_wrap = APP_STATE.with(|s| s.borrow().word_wrap);
            set_word_wrap(hwnd, !word_wrap);
        }
        IDM_FORMAT_FONT => do_select_font(hwnd),

        IDM_VIEW_STATUS_BAR => {
            let visible = APP_STATE.with(|s| s.borrow().status_visible);
            toggle_status_bar(hwnd, !visible);
        }

        IDM_HELP_VIEW_HELP => {
            info_msg(hwnd, "No help file is available for retropad.");
        }
        IDM_HELP_ABOUT => {
            let hinst = APP_HINST.with(|h| *h.borrow());
            unsafe {
                DialogBoxParamW(
                    hinst,
                    make_int_resource(IDD_ABOUT),
                    hwnd,
                    Some(about_dlg_proc),
                    0,
                );
            }
        }
        _ => {}
    }
}

unsafe extern "system" fn goto_dlg_proc(
    dlg: HWND,
    msg: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> isize {
    match msg {
        WM_INITDIALOG => {
            let hnd = unsafe { GetDlgItem(dlg, IDC_GOTO_EDIT as i32) };
            unsafe {
                SetDlgItemInt(dlg, IDC_GOTO_EDIT as i32, 1u32, TRUE);
                if hnd != 0 {
                    SendMessageW(hnd, EM_SETLIMITTEXT, 10, 0);
                }
            }
            1
        }
        WM_COMMAND => {
            let cmd = (wparam as u32 & 0xFFFF) as u16;
            match cmd {
                x if x as i32 == IDOK => {
                    let mut ok: BOOL = FALSE;
                    let line = unsafe {
                        GetDlgItemInt(dlg, IDC_GOTO_EDIT as i32, &mut ok, FALSE)
                    };
                    if ok == 0 || line == 0 {
                        warning_msg(dlg, "Enter a valid line number.");
                        return 1;
                    }
                    APP_STATE.with(|s| {
                        let state = s.borrow();
                        let max_line = unsafe {
                            SendMessageW(state.hwnd_edit, EM_GETLINECOUNT, 0, 0)
                        } as u32;
                        let line = line.min(max_line);
                        let char_idx = unsafe {
                            SendMessageW(
                                state.hwnd_edit,
                                EM_LINEINDEX,
                                unsigned_sub(line as usize, 1),
                                0,
                            )
                        } as i32;
                        if char_idx >= 0 {
                            unsafe {
                                SendMessageW(
                                    state.hwnd_edit,
                                    EM_SETSEL,
                                    char_idx as usize,
                                    char_idx as isize,
                                );
                                SendMessageW(state.hwnd_edit, EM_SCROLLCARET, 0, 0);
                            }
                        }
                    });
                    unsafe { EndDialog(dlg, IDOK as isize); }
                    1
                }
                x if x as i32 == IDCANCEL => {
                    unsafe { EndDialog(dlg, IDCANCEL as isize); }
                    1
                }
                _ => 0,
            }
        }
        _ => 0,
    }
}

unsafe extern "system" fn about_dlg_proc(
    dlg: HWND,
    msg: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> isize {
    match msg {
        WM_INITDIALOG => 1,
        WM_COMMAND => {
            let cmd = (wparam as u32 & 0xFFFF) as u16;
            if cmd as i32 == IDOK || cmd as i32 == IDCANCEL {
                unsafe { EndDialog(dlg, cmd as isize); }
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let find_msg = APP_FIND_MSG.with(|m| *m.borrow());

    if msg != 0 && msg == find_msg {
        handle_find_replace(lparam);
        return 0;
    }

    match msg {
        WM_CREATE => {
            let icc = INITCOMMONCONTROLSEX {
                dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_BAR_CLASSES,
            };
            unsafe { InitCommonControlsEx(&icc); }

            create_edit_control(hwnd);
            toggle_status_bar(hwnd, true);
            update_title(hwnd);
            update_status_bar();
            unsafe { DragAcceptFiles(hwnd, TRUE); }
            0
        }
        WM_SETFOCUS => {
            let edit = APP_STATE.with(|s| s.borrow().hwnd_edit);
            if edit != 0 {
                unsafe { SetFocus(edit); }
            }
            0
        }
        WM_SIZE => {
            update_layout(hwnd);
            update_status_bar();
            0
        }
        WM_DROPFILES => {
            let hdrop = wparam as HDROP;
            let mut buf = [0u16; MAX_PATH_BUFFER];
            let count = unsafe { DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0) };
            if count > 0 {
                unsafe { DragQueryFileW(hdrop, 0, buf.as_mut_ptr(), MAX_PATH_BUFFER as u32); }
                let end = buf.iter().position(|&c| c == 0).unwrap_or(MAX_PATH_BUFFER);
                let path = String::from_utf16_lossy(&buf[..end]);
                if prompt_save_changes(hwnd) {
                    load_document_from_path(hwnd, &path);
                }
            }
            unsafe { DragFinish(hdrop); }
            0
        }
        WM_COMMAND => {
            let notification = ((wparam as u32) >> 16) as u32;
            let ctrl_hwnd = lparam;

            let edit = APP_STATE.with(|s| s.borrow().hwnd_edit);

            if ctrl_hwnd == edit {
                if notification == EN_CHANGE {
                    let modified = unsafe { SendMessageW(edit, EM_GETMODIFY, 0, 0) != 0 };
                    APP_STATE.with(|s| s.borrow_mut().modified = modified);
                    update_title(hwnd);
                    update_status_bar();
                    return 0;
                } else if notification == 0x0400 { // EN_UPDATE
                    update_status_bar();
                    return 0;
                }
            }

            handle_command(hwnd, wparam, lparam);
            0
        }
        WM_INITMENUPOPUP => {
            update_menu_states(hwnd);
            0
        }
        WM_CLOSE => {
            if prompt_save_changes(hwnd) {
                unsafe { DestroyWindow(hwnd); }
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0); }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn run_app() -> i32 {
    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        APP_HINST.with(|h| *h.borrow_mut() = hinst);

        let find_msg_str = format!("{}\0", FINDMSGSTRING);
        let fm_wide: Vec<u16> = OsStr::new(&find_msg_str).encode_wide().collect();
        let fm = RegisterWindowMessageW(fm_wide.as_ptr());
        APP_FIND_MSG.with(|m| *m.borrow_mut() = fm);

        let class_name = wstr("RETROPAD_WINDOW");
        let icon = LoadIconW(hinst, make_int_resource(IDI_RETROPAD));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(main_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: icon,
            hCursor: LoadCursorW(0, IDC_IBEAM),
            hbrBackground: (COLOR_WINDOW + 1) as isize,
            lpszMenuName: make_int_resource(IDC_RETROPAD),
            lpszClassName: class_name.as_ptr(),
            hIconSm: icon,
        };

        if RegisterClassExW(&wc) == 0 {
            error_msg(0, "Failed to register window class.");
            return 0;
        }

        APP_STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.word_wrap = false;
            state.status_visible = true;
            state.status_before_wrap = true;
            state.encoding = TextEncoding::Utf8;
            state.find_flags = FR_DOWN;
        });

        let title = wstr(APP_TITLE);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            0,
            0,
            hinst,
            std::ptr::null(),
        );

        if hwnd == 0 {
            error_msg(0, "Failed to create main window.");
            return 0;
        }

        APP_STATE.with(|s| s.borrow_mut().hwnd_main = hwnd);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let accel = LoadAcceleratorsW(hinst, make_int_resource(IDC_RETROPAD));

        let mut msg: MSG = MSG {
            hwnd: 0, message: 0, wParam: 0, lParam: 0,
            time: 0, pt: POINT { x: 0, y: 0 },
        };

        loop {
            let ret = GetMessageW(&mut msg, 0, 0, 0);
            if ret <= 0 {
                break;
            }
            if accel == 0 || TranslateAcceleratorW(hwnd, accel, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        msg.wParam as i32
    }
}
