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

#[cfg(test)]
mod tests {
    use super::*;
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
