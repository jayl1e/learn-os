use sbi_rt;
pub fn console_put_char(c: usize){
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn shut_down(failure: bool)->!{
    use sbi_rt::{system_reset,Shutdown,NoReason,SystemFailure};
    if !failure{
        system_reset(Shutdown, NoReason);
    }else{
        sbi_rt::system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}