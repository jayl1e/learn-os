use core::{arch::asm, ffi, mem};

use lazy_static::lazy_static;

use crate::{println, sync::up::UPSafeCell, trap::context::TrapContext};

pub const MAX_APP_NUM: usize = 16;
//const APP_BASE_ADDRESS:usize = 0x80400000;
//const APP_SIZE_LIMIT:usize = 0x20000;
const KERNEL_STACK_LIMIT: usize = 8192;
const USER_STACK_LIMIT: usize = 8192;

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_LIMIT],
}
impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_LIMIT
    }
    fn push_trap_context(&self, cx: TrapContext) -> usize {
        let buf_addr = self.get_sp() - mem::size_of::<TrapContext>();
        let buf_ptr = buf_addr as *mut TrapContext;
        unsafe {
            *buf_ptr = cx;
        }
        buf_addr
    }
}

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_LIMIT],
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_LIMIT
    }
}

static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_LIMIT],
}; MAX_APP_NUM];
static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_LIMIT],
}; MAX_APP_NUM];

#[derive(Debug, Clone, Copy)]
struct AppInfoBuf {
    name: usize,
    start: usize,
    end: usize,
    mem_start: usize,
    mem_end: usize,
}

pub struct AppInfo {
    pub name: &'static str,
    mem: &'static [u8],
    dst: &'static mut [u8],
}

#[repr(C)]
struct AppManager {
    num_app: usize,
    app_infos: [AppInfoBuf; MAX_APP_NUM],
}

impl AppManager {
    pub fn print_apps_info(&self) {
        println!("sizeof AppInfoBuf is {}", size_of::<AppInfoBuf>());
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            self.print_app_info(i);
        }
        println!("[kernel] apps are above");
    }

    fn print_app_info(&self, i: usize) {
        let app = unsafe { self.get_app_info(i) };
        println!(
            "[kernel] app_{} named: {} from {:?} to {:?}",
            i,
            app.name,
            app.mem.as_ptr_range(),
            app.dst.as_ptr_range()
        )
    }

    unsafe fn get_app_info(&self, app_id: usize) -> AppInfo {
        let ref a = self.app_infos[app_id];
        let app_src = core::slice::from_raw_parts(a.start as *const u8, a.end - a.start);
        let app_dst =
            core::slice::from_raw_parts_mut(a.mem_start as *mut u8, a.mem_end - a.mem_start);

        let app_name = ffi::CStr::from_ptr(a.name as *const i8).to_str().unwrap();
        AppInfo {
            name: app_name,
            mem: app_src,
            dst: app_dst,
        }
    }

    pub unsafe fn load_app(&self, app_id: usize, app_info: AppInfo) {
        println!("[kernel] loading app_{}", app_id);
        let app_dst = app_info.dst;
        app_dst.fill(0);
        let app_src = app_info.mem;
        app_dst[..app_src.len()].copy_from_slice(app_src);
        asm!("fence.i");
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        extern "C" {
            pub fn _num_app();
        }
        let num_app_ptr = _num_app as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_infos = [AppInfoBuf {
            name: 0,
            start: 0,
            end: 0,
            mem_start: 0,
            mem_end: 0,
        }; MAX_APP_NUM];
        let app_info_raw =
            core::slice::from_raw_parts(num_app_ptr.add(1) as *const AppInfoBuf, num_app);
        app_infos[..num_app].copy_from_slice(app_info_raw);
        UPSafeCell::new(AppManager { num_app, app_infos })
    };
}

pub fn init() {
    print_apps_info();
}

pub fn print_apps_info() {
    APP_MANAGER.exclusive_access().print_apps_info();
}

pub unsafe fn load_all_apps() {
    let m = APP_MANAGER.exclusive_access();
    for i in 0..m.num_app {
        let app_info = m.get_app_info(i);
        m.load_app(i, app_info);
    }
}

pub fn get_app_info(app_id: usize) -> AppInfo {
    let m = APP_MANAGER.exclusive_access();
    unsafe { m.get_app_info(app_id) }
}

pub fn get_num_app() -> usize {
    let m = APP_MANAGER.exclusive_access();
    m.num_app
}

pub fn init_app_cx(app_id: usize) -> usize {
    let entry = get_app_info(app_id).dst.as_ptr() as usize;
    let sp = USER_STACK[app_id].get_sp();
    let ctx = TrapContext::init_new_app(sp, entry);
    let ksp = KERNEL_STACK[app_id].push_trap_context(ctx);
    ksp
}
