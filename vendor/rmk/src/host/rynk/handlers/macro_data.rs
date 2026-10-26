//! Whole-macro Rynk 0.2 operations, retaining the existing flash encoding.
use super::{
    super::{RynkService, legacy_macros},
    Handle,
};
use rmk_types::{
    constants::MACRO_SPACE_SIZE,
    keyboard_macros::Macro,
    protocol::rynk::{
        RynkError, SetMacroRequest,
        command::{GetMacro, SetMacro},
    },
};
impl Handle<GetMacro> for RynkService<'_> {
    async fn handle(&self, index: u8) -> Result<Macro, RynkError> {
        let mut bytes = [0; MACRO_SPACE_SIZE];
        self.ctx.read_macro_buffer(0, &mut bytes);
        legacy_macros::read(&bytes, index)
    }
}
impl Handle<SetMacro> for RynkService<'_> {
    async fn handle(&self, r: SetMacroRequest) -> Result<(), RynkError> {
        if !cfg!(feature = "storage") {
            return Err(RynkError::NotReady);
        }
        let mut bytes = [0; MACRO_SPACE_SIZE];
        self.ctx.read_macro_buffer(0, &mut bytes);
        let updated = legacy_macros::replace(&bytes, r.index, &r.macro_ops)?;
        self.ctx.write_macro_buffer(0, &updated).await;
        Ok(())
    }
}
