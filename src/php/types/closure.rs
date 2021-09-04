use std::marker::{PhantomData, Unsize};

use crate::php::{
    args::Arg, class::ClassBuilder, exceptions::PhpException, execution_data::ExecutionData,
    flags::MethodFlags, function::FunctionBuilder, types::object::ClassMetadata,
};

use super::{
    object::RegisteredClass,
    zval::{FromZval, IntoZval, Zval},
};

fn test() {
    // let x = Closure::wrap(Box::new(|a| {
    //     let y: f64 = a + 5.0;
    //     format!("{}", y)
    // }) as Box<dyn PhpClosure>);

    let y = Closure::wrap(Box::new(|| {
        println!("Hello");
    }) as Box<dyn Fn()>);
}

pub struct Closure {
    func: Box<dyn PhpClosure>,
}

unsafe impl Send for Closure {}
unsafe impl Sync for Closure {}

impl Closure {
    pub fn wrap<T>(func: T) -> Self
    where
        T: PhpClosure + 'static,
    {
        Self {
            func: Box::new(func) as Box<dyn PhpClosure>,
        }
    }

    pub fn build() {
        let x = ClassBuilder::new("RustClosure")
            .method(
                FunctionBuilder::new("__invoke", Self::invoke)
                    .build()
                    .expect("h"),
                MethodFlags::Public,
            )
            .object_override::<Self>()
            .build()
            .expect("ok");
        CLOSURE_META.set_ce(x);
    }

    extern "C" fn invoke(ex: &mut ExecutionData, ret: &mut Zval) {
        let this = unsafe { ex.get_object::<Self>() }.expect("asdf");
        this.func.invoke(ex, ret);
    }
}

impl Default for Closure {
    fn default() -> Self {
        panic!("can't instantiate closure");
    }
}

static CLOSURE_META: ClassMetadata<Closure> = ClassMetadata::new();

impl RegisteredClass for Closure {
    fn get_metadata() -> &'static super::object::ClassMetadata<Self> {
        &CLOSURE_META
    }
}

pub unsafe trait PhpClosure {
    fn invoke(&self, ex: &mut ExecutionData, ret: &mut Zval);
}

unsafe impl<R> PhpClosure for Box<dyn Fn() -> R>
where
    R: IntoZval,
{
    fn invoke(&self, _: &mut ExecutionData, ret: &mut Zval) {
        if let Err(e) = self().set_zval(ret, false) {
            PhpException::default(e.to_string())
                .throw()
                .unwrap_or_else(|_| panic!("Failed to throw exception: {}", e.to_string()))
        }
    }
}

unsafe impl<A, R> PhpClosure for Box<dyn Fn(A) -> R>
where
    A: FromZval<'static>,
    R: IntoZval,
{
    fn invoke(&self, ex: &mut ExecutionData, ret: &mut Zval) {
        let mut a = Arg::new("a", A::TYPE);
        parse_args!(ex, a);
        self(a.val().expect("hello"))
            .set_zval(ret, false)
            .expect("Hello");
    }
}
