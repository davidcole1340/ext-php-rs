//! Types used to interact with the Zend engine.

mod _type;
pub mod ce;
mod class;
mod ex;
mod function;
mod globals;
mod handlers;
mod ini_entry_def;
mod module;
mod try_catch;

use crate::{error::Result, ffi::php_printf};
use std::ffi::CString;

pub use _type::ZendType;
pub use class::ClassEntry;
pub use ex::ExecuteData;
pub use function::Function;
pub use function::FunctionEntry;
pub use globals::ExecutorGlobals;
pub use handlers::ZendObjectHandlers;
pub use ini_entry_def::IniEntryDef;
pub use module::ModuleEntry;
#[cfg(feature = "embed")]
pub(crate) use try_catch::panic_wrapper;
pub use try_catch::{bailout, try_catch};

// Used as the format string for `php_printf`.
const FORMAT_STR: &[u8] = b"%s\0";

/// Prints to stdout using the `php_printf` function.
///
/// Also see the [`php_print`] and [`php_println`] macros.
///
/// # Arguments
///
/// * message - The message to print to stdout.
///
/// # Returns
///
/// Nothing on success, error if the message could not be converted to a
/// [`CString`].
pub fn printf(message: &str) -> Result<()> {
    let message = CString::new(message)?;
    unsafe {
        php_printf(FORMAT_STR.as_ptr().cast(), message.as_ptr());
    };
    Ok(())
}
