# INI Settings

Your PHP Extension may want to provide it's own PHP INI settings to configure behaviour. This can be done in the `#[php_startup]` annotated startup function.

## Registering INI Settings

All PHP INI definitions must be registered with PHP to get / set their values via the `php.ini` file or `ini_get() / ini_set()`.


```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;

#[php_module]
mod module {
    # use ext_php_rs::zend::IniEntryDef;
    # use ext_php_rs::flags::IniEntryPermission;

    #[php_startup]
    pub fn startup_function(ty: i32, module_number: i32) {
        let ini_entries: Vec<IniEntryDef> = vec![
            IniEntryDef::new(
                "my_extension.display_emoji".to_owned(),
                "yes".to_owned(),
                IniEntryPermission::All,
            ),
        ];
        IniEntryDef::register(ini_entries, module_number);
    }
}
# fn main() {}
```

## Getting INI Settings

The INI values are stored as part of the `GlobalExecutor`, and can be accessed via the `ini_values()` function. To retrieve the value for a registered INI setting

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;

#[php_module]
mod module {
    # use ext_php_rs::zend::ExecutorGlobals;
    #[php_startup]
    pub fn startup_function(ty: i32, module_number: i32) {
        // Get all INI values
        let ini_values = ExecutorGlobals::get().ini_values(); // HashMap<String, Option<String>>
        let my_ini_value = ini_values.get("my_extension.display_emoji"); // Option<Option<String>>
    }
}
# fn main() {}
```
