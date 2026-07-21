//! Modified Run-Length Encoding (RLE) for radar-volume byte streams.
//!
//! Algorithm from Ru, Y. (2007), *Volumetric Visualization of NEXRAD Level II
//! Doppler Weather Data from Multiple Sites* (Purdue University) — see
//! `docs/post-v1-multi-site-animation.md`. The thesis reports 99%+
//! compression on sparse volumetric radar data (mostly-empty gates).
//!
//! ## Format
//!
//! The stream is a sequence of runs, each starting with a control byte:
//!
//! - **Repeat run** (bit 7 set): control byte `0x80`, followed by 1 value
//!   byte and a 32-bit little-endian run count. Used for runs of 2+
//!   identical bytes — the common case for sparse radar data, where long
//!   stretches of below-threshold gates encode to runs of `0x00`.
//! - **Literal run** (bit 7 clear): the lower 7 bits of the control byte
//!   are a length (1..=127), followed by that many raw bytes copied as-is.
//!   Used for non-repeating data, where a repeat run would cost more than
//!   it saves.
//!
//! This is a byte-level codec — callers who want to compress structured
//! data (like `ScanData`) must first flatten it to bytes in a layout that
//! puts identical values (e.g. a `None` marker byte) in long runs.

/// Minimum run length worth encoding as a repeat run (control + value + u32
/// count = 6 bytes, so anything shorter than that is cheaper as a literal).
const MIN_REPEAT: u32 = 6;

/// Compress `data` using the modified RLE scheme described above.
pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut literal: Vec<u8> = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let byte = data[i];
        let mut run_len = 1u32;
        while i + (run_len as usize) < data.len() && data[i + run_len as usize] == byte {
            run_len += 1;
        }

        if run_len >= MIN_REPEAT {
            flush_literal(&mut out, &mut literal);
            out.push(0x80);
            out.push(byte);
            out.extend_from_slice(&run_len.to_le_bytes());
        } else {
            for _ in 0..run_len {
                literal.push(byte);
                if literal.len() == 127 {
                    flush_literal(&mut out, &mut literal);
                }
            }
        }

        i += run_len as usize;
    }
    flush_literal(&mut out, &mut literal);
    out
}

fn flush_literal(out: &mut Vec<u8>, literal: &mut Vec<u8>) {
    if literal.is_empty() {
        return;
    }
    out.push(literal.len() as u8); // bit 7 clear, len <= 127
    out.extend_from_slice(literal);
    literal.clear();
}

/// Decompress a stream produced by [`compress`]. Returns an error string on
/// truncated/corrupt input (e.g. a corrupt cache entry).
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let control = data[i];
        i += 1;

        if control & 0x80 != 0 {
            let value = *data.get(i).ok_or("rle: truncated repeat run (value)")?;
            i += 1;
            let count_bytes: [u8; 4] = data
                .get(i..i + 4)
                .ok_or("rle: truncated repeat run (count)")?
                .try_into()
                .map_err(|_| "rle: truncated repeat run (count)")?;
            i += 4;
            let count = u32::from_le_bytes(count_bytes);
            out.resize(out.len() + count as usize, value);
        } else {
            let len = control as usize; // lower 7 bits == control, since bit 7 is clear
            let literal = data
                .get(i..i + len)
                .ok_or("rle: truncated literal run")?;
            out.extend_from_slice(literal);
            i += len;
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{compress, decompress};

    #[test]
    fn round_trips_empty() {
        let data: Vec<u8> = vec![];
        assert_eq!(decompress(&compress(&data)).unwrap(), data);
    }

    #[test]
    fn round_trips_mixed_runs() {
        let mut data = vec![0u8; 200]; // long repeat run
        data.extend([1, 2, 3, 4, 5]); // literal run
        data.extend(vec![9u8; 3]); // run below MIN_REPEAT threshold
        data.extend(vec![7u8; 1000]); // long repeat run
        assert_eq!(decompress(&compress(&data)).unwrap(), data);
    }

    #[test]
    fn round_trips_all_literal() {
        let data: Vec<u8> = (0..=255u8).collect();
        assert_eq!(decompress(&compress(&data)).unwrap(), data);
    }

    #[test]
    fn round_trips_long_literal_run_over_127_bytes() {
        // Forces flush_literal to split across multiple literal packets.
        let data: Vec<u8> = (0..300).map(|i| (i % 250) as u8).collect();
        assert_eq!(decompress(&compress(&data)).unwrap(), data);
    }

    #[test]
    fn rejects_truncated_input() {
        assert!(decompress(&[0x80]).is_err());
        assert!(decompress(&[0x80, 5, 1, 2, 3]).is_err()); // count needs 4 bytes
        assert!(decompress(&[3, 1, 2]).is_err()); // literal len 3, only 2 bytes present
    }

    #[test]
    fn compresses_sparse_data_by_at_least_90_percent() {
        // Simulates a mostly-empty gate stream: 1 zero byte per below-
        // threshold gate, 5 bytes (tag + f32) per real return, matching the
        // byte layout `cache::scan_to_bytes` uses for `Option<f32>` gates.
        // 1-in-200 gates carrying a value is realistic for a real radar
        // volume, where most gates at range are below threshold.
        let mut data = Vec::new();
        for i in 0..50_000u32 {
            if i % 200 == 0 {
                data.push(1);
                data.extend_from_slice(&(i as f32).to_le_bytes());
            } else {
                data.push(0);
            }
        }
        let compressed = compress(&data);
        assert!(
            compressed.len() * 10 < data.len(),
            "expected >=90% savings: {} -> {} bytes",
            data.len(),
            compressed.len()
        );
        assert_eq!(decompress(&compressed).unwrap(), data);
    }
}
