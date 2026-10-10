//! Madula status LEDs. SPI3 is independent of LPPS on TWISPI0.
use embassy_nrf::{bind_interrupts, gpio::Output, peripherals, spim};
use embassy_time::{Instant, Timer};
use rmk::embassy_futures::select::{select3, Either3};
use rmk::event::{ConnectionStatusChangeEvent, ConnectionType, EventSubscriber,
                 LayerChangeEvent, SubscribableEvent};

bind_interrupts!(pub struct Irqs { SPIM3 => spim::InterruptHandler<peripherals::SPI3>; });

fn layer_rgb(layer: u8) -> [u8; 3] {
    match layer {
        0 => [0, 0, 32], 1 => [32, 0, 0], 2 => [0, 32, 0],
        3 => [32, 32, 0], 4 => [0, 32, 32], _ => [32, 32, 32],
    }
}

fn encode(rgb: [u8; 3]) -> [u8; 24] {
    let mut out = [0x40; 24];
    for (i, value) in [rgb[1], rgb[0], rgb[2]].into_iter().enumerate() {
        for bit in 0..8 { if value & (0x80 >> bit) != 0 { out[i*8+bit] = 0x70; } }
    }
    out
}

#[embassy_executor::task]
pub async fn run(mut spi: spim::Spim<'static>, mut power: Output<'static>,
                 mut red: Output<'static>, mut green: Output<'static>, mut blue: Output<'static>) {
    // Subscribe before USB enumeration so the initial state is not missed.
    let mut connections = ConnectionStatusChangeEvent::subscriber();
    let mut layers = LayerChangeEvent::subscriber();
    let mut status = [32, 0, 0];
    let mut status_until = 0u64;
    let mut layer = 0;
    let mut layer_start = 0u64;
    let mut last_rgb = [255; 3];
    let mut powered = false;
    let mut last_lit = 0;
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
        let rgb = if now < status_until { status }
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
            // Match ZMK's 4MHz encoding and >=280us low reset/latch time.
            let result = spi.write(&encode(rgb)).await;
            let reset = [0u8; 140];
            let reset_result = spi.write(&reset).await;
            if result.is_err() || reset_result.is_err() { log::warn!("LED SPI write failed"); }
            else { last_rgb = rgb; }
        }
        if powered && rgb == [0; 3] && now.saturating_sub(last_lit) >= 15000 {
            power.set_high();
            powered = false;
        }
    }
}
