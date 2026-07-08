use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use crate::win32::*;

pub fn get_edit_text(edit: HWND) -> Option<String> {
    unsafe {
        let len = GetWindowTextLengthW(edit);
        if len <= 0 {
            return Some(String::new());
        }
        let mut buf: Vec<u16> = vec![0u16; (len + 1) as usize];
        GetWindowTextW(edit, buf.as_mut_ptr(), len + 1);
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16(&buf[..end]).ok()
    }
}

pub fn get_edit_sel(edit: HWND) -> (usize, usize) {
    unsafe {
        let mut start: u32 = 0;
        let mut end: u32 = 0;
        SendMessageW(
            edit,
            EM_GETSEL,
            &mut start as *mut u32 as usize,
            &mut end as *mut u32 as isize,
        );
        (start as usize, end as usize)
    }
}

pub fn select_match(edit: HWND, start: usize, end: usize) {
    unsafe {
        SendMessageW(edit, EM_SETSEL, start, end as isize);
        SendMessageW(edit, EM_SCROLLCARET, 0, 0);
    }
}

pub fn find_in_edit(
    edit: HWND,
    needle: &str,
    match_case: bool,
    search_down: bool,
    start_pos: usize,
) -> Option<(usize, usize)> {
    if needle.is_empty() {
        return None;
    }

    let text = get_edit_text(edit)?;
    let needle_len = needle.chars().count();
    let text_len = text.chars().count();

    let haystack: Vec<char> = if match_case {
        text.chars().collect()
    } else {
        text.to_lowercase().chars().collect()
    };
    let needle_chars: Vec<char> = if match_case {
        needle.chars().collect()
    } else {
        needle.to_lowercase().chars().collect()
    };

    let start = start_pos.min(text_len);

    if search_down {
        for i in start..haystack.len() {
            if haystack[i..].starts_with(&needle_chars) {
                return Some((i, i + needle_len));
            }
        }
        if start > 0 {
            for i in 0..start {
                if haystack[i..].starts_with(&needle_chars) {
                    return Some((i, i + needle_len));
                }
            }
        }
    } else {
        let mut found: Option<usize> = None;
        for i in 0..start {
            if haystack[i..].starts_with(&needle_chars) {
                found = Some(i);
            }
        }
        if let Some(idx) = found {
            return Some((idx, idx + needle_len));
        }
        for i in start..haystack.len() {
            if haystack[i..].starts_with(&needle_chars) {
                found = Some(i);
            }
        }
        if let Some(idx) = found {
            return Some((idx, idx + needle_len));
        }
    }

    None
}

pub fn replace_all_occurrences(
    edit: HWND,
    needle: &str,
    replacement: &str,
    match_case: bool,
) -> usize {
    if needle.is_empty() {
        return 0;
    }

    let text = match get_edit_text(edit) {
        Some(t) => t,
        None => return 0,
    };

    let nchars: Vec<char> = text.chars().collect();
    let haystack: Vec<char> = if match_case {
        nchars.clone()
    } else {
        text.to_lowercase().chars().collect()
    };
    let needle_chars: Vec<char> = if match_case {
        needle.chars().collect()
    } else {
        needle.to_lowercase().chars().collect()
    };
    let rchars: Vec<char> = replacement.chars().collect();

    let needle_len = needle_chars.len();
    let mut result: Vec<char> = Vec::with_capacity(nchars.len());
    let mut count: usize = 0;
    let mut src_idx = 0;
    let mut i = 0;

    while i + needle_len <= haystack.len() {
        if haystack[i..].starts_with(&needle_chars) {
            result.extend_from_slice(&nchars[src_idx..i]);
            result.extend_from_slice(&rchars);
            src_idx = i + needle_len;
            i += needle_len;
            count += 1;
        } else {
            i += 1;
        }
    }

    if count == 0 {
        return 0;
    }

    result.extend_from_slice(&nchars[src_idx..]);

    let result_str: String = result.into_iter().collect();
    let wide: Vec<u16> = OsStr::new(&result_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        SendMessageW(edit, 0x000C /* WM_SETTEXT */, 0, wide.as_ptr() as isize);
        SendMessageW(edit, EM_SETSEL, 0, -1isize as isize);
        SendMessageW(edit, EM_SCROLLCARET, 0, 0);
    }

    count
}
