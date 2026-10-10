//! Read-only A0 voltage diagnostics. No battery percentage or charging claims.
//! AIN0 (P0.02), no enable GPIO. External Bat board: RL2=1M, RL3=470k.
#[path = "battery_voltage.rs"]
mod voltage;
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
        // Divider verified in EasyEDA GeaconPolaris/Bat. No percentage estimate.
        let raw = sample[0];
        match voltage::millivolts(raw) {
            Some((pin, battery)) => log::info!(
                "Battery: raw={} pin_mv={} battery_mv={} chemistry=NiMH cells=1 divider=1470/470",
                raw, pin, battery),
            None => log::warn!("Battery: A0 out of range raw={}; voltage invalid", raw),
        }
        Timer::after_secs(10).await;
    }
}
