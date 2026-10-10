//! Read-only A0 voltage diagnostics. No battery percentage or charging claims.
//! ZMK Madula vbatt uses AIN0 (P0.02), no enable GPIO, and a 1:1 scale.
use embassy_nrf::{bind_interrupts, saadc};
use embassy_time::{with_timeout, Duration, Timer};

bind_interrupts!(pub struct Irqs { SAADC => saadc::InterruptHandler; });

#[embassy_executor::task]
pub async fn run(mut adc: saadc::Saadc<'static, 1>) {
    Timer::after_secs(5).await;
    if with_timeout(Duration::from_secs(1), adc.calibrate()).await.is_err() {
        log::warn!("Battery audit: ADC calibration timeout; diagnostics stopped");
        return;
    }
    loop {
        let mut sample = [0i16; 1];
        if with_timeout(Duration::from_secs(1), adc.sample(&mut sample)).await.is_err() {
            log::warn!("Battery audit: ADC sample timeout; diagnostics stopped");
            return;
        }
        // Internal 0.6V reference, gain 1/6, 12-bit. Report the pin voltage,
        // not a battery percentage: chemistry and scale need physical validation.
        let raw = sample[0];
        if !(0..4095).contains(&raw) {
            log::warn!("Battery audit: A0 out of range raw={}; voltage invalid", raw);
        } else {
            let mv = (i32::from(raw) * 3600 + 2048) / 4096;
            log::info!("Battery audit: A0 raw={} pin_mv={} scale_unverified=1:1", raw, mv);
        }
        Timer::after_secs(10).await;
    }
}
