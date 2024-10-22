use riscv::register::sstatus::{self, Sstatus, SPP};

//const  BASE_ADDRESS:usize = 0x80400000;
#[allow(unused)]
pub struct TrapContext {
    pub registers: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn init_new_app(
        sp: usize,
        entry: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let sepc = entry;
        let mut cx = TrapContext {
            registers: [0; 32],
            sstatus: sstatus,
            sepc,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
    fn set_sp(&mut self, sp: usize) {
        self.registers[2] = sp;
    }
}
