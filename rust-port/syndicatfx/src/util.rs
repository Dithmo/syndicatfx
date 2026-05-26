// util.rs — mirrors util.c: string helpers, path segment extraction.

pub fn strtolower(s: &mut [u8]) {
    for b in s.iter_mut() {
        if b.is_ascii_uppercase() {
            *b = b.to_ascii_lowercase();
        }
    }
}

pub fn strtocapwords(s: &mut [u8]) {
    let mut word_start = true;
    for b in s.iter_mut() {
        if *b == 0 { break; }
        if word_start {
            if b.is_ascii_alphabetic() {
                *b = b.to_ascii_uppercase();
                word_start = false;
            }
        } else {
            *b = b.to_ascii_lowercase();
            if *b == b' ' { word_start = true; }
        }
    }
}

/// Extract one path segment from `path`, writing it into `buf`.
/// Returns the remainder of `path` after the segment (or None if at end).
pub fn extract_path_segment<'a>(path: &'a str, buf: &mut [u8]) -> Option<&'a str> {
    let sep = std::path::MAIN_SEPARATOR;
    if let Some(pos) = path.find(sep) {
        let seg = &path[..pos];
        let seg_bytes = seg.as_bytes();
        let len = seg_bytes.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&seg_bytes[..len]);
        buf[len] = 0;
        let rest = &path[pos + 1..];
        if rest.is_empty() { None } else { Some(rest) }
    } else {
        let bytes = path.as_bytes();
        let len = bytes.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&bytes[..len]);
        buf[len] = 0;
        None
    }
}
