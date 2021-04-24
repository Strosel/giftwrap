use giftwrap::*;

#[derive(Debug, Wrap, Unwrap)]
pub struct MyStruct(i64);

#[derive(Debug, Wrap, Unwrap)]
pub struct MyNamedStruct {
    f: i64,
}

fn main() {
    println!("{:?}", MyStruct::from(12));
    println!("{:?}", i64::from(MyStruct(12)));

    println!("{:?}", MyNamedStruct::from(-23));
    println!("{:?}", i64::from(MyNamedStruct { f: -23 }));
}
