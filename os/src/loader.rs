use core::ffi;
use lazy_static::lazy_static;

use crate::{println, sync::UCell};

pub const MAX_APP_NUM: usize = 16;

#[derive(Debug, Clone, Copy)]
struct AppInfoBuf {
    name: usize,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: &'static str,
    pub mem: &'static [u8],
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
            "[kernel] app_{} named: {} from {:?}",
            i,
            app.name,
            app.mem.as_ptr_range(),
        )
    }

    unsafe fn get_app_info(&self, app_id: usize) -> AppInfo {
        let ref a = self.app_infos[app_id];
        let app_src = core::slice::from_raw_parts(a.start as *const u8, a.end - a.start);
        let app_name = ffi::CStr::from_ptr(a.name as *const i8).to_str().unwrap();
        AppInfo {
            name: app_name,
            mem: app_src,
        }
    }
    pub fn get_app_info_by_name(&self, name: &str) -> Option<AppInfo> {
        for i in 0..self.num_app {
            unsafe {
                let info = self.get_app_info(i);
                if info.name == name {
                    return Some(info);
                }
            }
        }
        None
    }
}

lazy_static! {
    static ref APP_MANAGER: UCell<AppManager> = unsafe {
        extern "C" {
            pub fn _num_app();
        }
        let num_app_ptr = _num_app as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_infos = [AppInfoBuf {
            name: 0,
            start: 0,
            end: 0,
        }; MAX_APP_NUM];
        let app_info_raw =
            core::slice::from_raw_parts(num_app_ptr.add(1) as *const AppInfoBuf, num_app);
        app_infos[..num_app].copy_from_slice(app_info_raw);
        UCell::new(AppManager { num_app, app_infos })
    };
}

pub fn init() {
    print_apps_info();
}

pub fn print_apps_info() {
    APP_MANAGER.exclusive_access().print_apps_info();
}

pub fn get_app_info_by_name(name: &str) -> Option<AppInfo> {
    let m = APP_MANAGER.exclusive_access();
    m.get_app_info_by_name(name)
}
