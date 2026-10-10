//! A0 voltage monitoring with a NiMH voltage-based estimate. No charging claims.
//! AIN0 (P0.02), no enable GPIO. External Bat board: RL2=1M, RL3=470k.
#[path = "battery_voltage.rs"]
mod voltage;
use embassy_nrf::{bind_interrupts, saadc};
use embassy_time::{with_timeout, Duration, Timer};
use core::sync::atomic::{AtomicU32, Ordering};
// Low byte: percentage + 1 (zero means unavailable); upper bits: sample time in seconds.
static LATEST: AtomicU32 = AtomicU32::new(0);

pub fn led_rgb(now_ms: u64) -> [u8; 3] {
    let value = LATEST.load(Ordering::Relaxed);
    let level = value & 255;
    let age = (now_ms / 1000).saturating_sub(u64::from(value >> 8));
    if level == 0 || age > 30 || !(5000..6000).contains(&(now_ms % 60000)) {
        return [0; 3];
    }
    voltage::battery_rgb((level - 1) as u8)
}

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
        // Divider verified in EasyEDA GeaconPolaris/Bat; percentage is only an estimate.
        let raw = sample[0];
        match voltage::millivolts(raw) {
            Some((pin, battery)) => {
                let estimate = voltage::estimated_percent(battery);
                let seconds = embassy_time::Instant::now().as_secs();
                LATEST.store(((seconds.min(0x00ff_ffff) as u32) << 8) |
                    estimate.map_or(0, |p| u32::from(p) + 1), Ordering::Relaxed);
                log::info!("Battery: raw={} pin_mv={} battery_mv={} estimated_pct={:?} chemistry=NiMH cells=1",
                    raw, pin, battery, estimate);
            }
            None => {
                LATEST.store(0, Ordering::Relaxed);
                log::warn!("Battery: A0 out of range raw={}; voltage invalid", raw);
            }
        }
        Timer::after_secs(10).await;
    }
}
