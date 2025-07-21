use crate::{convert::IntoZval, describe::DocComments};

pub trait RegisteredInterface {
    fn constants() -> &'static [(&'static str, &'static impl IntoZval, DocComments)];
}
