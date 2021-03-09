use crate::{
    functions::c_str, php_info_print_table_end, php_info_print_table_header,
    php_info_print_table_row, php_info_print_table_start,
};

pub struct InfoTable {
    rows: Vec<(&'static str, &'static str)>,
    header: Option<(&'static str, &'static str)>,
}

impl InfoTable {
    pub fn new(cols: u32) -> Self {
        InfoTable {
            rows: vec![],
            header: None,
        }
    }

    pub fn header<S>(&mut self, key: S, value: S) -> &mut Self
    where
        S: Into<&'static str>,
    {
        self.header = Some((key.into(), value.into()));
        self
    }

    pub fn row<I, S>(&mut self, key: S, value: S) -> &mut Self
    where
        S: Into<&'static str>,
    {
        self.rows.push((key.into(), value.into()));
        self
    }

    pub fn build(&self) {
        unsafe { php_info_print_table_start() };

        if let Some(header) = self.header {
            unsafe {
                php_info_print_table_header(2, c_str(header.0), c_str(header.1));
            }
        }

        for row in self.rows.iter() {
            unsafe { php_info_print_table_row(2, c_str(row.0), c_str(row.1)) };
        }

        unsafe { php_info_print_table_end() };
    }
}
