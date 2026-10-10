// GeaconPolaris Bat: RL2=1M above INPUT_VOLTAGE_B, RL3=470k to GND.
// Measures the pre-boost supply after QL8, not the boosted XIAO supply.
pub fn millivolts(raw: i16) -> Option<(u32, u32)> {
    if !(0..4095).contains(&raw) { return None; }
    let raw = raw as u64;
    let pin = (raw * 3600 + 2048) / 4096;
    // Convert directly from ADC counts, avoiding compounded rounding.
    let denominator = 4096 * 470;
    let battery = (raw * 3600 * 1470 + denominator / 2) / denominator;
    Some((pin as u32, battery as u32))
}

// Same 0..100%, 10%-step NiMH voltage table as Polaris_hardware.dtsi.
// A voltage-based indication, not coulomb-counted state of charge.
pub fn estimated_percent(mv: u32) -> Option<u8> {
    const LEVELS: [u32; 11] = [1100, 1150, 1200, 1220, 1240, 1260, 1280, 1300, 1320, 1350, 1400];
    // Near-zero/disconnected and implausibly high readings must not look full.
    if !(100..=1800).contains(&mv) { return None; }
    if mv <= LEVELS[0] { return Some(0); }
    for i in 1..LEVELS.len() {
        if mv < LEVELS[i] {
            return Some(((i as u32 - 1) * 10 +
                (mv - LEVELS[i-1]) * 10 / (LEVELS[i] - LEVELS[i-1])) as u8);
        }
    }
    Some(100)
}

pub fn battery_rgb(percent: u8) -> [u8; 3] {
    match percent { 0..=20 => [32, 0, 0], 21..=50 => [32, 16, 0], _ => [0, 32, 0] }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nimh_estimate_boundaries() {
        assert_eq!(estimated_percent(0), None);
        assert_eq!(estimated_percent(1801), None);
        assert_eq!(estimated_percent(1100), Some(0));
        assert_eq!(estimated_percent(1200), Some(20));
        assert_eq!(estimated_percent(1330), Some(83));
        assert_eq!(estimated_percent(1400), Some(100));
        assert_eq!(battery_rgb(20), [32, 0, 0]);
        assert_eq!(battery_rgb(50), [32, 16, 0]);
        assert_eq!(battery_rgb(83), [0, 32, 0]);
    }
    #[test]
    fn measured_nimh_sample() {
        assert_eq!(millivolts(483), Some((425, 1328)));
        assert_eq!(millivolts(484), Some((425, 1330)));
    }
    #[test]
    fn invalid_and_zero() {
        assert_eq!(millivolts(-1), None);
        assert_eq!(millivolts(4095), None);
        assert_eq!(millivolts(i16::MAX), None);
        assert_eq!(millivolts(0), Some((0, 0)));
    }
}
