//! Cornix's two WS2812 LEDs; wiring/power polarity match the ZMK board DTS.
use embassy_nrf::{gpio::Output, pwm::SequencePwm};
use embassy_time::{Instant, Timer};
use rmk::embassy_futures::select::{select, select3, Either, Either3};
use rmk::event::{CentralConnectedEvent, EventSubscriber, LayerChangeEvent, LedIndicatorEvent, SubscribableEvent};

#[path = "peripheral_led_pattern.rs"]
mod pattern;
#[path = "ws2812_pwm.rs"]
mod driver;

#[embassy_executor::task]
pub async fn run(mut pwm: SequencePwm<'static>, mut power: Output<'static>) {
    let mut links = CentralConnectedEvent::subscriber();
    let mut layers = LayerChangeEvent::subscriber();
    let mut indicators = LedIndicatorEvent::subscriber();
    #[cfg(feature = "led-self-test")]
    {
        power.set_high();
        Timer::after_millis(100).await;
        log::info!("Cornix LED PWM self-test: power=HIGH, green for 3s");
        for _ in 0..30 {
            if !driver::send(&mut pwm, [0, 32, 0]).await {
                log::warn!("Cornix LED self-test PWM error");
            }
            Timer::after_millis(100).await;
        }
        let _ = driver::send(&mut pwm, [0; 3]).await;
        power.set_low();
        log::info!("Cornix LED self-test complete");
    }
    let mut state = pattern::State::new(Instant::now().as_millis());
    let mut fade = pattern::Fade::new();
    let mut last = [[255;3];2];
    let mut powered = false;
    loop {
        match select3(links.next_event(), layers.next_event(), select(indicators.next_event(), Timer::after_millis(20))).await {
            Either3::First(e) => {
                state.link(e.connected, Instant::now().as_millis());
                log::info!("Cornix LED central_connected={}", e.connected);
            }
            Either3::Second(e) => {
                log::info!("Cornix LED layer={}", e.0);
                state.layer(e.0, Instant::now().as_millis());
            }
            Either3::Third(Either::First(e)) => {
                state.caps(e.caps_lock());
                log::info!("Cornix LED caps_lock={}", e.caps_lock());
            }
            Either3::Third(Either::Second(_)) => {}
        }
        let now = Instant::now().as_millis();
        let rgb = fade.pixels(state.pixels(now), now);
        if rgb != [[0;3];2] && !powered {
            power.set_high(); // Cornix is ACTIVE_HIGH, unlike Madula.
            Timer::after_millis(50).await;
            powered = true;
            last = [[255;3];2];
        }
        if powered && last != rgb {
            if driver::send_pixels(&mut pwm, rgb).await {
                log::debug!("Cornix LED PWM sent rgb={:?}", rgb);
                last = rgb;
                if rgb == [[0;3];2] {
                    power.set_low();
                    powered = false;
                }
            } else {
                log::warn!("Cornix LED PWM write failed");
            }
        }
    }
}
