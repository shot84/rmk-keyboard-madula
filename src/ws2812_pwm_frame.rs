//! Two GRB pixels, 16 MHz / 20 ticks per bit; 300 us low latch tail.
pub fn encode(rgb: [u8; 3]) -> [u16; 288] {
    encode_pixels([rgb,rgb])
}
pub fn encode_pixels(pixels: [[u8;3];2]) -> [u16;288] {
    let mut words = [0x8000; 288];
    let [a,b] = pixels;
    let bytes = [a[1],a[0],a[2],b[1],b[0],b[2]];
    for (i, byte) in bytes.into_iter().enumerate() {
        for bit in 0..8 {
            words[i * 8 + bit] = 0x8000 | if byte & (0x80 >> bit) != 0 { 13 } else { 6 };
        }
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_pixels() {
        let f = encode_pixels([[0;3],[255;3]]);
        assert!(f[..24].iter().all(|&v| v == (0x8000 | 6)));
        assert!(f[24..48].iter().all(|&v| v == (0x8000 | 13)));
    }
    #[test]
    fn grb_polarity_and_latch() {
        let f = encode([0x80, 1, 0]);
        assert_eq!(&f[..24], &f[24..48]);
        assert_eq!(f[7], 0x8000 | 13);
        assert_eq!(f[8], 0x8000 | 13);
        assert_eq!(f[..48].iter().filter(|&&v| v == 0x8000 | 13).count(), 4);
        assert!(f[48..].iter().all(|&v| v == 0x8000));
        assert!(encode([0; 3])[..48].iter().all(|&v| v == 0x8000 | 6));
    }
}
