use crate::builders::FunctionBuilder;

/// Implemented on ZSTs that represent PHP functions.
pub trait PhpFunction {
    /// Function used to 'build' the PHP function, returning a [`FunctionEntry`]
    /// to pass to the PHP interpreter.
    const FUNCTION_ENTRY: fn() -> FunctionBuilder<'static>;
}
