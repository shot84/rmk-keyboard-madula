//! Pixel 0: layer/Caps Lock. Pixel 1: central link, not PC or charging.
pub struct State { connected: bool, status_until: u64, layer: u8, caps: bool }
impl State {
    pub fn new(now: u64) -> Self {
        Self { connected: false, status_until: now, layer: 0, caps: false }
    }
    pub fn link(&mut self, connected: bool, now: u64) {
        self.connected = connected;
        self.status_until = now + 1500;
        if !connected { self.layer = 0; self.caps = false; }
    }
    pub fn layer(&mut self, layer: u8, _now: u64) { self.layer = layer; }
    pub fn caps(&mut self, caps: bool) { self.caps = caps; }
    pub fn pixels(&self, now: u64) -> [[u8;3];2] {
        let inner = if !self.connected { [0;3] }
            else if self.caps { [12;3] }
            else { match self.layer {
                0 => [0;3], 1 => [0,0,12], 2 => [10,0,12],
                3 => [0,10,10], 4 => [12,6,0], _ => [8;3],
            }};
        let outer = if !self.connected {
            if now % 3000 < 400 { [20,7,0] } else { [0;3] }
        } else if now < self.status_until { [0,20,0] } else { [0;3] };
        [inner, outer]
    }
}
/// Independent, interruptible fades: 300 ms in/color change, 600 ms out.
pub struct Fade {
    from: [[u8; 3]; 2],
    target: [[u8; 3]; 2],
    started: [u64; 2],
}
impl Fade {
    pub fn new() -> Self {
        Self { from: [[0; 3]; 2], target: [[0; 3]; 2], started: [0; 2] }
    }
    pub fn pixels(&mut self, target: [[u8; 3]; 2], now: u64) -> [[u8; 3]; 2] {
        let mut output = [[0; 3]; 2];
        for p in 0..2 {
            let duration = if self.target[p] == [0; 3] { 600 } else { 300 };
            let t = (now.saturating_sub(self.started[p]).min(duration) * 1000 / duration) as i32;
            // Smoothstep gives gentle acceleration and deceleration, without floats.
            let weight = t * t / 1000 * (3000 - 2 * t) / 1000;
            for c in 0..3 {
                output[p][c] = ((self.from[p][c] as i32 * (1000 - weight)
                    + self.target[p][c] as i32 * weight + 500) / 1000) as u8;
            }
            if target[p] != self.target[p] {
                self.from[p] = output[p];
                self.target[p] = target[p];
                self.started[p] = now;
            }
        }
        output
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fade_in_out_and_interruption() {
        let mut f = Fade::new();
        let blue = [[0, 0, 12], [0; 3]];
        assert_eq!(f.pixels(blue, 0), [[0; 3]; 2]);
        assert_eq!(f.pixels(blue, 150)[0], [0, 0, 6]);
        assert_eq!(f.pixels(blue, 300), blue);
        assert_eq!(f.pixels([[0; 3]; 2], 400), blue);
        assert_eq!(f.pixels([[0; 3]; 2], 700)[0], [0, 0, 6]);
        assert_eq!(f.pixels(blue, 700)[0], [0, 0, 6]);
        assert_eq!(f.pixels(blue, 1000), blue);
        f.pixels([[0; 3]; 2], 1000);
        assert_eq!(f.pixels([[0; 3]; 2], 1600), [[0; 3]; 2]);
    }
    #[test]
    fn pixels_fade_independently() {
        let mut f = Fade::new();
        f.pixels([[12; 3], [0; 3]], 0);
        f.pixels([[12; 3], [20; 3]], 150);
        assert_eq!(f.pixels([[12; 3], [20; 3]], 300), [[12; 3], [10; 3]]);
    }
    #[test]
    fn link_pulse_and_idle() {
        let mut s = State::new(0);
        assert_eq!(s.pixels(0), [[0;3],[20,7,0]]);
        assert_eq!(s.pixels(500), [[0;3];2]);
        s.link(true,600);
        assert_eq!(s.pixels(700), [[0;3],[0,20,0]]);
        assert_eq!(s.pixels(2100), [[0;3];2]);
    }
    #[test]
    fn held_layer_caps_and_disconnect() {
        let mut s = State::new(0);
        s.link(true,0); s.layer(1,2000);
        assert_eq!(s.pixels(60000), [[0,0,12],[0;3]]);
        s.caps(true);
        assert_eq!(s.pixels(60000)[0], [12;3]);
        s.layer(2,60000); s.caps(false);
        assert_eq!(s.pixels(60000)[0], [10,0,12]);
        s.layer(0,60000);
        assert_eq!(s.pixels(60000), [[0;3];2]);
        s.caps(true); s.link(false,60000);
        assert_eq!(s.pixels(60000), [[0;3],[20,7,0]]);
    }
}
