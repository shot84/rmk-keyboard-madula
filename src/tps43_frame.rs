//! IQS5xx 0x000C motion block, big endian. Cornix enclosure rotation.
#[derive(Default, Debug, PartialEq)]
pub struct Frame {
    pub pointer: [i16; 2],
    pub scroll: [i16; 2],
    pub tap: u8,
    pub hold: bool,
}
#[derive(Default)]
pub struct Decoder {
    remainder: [i32; 2],
}
impl Decoder {
    pub fn decode(&mut self, data: [u8; 10]) -> Option<Frame> {
        // Reset, ATI error, or palm/too many fingers: discard stale input.
        if data[3] & 0x88 != 0 {
            self.remainder = [0; 2];
            return None;
        }
        if data[4] & 6 != 0 {
            self.remainder = [0; 2];
            return Some(Frame::default());
        }
        let x = i16::from_be_bytes([data[6], data[7]]);
        let y = i16::from_be_bytes([data[8], data[9]]);
        let mut out = Frame::default();
        out.tap = if data[1] & 1 != 0 {
            1
        } else if data[2] & 1 != 0 {
            2
        } else {
            0
        };
        out.hold = data[1] & 2 != 0;
        if data[2] & 2 != 0 {
            // ZMK natural-scroll: raw (x,-y), followed by swap+invert both.
            let motion = [i32::from(y), -i32::from(x)];
            for axis in 0..2 {
                self.remainder[axis] += motion[axis];
                out.scroll[axis] = (self.remainder[axis] / 32).clamp(-8, 8) as i16;
                self.remainder[axis] %= 32;
            }
        } else {
            self.remainder = [0; 2];
            if data[4] & 1 != 0 {
                out.pointer = [y.saturating_neg(), x.saturating_neg()];
            }
        }
        Some(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(x: i16, y: i16) -> [u8; 10] {
        let mut d = [0; 10];
        d[4] = 1;
        d[6..8].copy_from_slice(&x.to_be_bytes());
        d[8..10].copy_from_slice(&y.to_be_bytes());
        d
    }
    #[test]
    fn signed_rotation() {
        assert_eq!(
            Decoder::default().decode(frame(10, -20)).unwrap().pointer,
            [20, -10]
        );
    }
    #[test]
    fn extreme_deltas_do_not_overflow() {
        assert_eq!(
            Decoder::default()
                .decode(frame(i16::MIN, 0))
                .unwrap()
                .pointer,
            [0, i16::MAX]
        );
    }
    #[test]
    fn stationary_frame_does_not_move() {
        let mut d = frame(30, 40);
        d[4] = 0;
        assert_eq!(Decoder::default().decode(d).unwrap().pointer, [0, 0]);
    }
    #[test]
    fn taps_and_drag_release() {
        let mut s = Decoder::default();
        let mut d = [0; 10];
        d[1] = 1;
        assert_eq!(s.decode(d).unwrap().tap, 1);
        d[1] = 0;
        d[2] = 1;
        assert_eq!(s.decode(d).unwrap().tap, 2);
        d[2] = 0;
        d[1] = 2;
        assert!(s.decode(d).unwrap().hold);
        assert!(!s.decode([0; 10]).unwrap().hold);
    }
    #[test]
    fn scroll_rotates_and_accumulates_without_cursor() {
        let mut s = Decoder::default();
        let mut d = frame(16, -16);
        d[2] = 2;
        assert_eq!(s.decode(d).unwrap().scroll, [0, 0]);
        let o = s.decode(d).unwrap();
        assert_eq!(o.scroll, [-1, -1]);
        assert_eq!(o.pointer, [0, 0]);
        s.decode(d);
        s.decode([0; 10]);
        assert_eq!(s.decode(d).unwrap().scroll, [0, 0]);
    }
    #[test]
    fn reset_ati_and_palm_discard_input() {
        let mut s = Decoder::default();
        for flag in [0x80, 8] {
            let mut d = frame(20, 20);
            d[3] = flag;
            assert!(s.decode(d).is_none());
        }
        let mut d = frame(20, 20);
        d[4] |= 2;
        d[1] = 3;
        assert_eq!(s.decode(d), Some(Frame::default()));
    }
}
