//! Madula ADS1220 bring-up. No pointer output until hardware data is validated.
//! Settings follow te9no/ads1220-zephyr-module and Cornix madula-trackpoint.
use embassy_nrf::{bind_interrupts, peripherals, spim};
use embassy_time::Timer;

bind_interrupts!(pub struct Irqs {
    TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
});

async fn registers(spi: &mut spim::Spim<'static>) -> Result<[u8; 4], spim::Error> {
    let mut result = [0; 4];
    for address in 0..4u8 {
        let mut rx = [0; 2];
        spi.transfer(&mut rx, &[0x20 | (address << 2), 0]).await?;
        result[address as usize] = rx[1];
    }
    Ok(result)
}

async fn sample(spi: &mut spim::Spim<'static>, mux: u8) -> Result<i32, spim::Error> {
    spi.write(&[0x40, (mux << 4) | 0x05]).await?;
    spi.write(&[0x08]).await?;
    // 330 SPS single-shot, margin above the nominal 3.03 ms conversion.
    Timer::after_micros(3500).await;
    let mut rx = [0; 4];
    spi.transfer(&mut rx, &[0x10, 0, 0, 0]).await?;
    Ok(i32::from_be_bytes([rx[1], rx[2], rx[3], 0]) >> 8)
}

#[embassy_executor::task]
pub async fn run(mut spi: spim::Spim<'static>) {
    // Keep keyboard/USB startup independent of sensor availability.
    Timer::after_secs(3).await;
    loop {
        let result = async {
            let mut reset_rx = [0; 1];
            spi.transfer(&mut reset_rx, &[0x06]).await?;
            Timer::after_millis(2).await;
            log::info!("LPPS reset registers={:02x?} (expected all zero)", registers(&mut spi).await?);
            // Gain 4, PGA bypass, 330 SPS single shot, REFP0, IDAC 100uA to REFP0.
            let expected = [0x05, 0x80, 0x43, 0xa0];
            for address in 0..4u8 {
                let mut rx = [0; 2];
                spi.transfer(&mut rx, &[0x40 | (address << 2), expected[address as usize]]).await?;
            }
            Timer::after_millis(10).await;
            let actual = registers(&mut spi).await?;
            log::info!("LPPS registers={:02x?}; expected={:02x?}", actual, expected);
            if actual != expected {
                log::warn!("LPPS register mismatch; no pointer output");
                return Ok::<(), spim::Error>(());
            }
            let mut sums = [0i64; 2];
            for _ in 0..100 {
                sums[0] += sample(&mut spi, 0).await? as i64;
                sums[1] += sample(&mut spi, 7).await? as i64;
            }
            let zero = [(sums[0] / 100) as i32, (sums[1] / 100) as i32];
            log::info!("LPPS diagnostic baseline={:?}; pointer output disabled", zero);
            let mut count = 0u32;
            loop {
                let x = sample(&mut spi, 0).await?;
                let y = sample(&mut spi, 7).await?;
                count += 1;
                if count % 125 == 0 {
                    log::info!("LPPS raw=({}, {}) delta=({}, {})", x, y, x-zero[0], y-zero[1]);
                    let actual = registers(&mut spi).await?;
                    if actual != [0x75, 0x80, 0x43, 0xa0] {
                        log::warn!("LPPS configuration lost: {:02x?}", actual);
                        return Ok(());
                    }
                }
                Timer::after_micros(500).await;
            }
        }.await;
        if let Err(error) = result { log::warn!("LPPS SPI error: {:?}", error); }
        Timer::after_secs(5).await;
    }
}
