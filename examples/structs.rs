use giftwrap::Wrap;

#[derive(Debug, Wrap)]
pub struct MyStruct(i64);

#[derive(Debug, Wrap)]
pub struct MyNamedStruct {
    f: i64,
}

fn main() {
    println!("{:?}", MyStruct::from(12));
    println!("{:?}", i64::from(MyStruct(12)));

    println!("{:?}", MyNamedStruct::from(-23));
    println!("{:?}", i64::from(MyNamedStruct { f: -23 }));
}
