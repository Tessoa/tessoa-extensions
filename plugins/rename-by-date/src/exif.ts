// A minimal EXIF reader: the capture date of a JPEG, nothing else.
//
// The capture date is DateTimeOriginal (tag 0x9003) in the EXIF sub-IFD. When
// it is missing, IFD0's DateTime (tag 0x0132) is used instead. The two can
// differ, and a reader that gets the sub-IFD pointer wrong silently returns
// the wrong one; the fixture DSC_9999.jpeg has different values for the two
// tags for exactly that reason.

/** The capture date as EXIF writes it, "YYYY:MM:DD HH:MM:SS", or null. */
export function captureDate(bytes: Uint8Array): string | null {
  if (bytes.length < 4 || bytes[0] !== 0xff || bytes[1] !== 0xd8) return null;
  let i = 2;
  while (i + 4 <= bytes.length) {
    if (bytes[i] !== 0xff) return null;
    const marker = bytes[i + 1];
    const len = (bytes[i + 2] << 8) | bytes[i + 3];
    if (marker === 0xe1) {
      const end = Math.min(i + 2 + len, bytes.length);
      const segment = bytes.subarray(Math.min(i + 4, end), end);
      if (segment.length > 6 && ascii(segment, 0, 4) === "Exif" && segment[4] === 0 && segment[5] === 0) {
        return parseTiff(segment.subarray(6));
      }
    }
    if (marker === 0xda) return null; // start of scan: no more headers
    i += 2 + len;
  }
  return null;
}

function ascii(b: Uint8Array, offset: number, length: number): string {
  let s = "";
  for (let k = offset; k < offset + length && k < b.length; k++) s += String.fromCharCode(b[k]);
  return s;
}

function parseTiff(t: Uint8Array): string | null {
  if (t.length < 8) return null;
  const order = ascii(t, 0, 2);
  const little = order === "II";
  if (!little && order !== "MM") return null;
  const view = new DataView(t.buffer, t.byteOffset, t.byteLength);
  const r16 = (o: number) => (o + 2 <= t.length ? view.getUint16(o, little) : -1);
  const r32 = (o: number) => (o + 4 <= t.length ? view.getUint32(o, little) : -1);
  if (r16(2) !== 42) return null;
  const ifd0 = r32(4);

  // An entry: [type, count, offset of the value slot].
  const find = (ifd: number, tag: number): [number, number, number] | null => {
    const n = r16(ifd);
    if (n < 0) return null;
    for (let k = 0; k < Math.min(n, 4096); k++) {
      const e = ifd + 2 + k * 12;
      if (r16(e) === tag) return [r16(e + 2), r32(e + 4), e + 8];
    }
    return null;
  };
  const asciiValue = (entry: [number, number, number] | null): string | null => {
    if (!entry || entry[0] !== 2) return null;
    const count = entry[1];
    // Values of four bytes or fewer live in the slot itself.
    const offset = count <= 4 ? entry[2] : r32(entry[2]);
    if (offset < 0 || offset + count > t.length) return null;
    const s = ascii(t, offset, count).replace(/\0+$/, "").trim();
    return s.length >= 19 && /^\d/.test(s) ? s : null;
  };

  // The EXIF sub-IFD pointer is a LONG with count 1, so the pointer is the
  // value slot itself, not the count field before it.
  const exifPointer = find(ifd0, 0x8769);
  if (exifPointer) {
    const date = asciiValue(find(r32(exifPointer[2]), 0x9003));
    if (date) return date;
  }
  return asciiValue(find(ifd0, 0x0132));
}
