use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use crate::win32::*;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum TextEncoding {
    #[default]
    Utf8 = 1,
    Utf16Le = 2,
    Utf16Be = 3,
    Ansi = 4,
}

pub fn detect_encoding(data: &[u8]) -> TextEncoding {
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
        return TextEncoding::Utf16Le;
    }
    if data.len() >= 2 && data[0] == 0xFE && data[1] == 0xFF {
        return TextEncoding::Utf16Be;
    }
    if data.len() >= 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
        return TextEncoding::Utf8;
    }
    if std::str::from_utf8(data).is_ok() {
        TextEncoding::Utf8
    } else {
        TextEncoding::Ansi
    }
}

fn decode_to_string(data: &[u8], encoding: TextEncoding) -> Option<String> {
    unsafe {
        match encoding {
            TextEncoding::Utf16Le => {
                if data.len() < 2 {
                    return None;
                }
                let bom_offset = if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
                    2
                } else {
                    0
                };
                let wchar_count = (data.len() - bom_offset) / 2;
                let wchars = std::slice::from_raw_parts(
                    data[bom_offset..].as_ptr() as *const u16,
                    wchar_count,
                );
                Some(String::from_utf16_lossy(wchars))
            }
            TextEncoding::Utf16Be => {
                if data.len() < 2 {
                    return None;
                }
                let bom_offset = if data.len() >= 2 && data[0] == 0xFE && data[1] == 0xFF {
                    2
                } else {
                    0
                };
                let wchars: Vec<u16> = data[bom_offset..]
                    .chunks_exact(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                    .collect();
                Some(String::from_utf16_lossy(&wchars))
            }
            TextEncoding::Utf8 => {
                let bom_offset = if data.len() >= 3
                    && data[0] == 0xEF
                    && data[1] == 0xBB
                    && data[2] == 0xBF
                {
                    3
                } else {
                    0
                };
                String::from_utf8(data[bom_offset..].to_vec()).ok()
            }
            TextEncoding::Ansi => {
                let chars = MultiByteToWideChar(
                    CP_ACP,
                    0,
                    data.as_ptr(),
                    data.len() as i32,
                    std::ptr::null_mut(),
                    0,
                );
                if chars <= 0 {
                    return None;
                }
                let mut buf: Vec<u16> = vec![0; chars as usize];
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    data.as_ptr(),
                    data.len() as i32,
                    buf.as_mut_ptr(),
                    chars,
                );
                String::from_utf16(&buf).ok()
            }
        }
    }
}

fn error_msg(owner: HWND, msg: &str) {
    let title = wide_from_str("retropad");
    let msg_wide = wide_from_str(msg);
    unsafe {
        MessageBoxW(owner, msg_wide.as_ptr(), title.as_ptr(), MB_ICONERROR);
    }
}

pub fn load_text_file(owner: HWND, path: &str) -> Option<(String, TextEncoding)> {
    let path_wide = wide_from_str(path);

    let file = unsafe {
        CreateFileW(
            path_wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    };

    if file == INVALID_HANDLE_VALUE {
        error_msg(owner, "Unable to open file.");
        return None;
    }

    unsafe {
        let mut size: i64 = 0;
        if GetFileSizeEx(file, &mut size) == 0 || size > u32::MAX as i64 {
            CloseHandle(file);
            error_msg(owner, "Unsupported file size.");
            return None;
        }

        let bytes = size as usize;
        let mut buffer: Vec<u8> = vec![0u8; bytes + 3];
        let mut read: u32 = 0;
        if ReadFile(file, buffer.as_mut_ptr(), bytes as u32, &mut read, std::ptr::null()) == 0 {
            CloseHandle(file);
            error_msg(owner, "Failed reading file.");
            return None;
        }
        CloseHandle(file);
        buffer.truncate(read as usize);

        if read == 0 {
            return Some((String::new(), TextEncoding::Utf8));
        }

        let enc = detect_encoding(&buffer);
        match decode_to_string(&buffer, enc) {
            Some(text) => Some((text, enc)),
            None => {
                error_msg(owner, "Unable to decode file.");
                None
            }
        }
    }
}

pub fn save_text_file(owner: HWND, path: &str, text: &str, encoding: TextEncoding) -> bool {
    let path_wide = wide_from_str(path);

    let file = unsafe {
        CreateFileW(
            path_wide.as_ptr(),
            GENERIC_WRITE,
            0,
            std::ptr::null(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            0,
        )
    };

    if file == INVALID_HANDLE_VALUE {
        error_msg(owner, "Unable to create file.");
        return false;
    }

    let ok = match encoding {
        TextEncoding::Utf16Le => write_utf16le(file, text),
        TextEncoding::Utf16Be => write_utf8_with_bom(file, text),
        TextEncoding::Ansi => write_ansi(file, text),
        TextEncoding::Utf8 => write_utf8_with_bom(file, text),
    };

    unsafe {
        CloseHandle(file);
    }

    if !ok {
        error_msg(owner, "Failed writing file.");
    }
    ok
}

fn write_utf8_with_bom(file: isize, text: &str) -> bool {
    let bom: [u8; 3] = [0xEF, 0xBB, 0xBF];
    let mut written: u32 = 0;
    unsafe {
        if WriteFile(file, bom.as_ptr(), 3, &mut written, std::ptr::null()) == 0 {
            return false;
        }
        let bytes = text.as_bytes();
        WriteFile(file, bytes.as_ptr(), bytes.len() as u32, &mut written, std::ptr::null()) != 0
    }
}

fn write_utf16le(file: isize, text: &str) -> bool {
    let bom: [u8; 2] = [0xFF, 0xFE];
    let mut written: u32 = 0;
    unsafe {
        if WriteFile(file, bom.as_ptr(), 2, &mut written, std::ptr::null()) == 0 {
            return false;
        }
        let wide: Vec<u16> = text.encode_utf16().collect();
        let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
        WriteFile(file, bytes.as_ptr(), bytes.len() as u32, &mut written, std::ptr::null()) != 0
    }
}

fn write_ansi(file: isize, text: &str) -> bool {
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let byte_count = WideCharToMultiByte(
            CP_ACP,
            0,
            wide.as_ptr(),
            wide.len() as i32,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
        );
        if byte_count <= 0 {
            return false;
        }
        let mut buf: Vec<u8> = vec![0; byte_count as usize];
        WideCharToMultiByte(
            CP_ACP,
            0,
            wide.as_ptr(),
            wide.len() as i32,
            buf.as_mut_ptr(),
            byte_count,
            std::ptr::null(),
            std::ptr::null_mut(),
        );
        let mut written: u32 = 0;
        WriteFile(
            file,
            buf.as_ptr(),
            buf.len() as u32,
            &mut written,
            std::ptr::null(),
        ) != 0
    }
}

pub fn open_file_dialog(owner: HWND) -> Option<String> {
    let mut buf: Vec<u16> = vec![0u16; MAX_PATH];
    let title = wide_from_str("retropad");

    let filter: Vec<u16> = {
        let txt = "Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
        let mut result = Vec::new();
        for part in txt.split('\0') {
            result.extend(OsStr::new(part).encode_wide());
            result.push(0);
        }
        result.push(0);
        result
    };

    let def_ext = wide_from_str("txt");

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        hInstance: 0,
        lpstrFilter: filter.as_ptr(),
        lpstrCustomFilter: std::ptr::null_mut(),
        nMaxCustFilter: 0,
        nFilterIndex: 0,
        lpstrFile: buf.as_mut_ptr(),
        nMaxFile: buf.len() as u32,
        lpstrFileTitle: std::ptr::null_mut(),
        nMaxFileTitle: 0,
        lpstrInitialDir: std::ptr::null(),
        lpstrTitle: title.as_ptr(),
        Flags: OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST,
        nFileOffset: 0,
        nFileExtension: 0,
        lpstrDefExt: def_ext.as_ptr(),
        lCustData: 0,
        lpfnHook: None,
        lpTemplateName: std::ptr::null(),
    };

    unsafe {
        if GetOpenFileNameW(&mut ofn) != 0 {
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..end]))
        } else {
            None
        }
    }
}

pub fn save_file_dialog(owner: HWND, current_path: &str) -> Option<String> {
    let mut buf: Vec<u16> = vec![0u16; MAX_PATH];

    if !current_path.is_empty() {
        let wide = wide_from_str(current_path);
        let copy_len = wide.len().min(buf.len());
        buf[..copy_len].copy_from_slice(&wide[..copy_len]);
    } else {
        let default = wide_from_str("*.txt");
        let copy_len = default.len().min(buf.len());
        buf[..copy_len].copy_from_slice(&default[..copy_len]);
    }

    let title = wide_from_str("retropad");

    let filter: Vec<u16> = {
        let txt = "Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0";
        let mut result = Vec::new();
        for part in txt.split('\0') {
            result.extend(OsStr::new(part).encode_wide());
            result.push(0);
        }
        result.push(0);
        result
    };

    let def_ext = wide_from_str("txt");

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        hInstance: 0,
        lpstrFilter: filter.as_ptr(),
        lpstrCustomFilter: std::ptr::null_mut(),
        nMaxCustFilter: 0,
        nFilterIndex: 0,
        lpstrFile: buf.as_mut_ptr(),
        nMaxFile: buf.len() as u32,
        lpstrFileTitle: std::ptr::null_mut(),
        nMaxFileTitle: 0,
        lpstrInitialDir: std::ptr::null(),
        lpstrTitle: title.as_ptr(),
        Flags: OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST,
        nFileOffset: 0,
        nFileExtension: 0,
        lpstrDefExt: def_ext.as_ptr(),
        lCustData: 0,
        lpfnHook: None,
        lpTemplateName: std::ptr::null(),
    };

    unsafe {
        if GetSaveFileNameW(&mut ofn) != 0 {
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..end]))
        } else {
            None
        }
    }
}
