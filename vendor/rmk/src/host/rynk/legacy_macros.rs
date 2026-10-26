//! Rynk 0.2 wire macros mapped to the existing RMK 0.9 Vial byte buffer.
//! Validate and build a replacement before changing the live buffer. Storage
//! enums, key bytes, and macro playback remain compatible with rc.5.
use heapless::Vec;
use rmk_types::{
    action::Action,
    constants::{MACRO_MAX_NUM, MACRO_SPACE_SIZE},
    keyboard_macros::{Macro, MacroOp},
    keycode::KeyCode,
    protocol::rynk::RynkError,
};

fn span(bytes: &[u8], index: u8) -> Result<core::ops::Range<usize>, RynkError> {
    if index as usize >= MACRO_MAX_NUM {
        return Err(RynkError::Invalid);
    }
    let mut start = 0;
    for slot in 0..=index {
        let end = bytes[start..]
            .iter()
            .position(|b| *b == 0)
            .map(|n| start + n)
            .ok_or(RynkError::Invalid)?;
        if slot == index {
            return Ok(start..end);
        }
        start = end + 1;
    }
    Err(RynkError::Invalid)
}

pub fn read(bytes: &[u8], index: u8) -> Result<Macro, RynkError> {
    let mut bytes = &bytes[span(bytes, index)?];
    let mut ops = Macro::default();
    while let Some((&first, rest)) = bytes.split_first() {
        let op = if first == 1 {
            let kind = *rest.first().ok_or(RynkError::Invalid)?;
            match kind {
                1..=3 if rest.len() >= 2 => {
                    let action = Action::Key(KeyCode::Hid(rest[1].into()));
                    bytes = &rest[2..];
                    match kind {
                        1 => MacroOp::Tap(action),
                        2 => MacroOp::Press(action),
                        _ => MacroOp::Release(action),
                    }
                }
                4 if rest.len() >= 3 => {
                    let ms = (rest[1] as u16 - 1) + (rest[2] as u16 - 1) * 255;
                    bytes = &rest[3..];
                    MacroOp::Delay(ms)
                }
                // Extended Vial actions cannot run on the legacy Rynk engine.
                _ => return Err(RynkError::Invalid),
            }
        } else if first.is_ascii() {
            bytes = rest;
            MacroOp::Char(first)
        } else {
            return Err(RynkError::Invalid);
        };
        ops.0.push(op).map_err(|_| RynkError::Invalid)?;
    }
    Ok(ops)
}

pub fn replace(
    bytes: &[u8; MACRO_SPACE_SIZE],
    index: u8,
    ops: &Macro,
) -> Result<[u8; MACRO_SPACE_SIZE], RynkError> {
    let mut encoded: Vec<u8, MACRO_SPACE_SIZE> = Vec::new();
    for op in ops.iter() {
        match *op {
            MacroOp::Tap(Action::Key(KeyCode::Hid(key)))
            | MacroOp::Press(Action::Key(KeyCode::Hid(key)))
            | MacroOp::Release(Action::Key(KeyCode::Hid(key)))
                if key as u8 != 0 =>
            {
                let kind = match op {
                    MacroOp::Tap(_) => 1,
                    MacroOp::Press(_) => 2,
                    _ => 3,
                };
                encoded
                    .extend_from_slice(&[1, kind, key as u8])
                    .map_err(|_| RynkError::Invalid)?;
            }
            MacroOp::Delay(ms) if ms <= 65024 => {
                encoded
                    .extend_from_slice(&[1, 4, (ms % 255 + 1) as u8, (ms / 255 + 1) as u8])
                    .map_err(|_| RynkError::Invalid)?;
            }
            MacroOp::Char(c) if c.is_ascii() && c > 1 => {
                encoded.push(c).map_err(|_| RynkError::Invalid)?;
            }
            _ => return Err(RynkError::Invalid),
        }
    }
    let old = span(bytes, index)?;
    // Preserve all other slots, including empty macros and their terminators.
    let used = span(bytes, (MACRO_MAX_NUM - 1) as u8)?.end + 1;
    let new_len = used - old.len() + encoded.len();
    if new_len > MACRO_SPACE_SIZE {
        return Err(RynkError::Invalid);
    }
    let mut result = [0; MACRO_SPACE_SIZE];
    result[..old.start].copy_from_slice(&bytes[..old.start]);
    let end = old.start + encoded.len();
    result[old.start..end].copy_from_slice(&encoded);
    result[end..new_len].copy_from_slice(&bytes[old.end..used]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmk_types::keycode::HidKeyCode;
    #[test]
    fn edit_one_slot_preserves_neighbors_and_round_trips() {
        let a = Macro::from_slice(&[
            MacroOp::Char(b'a'),
            MacroOp::Delay(300),
            MacroOp::Tap(Action::Key(KeyCode::Hid(HidKeyCode::A))),
        ])
        .unwrap();
        let b = Macro::from_slice(&[MacroOp::Char(b'b')]).unwrap();
        let bytes = replace(&[0; MACRO_SPACE_SIZE], 31, &b).unwrap();
        let bytes = replace(&bytes, 0, &a).unwrap();
        assert_eq!(read(&bytes, 0).unwrap(), a);
        assert_eq!(read(&bytes, 31).unwrap(), b);
        let bytes = replace(&bytes, 0, &Macro::default()).unwrap();
        assert!(read(&bytes, 0).unwrap().is_empty());
        assert_eq!(read(&bytes, 31).unwrap(), b);
    }
    #[test]
    fn invalid_and_unsupported_inputs_never_modify_storage() {
        let bytes = [0; MACRO_SPACE_SIZE];
        for op in [
            MacroOp::PauseForRelease,
            MacroOp::Delay(65535),
            MacroOp::Char(0),
            MacroOp::Char(255),
        ] {
            assert!(replace(&bytes, 0, &Macro::from_slice(&[op]).unwrap()).is_err());
        }
        assert!(read(&bytes, 32).is_err());
        assert!(replace(&bytes, 32, &Macro::default()).is_err());
        assert!(read(&[1, 4, 1, 0], 0).is_err());
        assert_eq!(bytes, [0; MACRO_SPACE_SIZE]);
    }
    #[test]
    fn shared_capacity_overflow_is_rejected() {
        let full = Macro::from_slice(&[MacroOp::Delay(65024); 32]).unwrap();
        let mut bytes = [0; MACRO_SPACE_SIZE];
        loop {
            let next = (0..32)
                .find(|&i| read(&bytes, i).unwrap().is_empty())
                .unwrap();
            match replace(&bytes, next, &full) {
                Ok(new) => bytes = new,
                Err(_) => {
                    assert!(read(&bytes, next).unwrap().is_empty());
                    break;
                }
            }
        }
    }
}
