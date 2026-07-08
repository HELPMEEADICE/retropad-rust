#![allow(non_snake_case, dead_code)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HMENU = isize;
pub type HFONT = isize;
pub type HACCEL = isize;
pub type HDROP = isize;
pub type HGDIOBJ = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type BOOL = i32;
pub type DWORD = u32;
pub type UINT = u32;
pub type LONG = i32;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

pub const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
pub const SW_SHOW: i32 = 5;
pub const SW_HIDE: i32 = 0;
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_SETFOCUS: u32 = 0x0007;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_CUT: u32 = 0x0300;
pub const WM_COPY: u32 = 0x0301;
pub const WM_PASTE: u32 = 0x0302;
pub const WM_CLEAR: u32 = 0x0303;
pub const WM_DROPFILES: u32 = 0x0233;
pub const WM_INITMENUPOPUP: u32 = 0x0117;
pub const WM_INITDIALOG: u32 = 0x0110;
pub const WM_USER: u32 = 0x0400;

pub const EM_GETSEL: u32 = 0x00B0;
pub const EM_SETSEL: u32 = 0x00B1;
pub const EM_GETMODIFY: u32 = 0x00B8;
pub const EM_SETMODIFY: u32 = 0x00B9;
pub const EM_GETLINECOUNT: u32 = 0x00BA;
pub const EM_LINEINDEX: u32 = 0x00BB;
pub const EM_LINEFROMCHAR: u32 = 0x00C9;
pub const EM_REPLACESEL: u32 = 0x00C2;
pub const EM_SCROLLCARET: u32 = 0x00B7;
pub const EM_UNDO: u32 = 0x00C7;
pub const EM_SETLIMITTEXT: u32 = 0x00C5;
pub const EN_CHANGE: u32 = 0x0300;
pub const EN_UPDATE: u32 = 0x0400;

pub const SB_SETTEXT: u32 = WM_USER + 1;

pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_VREDRAW: u32 = 0x0001;

pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_HSCROLL: u32 = 0x00100000;
pub const WS_EX_CLIENTEDGE: u32 = 0x00000200;

pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_AUTOVSCROLL: u32 = 0x0040;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const ES_NOHIDESEL: u32 = 0x0100;
pub const ES_WANTRETURN: u32 = 0x1000;
pub const ES_NUMBER: u32 = 0x2000;

pub const SBARS_SIZEGRIP: u32 = 0x0100;

pub const MB_OK: u32 = 0x00000000;
pub const MB_ICONERROR: u32 = 0x00000010;
pub const MB_ICONQUESTION: u32 = 0x00000020;
pub const MB_ICONWARNING: u32 = 0x00000030;
pub const MB_ICONINFORMATION: u32 = 0x00000040;
pub const MB_YESNOCANCEL: u32 = 0x00000003;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;
pub const IDCANCEL: i32 = 2;
pub const IDOK: i32 = 1;

pub const MF_BYCOMMAND: u32 = 0x00000000;
pub const MF_CHECKED: u32 = 0x00000008;
pub const MF_UNCHECKED: u32 = 0x00000000;
pub const MF_ENABLED: u32 = 0x00000000;
pub const MF_GRAYED: u32 = 0x00000001;

pub const IDC_IBEAM: LPCWSTR = 32513 as LPCWSTR;

pub const ICC_BAR_CLASSES: u32 = 0x00000004;

pub const COLOR_WINDOW: i32 = 5;

pub const OFN_FILEMUSTEXIST: u32 = 0x00001000;
pub const OFN_HIDEREADONLY: u32 = 0x00000004;
pub const OFN_OVERWRITEPROMPT: u32 = 0x00000002;
pub const OFN_PATHMUSTEXIST: u32 = 0x00000800;

pub const FR_DOWN: u32 = 0x00000001;
pub const FR_MATCHCASE: u32 = 0x00000004;
pub const FR_FINDNEXT: u32 = 0x00000008;
pub const FR_REPLACE: u32 = 0x00000010;
pub const FR_REPLACEALL: u32 = 0x00000020;
pub const FR_DIALOGTERM: u32 = 0x00000040;

pub const CF_SCREENFONTS: u32 = 0x00000001;
pub const CF_INITTOLOGFONTSTRUCT: u32 = 0x00000040;

pub const CP_UTF8: u32 = 65001;
pub const CP_ACP: u32 = 0;
pub const MB_ERR_INVALID_CHARS: u32 = 0x00000008;

pub const DATE_SHORTDATE: u32 = 0x00000001;
pub const LOCALE_USER_DEFAULT: u32 = 0x0400;

pub const GENERIC_READ: u32 = 0x80000000;
pub const GENERIC_WRITE: u32 = 0x40000000;
pub const FILE_SHARE_READ: u32 = 0x00000001;
pub const OPEN_EXISTING: u32 = 3;
pub const CREATE_ALWAYS: u32 = 2;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;
pub const INVALID_HANDLE_VALUE: isize = -1isize;

pub const MAX_PATH: usize = 260;

pub const SPI_GETICONTITLELOGFONT: u32 = 0x001F;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: isize,
    pub hCursor: isize,
    pub hbrBackground: isize,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: isize,
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: u32,
    pub dwICC: u32,
}

#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: LPCWSTR,
    pub lpstrCustomFilter: LPWSTR,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: LPWSTR,
    pub nMaxFile: u32,
    pub lpstrFileTitle: LPWSTR,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: LPCWSTR,
    pub lpstrTitle: LPCWSTR,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: LPCWSTR,
    pub lCustData: LPARAM,
    pub lpfnHook: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> UINT>,
    pub lpTemplateName: LPCWSTR,
}

#[repr(C)]
pub struct LOGFONTW {
    pub lfHeight: i32,
    pub lfWidth: i32,
    pub lfEscapement: i32,
    pub lfOrientation: i32,
    pub lfWeight: i32,
    pub lfItalic: u8,
    pub lfUnderline: u8,
    pub lfStrikeOut: u8,
    pub lfCharSet: u8,
    pub lfOutPrecision: u8,
    pub lfClipPrecision: u8,
    pub lfQuality: u8,
    pub lfPitchAndFamily: u8,
    pub lfFaceName: [u16; 32],
}

#[repr(C)]
pub struct CHOOSEFONTW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hDC: isize,
    pub lpLogFont: *mut LOGFONTW,
    pub iPointSize: i32,
    pub Flags: u32,
    pub rgbColors: u32,
    pub lCustData: LPARAM,
    pub lpfnHook: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> UINT>,
    pub lpTemplateName: LPCWSTR,
    pub hInstance: HINSTANCE,
    pub lpszStyle: LPWSTR,
    pub nFontType: u16,
    pub ___MISSING_ALIGNMENT__: u16,
    pub nSizeMin: i32,
    pub nSizeMax: i32,
}

#[repr(C)]
pub struct FINDREPLACEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub Flags: u32,
    pub lpstrFindWhat: LPWSTR,
    pub lpstrReplaceWith: LPWSTR,
    pub wFindWhatLen: u16,
    pub wReplaceWithLen: u16,
    pub lCustData: LPARAM,
    pub lpfnHook: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> UINT>,
    pub lpTemplateName: LPCWSTR,
}

#[repr(C)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}

pub const FINDMSGSTRING: &str = "commdlg_FindReplace";

extern "system" {
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *const std::ffi::c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn TranslateAcceleratorW(hWnd: HWND, hAccTable: HACCEL, lpMsg: *const MSG) -> i32;
    pub fn LoadAcceleratorsW(hInstance: HINSTANCE, lpTableName: LPCWSTR) -> HACCEL;
    pub fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
    pub fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: i32) -> i32;
    pub fn GetWindowTextLengthW(hWnd: HWND) -> i32;
    pub fn MessageBoxW(hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: u32) -> i32;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn MoveWindow(hWnd: HWND, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: BOOL) -> BOOL;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn GetMenu(hWnd: HWND) -> HMENU;
    pub fn CheckMenuItem(hMenu: HMENU, uIDCheckItem: u32, uCheck: u32) -> u32;
    pub fn EnableMenuItem(hMenu: HMENU, uIDEnableItem: u32, uEnable: u32) -> BOOL;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> isize;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> isize;
    pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> BOOL;
    pub fn CreateStatusWindowW(style: i32, lpszText: LPCWSTR, hwndParent: HWND, wID: u32) -> HWND;
    pub fn DragAcceptFiles(hWnd: HWND, fAccept: BOOL);
    pub fn DragQueryFileW(hDrop: HDROP, iFile: u32, lpszFile: LPWSTR, cch: u32) -> u32;
    pub fn DragFinish(hDrop: HDROP);
    pub fn GetOpenFileNameW(lpofn: *mut OPENFILENAMEW) -> BOOL;
    pub fn GetSaveFileNameW(lpofn: *mut OPENFILENAMEW) -> BOOL;
    pub fn FindTextW(lpfr: *mut FINDREPLACEW) -> HWND;
    pub fn ReplaceTextW(lpfr: *mut FINDREPLACEW) -> HWND;
    pub fn RegisterWindowMessageW(lpString: LPCWSTR) -> u32;
    pub fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize>,
        dwInitParam: LPARAM,
    ) -> isize;
    pub fn EndDialog(hDlg: HWND, nResult: isize) -> BOOL;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
    pub fn GetDlgItemInt(hDlg: HWND, nIDDlgItem: i32, lpTranslated: *mut BOOL, bSigned: BOOL) -> u32;
    pub fn SetDlgItemInt(hDlg: HWND, nIDDlgItem: i32, uValue: u32, bSigned: BOOL) -> BOOL;
    pub fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn GetObjectW(h: HGDIOBJ, c: i32, pv: *mut std::ffi::c_void) -> i32;
    pub fn ChooseFontW(lpcf: *mut CHOOSEFONTW) -> BOOL;
    pub fn SystemParametersInfoW(
        uiAction: u32,
        uiParam: u32,
        pvParam: *mut std::ffi::c_void,
        fWinIni: u32,
    ) -> BOOL;
    pub fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
    pub fn GetDateFormatW(
        Locale: u32,
        dwFlags: u32,
        lpDate: *const SYSTEMTIME,
        lpFormat: LPCWSTR,
        lpDateStr: LPWSTR,
        cchDate: i32,
    ) -> i32;
    pub fn GetTimeFormatW(
        Locale: u32,
        dwFlags: u32,
        lpTime: *const SYSTEMTIME,
        lpFormat: LPCWSTR,
        lpTimeStr: LPWSTR,
        cchTime: i32,
    ) -> i32;
    pub fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *const std::ffi::c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: isize,
    ) -> isize;
    pub fn ReadFile(
        hFile: isize,
        lpBuffer: *mut u8,
        nNumberOfBytesToRead: u32,
        lpNumberOfBytesRead: *mut u32,
        lpOverlapped: *const std::ffi::c_void,
    ) -> BOOL;
    pub fn WriteFile(
        hFile: isize,
        lpBuffer: *const u8,
        nNumberOfBytesToWrite: u32,
        lpNumberOfBytesWritten: *mut u32,
        lpOverlapped: *const std::ffi::c_void,
    ) -> BOOL;
    pub fn CloseHandle(hObject: isize) -> BOOL;
    pub fn GetFileSizeEx(hFile: isize, lpFileSize: *mut i64) -> BOOL;
    pub fn MultiByteToWideChar(
        CodePage: u32,
        dwFlags: u32,
        lpMultiByteStr: *const u8,
        cbMultiByte: i32,
        lpWideCharStr: LPWSTR,
        cchWideChar: i32,
    ) -> i32;
    pub fn WideCharToMultiByte(
        CodePage: u32,
        dwFlags: u32,
        lpWideCharStr: LPCWSTR,
        cchWideChar: i32,
        lpMultiByteStr: *mut u8,
        cbMultiByte: i32,
        lpDefaultChar: LPCWSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> i32;
    pub fn wsprintfW(lpOut: LPWSTR, lpFmt: LPCWSTR, ...) -> i32;
}

pub fn make_int_resource(id: u16) -> LPCWSTR {
    id as usize as LPCWSTR
}

pub fn wide_from_str(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}
