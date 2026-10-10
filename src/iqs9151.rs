//! IQS9151 single-finger pointer bring-up. No gesture emulation yet.
//! Init sequence adapted from ShiniNet/zmk-driver-iqs9151 (MIT; see licenses).
use embassy_nrf::{bind_interrupts, peripherals::TWISPI0, gpio::Input, twim::{self, Twim}};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use rmk::event::{publish_event, Axis, AxisEvent, AxisValType, PointingEvent};
use crate::iqs9151_config::*;
#[path = "iqs9151_frame.rs"]
mod frame;

bind_interrupts!(pub struct Irqs { TWISPI0 => twim::InterruptHandler<TWISPI0>; });
#[derive(Debug)]
#[allow(dead_code)] // Payloads are retained for the CDC Debug formatter.
enum Error { ReadyTimeout, TransferTimeout, Bus(twim::Error), Product(u16), ResetTimeout, AtiTimeout, Reset }
struct Sensor { bus: Twim<'static>, ready: Input<'static> }
impl Sensor {
    async fn ready(&mut self) -> Result<(), Error> {
        with_timeout(Duration::from_millis(500), self.ready.wait_for_low()).await
            .map_err(|_| Error::ReadyTimeout)
    }
    async fn read(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Error> {
        self.ready().await?;
        with_timeout(Duration::from_millis(100), self.bus.write_read(0x56, &reg.to_le_bytes(), data))
            .await.map_err(|_| Error::TransferTimeout)?.map_err(Error::Bus)
    }
    async fn word(&mut self, reg: u16) -> Result<u16, Error> {
        let mut data = [0; 2]; self.read(reg, &mut data).await?;
        Ok(u16::from_le_bytes(data))
    }
    async fn write(&mut self, reg: u16, data: &[u8]) -> Result<(), Error> {
        for (index, chunk) in data.chunks(30).enumerate() {
            self.ready().await?;
            let mut tx = [0; 32];
            tx[..2].copy_from_slice(&(reg + (index * 30) as u16).to_le_bytes());
            tx[2..2 + chunk.len()].copy_from_slice(chunk);
            with_timeout(Duration::from_millis(100), self.bus.write(0x56, &tx[..2 + chunk.len()]))
                .await.map_err(|_| Error::TransferTimeout)?.map_err(Error::Bus)?;
        }
        Ok(())
    }
    async fn set(&mut self, reg: u16, word: u16) -> Result<(), Error> {
        self.write(reg, &word.to_le_bytes()).await
    }
    async fn initialize(&mut self) -> Result<(), Error> {
        let id = self.word(0x1000).await?;
        log::info!("IQS9151 product=0x{:04x}", id);
        if id != 0x09bc { return Err(Error::Product(id)); }
        let ctrl = self.word(0x11bc).await?;
        self.set(0x11bc, ctrl | (1 << 9)).await?;
        let start = Instant::now();
        loop {
            if self.word(0x1020).await? & 0x80 != 0 { break; }
            if start.elapsed().as_millis() > 3000 { return Err(Error::ResetTimeout); }
            Timer::after_millis(10).await;
        }
        let ctrl = self.word(0x11bc).await?;
        self.set(0x11bc, ctrl | 0x80).await?;
        Timer::after_millis(100).await;
        for (reg, data) in [(0x115c, ALP_COMPENSATION), (0x1178, MAIN_CONFIG),
                            (0x1218, RXTX_MAP), (0x1246, CHANNEL_DISABLE), (0x129e, SNAP_ENABLE)] {
            self.write(reg, data).await?;
        }
        // Same hardware rotation (270 degrees CCW) and tuning as Madula ZMK.
        let settings = self.word(0x11e2).await?;
        self.set(0x11e2, (settings & !7) | 5).await?;
        for (reg, val) in [(0x11e6,2457), (0x11e8,3072), (0x1196,400),
                           (0x11ea,30), (0x11ec,511), (0x11f6,0), (0x11f8,0)] {
            self.set(reg, val).await?;
        }
        self.write(0x11ee, &[80]).await?;
        // System control is LE16: ATI request bits 5 and 6 belong to the low byte.
        self.set(0x11bc, 0x60).await?;
        let start = Instant::now();
        loop {
            if self.word(0x11bc).await? & 0x60 == 0 { break; }
            if start.elapsed().as_millis() > 3000 { return Err(Error::AtiTimeout); }
            Timer::after_millis(10).await;
        }
        // Keep streaming mode for bring-up: read one frame per RDY window.
        // Idle event-mode silence must not be treated as a failure.
        let settings = self.word(0x11be).await?;
        self.set(0x11be, settings & !0x100).await?;
        log::info!("IQS9151 configured; single-finger pointer; gestures disabled");
        Ok(())
    }
    async fn sample(&mut self) -> Result<(i16, i16, u8), Error> {
        let mut frame = [0; 16];
        self.read(0x1014, &mut frame).await?;
        frame::decode(&frame).ok_or(Error::Reset)
    }
}

#[embassy_executor::task]
pub async fn run(bus: Twim<'static>, ready: Input<'static>) {
    let mut sensor = Sensor { bus, ready };
    Timer::after_secs(3).await;
    loop {
        if let Err(e) = sensor.initialize().await {
            log::warn!("IQS9151 init failed: {:?}; retry in 5s", e);
            Timer::after_secs(5).await; continue;
        }
        let mut report = Instant::now();
        let mut sum = [0i32; 2];
        loop {
            match sensor.sample().await {
                Ok((dx, dy, fingers)) => {
                    if dx != 0 || dy != 0 {
                        publish_event(PointingEvent { device_id: 0, axes: [
                            AxisEvent { typ: AxisValType::Rel, axis: Axis::X, value: dx },
                            AxisEvent { typ: AxisValType::Rel, axis: Axis::Y, value: dy },
                            AxisEvent { typ: AxisValType::Rel, axis: Axis::Z, value: 0 },
                        ] });
                        sum[0] += i32::from(dx); sum[1] += i32::from(dy);
                    }
                    if report.elapsed().as_secs() >= 1 {
                        log::info!("IQS9151 fingers={} delta_sum={:?}", fingers, sum);
                        sum = [0; 2]; report = Instant::now();
                    }
                }
                Err(e) => {
                    log::warn!("IQS9151 read failed: {:?}; output disabled", e);
                    break;
                }
            }
            // Allow RDY deassertion after STOP and yield to BLE/USB tasks.
            Timer::after_millis(1).await;
        }
        Timer::after_secs(2).await;
    }
}
