use giftwrap::*;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

#[derive(Wrap, Unwrap, Debug)]
pub enum MyEnum {
    #[giftwrap(noWrap = true)]
    UnwrappedNumber {
        n: i64,
    },
    #[giftwrap(noUnwrap = true)]
    WrappedNumber {
        n: i64,
    },
    Text(String),
}

#[derive(Debug)]
pub struct Str<'a>(&'a str);

#[derive(Wrap, Unwrap, Debug)]
pub enum MyGenericEnum<'a, T> {
    Str(Str<'a>),
    #[giftwrap(wrapDepth = 1)]
    Gen(Option<T>),
    #[giftwrap(wrapDepth = 0)]
    Dep(Arc<Mutex<i32>>),
    #[giftwrap(noWrap = true, noUnwrap = true)]
    T(T),
}

fn main() {
    println!("{:?}", MyEnum::from(12));
    println!("{:?}", i64::try_from(MyEnum::UnwrappedNumber { n: 12 }));

    println!("{:?}", MyEnum::from(String::from("<-")));
    println!("{:?}", String::try_from(MyEnum::Text(String::from("->"))));

    println!("{:?}", MyGenericEnum::<()>::from(Str("<=")));
    println!("{:?}", Str::try_from(MyGenericEnum::<()>::Str(Str("=>"))));

    println!("{:?}", MyGenericEnum::<bool>::from(Some(true)));
    println!(
        "{:?}",
        Option::<bool>::try_from(MyGenericEnum::Gen(Some(false)))
    );

    println!("{:?}", MyGenericEnum::<()>::from(1i32));
    println!("{:?}", MyGenericEnum::<()>::from(Mutex::new(2i32)));
    println!(
        "{:?}",
        MyGenericEnum::<()>::from(Arc::new(Mutex::new(3i32)))
    );
}
