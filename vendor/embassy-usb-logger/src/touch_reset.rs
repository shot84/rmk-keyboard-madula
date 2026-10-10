// Local addition: require a deliberate 1200-baud DTR high -> low sequence.
#[derive(Default)]
pub(crate) struct TouchReset {
    armed: bool,
}

impl TouchReset {
    pub(crate) fn observe(&mut self, baud: u32, dtr: bool) -> bool {
        if baud != 1200 {
            self.armed = false;
            return false;
        }
        if dtr {
            self.armed = true;
            return false;
        }
        core::mem::take(&mut self.armed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_touch_only() {
        let mut state = TouchReset::default();
        assert!(!state.observe(1200, false));
        assert!(!state.observe(115200, true));
        assert!(!state.observe(115200, false));
        assert!(!state.observe(1200, true));
        assert!(state.observe(1200, false));
        assert!(!state.observe(1200, false));
    }
    #[test]
    fn changing_baud_or_usb_reset_disarms() {
        let mut state = TouchReset::default();
        assert!(!state.observe(1200, true));
        assert!(!state.observe(9600, false));
        assert!(!state.observe(1200, false));
    }
}
