//! PWM timing reference: numachang/cornix-rmk-custom src/ws2812.rs (MIT).
//! See licenses/cornix-rmk-custom-MIT.txt. No CPU timing/interrupt masking.
use embassy_nrf::pwm::{SequenceConfig, SequencePwm, SingleSequenceMode, SingleSequencer};
use embassy_time::Timer;

#[path = "ws2812_pwm_frame.rs"]
mod frame;

pub async fn send(pwm: &mut SequencePwm<'static>, rgb: [u8; 3]) -> bool {
    send_pixels(pwm, [rgb,rgb]).await
}
pub async fn send_pixels(pwm: &mut SequencePwm<'static>, pixels: [[u8;3];2]) -> bool {
    let words = frame::encode_pixels(pixels);
    let sequence = SingleSequencer::new(pwm, &words, SequenceConfig::default());
    if sequence.start(SingleSequenceMode::Times(1)).is_err() {
        return false;
    }
    // 288 periods at 1.25 us = 360 us. Keep DMA buffer/sequence alive.
    Timer::after_millis(1).await;
    true
}
