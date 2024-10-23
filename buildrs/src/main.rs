#[no_mangle]
#[inline(never)]
fn foo(s:i32) -> i32 {
    let foo=[1,2,3];
    let bar = &foo[..];
    let far = &bar[3..];
    println!("len {}", far.len());
    s
}

fn main() {
    foo(4);
}
