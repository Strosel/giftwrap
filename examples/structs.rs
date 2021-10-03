use giftwrap::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Wrap, Unwrap)]
pub struct MyStruct<'a, T>(Option<&'a T>);

#[derive(Debug, Wrap, Unwrap)]
pub struct MyNamedStruct {
    f: i64,
}

#[derive(Debug, Wrap, Unwrap)]
pub struct Depth(#[wrapDepth(0)] Arc<Mutex<i32>>);

fn main() {
    println!("{:?}", MyStruct::<i64>::from(Some(&12)));
    println!(
        "{:?}",
        Option::<&'_ i64>::from(MyStruct::<'_, i64>(Some(&12)))
    );

    println!("{:?}", MyNamedStruct::from(-23));
    println!("{:?}", i64::from(MyNamedStruct { f: -23 }));

    println!("{:?}", Depth::from(1i32));
    println!("{:?}", Depth::from(Mutex::new(2i32)));
    println!("{:?}", Depth::from(Arc::new(Mutex::new(3i32))));
}
