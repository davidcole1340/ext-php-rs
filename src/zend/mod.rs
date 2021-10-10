//! Types used to interact with the Zend engine.

mod _type;
pub mod ce;
mod class;
mod ex;
mod function;
mod globals;
mod handlers;
mod module;

pub use _type::ZendType;
pub use class::ClassEntry;
pub use ex::ExecuteData;
pub use function::FunctionEntry;
pub use globals::ExecutorGlobals;
pub use handlers::ZendObjectHandlers;
pub use module::ModuleEntry;
