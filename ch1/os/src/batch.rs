use core::{arch::asm, mem, ffi};

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

#[derive(Debug, Clone, Copy)]
struct AppInfoBuf{
    name: usize,
    start: usize,
    end: usize,
}

pub struct AppInfo{
    pub name: &'static str,
    mem: &'static [u8]
}

#[repr(C)]
struct AppManager{
    num_app: usize,
    current_app: usize,
    app_infos: [AppInfoBuf;MAX_APP_NUM],
}

impl AppManager {
    pub fn get_current_app(&self)->usize{
        self.current_app
    }
    pub fn print_apps_info(&self){
        println!("sizeof AppInfoBuf is {}", size_of::<AppInfoBuf>());
        println!("[kernel] num_app = {}", self.num_app);
        for i in 1..=self.num_app{
            self.print_app_info(i);
        }
        println!("[kernel] apps are above");
    }
    fn print_app_info(&self, i:usize){
        let app = unsafe {self.get_app_info(i)};
        println!(
            "[kernel] app_{} [{:p}, {:p}], named: {}",
            i,
            app.mem.as_ptr_range().start,
            app.mem.as_ptr_range().end,
            app.name
        )
    }
    pub fn move_to_next_app(&mut self)->bool{
        if self.current_app>=self.num_app{
            false
        }else{
            self.current_app+=1;
            true
        }
    }

    unsafe fn get_app_info(&self, app_id:usize)->AppInfo{
        let ref a = self.app_infos[app_id-1];
        let app_src =
            core::slice::from_raw_parts(a.start as *const u8, a.end- a.start);
        let app_name = ffi::CStr::from_ptr(a.name as *const i8).to_str().unwrap();
        AppInfo { name: app_name, mem: app_src}
    }

    pub unsafe fn load_app(&self, app_id:usize){
        if app_id>self.num_app{
            panic!("bad app id to load");
        }
        println!("[kernel] loading app_{}", app_id);
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT);
        app_dst.fill(0);
        let app_src = self.get_app_info(app_id).mem;
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
        let mut app_infos = [AppInfoBuf{name:0, start:0, end:0};MAX_APP_NUM];
        let app_info_raw = core::slice::from_raw_parts(num_app_ptr.add(1) as *const AppInfoBuf, num_app);
        app_infos[..num_app].copy_from_slice(app_info_raw);
        UPSafeCell::new(AppManager{
            num_app,
            current_app:0,
            app_infos
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
    if !m.move_to_next_app(){
        println!("[kernel] finished all app, shutting down");
        shut_down(false)
    }
    extern "C"{fn __restore(ctx_ptr: usize)->!;}
    unsafe {
        m.load_app(m.get_current_app());
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

pub fn get_current_app()->AppInfo{
    let m = APP_MANAGER.exclusive_access();
    unsafe {
        m.get_app_info(m.get_current_app())
    }
}
    

