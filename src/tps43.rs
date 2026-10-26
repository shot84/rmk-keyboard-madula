//! TPS43/IQS5xx. Register protocol follows Azoteq IQS5xx B000 datasheet
//! and vendor/rmk/src/input_device/iqs5xx.rs (MIT/Apache-2.0).
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Output},
    peripherals::TWISPI0,
    twim::{self, Twim},
};
use embassy_time::{Duration, Timer, with_timeout};
use rmk::event::{
    Axis, AxisEvent, AxisValType, KeyboardEvent, PointingEvent, publish_event, publish_event_async,
};
#[path = "tps43_frame.rs"]
mod frame;
bind_interrupts!(pub struct Irqs { TWISPI0 => twim::InterruptHandler<TWISPI0>; });
#[derive(Debug)]
#[allow(dead_code)]
enum Error {
    Timeout,
    Bus(twim::Error),
    Product([u8; 4]),
    SensorReset,
}
struct Sensor {
    bus: Twim<'static>,
    ready: Input<'static>,
}
impl Sensor {
    async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        with_timeout(Duration::from_millis(200), self.bus.write(0x74, bytes))
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(Error::Bus)
    }
    async fn read(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Error> {
        with_timeout(
            Duration::from_millis(200),
            self.bus.write_read(0x74, &reg.to_be_bytes(), data),
        )
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(Error::Bus)
    }
    async fn initialize(&mut self) -> Result<(), Error> {
        let mut id = [0; 4];
        // Force comms can initially NAK after low-power mode.
        let mut result = self.read(0, &mut id).await;
        for _ in 0..2 {
            if result.is_ok() {
                break;
            }
            Timer::after_micros(200).await;
            result = self.read(0, &mut id).await;
        }
        result?;
        if !matches!(id, [0, 40, 0, 15] | [0, 58, 0, 15] | [0, 52, 0, 15]) {
            return Err(Error::Product(id));
        }
        // Separate writes are essential: each block begins with a new register address.
        // Streaming mode provides explicit drag release frames; active/idle 15/25ms.
        for bytes in [
            &[0x05, 0x8a, 30][..],
            &[0x05, 0x8e, 0x64, 0x00],
            &[0x05, 0x7a, 0, 15, 0, 15, 0, 25],
            &[0x06, 0x37, 5],
            &[0x06, 0x72, 1],
            &[0x06, 0x32, 0x0b],
            &[0x06, 0xb7, 3, 3],
            &[0x06, 0xbd, 0, 250],
            &[0x04, 0x31, 0x80],
            &[0xee, 0xee],
        ] {
            self.write(bytes).await?;
        }
        log::info!(
            "CornixTP TPS43 initialized: product={:?}, tap/hold/scroll enabled",
            id
        );
        Ok(())
    }
    async fn sample(&mut self) -> Result<[u8; 10], Error> {
        with_timeout(Duration::from_secs(2), self.ready.wait_for_high())
            .await
            .map_err(|_| Error::Timeout)?;
        let mut data = [0; 10];
        let read = self.read(0x000c, &mut data).await;
        // Always attempt to close the window, including after a failed read.
        let end = self.write(&[0xee, 0xee]).await;
        read?;
        end?;
        // Do not consume the same RDY-high window twice.
        with_timeout(Duration::from_millis(100), self.ready.wait_for_low())
            .await
            .map_err(|_| Error::Timeout)?;
        Ok(data)
    }
}
async fn key(col: u8, down: bool) {
    publish_event_async(KeyboardEvent::key(4, col, down)).await;
}
async fn tap(col: u8) {
    key(col, true).await;
    Timer::after_millis(8).await;
    key(col, false).await;
}
#[embassy_executor::task]
pub async fn run(bus: Twim<'static>, ready: Input<'static>, mut reset: Output<'static>) {
    let mut sensor = Sensor { bus, ready };
    loop {
        reset.set_low();
        Timer::after_millis(10).await;
        reset.set_high();
        Timer::after_millis(200).await;
        if let Err(e) = sensor.initialize().await {
            let _ = sensor.write(&[0xee, 0xee]).await;
            log::warn!("TPS43 init failed: {:?}; retrying", e);
            Timer::after_secs(2).await;
            continue;
        }
        let mut decoder = frame::Decoder::default();
        let mut holding = false;
        loop {
            let result = sensor
                .sample()
                .await
                .and_then(|data| decoder.decode(data).ok_or(Error::SensorReset));
            let out = match result {
                Ok(out) => out,
                Err(e) => {
                    log::warn!("TPS43 input stopped: {:?}; resetting", e);
                    break;
                }
            };
            if out.hold != holding {
                key(3, out.hold).await;
                holding = out.hold;
            }
            if out.tap != 0 && !holding {
                tap(if out.tap == 1 { 3 } else { 4 }).await;
            }
            for axis in 0..2 {
                let value = out.scroll[axis];
                let col = if axis == 0 {
                    if value < 0 { 8 } else { 9 }
                } else if value < 0 {
                    7
                } else {
                    6
                };
                for _ in 0..value.unsigned_abs() {
                    tap(col).await;
                }
            }
            let [x, y] = out.pointer;
            if x != 0 || y != 0 {
                publish_event(PointingEvent {
                    device_id: 0,
                    axes: [
                        AxisEvent {
                            typ: AxisValType::Rel,
                            axis: Axis::X,
                            value: x,
                        },
                        AxisEvent {
                            typ: AxisValType::Rel,
                            axis: Axis::Y,
                            value: y,
                        },
                        AxisEvent {
                            typ: AxisValType::Rel,
                            axis: Axis::Z,
                            value: 0,
                        },
                    ],
                });
            }
        }
        // A lost sensor must never leave the host's drag button pressed.
        if holding {
            key(3, false).await;
        }
        Timer::after_secs(1).await;
    }
}
