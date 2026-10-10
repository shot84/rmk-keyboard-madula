//! Hardware-independent IQS gesture state; absolute coordinates are chip-rotated.
#[derive(Clone, Copy, Default)]
pub struct Sample { pub fingers: u8, pub delta: [i16; 2], pub points: [[u16; 2]; 3] }
pub fn decode(f: &[u8; 40]) -> Option<Sample> {
    if f[12] & 0x80 != 0 { return None; }
    let mut s = Sample { fingers: f[14] & 15, ..Sample::default() };
    if f[13] & 2 == 0 { s.fingers = 0; }
    s.delta = [i16::from_le_bytes([f[0], f[1]]), i16::from_le_bytes([f[2], f[3]])];
    for n in 0..3 { for axis in 0..2 {
        let i = 16 + n * 8 + axis * 2;
        s.points[n][axis] = u16::from_le_bytes([f[i], f[i+1]]);
    }}
    Some(s)
}
#[derive(Default, Debug, PartialEq)]
pub struct Output { pub pointer: [i16; 2], pub scroll: [i16; 2], pub tap: u8 }
#[derive(Default)]
pub struct Gestures {
    count: u8, peak: u8, start: u64, travel: u32, invalid: bool,
    previous: [i32; 2], remainder: [i32; 2],
}
impl Gestures {
    pub fn update(&mut self, s: Sample, now: u64) -> Output {
        let mut out = Output::default();
        if s.fingers == 0 {
            if self.count != 0 && !self.invalid && now.saturating_sub(self.start) <= 250 && self.travel <= 40 {
                out.tap = self.peak;
            }
            *self = Self::default();
            return out;
        }
        if self.count == 0 { self.start = now; }
        if s.fingers > 3 { self.invalid = true; }
        if s.fingers > self.peak && now.saturating_sub(self.start) > 100 { self.invalid = true; }
        self.peak = self.peak.max(s.fingers);
        let n = usize::from(s.fingers.min(3));
        let center = core::array::from_fn(|axis| s.points[..n].iter().map(|p| i32::from(p[axis])).sum::<i32>() / n as i32);
        if self.count == s.fingers {
            let delta = [center[0]-self.previous[0], center[1]-self.previous[1]];
            self.travel = self.travel.saturating_add(delta[0].unsigned_abs() + delta[1].unsigned_abs());
            if !self.invalid && s.fingers == self.peak {
                let (motion, divisor) = if s.fingers == 1 {
                    ([i32::from(s.delta[0]), i32::from(s.delta[1])], 2)
                } else { (delta, 32) };
                if s.fingers <= 2 { for axis in 0..2 {
                    self.remainder[axis] += motion[axis];
                    let value = self.remainder[axis] / divisor;
                    self.remainder[axis] %= divisor;
                    if s.fingers == 1 { out.pointer[axis] = value.clamp(-127,127) as i16; }
                    else { out.scroll[axis] = value.clamp(-3,3) as i16; }
                }}
            }
        } else { self.remainder = [0;2]; }
        self.previous = center; self.count = s.fingers;
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn s(n:u8,x:u16)->Sample { Sample { fingers:n, points:[[x,100];3], delta:[1,0] } }
    #[test] fn taps() { for n in 1..=3 {let mut g=Gestures::default();g.update(s(n,100),0);assert_eq!(g.update(s(0,0),100).tap,n);} }
    #[test] fn moving_or_long_touch_does_not_click() {let mut g=Gestures::default();g.update(s(1,100),0);g.update(s(1,150),20);assert_eq!(g.update(s(0,0),100).tap,0);g.update(s(2,100),200);assert_eq!(g.update(s(0,0),600).tap,0);}
    #[test] fn half_speed_preserves_fraction() {let mut g=Gestures::default();g.update(s(1,100),0);assert_eq!(g.update(s(1,101),10).pointer,[0,0]);assert_eq!(g.update(s(1,102),20).pointer,[1,0]);}
    #[test] fn scroll_and_lift_tail_do_not_move_cursor() {let mut g=Gestures::default();g.update(s(2,100),0);let o=g.update(s(2,164),20);assert_eq!(o.scroll,[2,0]);assert_eq!(o.pointer,[0,0]);assert_eq!(g.update(s(1,164),40).pointer,[0,0]);assert_eq!(g.update(s(1,165),50).pointer,[0,0]);assert_eq!(g.update(s(0,0),60).tap,0);}
    #[test] fn staggered_three_finger_tap_is_one_middle_click() {let mut g=Gestures::default();g.update(s(1,100),0);g.update(s(2,100),20);g.update(s(3,100),40);g.update(s(2,100),70);g.update(s(1,100),90);assert_eq!(g.update(s(0,0),120).tap,3);assert_eq!(g.update(s(0,0),130).tap,0);}
    #[test] fn palm_cancels_click() {let mut g=Gestures::default();g.update(s(4,100),0);g.update(s(1,100),20);assert_eq!(g.update(s(0,0),50).tap,0);}
    #[test] fn reset_frame_rejected() {let mut f=[0;40];f[12]=128;assert!(decode(&f).is_none());}
}
