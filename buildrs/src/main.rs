#[derive(Debug, Clone, Copy)]
struct A(i32);

impl A {
    fn m(&self) -> i32 {
        println!("m");
        self.0 * 6
    }
}

#[no_mangle]
#[inline(never)]
fn foo(s:i32) -> i32 {
    let a = A(s);
    let c= a.m();
    println!("c is {}",c);
    let b = a;
    b.m();
    a.0
}

fn main() {
    foo(4);
}
