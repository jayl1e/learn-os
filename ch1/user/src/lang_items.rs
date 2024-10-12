use core::panic::PanicInfo;

use crate::println;
use crate::exit;

#[panic_handler]
fn panic(info: &PanicInfo)->!{
    if let Some(location)= info.location(){
        println!("Panic at {}:{} {}", location.file(),location.line(), info.message())
    }else{
        println!("Panic: {}", info.message())
    }
    exit(1);
    loop {
        
    }
}