use giftwrap::*;

#[derive(Debug, Wrap, Unwrap)]
pub struct MyStruct<'a, T>(Option<&'a T>);

#[derive(Debug, Wrap, Unwrap)]
pub struct MyNamedStruct {
    f: i64,
}

fn main() {
    println!("{:?}", MyStruct::<i64>::from(Some(&12)));
    println!(
        "{:?}",
        Option::<&'_ i64>::from(MyStruct::<'_, i64>(Some(&12)))
    );

    println!("{:?}", MyNamedStruct::from(-23));
    println!("{:?}", i64::from(MyNamedStruct { f: -23 }));
}
