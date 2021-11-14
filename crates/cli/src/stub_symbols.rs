/// Generates mock symbols required to generate stub files from a downstream
/// crates CLI application.
#[macro_export]
macro_rules! stub_symbols {
    ($($s: ident),*) => {
        $(
            $crate::stub_symbols!(@INTERNAL; $s);
        )*
    };
    (@INTERNAL; $s: ident) => {
        #[allow(non_upper_case_globals)]
        #[no_mangle]
        pub static mut $s: *mut () = ::std::ptr::null_mut();
    };
}
