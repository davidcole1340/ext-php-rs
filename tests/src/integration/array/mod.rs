use std::collections::{BTreeMap, HashMap};

use ext_php_rs::{
    convert::IntoZval,
    ffi::HashTable,
    php_function,
    prelude::ModuleBuilder,
    types::{ArrayKey, Zval},
    wrap_function,
};

#[php_function]
pub fn test_array(a: Vec<String>) -> Vec<String> {
    a
}

#[php_function]
pub fn test_array_assoc(a: HashMap<String, String>) -> HashMap<String, String> {
    a
}

#[php_function]
pub fn test_array_assoc_array_keys(a: Vec<(ArrayKey, String)>) -> Vec<(ArrayKey, String)> {
    a
}

#[php_function]
pub fn test_btree_map(a: BTreeMap<ArrayKey, String>) -> BTreeMap<ArrayKey, String> {
    a
}

#[php_function]
pub fn test_array_keys() -> Zval {
    let mut ht = HashTable::new();
    ht.insert(-42, "foo").unwrap();
    ht.insert(0, "bar").unwrap();
    ht.insert(5, "baz").unwrap();
    ht.insert("10", "qux").unwrap();
    ht.insert("quux", "quuux").unwrap();

    ht.into_zval(false).unwrap()
}

pub fn build_module(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(test_array))
        .function(wrap_function!(test_array_assoc))
        .function(wrap_function!(test_array_assoc_array_keys))
        .function(wrap_function!(test_btree_map))
        .function(wrap_function!(test_array_keys))
}

#[cfg(test)]
mod tests {
    #[test]
    fn array_works() {
        assert!(crate::integration::test::run_php("array/array.php"));
    }
}
