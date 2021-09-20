use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use regex::Regex;

const MIN_PHP_API_VER: u32 = 20200930;
const MAX_PHP_API_VER: u32 = 20200930;

fn main() {
    // rerun if wrapper header is changed
    println!("cargo:rerun-if-changed=src/wrapper/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper/wrapper.c");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    // check for docs.rs and use stub bindings if required
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=docs.rs detected - using stub bindings");
        println!("cargo:rustc-cfg=php_debug");

        std::fs::copy("docsrs_bindings.rs", out_path)
            .expect("Unable to copy docs.rs stub bindings to output directory.");
        return;
    }

    // use php-config to fetch includes
    let includes_cmd = Command::new("php-config")
        .arg("--includes")
        .output()
        .expect("Unable to run `php-config`. Please ensure it is visible in your PATH.");

    if !includes_cmd.status.success() {
        let stderr = String::from_utf8(includes_cmd.stderr)
            .unwrap_or_else(|_| String::from("Unable to read stderr"));
        panic!("Error running `php-config`: {}", stderr);
    }

    // Ensure the PHP API version is supported.
    // We could easily use grep and sed here but eventually we want to support Windows,
    // so it's easier to just use regex.
    let php_i_cmd = Command::new("php")
        .arg("-i")
        .output()
        .expect("Unable to run `php -i`. Please ensure it is visible in your PATH.");

    if !php_i_cmd.status.success() {
        let stderr = str::from_utf8(&includes_cmd.stderr).unwrap_or("Unable to read stderr");
        panic!("Error running `php -i`: {}", stderr);
    }

    let api_ver = Regex::new(r"PHP API => ([0-9]+)")
        .unwrap()
        .captures_iter(
            str::from_utf8(&php_i_cmd.stdout).expect("Unable to parse `php -i` stdout as UTF-8"),
        )
        .next()
        .and_then(|ver| ver.get(1))
        .and_then(|ver| ver.as_str().parse::<u32>().ok())
        .expect("Unable to retrieve PHP API version from `php -i`.");

    if api_ver < MIN_PHP_API_VER || api_ver > MAX_PHP_API_VER {
        panic!("The current version of PHP is not supported. Current PHP API version: {}, requires a version between {} and {}", api_ver, MIN_PHP_API_VER, MAX_PHP_API_VER);
    }

    let includes =
        String::from_utf8(includes_cmd.stdout).expect("unable to parse `php-config` stdout");

    // Build `wrapper.c` and link to Rust.
    cc::Build::new()
        .file("src/wrapper/wrapper.c")
        .includes(
            str::replace(includes.as_ref(), "-I", "")
                .split(' ')
                .map(|path| Path::new(path)),
        )
        .compile("wrapper");

    let mut bindgen = bindgen::Builder::default()
        .header("src/wrapper/wrapper.h")
        .clang_args(includes.split(' '))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .no_copy("_zend_value")
        .no_copy("_zend_string")
        .layout_tests(env::var("EXT_PHP_RS_TEST").is_ok());

    for binding in ALLOWED_BINDINGS.iter() {
        bindgen = bindgen
            .allowlist_function(binding)
            .allowlist_type(binding)
            .allowlist_var(binding);
    }

    bindgen
        .generate()
        .expect("Unable to generate bindings for PHP")
        .write_to_file(out_path)
        .expect("Unable to write bindings file.");

    let configure = Configure::get();

    if configure.has_zts() {
        println!("cargo:rustc-cfg=php_zts");
    }

    if configure.debug() {
        println!("cargo:rustc-cfg=php_debug");
    }
}

struct Configure(String);

impl Configure {
    pub fn get() -> Self {
        let cmd = Command::new("php-config")
        .arg("--configure-options")
        .output()
        .expect("Unable to run `php-config --configure-options`. Please ensure it is visible in your PATH.");

        if !cmd.status.success() {
            let stderr = String::from_utf8(cmd.stderr)
                .unwrap_or_else(|_| String::from("Unable to read stderr"));
            panic!("Error running `php -i`: {}", stderr);
        }

        // check for the ZTS feature flag in configure
        let stdout =
            String::from_utf8(cmd.stdout).expect("Unable to read stdout from `php-config`.");
        Self(stdout)
    }

    pub fn has_zts(&self) -> bool {
        self.0.contains("--enable-zts")
    }

    pub fn debug(&self) -> bool {
        self.0.contains("--enable-debug")
    }
}

/// Array of functions/types used in `ext-php-rs` - used to allowlist when generating
/// bindings, as we don't want to generate bindings for everything (i.e. stdlib headers).
const ALLOWED_BINDINGS: &[&str] = &[
    "HashTable",
    "_Bucket",
    "_call_user_function_impl",
    "_efree",
    "_emalloc",
    "_zend_executor_globals",
    "_zend_expected_type",
    "_zend_expected_type_Z_EXPECTED_ARRAY",
    "_zend_expected_type_Z_EXPECTED_BOOL",
    "_zend_expected_type_Z_EXPECTED_DOUBLE",
    "_zend_expected_type_Z_EXPECTED_LONG",
    "_zend_expected_type_Z_EXPECTED_OBJECT",
    "_zend_expected_type_Z_EXPECTED_RESOURCE",
    "_zend_expected_type_Z_EXPECTED_STRING",
    "_zend_new_array",
    "_zval_struct__bindgen_ty_1",
    "_zval_struct__bindgen_ty_2",
    "ext_php_rs_executor_globals",
    "ext_php_rs_php_build_id",
    "ext_php_rs_zend_object_alloc",
    "ext_php_rs_zend_object_release",
    "ext_php_rs_zend_string_init",
    "ext_php_rs_zend_string_release",
    "object_properties_init",
    "php_info_print_table_end",
    "php_info_print_table_header",
    "php_info_print_table_row",
    "php_info_print_table_start",
    "std_object_handlers",
    "zend_array_destroy",
    "zend_array_dup",
    "zend_ce_argument_count_error",
    "zend_ce_arithmetic_error",
    "zend_ce_compile_error",
    "zend_ce_division_by_zero_error",
    "zend_ce_error_exception",
    "zend_ce_exception",
    "zend_ce_parse_error",
    "zend_ce_throwable",
    "zend_ce_type_error",
    "zend_ce_unhandled_match_error",
    "zend_ce_value_error",
    "zend_class_entry",
    "zend_declare_class_constant",
    "zend_declare_property",
    "zend_do_implement_interface",
    "zend_execute_data",
    "zend_function_entry",
    "zend_hash_clean",
    "zend_hash_index_del",
    "zend_hash_index_find",
    "zend_hash_index_update",
    "zend_hash_next_index_insert",
    "zend_hash_str_del",
    "zend_hash_str_find",
    "zend_hash_str_update",
    "zend_internal_arg_info",
    "zend_is_callable",
    "zend_long",
    "zend_lookup_class_ex",
    "zend_module_entry",
    "zend_object",
    "zend_object_handlers",
    "zend_object_std_init",
    "zend_objects_clone_members",
    "zend_register_bool_constant",
    "zend_register_double_constant",
    "zend_register_internal_class_ex",
    "zend_register_long_constant",
    "zend_register_string_constant",
    "zend_resource",
    "zend_string",
    "zend_string_init_interned",
    "zend_throw_exception_ex",
    "zend_type",
    "zend_value",
    "zend_wrong_parameters_count_error",
    "zval",
    "CONST_CS",
    "CONST_DEPRECATED",
    "CONST_NO_FILE_CACHE",
    "CONST_PERSISTENT",
    "HT_MIN_SIZE",
    "IS_ARRAY",
    "IS_ARRAY_EX",
    "IS_CALLABLE",
    "IS_CONSTANT_AST",
    "IS_CONSTANT_AST_EX",
    "IS_DOUBLE",
    "IS_FALSE",
    "IS_INTERNED_STRING_EX",
    "IS_LONG",
    "IS_MIXED",
    "IS_NULL",
    "IS_OBJECT",
    "IS_OBJECT_EX",
    "IS_REFERENCE",
    "IS_REFERENCE_EX",
    "IS_RESOURCE",
    "IS_RESOURCE_EX",
    "IS_STRING",
    "IS_STRING_EX",
    "IS_TRUE",
    "IS_TYPE_COLLECTABLE",
    "IS_TYPE_REFCOUNTED",
    "IS_UNDEF",
    "IS_VOID",
    "MAY_BE_ANY",
    "MAY_BE_BOOL",
    "USING_ZTS",
    "ZEND_ACC_ABSTRACT",
    "ZEND_ACC_ANON_CLASS",
    "ZEND_ACC_CALL_VIA_TRAMPOLINE",
    "ZEND_ACC_CHANGED",
    "ZEND_ACC_CLOSURE",
    "ZEND_ACC_CONSTANTS_UPDATED",
    "ZEND_ACC_CTOR",
    "ZEND_ACC_DEPRECATED",
    "ZEND_ACC_DONE_PASS_TWO",
    "ZEND_ACC_EARLY_BINDING",
    "ZEND_ACC_FAKE_CLOSURE",
    "ZEND_ACC_FINAL",
    "ZEND_ACC_GENERATOR",
    "ZEND_ACC_HAS_FINALLY_BLOCK",
    "ZEND_ACC_HAS_RETURN_TYPE",
    "ZEND_ACC_HAS_TYPE_HINTS",
    "ZEND_ACC_HAS_UNLINKED_USES",
    "ZEND_ACC_HEAP_RT_CACHE",
    "ZEND_ACC_IMMUTABLE",
    "ZEND_ACC_IMPLICIT_ABSTRACT_CLASS",
    "ZEND_ACC_INTERFACE",
    "ZEND_ACC_LINKED",
    "ZEND_ACC_NEARLY_LINKED",
    "ZEND_ACC_NEVER_CACHE",
    "ZEND_ACC_NO_DYNAMIC_PROPERTIES",
    "ZEND_ACC_PRELOADED",
    "ZEND_ACC_PRIVATE",
    "ZEND_ACC_PROMOTED",
    "ZEND_ACC_PROPERTY_TYPES_RESOLVED",
    "ZEND_ACC_PROTECTED",
    "ZEND_ACC_PUBLIC",
    "ZEND_ACC_RESOLVED_INTERFACES",
    "ZEND_ACC_RESOLVED_PARENT",
    "ZEND_ACC_RETURN_REFERENCE",
    "ZEND_ACC_REUSE_GET_ITERATOR",
    "ZEND_ACC_STATIC",
    "ZEND_ACC_STRICT_TYPES",
    "ZEND_ACC_TOP_LEVEL",
    "ZEND_ACC_TRAIT",
    "ZEND_ACC_TRAIT_CLONE",
    "ZEND_ACC_UNRESOLVED_VARIANCE",
    "ZEND_ACC_USES_THIS",
    "ZEND_ACC_USE_GUARDS",
    "ZEND_ACC_VARIADIC",
    "ZEND_DEBUG",
    "ZEND_HAS_STATIC_IN_METHODS",
    "ZEND_ISEMPTY",
    "ZEND_MM_ALIGNMENT",
    "ZEND_MM_ALIGNMENT_MASK",
    "ZEND_MODULE_API_NO",
    "ZEND_PROPERTY_EXISTS",
    "ZEND_PROPERTY_ISSET",
    "Z_TYPE_FLAGS_SHIFT",
    "_IS_BOOL",
    "_ZEND_IS_VARIADIC_BIT",
    "_ZEND_SEND_MODE_SHIFT",
    "_ZEND_TYPE_NULLABLE_BIT",
    "ts_rsrc_id",
    "_ZEND_TYPE_NAME_BIT",
    "zval_ptr_dtor",
    "zend_refcounted_h",
];
