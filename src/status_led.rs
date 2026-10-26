//! Madula external LEDs use PWM0, independent of input module buses.
use embassy_nrf::{gpio::Output, pwm::SequencePwm};
use embassy_time::{Instant, Timer};
use rmk::embassy_futures::select::{select3, Either3};
use rmk::event::{ConnectionStatusChangeEvent, ConnectionType, EventSubscriber,
                 LayerChangeEvent, SubscribableEvent};

#[path = "ws2812_pwm.rs"]
mod driver;

// Pulse count requested by other tasks (pad tuning keys); the LED loop picks it up.
static PULSES: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);
#[allow(dead_code)] // Only the IQS build calls it.
pub fn pulse(count: u8) { PULSES.store(count, core::sync::atomic::Ordering::Relaxed); }

fn layer_rgb(layer: u8) -> [u8; 3] {
    match layer {
        0 => [0, 0, 32], 1 => [32, 0, 0], 2 => [0, 32, 0],
        3 => [32, 32, 0], 4 => [0, 32, 32], _ => [32, 32, 32],
    }
}

#[embassy_executor::task]
pub async fn run(mut pwm: SequencePwm<'static>, mut power: Output<'static>,
                 mut red: Output<'static>, mut green: Output<'static>, mut blue: Output<'static>) {
    // Subscribe before USB enumeration so the initial state is not missed.
    let mut connections = ConnectionStatusChangeEvent::subscriber();
    let mut layers = LayerChangeEvent::subscriber();
    #[cfg(feature = "led-self-test")]
    {
        power.set_low(); // Madula LED_EN_L is active low.
        Timer::after_millis(100).await;
        log::info!("Madula external LED PWM self-test: green for 3s");
        for _ in 0..30 {
            if !driver::send(&mut pwm, [0, 32, 0]).await {
                log::warn!("Madula external LED PWM self-test failed");
            }
            Timer::after_millis(100).await;
        }
        let _ = driver::send(&mut pwm, [0; 3]).await;
        power.set_high();
        log::info!("Madula external LED PWM self-test complete");
    }
    let mut status = [32, 0, 0];
    let mut status_until = 0u64;
    let mut layer = 0;
    let mut layer_start = 0u64;
    let mut last_rgb = [255; 3];
    let mut powered = false;
    let mut last_lit = 0;
    let (mut pulses, mut pulse_start) = (0u8, 0u64);
    loop {
        match select3(connections.next_event(), layers.next_event(), Timer::after_millis(10)).await {
            Either3::First(event) => {
                status = match event.0.decide_active() {
                    Some(ConnectionType::Usb) => [0, 32, 32],
                    Some(ConnectionType::Ble) => [0, 0, 32],
                    None => [32, 0, 0],
                };
                status_until = Instant::now().as_millis() + 1000;
                log::info!("LED transport={:?}", event.0.decide_active());
            }
            Either3::Second(event) => {
                layer = event.0;
                layer_start = Instant::now().as_millis();
                log::info!("LED layer={} pulses={}", layer, layer);
            }
            Either3::Third(_) => {}
        }
        let now = Instant::now().as_millis();
        let requested = PULSES.swap(0, core::sync::atomic::Ordering::Relaxed);
        if requested != 0 { pulses = requested; pulse_start = now; }
        let pulse_elapsed = now.saturating_sub(pulse_start);
        let elapsed = now.saturating_sub(layer_start);
        let onboard = if now < status_until { status }
            else if layer_start != 0 && elapsed < 120 { layer_rgb(layer) }
            else { [0; 3] };
        red.set_level(if onboard[0] != 0 { embassy_nrf::gpio::Level::Low } else { embassy_nrf::gpio::Level::High });
        green.set_level(if onboard[1] != 0 { embassy_nrf::gpio::Level::Low } else { embassy_nrf::gpio::Level::High });
        blue.set_level(if onboard[2] != 0 { embassy_nrf::gpio::Level::Low } else { embassy_nrf::gpio::Level::High });
        #[cfg(feature = "battery-audit")]
        let battery_rgb = crate::battery_audit::led_rgb(now);
        #[cfg(not(feature = "battery-audit"))]
        let battery_rgb = [0; 3];
        let rgb = if pulse_elapsed < u64::from(pulses)*240 {
                if (pulse_elapsed/120)%2 == 0 { [32, 16, 0] } else { [0; 3] }
            }
            else if now < status_until { status }
            else if elapsed < u64::from(layer)*240 {
                if (elapsed/120)%2 == 0 { [32; 3] } else { [0; 3] }
            }
            else { battery_rgb };
        if rgb != [0; 3] {
            last_lit = now;
            if !powered {
                power.set_low();
                Timer::after_millis(10).await;
                powered = true;
                last_rgb = [255; 3];
            }
        }
        if powered && rgb != last_rgb {
            if driver::send(&mut pwm, rgb).await {
                last_rgb = rgb;
                log::info!("Madula external LED PWM sent rgb={:?}", rgb);
            } else { log::warn!("Madula external LED PWM write failed"); }
        }
        if powered && rgb == [0; 3] && now.saturating_sub(last_lit) >= 15000 {
            power.set_high();
            powered = false;
        }
    }
}
