use riscv::register::sstatus::{self, Sstatus, SPP};
const  BASE_ADDRESS:usize = 0x80400000;
pub struct TrapContext{
    pub registers: [usize;32],
    #[allow(unused)]
    pub sstatus: Sstatus,
    pub sepc: usize
}

impl TrapContext {
    pub fn init_new_app(sp:usize)->Self{
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let sepc =BASE_ADDRESS;
        let mut cx = TrapContext { registers: [0;32], sstatus: sstatus, sepc};
        cx.set_sp(sp);
        cx
    }
    fn set_sp(&mut self, sp:usize){
        self.registers[2] = sp;
    }
}