//! Host-only wire peer for the deployed GUI WASM. Not a hardware emulator.
#[path = "../../vendor/rmk/src/host/rynk/legacy_macros.rs"]
mod legacy_macros;
use rmk_types::{action::KeyAction, constants::*, protocol::rynk::*};
use std::io::{self, BufRead};
fn answer<T: serde::Serialize>(h: RynkHeader, value: Result<T, RynkError>) -> Vec<u8> {
    let mut buf = [0; 4096];
    let n = encode_frame(&mut buf, h, &value).unwrap();
    buf[..n].to_vec()
}
fn main() {
    let mut macros = [0; MACRO_SPACE_SIZE];
    let mut key = KeyAction::No;
    for line in io::stdin().lock().lines() {
        let text = line.unwrap();
        let frame: Vec<u8> = text
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect();
        let h = RynkHeader::peek(&frame).unwrap();
        let decoded = cobs::decode_vec(&frame).unwrap();
        let payload = &decoded[3..];
        let out = match h.cmd {
            Cmd::GetVersion => answer(
                h,
                Ok(if std::env::var_os("TEST_OLD_PROTOCOL").is_some() {
                    ProtocolVersion { major: 0, minor: 1 }
                } else {
                    ProtocolVersion::CURRENT
                }),
            ),
            Cmd::GetCapabilities => answer(
                h,
                Ok(DeviceCapabilities {
                    num_layers: 5,
                    num_rows: 5,
                    num_cols: 14,
                    num_encoders: 2,
                    max_macros: 32,
                    macro_max_size: 32,
                    macros_writable: true,
                    storage_enabled: true,
                    is_split: true,
                    num_split_peripherals: 2,
                    ble_enabled: true,
                    num_ble_profiles: 8,
                    max_payload_size: RYNK_MAX_PAYLOAD_SIZE as u16,
                    max_bulk_keys: MAX_BULK_KEYS as u8,
                    max_bulk_items: MAX_BULK_ITEMS as u8,
                    bulk_transfer_supported: true,
                    ..Default::default()
                }),
            ),
            Cmd::GetDeviceInfo => answer(
                h,
                Ok(DeviceInfo {
                    rmk_version: FirmwareVersion {
                        major: 0,
                        minor: 9,
                        patch: 0,
                    },
                    vendor_id: 0x4c4b,
                    product_id: 0x4643,
                    manufacturer: "Madula".try_into().unwrap(),
                    product_name: "Madula RMK".try_into().unwrap(),
                    serial_number: "wire-test".try_into().unwrap(),
                }),
            ),
            Cmd::GetKeyAction => answer(h, Ok(key)),
            Cmd::SetKeyAction => {
                let r: SetKeyRequest = postcard::from_bytes(payload).unwrap();
                key = r.action;
                answer(h, Ok(()))
            }
            Cmd::GetMacro => {
                let i = postcard::from_bytes(payload).unwrap();
                answer(h, legacy_macros::read(&macros, i))
            }
            Cmd::SetMacro => {
                let r: SetMacroRequest = postcard::from_bytes(payload).unwrap();
                let result =
                    legacy_macros::replace(&macros, r.index, &r.macro_ops).map(|b| macros = b);
                answer(h, result)
            }
            Cmd::GetBleStatus => answer(h, Ok(rmk_types::ble::BleStatus::default())),
            _ => answer::<()>(h, Err(RynkError::UnknownCmd)),
        };
        println!(
            "{}",
            out.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
    }
}
