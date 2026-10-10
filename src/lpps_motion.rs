// Fixed-point velocity mapping; each axis keeps its own fractional motion.
#[derive(Default)]
pub struct AxisMotion { remainder: i32 }
impl AxisMotion {
    pub fn step(&mut self, delta: i32) -> i16 {
        const DEADZONE: i32 = 1500;
        const RANGE: i32 = 12000 - DEADZONE;
        let delta = delta.clamp(-12000, 12000);
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
        for n in [-1500, -1, 0, 1, 1500] { assert_eq!(axis.step(n), 0); }
        assert_eq!(axis.step(12000), 20);
        assert_eq!(axis.step(-12000), -20);
        assert_eq!(axis.step(i32::MIN), -20);
    }
    #[test]
    fn preserves_slow_motion_and_diagonals() {
        let mut x = AxisMotion::default();
        let mut y = AxisMotion::default();
        let mut total = (0, 0);
        for _ in 0..525 { total.0 += x.step(1501); total.1 += y.step(-1501); }
        assert_eq!(total, (1, -1));
    }
}
