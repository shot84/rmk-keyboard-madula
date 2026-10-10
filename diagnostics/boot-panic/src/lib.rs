#![no_std]

#[cfg(not(feature = "boot-recovery"))]
use original_panic_probe as _;

// Only boot-audit images reserve this address in their linker memory map.
#[cfg(feature = "boot-recovery")]
const ADDRESS: usize = 0x20020000;
#[cfg(feature = "boot-recovery")]
const MAGIC: u32 = 0x4D424352;

/// Copy a retained diagnostic, if present. It is invalid after a power cycle.
#[cfg(feature = "boot-recovery")]
pub fn read(out: &mut [u8; 384]) -> Option<usize> {
    unsafe {
        if core::ptr::read_volatile(ADDRESS as *const u32) != MAGIC { return None; }
        let count = core::ptr::read_volatile((ADDRESS + 4) as *const u32) as usize;
        if count > out.len() { return None; }
        for (i, byte) in out[..count].iter_mut().enumerate() {
            *byte = core::ptr::read_volatile((ADDRESS + 8 + i) as *const u8);
        }
        Some(count)
    }
}

#[cfg(feature = "boot-recovery")]
fn capture(args: core::fmt::Arguments<'_>) -> ! {
    cortex_m::interrupt::disable();
    struct Writer(usize);
    impl core::fmt::Write for Writer {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            for byte in text.bytes() {
                if self.0 >= 384 { break; }
                unsafe { core::ptr::write_volatile((ADDRESS + 8 + self.0) as *mut u8, byte); }
                self.0 += 1;
            }
            Ok(())
        }
    }
    let mut writer = Writer(0);
    let _ = core::fmt::write(&mut writer, args);
    unsafe {
        core::ptr::write_volatile((ADDRESS + 4) as *mut u32, writer.0 as u32);
        core::ptr::write_volatile(ADDRESS as *mut u32, MAGIC);
    }
    embassy_nrf::pac::POWER.gpregret().write_value(embassy_nrf::pac::power::regs::Gpregret(0x57));
    cortex_m::asm::dsb();
    cortex_m::peripheral::SCB::sys_reset()
}

#[cfg(feature = "boot-recovery")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! { capture(format_args!("PANIC: {info}")) }

#[cfg(feature = "boot-recovery")]
#[cortex_m_rt::exception]
unsafe fn HardFault(frame: &cortex_m_rt::ExceptionFrame) -> ! {
    capture(format_args!("HARDFAULT: pc={:#010x} lr={:#010x}", frame.pc(), frame.lr()))
}
