//! Origin of one compiled CSS rule from a Sass source map.
//!
//! A compiled rule is planned only when that origin is a file under
//! `ref/mempool/frontend`. The decoder does not guess from selector names.

/// Embedded source map taken from compiler CSS.
pub struct EmbeddedSourceMap {
    sources: Vec<String>,
    lines: Vec<Vec<MappingSegment>>,
}

struct MappingSegment {
    generated_column: u32,
    source_index: Option<usize>,
}

/// Parses `sourceMappingURL=` when Sass embedded it in the CSS.
/// `None` means this CSS did not name a source map.
pub fn parse_embedded_source_map(css: &str) -> Option<EmbeddedSourceMap> {
    let marker = "sourceMappingURL=";
    let start = css.rfind(marker)?;
    let mut payload = css[start + marker.len()..].trim();
    if let Some(end) = payload.find("*/") {
        payload = payload[..end].trim();
    }
    let json = decode_source_map_payload(payload)?;
    parse_source_map_json(&json)
}

/// Source file for generated line and column, both 0-based.
/// `None` when this position has no named origin.
pub fn origin_at(map: &EmbeddedSourceMap, line: u32, column: u32) -> Option<&str> {
    let segments = map.lines.get(line as usize)?;
    let mut chosen: Option<usize> = None;
    let mut found = false;
    for segment in segments {
        if segment.generated_column > column {
            break;
        }
        found = true;
        chosen = segment.source_index;
    }
    if !found {
        return None;
    }
    let source = map.sources.get(chosen?)?;
    if source.is_empty() {
        None
    } else {
        Some(source.as_str())
    }
}

/// True when `origin` names a file outside `ref/mempool/frontend`.
/// The style proof skips that compiled rule. An empty origin is not skipped.
pub fn skip_non_upstream_origin(origin: &str) -> bool {
    let path = origin_file_path(origin);
    if path.is_empty() {
        return false;
    }
    !path_is_under_mempool_frontend(&path)
}

fn path_is_under_mempool_frontend(path: &str) -> bool {
    let path = path.replace('\\', "/");
    path.contains("/ref/mempool/frontend/") || path.ends_with("/ref/mempool/frontend")
}

fn origin_file_path(origin: &str) -> String {
    let decoded = percent_decode(origin.trim());
    let Some(rest) = decoded.strip_prefix("file://") else {
        return decoded;
    };
    if let Some(after_host) = rest.strip_prefix("localhost") {
        if let Some(path) = after_host.strip_prefix('/') {
            return format!("/{path}");
        }
    }
    if rest.starts_with('/') {
        rest.to_string()
    } else {
        decoded
    }
}

fn parse_source_map_json(json: &str) -> Option<EmbeddedSourceMap> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let root = value
        .get("sourceRoot")
        .and_then(|item| item.as_str())
        .unwrap_or("");
    let sources_json = value.get("sources")?.as_array()?;
    let sources = sources_json
        .iter()
        .map(|item| match item.as_str() {
            Some(source) => join_source_root(root, source),
            None => String::new(),
        })
        .collect::<Vec<_>>();
    let mappings = value
        .get("mappings")
        .and_then(|item| item.as_str())
        .unwrap_or("");
    let lines = decode_mappings(mappings, sources.len());
    Some(EmbeddedSourceMap { sources, lines })
}

fn join_source_root(root: &str, source: &str) -> String {
    if root.is_empty() || source.contains("://") || source.starts_with('/') {
        return source.to_string();
    }
    let root = root.trim_end_matches('/');
    format!("{root}/{source}")
}

fn decode_source_map_payload(url: &str) -> Option<String> {
    let rest = url.trim().strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    if meta
        .split(';')
        .any(|part| part.eq_ignore_ascii_case("base64"))
    {
        let bytes = decode_base64(data.trim())?;
        String::from_utf8(bytes).ok()
    } else {
        Some(percent_decode(data.trim()))
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    fn value(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    let mut filled = 0usize;
    for byte in input.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b'=' {
            break;
        }
        buf[filled] = value(byte)?;
        filled += 1;
        if filled == 4 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            out.push((buf[1] << 4) | (buf[2] >> 2));
            out.push((buf[2] << 6) | buf[3]);
            filled = 0;
        }
    }
    if filled == 2 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
    } else if filled == 3 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
        out.push((buf[1] << 4) | (buf[2] >> 2));
    } else if filled != 0 {
        return None;
    }
    Some(out)
}

fn decode_mappings(mappings: &str, source_count: usize) -> Vec<Vec<MappingSegment>> {
    let chars: Vec<char> = mappings.chars().collect();
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut index = 0usize;
    let mut generated_column = 0i64;
    let mut source_index = 0i64;
    let mut original_line = 0i64;
    let mut original_column = 0i64;
    let mut name_index = 0i64;
    while index < chars.len() {
        match chars[index] {
            ';' => {
                line.sort_by_key(|segment: &MappingSegment| segment.generated_column);
                lines.push(std::mem::take(&mut line));
                generated_column = 0;
                index += 1;
            }
            ',' => index += 1,
            _ => {
                let mut fields = Vec::new();
                let start = index;
                while index < chars.len() && chars[index] != ',' && chars[index] != ';' {
                    let Some(field) = decode_vlq(&chars, &mut index) else {
                        if index == start {
                            index += 1;
                        }
                        break;
                    };
                    fields.push(field);
                }
                if fields.is_empty() {
                    continue;
                }
                generated_column += fields[0];
                let mut mapped = None;
                if fields.len() >= 4 {
                    source_index += fields[1];
                    original_line += fields[2];
                    original_column += fields[3];
                    if fields.len() >= 5 {
                        name_index += fields[4];
                    }
                    if source_index >= 0 {
                        let idx = source_index as usize;
                        if idx < source_count {
                            mapped = Some(idx);
                        }
                    }
                }
                if generated_column >= 0 {
                    line.push(MappingSegment {
                        generated_column: generated_column as u32,
                        source_index: mapped,
                    });
                }
            }
        }
    }
    line.sort_by_key(|segment: &MappingSegment| segment.generated_column);
    lines.push(line);
    let _state = (original_line, original_column, name_index);
    lines
}

fn decode_vlq(chars: &[char], index: &mut usize) -> Option<i64> {
    let mut value = 0i64;
    let mut shift = 0;
    loop {
        if *index >= chars.len() {
            return None;
        }
        let digit = vlq_digit(chars[*index])?;
        *index += 1;
        value += (digit & 31) << shift;
        if digit & 32 == 0 {
            break;
        }
        shift += 5;
        if shift > 32 {
            return None;
        }
    }
    let negate = value & 1 == 1;
    value >>= 1;
    if negate {
        value = -value;
    }
    Some(value)
}

fn vlq_digit(ch: char) -> Option<i64> {
    let value = match ch {
        'A'..='Z' => ch as i64 - 'A' as i64,
        'a'..='z' => ch as i64 - 'a' as i64 + 26,
        '0'..='9' => ch as i64 - '0' as i64 + 52,
        '+' => 62,
        '/' => 63,
        _ => return None,
    };
    Some(value)
}
