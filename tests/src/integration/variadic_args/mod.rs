//! Rust type &[&Zval] must be converted to Vec<Zval> because of
//! lifetime hell.

use ext_php_rs::{prelude::*, types::Zval};

#[php_function]
pub fn test_variadic_args(params: &[&Zval]) -> Vec<Zval> {
    params.iter().map(|x| x.shallow_clone()).collect()
}

#[php_function]
pub fn test_variadic_add_required(number: u32, numbers: &[&Zval]) -> u32 {
    number
        + numbers
            .iter()
            .map(|x| u32::try_from(x.long().unwrap()).unwrap())
            .sum::<u32>()
}

#[php_function]
pub fn test_variadic_count(items: &[&Zval]) -> usize {
    items.len()
}

#[php_function]
pub fn test_variadic_types(values: &[&Zval]) -> Vec<String> {
    values
        .iter()
        .map(|v| {
            if v.is_long() {
                "long".to_string()
            } else if v.is_string() {
                "string".to_string()
            } else if v.is_double() {
                "double".to_string()
            } else if v.is_true() || v.is_false() {
                "bool".to_string()
            } else if v.is_array() {
                "array".to_string()
            } else if v.is_object() {
                "object".to_string()
            } else if v.is_null() {
                "null".to_string()
            } else {
                "unknown".to_string()
            }
        })
        .collect()
}

#[php_function]
pub fn test_variadic_strings(prefix: String, suffixes: &[&Zval]) -> Vec<String> {
    suffixes
        .iter()
        .filter_map(|v| v.str())
        .map(|s| format!("{prefix}{s}"))
        .collect()
}

#[php_function]
pub fn test_variadic_sum_all(nums: &[&Zval]) -> i64 {
    nums.iter().filter_map(|v| v.long()).sum()
}

#[php_function]
pub fn test_variadic_optional(required: String, optional: Option<i64>, extras: &[&Zval]) -> String {
    let opt_str = optional.map_or_else(|| "none".to_string(), |v| v.to_string());
    format!("{required}-{opt_str}-{}", extras.len())
}

#[php_function]
pub fn test_variadic_empty_check(items: &[&Zval]) -> bool {
    items.is_empty()
}

#[php_function]
pub fn test_variadic_first_last(items: &[&Zval]) -> Vec<Zval> {
    let mut result = Vec::new();
    if let Some(first) = items.first() {
        result.push(first.shallow_clone());
    }
    if let Some(last) = items.last() {
        if items.len() > 1 {
            result.push(last.shallow_clone());
        }
    }
    result
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_variadic_args))
        .function(wrap_function!(test_variadic_add_required))
        .function(wrap_function!(test_variadic_count))
        .function(wrap_function!(test_variadic_types))
        .function(wrap_function!(test_variadic_strings))
        .function(wrap_function!(test_variadic_sum_all))
        .function(wrap_function!(test_variadic_optional))
        .function(wrap_function!(test_variadic_empty_check))
        .function(wrap_function!(test_variadic_first_last))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_variadic_args() {
        assert!(crate::integration::test::run_php(
            "variadic_args/variadic_args.php"
        ));
    }
}
