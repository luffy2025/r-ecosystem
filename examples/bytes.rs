use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"hello world!");
    println!("buf: {:?}", buf);

    buf.put(&b"pillow talk"[..]);
    buf.put(&b"|"[..]);
    buf.put_i64(0xdeadbeaf);
    buf.put(&b"|"[..]);
    buf.put_i64_le(0xdeadbeaf);
    buf.put(&b"|"[..]);
    println!("buf: {:?}", buf);

    let mut a = buf.split(); // buf is empty now
    a.extend_from_slice(b"a str");
    println!("a: {:?}", a);
    println!("buf: {:?}", buf);

    let mut b = a.freeze(); // a is moved to b
    println!("b: {:?}", b);

    let c = b.split_to(5); // pointer in b is forward by 5 bytes
    println!("c: {:?}", c);

    let d = b.split_at(6); // b is split into two parts at index 6

    println!("d0: {:?}", String::from_utf8_lossy(d.0));
    println!("d1: {:?}", String::from_utf8_lossy(d.1));

    Ok(())
}
