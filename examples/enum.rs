use giftwrap::*;
use std::convert::TryFrom;

#[derive(Wrap, Unwrap, Debug)]
pub enum MyEnum {
    #[noWrap]
    UnwrappedNumber {
        n: i64,
    },
    #[noUnwrap]
    WrappedNumber {
        n: i64,
    },
    Text(String),
}

fn main() {
    println!("{:?}", MyEnum::from(12));
    println!("{:?}", i64::try_from(MyEnum::UnwrappedNumber { n: 12 }));

    println!("{:?}", MyEnum::from(String::from("<-")));
    println!("{:?}", String::try_from(MyEnum::Text(String::from("->"))));
}
