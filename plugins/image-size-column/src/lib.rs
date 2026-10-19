//! Example plugin: a column with the width and height of PNG and JPEG images.
//!
//! The user adds the column "Dimensions" from the column menu of tessoa's
//! file list. tessoa asks the plugin only for the files on screen, and only
//! for those the manifest's `when` admits (`.png`, `.jpg`, `.jpeg`); it keeps
//! the values until a file changes. The value is text such as `1920×1080`
//! that sorts by the number of pixels, so the largest images come last.
//!
//! The values are computed in the plugin's column worker, never in a command.
//! For each file the column reads only the few bytes that say how large the
//! image is: the start of a PNG, or the headers of a JPEG one after the
//! other until the frame header, skipping the rest (a camera's EXIF block
//! alone can be 64 KiB). A file that is not such an image, or that cannot be
//! read, shows nothing. The size is the one stored in the file; the EXIF
//! orientation that turns a photo upright is not applied.
//!
//! Reading the files of the list needs `files.read-listed`, the only
//! permission the manifest declares. The plugin has no commands.

use tessoa_plugin_sdk::prelude::*;

struct ImageSize;

impl Plugin for ImageSize {
    fn run_command(command_id: &str) -> Result<(), String> {
        Err(format!("unknown command {command_id}: this plugin has a column only"))
    }
}

/// The column's values. The worker keeps this value from one request to the
/// next; this column has nothing to remember.
struct Dimensions;

impl Columns for Dimensions {
    fn value(&mut self, column: &str, file: &ColumnFile) -> ColumnValue {
        if column != "dimensions" {
            return ColumnValue::Empty;
        }
        // The handle is valid while tessoa waits for this request.
        let mut read = |offset, len| files::read(file.handle, offset, len).unwrap_or_default();
        match image_size(&mut read) {
            Some((w, h)) => ColumnValue::sorted_text(format!("{w}×{h}"), f64::from(w) * f64::from(h)),
            None => ColumnValue::Empty,
        }
    }
}

tessoa_plugin_sdk::export!(ImageSize; columns: Dimensions);

/// The width and height of a PNG or JPEG image. `read(offset, len)` returns
/// up to `len` bytes of the file from `offset`: fewer at its end, none when
/// it cannot be read.
pub fn image_size(read: &mut dyn FnMut(u64, u32) -> Vec<u8>) -> Option<(u32, u32)> {
    let head = read(0, 24);
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        png_size(&head)
    } else if head.starts_with(&[0xFF, 0xD8]) {
        jpeg_size(read)
    } else {
        None
    }
}

/// A PNG starts with its signature and the IHDR chunk, whose first fields
/// are the width and the height.
fn png_size(head: &[u8]) -> Option<(u32, u32)> {
    if head.len() < 24 || &head[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([head[16], head[17], head[18], head[19]]);
    let h = u32::from_be_bytes([head[20], head[21], head[22], head[23]]);
    (w > 0 && h > 0).then_some((w, h))
}

/// The most headers a JPEG may have before its frame header; a file with
/// more is taken for something else.
const MAX_SEGMENTS: usize = 256;

/// A JPEG is a row of segments, each `FF`, a marker and, for most markers, a
/// length. The frame header (a start-of-frame marker) holds the height and
/// the width; the image data starts at the start-of-scan marker, after it.
fn jpeg_size(read: &mut dyn FnMut(u64, u32) -> Vec<u8>) -> Option<(u32, u32)> {
    let mut at = 2;
    for _ in 0..MAX_SEGMENTS {
        let m = read(at, 4);
        if m.len() < 2 || m[0] != 0xFF {
            return None;
        }
        match m[1] {
            // A fill byte before a marker.
            0xFF => {
                at += 1;
                continue;
            }
            // Markers without a length.
            0x01 | 0xD0..=0xD8 => {
                at += 2;
                continue;
            }
            // The end of the image, or its data, before any frame header.
            0xD9 | 0xDA => return None,
            _ => {}
        }
        if m.len() < 4 {
            return None;
        }
        let len = u16::from_be_bytes([m[2], m[3]]);
        if len < 2 {
            return None;
        }
        if is_start_of_frame(m[1]) {
            // FF, marker, length (2 bytes), precision (1), height, width.
            let f = read(at + 5, 4);
            if f.len() < 4 {
                return None;
            }
            let h = u32::from(u16::from_be_bytes([f[0], f[1]]));
            let w = u32::from(u16::from_be_bytes([f[2], f[3]]));
            // A height of 0 is given later in the file; such files are rare.
            return (w > 0 && h > 0).then_some((w, h));
        }
        at += 2 + u64::from(len);
    }
    None
}

/// The start-of-frame markers: C0 to CF, except C4 (Huffman tables), C8
/// (reserved) and CC (arithmetic coding).
fn is_start_of_frame(marker: u8) -> bool {
    (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC)
}

#[cfg(test)]
mod tests {
    use super::image_size;

    /// The size of an image held in memory, and how many bytes were read.
    fn size_of(bytes: &[u8]) -> (Option<(u32, u32)>, usize) {
        let mut read_bytes = 0;
        let mut read = |offset: u64, len: u32| {
            let rest = bytes.get(offset as usize..).unwrap_or_default();
            let piece = rest[..rest.len().min(len as usize)].to_vec();
            read_bytes += piece.len();
            piece
        };
        let size = image_size(&mut read);
        (size, read_bytes)
    }

    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&h.to_be_bytes());
        b.extend_from_slice(&[1, 0, 0, 0, 0]);
        b
    }

    fn segment(marker: u8, body: &[u8]) -> Vec<u8> {
        let mut s = vec![0xFF, marker];
        s.extend_from_slice(&(body.len() as u16 + 2).to_be_bytes());
        s.extend_from_slice(body);
        s
    }

    fn frame(marker: u8, w: u16, h: u16) -> Vec<u8> {
        let mut body = vec![8];
        body.extend_from_slice(&h.to_be_bytes());
        body.extend_from_slice(&w.to_be_bytes());
        body.extend_from_slice(&[1, 1, 0x11, 0]);
        segment(marker, &body)
    }

    fn jpeg(parts: &[Vec<u8>]) -> Vec<u8> {
        let mut b = vec![0xFF, 0xD8];
        for p in parts {
            b.extend_from_slice(p);
        }
        b.extend_from_slice(&[0xFF, 0xD9]);
        b
    }

    #[test]
    fn reads_the_size_of_a_png() {
        assert_eq!(size_of(&png(1920, 1080)).0, Some((1920, 1080)));
        assert_eq!(size_of(&png(0, 1080)).0, None);
        assert_eq!(size_of(&png(1920, 1080)[..20]).0, None);
    }

    #[test]
    fn skips_the_headers_of_a_jpeg_without_reading_them() {
        let exif = segment(0xE1, &vec![0; 60_000]);
        let tables = segment(0xDB, &[0; 65]);
        let photo = jpeg(&[exif, tables, frame(0xC0, 4032, 3024), segment(0xDA, &[0; 10])]);
        let (size, read) = size_of(&photo);
        assert_eq!(size, Some((4032, 3024)));
        assert!(read < 100, "read {read} bytes of {}", photo.len());
    }

    #[test]
    fn reads_a_progressive_jpeg_with_fill_bytes() {
        // One fill byte: a reader that skips two lands past the marker's FF.
        let mut fill = vec![0xFF];
        fill.extend(frame(0xC2, 2480, 3508));
        assert_eq!(size_of(&jpeg(&[segment(0xE0, b"JFIF\0"), fill])).0, Some((2480, 3508)));
    }

    #[test]
    fn finds_nothing_in_what_is_not_such_an_image() {
        // The data starts before any frame header.
        assert_eq!(size_of(&jpeg(&[segment(0xDA, &[0; 4]), frame(0xC0, 10, 10)])).0, None);
        // Only the start of the image.
        assert_eq!(size_of(&jpeg(&[])).0, None);
        assert_eq!(size_of(&[0xFF, 0xD8, 0xFF]).0, None);
        // A table of Huffman codes is not a frame header.
        assert_eq!(size_of(&jpeg(&[frame(0xC4, 10, 10)])).0, None);
        assert_eq!(size_of(b"GIF89a\x10\0\x10\0").0, None);
        assert_eq!(size_of(b"").0, None);
    }
}
