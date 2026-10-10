// Fixed-point velocity mapping; each axis keeps its own fractional motion.
#[derive(Default)]
pub struct AxisMotion { remainder: i32, filtered: i32 }
impl AxisMotion {
    pub fn step(&mut self, delta: i32) -> i16 {
        const DEADZONE: i32 = 2500;
        const RANGE: i32 = 12000 - DEADZONE;
        // One-pole low-pass (~24 ms at an 8 ms sample period), independently
        // per axis. Clamp first to keep arithmetic bounded on invalid inputs.
        let input = delta.clamp(-12000, 12000) * 256;
        self.filtered += (input - self.filtered) / 4;
        let delta = self.filtered / 256;
        if delta.abs() <= DEADZONE {
            self.remainder = 0;
            return 0;
        }
        let velocity = (delta.abs() - DEADZONE) * delta.signum() * 20;
        // Do not let fractional motion from one direction delay a reversal.
        if self.remainder.signum() != velocity.signum() { self.remainder = 0; }
        self.remainder += velocity;
        let result = self.remainder / RANGE;
        self.remainder %= RANGE;
        result as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deadzone_and_limits() {
        let mut axis = AxisMotion::default();
        for n in [-2500, -1, 0, 1, 2500] { assert_eq!(axis.step(n), 0); }
        for _ in 0..100 { assert!((0..=20).contains(&axis.step(i32::MAX))); }
        for _ in 0..100 { assert!((-20..=20).contains(&axis.step(i32::MIN))); }
        assert!(axis.step(i32::MIN) <= -19);
    }
    #[test]
    fn preserves_slow_motion_and_diagonals() {
        let mut x = AxisMotion::default();
        let mut y = AxisMotion::default();
        let mut total = (0, 0);
        for _ in 0..1000 { total.0 += x.step(2600); total.1 += y.step(-2600); }
        assert!(total.0 > 0);
        assert_eq!(total.0, -total.1);
    }
    #[test]
    fn idle_noise_and_release_stop() {
        let mut axis = AxisMotion::default();
        for _ in 0..1000 {
            for n in [-2700, -600, 1200, -1500, 1900, -200] {
                assert_eq!(axis.step(n), 0);
            }
        }
        for _ in 0..100 { axis.step(12000); }
        for _ in 0..20 { axis.step(0); }
        for _ in 0..100 { assert_eq!(axis.step(0), 0); }
    }
}
