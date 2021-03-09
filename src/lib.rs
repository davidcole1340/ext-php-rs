#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
pub mod macros;
pub mod functions;
pub mod module;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use crate::info_table_start;

    #[test]
    fn test() {
        info_table_start!();

        info_table_header!("Hello", "World", "From", "Macro");
        info_table_header!();
        info_table_row!("Hello", "World!");

        info_table_end!();
    }
}
