# INI Builder

When configuring a SAPI you may use `IniBuilder` to load INI settings as text.
This is useful for setting up configurations required by the SAPI capabilities.

INI settings applied to a SAPI through `sapi.ini_entries` will be immutable,
meaning they cannot be changed at runtime. This is useful for applying settings
to match hard requirements of the way your SAPI works.

To apply _configurable_ defaults it is recommended to use a `sapi.ini_defaults`
callback instead, which will allow settings to be changed at runtime.

```rust,no_run,ignore
use ext_php_rs::builder::{IniBuilder, SapiBuilder};

# fn main() {
// Create a new IniBuilder instance.
let mut builder = IniBuilder::new();

// Append a single key/value pair to the INIT buffer with an unquoted value.
builder.unquoted("log_errors", "1");

// Append a single key/value pair to the INI buffer with a quoted value.
builder.quoted("default_mimetype", "text/html");

// Append INI line text as-is. A line break will be automatically appended.
builder.define("memory_limit=128MB");

// Prepend INI line text as-is. No line break insertion will occur.
builder.prepend("error_reporting=0\ndisplay_errors=1\n");

// Construct a SAPI.
let mut sapi = SapiBuilder::new("name", "pretty_name").build()
  .expect("should build SAPI");

// Dump INI entries from the builder into the SAPI.
sapi.ini_entries = builder.finish();
# }
