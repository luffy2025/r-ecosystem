use anyhow::Result;
use strum::{
    EnumCount, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr, VariantNames,
};

#[allow(unused)]
#[derive(Debug, EnumString, EnumCount, EnumIs, EnumIter, VariantNames, IntoStaticStr)]
enum MyEnum {
    #[strum(serialize = "Variant A", to_string = "Enum A")]
    A,
    B(String),
    C,
}

fn main() -> Result<()> {
    println!("{:?}", MyEnum::VARIANTS);
    MyEnum::iter().for_each(|v| println!("{:?}", v));

    println!("{:?}", MyEnum::COUNT);

    let b = MyEnum::B("Hello".to_string());
    println!("{:?} {:?}", b, b.is_b());

    let s: &'static str = b.into();
    println!("{}", s);

    let e: MyEnum = "Variant A".parse()?;
    println!("{:?}", e);

    Ok(())
}
