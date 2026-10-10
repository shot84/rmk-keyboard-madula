//! Decode the 0x1014..0x1023 IQS9151 relative-coordinate block.
pub fn decode(frame: &[u8; 16]) -> Option<(i16, i16, u8)> {
    if frame[12] & 0x80 != 0 { return None; }
    let fingers = frame[14] & 0xf;
    if fingers != 1 || frame[13] & 2 == 0 { return Some((0, 0, fingers)); }
    Some((i16::from_le_bytes([frame[0], frame[1]]),
          i16::from_le_bytes([frame[2], frame[3]]), fingers))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_little_endian() {
        let mut f = [0; 16]; f[13] = 2; f[14] = 1;
        f[..2].copy_from_slice(&(-300i16).to_le_bytes());
        f[2..4].copy_from_slice(&400i16.to_le_bytes());
        assert_eq!(decode(&f), Some((-300, 400, 1)));
    }
    #[test]
    fn reset_discards_frame() {
        let mut f = [0; 16]; f[12] = 0x80;
        assert_eq!(decode(&f), None);
    }
    #[test]
    fn inactive_and_multifinger_cannot_move_pointer() {
        let mut f = [0xff; 16]; f[12] = 0; f[13] = 2;
        for n in [0, 2, 3, 15] { f[14] = n; assert_eq!(decode(&f), Some((0, 0, n))); }
        f[14] = 1; f[13] = 0;
        assert_eq!(decode(&f), Some((0, 0, 1)));
    }
}
