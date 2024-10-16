use riscv::register::time;

fn get_time() -> usize {
    time::read()
}
const CLOCK_FREQ: usize = 12500000; //qemu freq
const TICK_PER_SECOND: usize = 100;
const MILLI_PER_SEC: usize = 1000;

pub fn set_next_trigger() {
    crate::sbi::set_timer(get_time() + CLOCK_FREQ / TICK_PER_SECOND);
}

pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / MILLI_PER_SEC)
}
