//! Hardware-independent IQS gesture state; absolute coordinates are chip-rotated.
#[derive(Clone, Copy, Default)]
pub struct Sample {
    pub fingers: u8, pub touch: bool, pub delta: [i16; 2], pub points: [[u16; 2]; 3],
    /// Finger 1 touch strength and area, TP_FINGER1_CONFIDENCE and TP_MOVEMENT_DETECTED.
    pub strength: u16, pub area: u16, pub confident: bool, pub moved: bool,
}
pub fn decode(f: &[u8; 40]) -> Option<Sample> {
    if f[12] & 0x80 != 0 { return None; }
    // Finger count comes from the trackpad flags alone, as in the ZMK driver; the global
    // touch bit (info flags bit 9) can drop for a frame while fingers stay down.
    let mut s = Sample { fingers: f[14] & 15, touch: f[13] & 2 != 0, confident: f[15] & 1 != 0,
                         moved: f[14] & 0x10 != 0, ..Sample::default() };
    s.strength = u16::from_le_bytes([f[20], f[21]]);
    s.area = u16::from_le_bytes([f[22], f[23]]);
    s.delta = [i16::from_le_bytes([f[0], f[1]]), i16::from_le_bytes([f[2], f[3]])];
    for n in 0..3 { for axis in 0..2 {
        let i = 16 + n * 8 + axis * 2;
        s.points[n][axis] = u16::from_le_bytes([f[i], f[i+1]]);
    }}
    Some(s)
}
#[derive(Default, Debug, PartialEq)]
pub struct Output { pub pointer: [i16; 2], pub scroll: [i16; 2], pub tap: u8, pub left: Option<bool>, pub swipe: Option<Swipe> }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Swipe { Up = 0, Down = 1, Left = 2, Right = 3 }
// A touch ends only after this long with no fingers, so a one-frame dropout is not a tap.
pub const LIFT_MS: u64 = 30;
// Tap limits (max ms, max travel) for one, two and three fingers. The one-finger time comes
// from Params (250 ms by default; ZMK's 160 ms missed slower clicks); two and three follow
// madula-iqs.conf.
const TAP_LIMITS: [(u64, u32); 3] = [(250, 40), (170, 24), (170, 24)];
/// Runtime tuning: cursor speed is cursor_num/4 of the sensor delta, scroll steps once per
/// scroll_div centroid units, one-finger taps must lift within tap_ms; one-finger motion is
/// held back lift_hold frames (0 = off) and what is held is dropped when the finger lifts.
#[derive(Clone, Copy, Debug)]
pub struct Params { pub cursor_num: i32, pub scroll_div: i32, pub tap_ms: u64, pub tap_drag: bool, pub lift_hold: usize, pub three_drag: bool }
impl Default for Params { fn default() -> Self { Params { cursor_num: 4, scroll_div: 32, tap_ms: 250, tap_drag: true, lift_hold: 3, three_drag: false } } }
// A finger rolling off shrinks its contact toward the last edge touching, which the chip
// reports as a jump in the last two or three frames (about 30 ms, with strength still rising).
// Gating on TP_MOVEMENT_DETECTED (ZMK, Lalapad) or on falling strength dropped the small
// deltas of slow, fine motion, so motion is delayed instead and the held tail dropped.
// The delay is counted in frames (about 11 ms each), one frame out per frame in: timing it
// in ms let uneven frame gaps release zero or two frames at once, a visible stutter.
pub const HOLD: usize = 8;
// Tap-drag: after a one-finger tap the left button stays down this long; touching again in
// that window drags with it held (Lalapad Gen2 RMK uses 160 ms, madula-iqs.conf 120 ms).
pub const TAPDRAG_GAP_MS: u64 = 150;
// Three-finger drag (macOS style): once three fingers, all down within the first 100 ms of the
// touch, travel past the three-finger tap limit counted from when the third finger landed,
// the left button is held and the finger centroid moves the cursor, also while only two
// remain; lifting every finger lets go. A three-finger tap stays a middle click.
pub const THREE_DRAG_TRAVEL: u32 = TAP_LIMITS[2].1;
// With three-finger drag off, the same three fingers swipe instead: once per touch, when the
// centroid has moved this far on one axis (the ZMK driver's 3F swipe threshold, which counts
// finger 1 in the same chip units), the larger axis picks the direction.
pub const SWIPE_TRAVEL: i32 = 200;
/// Left button held by a tap: waiting for a second touch until `release_at`, or dragging.
#[derive(Default, Clone, Copy)]
struct Held { release_at: u64, dragging: bool }
#[derive(Default)]
pub struct Gestures {
    count: u8, peak: u8, start: u64, travel: u32, travel3: u32, invalid: bool, palm: bool, drag3: bool,
    sum3: [i32; 2], swiped: bool,
    previous: [i32; 2], remainder: [i32; 2], lifted: Option<u64>,
    held: Option<Held>,
    /// One-finger motion not yet reported (delta × cursor_num), oldest first.
    hold: [[i32; 2]; HOLD + 1], hold_len: usize, hold_depth: usize,
    /// One-finger frames whose held motion was dropped at lift (diagnostic counter).
    pub gated: u32,
    /// Dropouts shorter than LIFT_MS that were bridged (diagnostic counter).
    pub bridged: u32,
}
impl Gestures {
    pub fn update(&mut self, s: Sample, now: u64, p: Params) -> Output {
        let mut out = Output::default();
        if s.fingers != 1 {
            self.gated = self.gated.wrapping_add(self.hold.iter().take(self.hold_len).filter(|h| **h != [0; 2]).count() as u32);
            self.hold_len = 0;
        }
        if s.fingers == 0 {
            if self.count == 0 {
                if let Some(h) = self.held { if !h.dragging && now >= h.release_at {
                    self.held = None; out.left = Some(false);
                }}
                return out;
            }
            let lifted = *self.lifted.get_or_insert(now);
            if now.saturating_sub(lifted) < LIFT_MS { return out; }
            let (mut max_ms, max_travel) = TAP_LIMITS[usize::from(self.peak.clamp(1, 3) - 1)];
            if self.peak == 1 { max_ms = p.tap_ms; }
            if !self.invalid && lifted.saturating_sub(self.start) <= max_ms && self.travel <= max_travel {
                out.tap = self.peak;
            }
            if self.drag3 { out.tap = 0; out.left = Some(false); }
            match self.held {
                // End of the second touch: let go; a short second tap becomes a second click.
                Some(Held { dragging: true, .. }) => {
                    self.held = None; out.left = Some(false);
                    if out.tap != 1 { out.tap = 0; }
                }
                _ if out.tap == 1 && p.tap_drag => {
                    out.tap = 0; out.left = Some(true);
                    self.held = Some(Held { release_at: lifted + TAPDRAG_GAP_MS, dragging: false });
                }
                _ => {}
            }
            let (bridged, gated, held) = (self.bridged, self.gated, self.held);
            *self = Self::default();
            self.bridged = bridged; self.gated = gated; self.held = held;
            return out;
        }
        if self.lifted.take().is_some() { self.bridged = self.bridged.wrapping_add(1); }
        if self.count == 0 {
            self.start = now;
            if let Some(h) = self.held { if !h.dragging {
                if s.fingers == 1 && now < h.release_at { self.held = Some(Held { dragging: true, ..h }); }
                else { self.held = None; out.left = Some(false); }
            }}
        }
        if s.fingers > 1 { if let Some(Held { dragging: true, .. }) = self.held { self.held = None; out.left = Some(false); } }
        if s.fingers > 3 { self.invalid = true; self.palm = true; }
        if s.fingers > self.peak && now.saturating_sub(self.start) > 100 { self.invalid = true; }
        self.peak = self.peak.max(s.fingers);
        let n = usize::from(s.fingers.min(3));
        let center = core::array::from_fn(|axis| s.points[..n].iter().map(|p| i32::from(p[axis])).sum::<i32>() / n as i32);
        if self.count == s.fingers {
            let delta = [center[0]-self.previous[0], center[1]-self.previous[1]];
            self.travel = self.travel.saturating_add(delta[0].unsigned_abs() + delta[1].unsigned_abs());
            if s.fingers == 3 {
                self.travel3 = self.travel3.saturating_add(delta[0].unsigned_abs() + delta[1].unsigned_abs());
                for axis in 0..2 { self.sum3[axis] += delta[axis]; }
            }
            if !p.three_drag && !self.swiped && s.fingers == 3 && !self.invalid {
                let [x, y] = self.sum3;
                if x.abs().max(y.abs()) >= SWIPE_TRAVEL {
                    self.swiped = true;
                    out.swipe = Some(if x.abs() >= y.abs() { if x < 0 { Swipe::Left } else { Swipe::Right } }
                                     else if y < 0 { Swipe::Up } else { Swipe::Down });
                }
            }
            if !self.drag3 && p.three_drag && s.fingers == 3 && !self.invalid && self.travel3 > THREE_DRAG_TRAVEL {
                self.drag3 = true; out.left = Some(true);
            }
            if self.drag3 {
                if s.fingers >= 2 { for axis in 0..2 {
                    self.remainder[axis] += delta[axis] * p.cursor_num;
                    out.pointer[axis] = (self.remainder[axis] / 4).clamp(-127,127) as i16;
                    self.remainder[axis] %= 4;
                }}
            } else if !self.invalid && s.fingers == self.peak {
                let (motion, divisor) = if s.fingers == 1 {
                    (self.release([i32::from(s.delta[0]) * p.cursor_num, i32::from(s.delta[1]) * p.cursor_num], p), 4)
                } else { (delta, p.scroll_div.max(1)) };
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
    /// Queues this frame's one-finger motion and returns what falls out beyond lift_hold frames.
    fn release(&mut self, motion: [i32; 2], p: Params) -> [i32; 2] {
        let depth = p.lift_hold.min(HOLD);
        // A tuning change starts a fresh delay window; never replay old motion.
        if depth != self.hold_depth {
            self.hold_len = 0;
            self.hold_depth = depth;
        }
        if depth == 0 { return motion; }
        self.hold[self.hold_len] = motion; self.hold_len += 1;
        let mut out = [0; 2];
        while self.hold_len > depth {
            for axis in 0..2 { out[axis] += self.hold[0][axis]; }
            self.hold.copy_within(1.., 0); self.hold_len -= 1;
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const P: Params = Params { cursor_num: 2, scroll_div: 32, tap_ms: 250, tap_drag: false, lift_hold: 0, three_drag: false };
    const D: Params = Params { tap_drag: true, ..P };
    fn s(n:u8,x:u16)->Sample { Sample { fingers:n, touch:n!=0, points:[[x,100];3], delta:[1,0], strength:100, area:10, confident:n!=0, moved:true } }
    // Lift and wait out LIFT_MS; returns the tap from the frame that ends the touch.
    fn lift(g:&mut Gestures,t:u64)->u8 { assert_eq!(g.update(s(0,0),t, P).tap,0); g.update(s(0,0),t+LIFT_MS, P).tap }
    #[test] fn taps() { for n in 1..=3 {let mut g=Gestures::default();g.update(s(n,100),0, P);assert_eq!(lift(&mut g,100),n);} }
    #[test] fn moving_or_long_touch_does_not_click() {let mut g=Gestures::default();g.update(s(1,100),0, P);g.update(s(1,150),20, P);assert_eq!(lift(&mut g,100),0);g.update(s(2,100),200, P);assert_eq!(lift(&mut g,600),0);}
    #[test] fn half_speed_preserves_fraction() {let mut g=Gestures::default();g.update(s(1,100),0, P);assert_eq!(g.update(s(1,101),10, P).pointer,[0,0]);assert_eq!(g.update(s(1,102),20, P).pointer,[1,0]);}
    #[test] fn scroll_and_lift_tail_do_not_move_cursor() {let mut g=Gestures::default();g.update(s(2,100),0, P);let o=g.update(s(2,164),20, P);assert_eq!(o.scroll,[2,0]);assert_eq!(o.pointer,[0,0]);assert_eq!(g.update(s(1,164),40, P).pointer,[0,0]);assert_eq!(g.update(s(1,165),50, P).pointer,[0,0]);assert_eq!(lift(&mut g,60),0);}
    #[test] fn staggered_three_finger_tap_is_one_middle_click() {let mut g=Gestures::default();g.update(s(1,100),0, P);g.update(s(2,100),20, P);g.update(s(3,100),40, P);g.update(s(2,100),70, P);g.update(s(1,100),90, P);assert_eq!(lift(&mut g,120),3);assert_eq!(g.update(s(0,0),200, P).tap,0);}
    #[test] fn palm_cancels_click() {let mut g=Gestures::default();g.update(s(4,100),0, P);g.update(s(1,100),20, P);assert_eq!(lift(&mut g,50),0);}
    #[test] fn dropout_while_resting_does_not_click() {
        let mut g=Gestures::default(); let mut taps=0;
        for i in 0..100u64 { let n = if i % 3 == 2 {0} else {3}; taps += u32::from(g.update(s(n,100),i*10, P).tap != 0); }
        assert_eq!(taps,0); assert!(g.bridged > 0);
    }
    #[test] fn slower_tap_needs_longer_tap_ms() {let mut g=Gestures::default();g.update(s(1,100),0,P);assert_eq!(lift(&mut g,300),0);
        let long=Params{tap_ms:350,..P};g.update(s(1,100),1000,long);assert_eq!(g.update(s(0,0),1300,long).tap,0);assert_eq!(g.update(s(0,0),1300+LIFT_MS,long).tap,1);}
    #[test] fn cursor_num_scales_pointer() {let mut g=Gestures::default();let fast=Params{cursor_num:4,..P};g.update(s(1,100),0,fast);assert_eq!(g.update(s(1,101),10,fast).pointer,[1,0]);}
    #[test] fn tap_holds_left_until_gap_then_releases() {let mut g=Gestures::default();g.update(s(1,100),0,D);
        assert_eq!(g.update(s(0,0),80,D).left,None);let o=g.update(s(0,0),80+LIFT_MS,D);assert_eq!((o.tap,o.left),(0,Some(true)));
        assert_eq!(g.update(s(0,0),80+TAPDRAG_GAP_MS-1,D).left,None);assert_eq!(g.update(s(0,0),80+TAPDRAG_GAP_MS,D).left,Some(false));}
    #[test] fn second_touch_drags_with_left_held() {let mut g=Gestures::default();g.update(s(1,100),0,D);g.update(s(0,0),80,D);g.update(s(0,0),80+LIFT_MS,D);
        let o=g.update(s(1,100),150,D);assert_eq!(o.left,None);
        for i in 1..20u16 { let o=g.update(s(1,100+i*10),150+u64::from(i)*10,D); assert_eq!(o.left,None); }
        assert_eq!(g.update(s(0,0),500,D).left,None);let o=g.update(s(0,0),500+LIFT_MS,D);assert_eq!((o.tap,o.left),(0,Some(false)));}
    #[test] fn double_tap_with_drag_on_is_two_clicks() {let mut g=Gestures::default();g.update(s(1,100),0,D);g.update(s(0,0),80,D);g.update(s(0,0),80+LIFT_MS,D);
        g.update(s(1,100),150,D);g.update(s(0,0),220,D);let o=g.update(s(0,0),220+LIFT_MS,D);assert_eq!((o.tap,o.left),(1,Some(false)));}
    #[test] fn two_fingers_after_tap_release_left() {let mut g=Gestures::default();g.update(s(1,100),0,D);g.update(s(0,0),80,D);g.update(s(0,0),80+LIFT_MS,D);
        assert_eq!(g.update(s(2,100),150,D).left,Some(false));}
    #[test] fn double_tap_still_gives_two_clicks() {let mut g=Gestures::default();g.update(s(1,100),0, P);assert_eq!(lift(&mut g,80),1);g.update(s(1,100),200, P);assert_eq!(lift(&mut g,280),1);}
    #[test] fn held_motion_arrives_late_and_lift_drops_tail() {let mut g=Gestures::default();let h=Params{cursor_num:4,lift_hold:3,..P};
        g.update(s(1,100),0,h);
        for t in [10,20,30] { assert_eq!(g.update(s(1,100),t,h).pointer,[0,0]); }
        // Uneven gaps still release exactly one frame per frame.
        assert_eq!(g.update(s(1,100),31,h).pointer,[1,0]);
        assert_eq!(g.update(Sample{delta:[0,0],..s(1,100)},70,h).pointer,[1,0]);
        assert_eq!(g.update(s(0,0),75,h).pointer,[0,0]);assert_eq!(g.gated,2);}
    #[test] fn small_unflagged_deltas_still_move() {let mut g=Gestures::default();let still=Sample{moved:false,..s(1,100)};
        g.update(still,0,P);assert_eq!(g.update(still,10,P).pointer,[0,0]);assert_eq!(g.update(still,20,P).pointer,[1,0]);}
    #[test] fn three_finger_drag_holds_left_and_moves() {let mut g=Gestures::default();let t=Params{three_drag:true,..P};
        g.update(s(3,100),0,t);assert_eq!(g.update(s(3,110),10,t).left,None);
        let o=g.update(s(3,130),20,t);assert_eq!(o.left,Some(true));assert_eq!(o.pointer,[10,0]);
        let o=g.update(s(2,130),30,t);assert_eq!((o.left,o.scroll),(None,[0,0]));
        assert_eq!(g.update(s(2,150),40,t).pointer,[10,0]);
        assert_eq!(g.update(s(0,0),50,t).left,None);let o=g.update(s(0,0),50+LIFT_MS,t);assert_eq!((o.tap,o.left),(0,Some(false)));}
    #[test] fn third_finger_during_scroll_does_not_drag() {let mut g=Gestures::default();let t=Params{three_drag:true,..P};
        g.update(s(2,100),0,t);g.update(s(2,150),20,t);g.update(s(2,200),40,t);
        for i in 0..5u16 { assert_eq!(g.update(s(3,200+i*20),200+u64::from(i)*10,t).left,None); }}
    #[test] fn three_finger_tap_is_still_middle_click_with_drag_on() {let mut g=Gestures::default();let t=Params{three_drag:true,..P};
        g.update(s(3,100),0,t);g.update(s(3,105),10,t);assert_eq!(lift(&mut g,60),3);}
    #[test] fn three_finger_swipe_fires_once_when_drag_is_off() {let mut g=Gestures::default();
        let mut swipes=0;
        for i in 0..10u16 { let o=g.update(s(3,100+i*30),u64::from(i)*10,P); assert_eq!(o.left,None);
            if let Some(d)=o.swipe { assert_eq!((swipes,d),(0,Swipe::Right)); swipes+=1; } }
        assert_eq!(swipes,1);assert_eq!(lift(&mut g,200),0);}
    #[test] fn three_finger_swipe_up_and_none_while_dragging() {let mut g=Gestures::default();
        let up=|y:u16| Sample{points:[[100,y];3],..s(3,100)};
        g.update(up(400),0,P);assert_eq!(g.update(up(300),10,P).swipe,None);assert_eq!(g.update(up(190),20,P).swipe,Some(Swipe::Up));
        let mut g=Gestures::default();let t=Params{three_drag:true,..P};
        for i in 0..10u16 { assert_eq!(g.update(s(3,100+i*30),u64::from(i)*10,t).swipe,None); }}
    #[test] fn reset_frame_rejected() {let mut f=[0;40];f[12]=128;assert!(decode(&f).is_none());}
}

#[cfg(test)]
mod tuning_regressions {
    use super::*;
    fn sample(dx: i16) -> Sample { Sample { fingers: 1, points: [[100,100];3], delta: [dx,0], ..Sample::default() } }
    #[test]
    fn changing_hold_does_not_replay_old_motion() {
        let mut g = Gestures::default();
        let on = Params { lift_hold: 3, tap_drag: false, ..Params::default() };
        g.update(sample(0),0,on);
        for t in [10,20,30] { g.update(sample(10),t,on); }
        let off = Params { lift_hold: 0, ..on };
        assert_eq!(g.update(sample(0),40,off).pointer,[0,0]);
        assert_eq!(g.update(sample(0),50,on).pointer,[0,0]);
        for t in [60,70,80,90] { assert_eq!(g.update(sample(0),t,on).pointer,[0,0]); }
    }
    #[test]
    fn maximum_hold_depth_is_bounded() {
        let mut g=Gestures::default();
        let p=Params{lift_hold:HOLD,..Params::default()};
        for t in 0..100 { g.update(sample(1),t*10,p); }
    }
}
