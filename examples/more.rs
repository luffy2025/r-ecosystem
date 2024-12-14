use anyhow::Result;
use derive_more::{Add, Display, From, Into};

#[derive(PartialEq, Clone, Copy, From, Add, Into, Display)]
struct MyInt(i32);

#[derive(PartialEq, From, Into)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, From, Add, Display)]
enum MyEnum {
    #[display("Int32({})", _0)]
    Int(i32),
    UInt(u32),
    #[display("There is nothing")]
    Nothing,
}

fn main() -> Result<()> {
    let my_int: MyInt = 10.into();
    let v = my_int + 20.into();
    let v_i32: i32 = v.into();
    println!("my_int: {} v: {} v_i32: {}", my_int, v, v_i32);

    let e1: MyEnum = 10i32.into();
    let e2: MyEnum = 20u32.into();
    println!("e1: {} e2: {}", e1, e2);

    let e3: MyEnum = 30i32.into();
    let e4 = e1 + e3;
    println!("e4: {}", e4?);

    println!("nothing: {}", MyEnum::Nothing);

    Ok(())
}
