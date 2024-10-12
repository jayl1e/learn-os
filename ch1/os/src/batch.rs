use core::{arch::asm, mem};

use lazy_static::lazy_static;

use crate::{println, sbi::shut_down, sync::up::UPSafeCell, trap::context::TrapContext};

const MAX_APP_NUM:usize=16;
const APP_BASE_ADDRESS:usize = 0x80400000;
const APP_SIZE_LIMIT:usize = 0x20000;
const KERNEL_STACK_LIMIT:usize = 8192;
const USER_STACK_LIMIT:usize = 8192;
#[repr(align(4096))]
struct KernelStack{
    data: [u8;KERNEL_STACK_LIMIT]
}
impl KernelStack {
    fn get_sp(&self)->usize{
        self.data.as_ptr() as usize + KERNEL_STACK_LIMIT
    }
    fn push_trap_context(&self, cx: TrapContext)->usize{
        let buf_addr = self.get_sp()-mem::size_of::<TrapContext>();
        let buf_ptr = buf_addr as *mut TrapContext;
        unsafe {
            *buf_ptr = cx;
        }
        buf_addr
    }
}

#[repr(align(4096))]
struct UserStack{
    data: [u8;USER_STACK_LIMIT]
}

impl UserStack {
    fn get_sp(&self)->usize{
        self.data.as_ptr() as usize + KERNEL_STACK_LIMIT
    }
}

static KERNEL_STACK:KernelStack = KernelStack{data:[0;KERNEL_STACK_LIMIT]};
static USER_STACK:UserStack = UserStack{data:[0;USER_STACK_LIMIT]};



struct AppManager{
    num_app: usize,
    next_app: usize,
    app_start: [usize;MAX_APP_NUM+1],
}

impl AppManager {
    pub fn get_current_app(&self)->usize{
        self.next_app
    }
    pub fn print_apps_info(&self){
        println!("[kernel] num_app = {}", self.num_app);
        for i in 1..=self.num_app{
            println!(
                "[kernel] app_{} [{:#x}, {:#x}]",
                i,
                self.app_start[i],
                self.app_start[i+1]
            );
        }
    }
    pub fn move_to_next_app(&mut self)->bool{
        if self.next_app>=self.num_app{
            false
        }else{
            self.next_app+=1;
            true
        }
    }
    pub unsafe fn load_app(&self, app_id:usize){
        if app_id>=self.num_app{
            panic!("bad app id to load");
        }
        println!("[kernel] loading app_{}", app_id+1);
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT);
        app_dst.fill(0);
        let app_src = core::slice::from_raw_parts(self.app_start[app_id] as *const u8, self.app_start[app_id+1]- self.app_start[app_id]);
        app_dst[..app_src.len()].copy_from_slice(app_src);
        asm!("fence.i");
    }
}

lazy_static!{
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        extern "C" {
            pub fn _num_app();
        }
        let num_app_ptr = _num_app as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_start = [0;MAX_APP_NUM+1];
        let app_start_raw = core::slice::from_raw_parts(num_app_ptr.add(1), num_app+1);
        app_start[..=num_app].copy_from_slice(app_start_raw);
        UPSafeCell::new(AppManager{
            num_app,
            next_app:0,
            app_start
        })
    };
}

pub fn init(){
    print_apps_info();
}

pub fn print_apps_info(){
    APP_MANAGER.exclusive_access().print_apps_info();
}
    
pub fn run_next_app()->!{
    let mut m = APP_MANAGER.exclusive_access();
    let current = m.get_current_app();
    if !m.move_to_next_app(){
        println!("[kernel] finished all app, shutting down");
        shut_down(false)
    }
    extern "C"{fn __restore(ctx_ptr: usize)->!;}
    unsafe {
        m.load_app(current);
    }
    //function never return, the destructor wont work, drop it manually
    drop(m);
    
    unsafe {
        __restore(
            KERNEL_STACK.push_trap_context(
                TrapContext::init_new_app(USER_STACK.get_sp())
            )
        )
    }
}

    

