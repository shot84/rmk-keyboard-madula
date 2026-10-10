//! Madula 3-wire PMW3610. Pin/timing reference: te9no/zmk-driver-spi-three-wire.
//! Initialization sequence follows RMK 0.9 PMW3610; individual reads avoid burst mode.
use embassy_nrf::gpio::{Output, Flex, Input, OutputDrive, Pull};
use embassy_time::Timer;
use rmk::event::{publish_event, Axis, AxisEvent, AxisValType, PointingEvent};

struct Bus { clock: Output<'static>, data: Flex<'static>, cs: Output<'static> }
fn delay(us: u32) { cortex_m::asm::delay(64 * us); }
impl Bus {
    fn tx(&mut self, byte: u8) {
        self.data.set_as_output(OutputDrive::Standard);
        for bit in (0..8).rev() {
            self.clock.set_low();
            if byte & (1 << bit) != 0 { self.data.set_high(); } else { self.data.set_low(); }
            delay(1);
            self.clock.set_high();
            delay(1);
        }
    }
    fn finish(&mut self) {
        delay(4); self.cs.set_high(); self.data.set_as_input(Pull::None); delay(30);
    }
    fn read(&mut self, reg: u8) -> u8 {
        self.cs.set_low(); delay(1); self.tx(reg & 0x7f);
        self.data.set_as_input(Pull::None); delay(4);
        let mut byte = 0;
        for _ in 0..8 {
            self.clock.set_low(); delay(1); self.clock.set_high(); delay(1);
            byte = (byte << 1) | u8::from(self.data.is_high());
        }
        self.finish(); byte
    }
    fn write(&mut self, reg: u8, value: u8) {
        self.cs.set_low(); delay(1); self.tx(reg | 0x80); self.tx(value);
        delay(10); self.finish();
    }
    async fn initialize(&mut self) -> bool {
        self.write(0x3a, 0x5a); Timer::after_millis(50).await;
        let id = self.read(0x00);
        let revision = self.read(0x01);
        log::info!("PMW3610 id=0x{:02x} revision=0x{:02x}", id, revision);
        if id != 0x3e { return false; }
        self.write(0x41, 0xba); Timer::after_micros(300).await;
        self.write(0x2d, 0); Timer::after_millis(10).await;
        let observation = self.read(0x2d);
        if observation & 0x0f != 0x0f {
            self.write(0x41, 0xb5);
            log::warn!("PMW3610 initialization observation=0x{:02x}", observation);
            return false;
        }
        for reg in 0x02..=0x05 { self.read(reg); }
        self.write(0x11, 0xfd); // default performance, force awake
        self.write(0x1b, 4); self.write(0x1c, 4); self.write(0x1d, 15);
        self.write(0x7f, 0xff);
        self.write(0x05, 4); // 800 CPI; orientation applied by PointingProcessor
        let resolution = self.read(0x05);
        self.write(0x7f, 0); self.write(0x41, 0xb5);
        log::info!("PMW3610 CPI register=0x{:02x}; individual reads; force awake", resolution);
        resolution == 4
    }
}
fn signed12(value: u16) -> i16 { ((value << 4) as i16) >> 4 }

#[embassy_executor::task]
pub async fn run(clock: Output<'static>, data: Flex<'static>, cs: Output<'static>, irq: Input<'static>) {
    let mut bus = Bus { clock, data, cs };
    Timer::after_secs(3).await;
    loop {
        if !bus.initialize().await {
            log::warn!("PMW3610 not ready; output disabled, retry in 5s");
            Timer::after_secs(5).await; continue;
        }
        log::info!("PMW3610 ready; 800 CPI; swap XY + invert XY");
        let mut ticks = 0u32;
        let mut sum = [0i32; 2];
        loop {
            if irq.is_low() {
                let motion = bus.read(0x02);
                if motion == 0xff || bus.read(0x00) != 0x3e {
                    log::warn!("PMW3610 invalid response; motion discarded");
                    Timer::after_secs(1).await;
                    break;
                }
                if motion & 0x80 != 0 {
                    let xl = bus.read(0x03); let yl = bus.read(0x04); let high = bus.read(0x05);
                    let dx = signed12(u16::from(xl) | (u16::from(high & 0xf0) << 4));
                    let dy = signed12(u16::from(yl) | (u16::from(high & 0x0f) << 8));
                    sum[0] += i32::from(dx); sum[1] += i32::from(dy);
                    if dx != 0 || dy != 0 {
                        publish_event(PointingEvent { device_id: 0, axes: [
                            AxisEvent { typ: AxisValType::Rel, axis: Axis::X, value: dx },
                            AxisEvent { typ: AxisValType::Rel, axis: Axis::Y, value: dy },
                            AxisEvent { typ: AxisValType::Rel, axis: Axis::Z, value: 0 },
                        ] });
                    }
                }
            }
            ticks += 1;
            if ticks % 125 == 0 {
                let id = bus.read(0x00);
                log::info!("PMW3610 delta_sum={:?} irq={} id=0x{:02x}", sum, irq.is_low(), id);
                sum = [0; 2];
                if id != 0x3e { break; }
            }
            Timer::after_millis(8).await;
        }
    }
}
