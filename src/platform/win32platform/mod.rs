
use ::{RcCell, WeakCell};
use platform::{Platform, EventLoop, WinId};
use window::{Window, State, WindowBase};
use geometry::{Area, IRect};

use winapi::*;
use kernel32::*;
use user32::*;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::mem;
use std::ptr;


// 32/64 bits compatibility
#[cfg(target_arch = "x86_64")]
unsafe fn get_window_long_ptr(hwnd: HWND, index: c_int) -> LONG_PTR {
    GetWindowLongPtrW(hwnd, index)
}
#[cfg(not(target_arch = "x86_64"))]
unsafe fn get_window_long_ptr(hwnd: HWND, index: c_int) -> LONG_PTR {
    GetWindowLongW(hwnd, index)
}
#[cfg(target_arch = "x86_64")]
unsafe fn set_window_long_ptr(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    SetWindowLongPtrW(hwnd, index, value)
}
#[cfg(not(target_arch = "x86_64"))]
unsafe fn set_window_long_ptr(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(hwnd, index, value)
}



/// convert a String or &str into utf16 string usable in Windows Unicode API
fn to_u16<S : AsRef<str>>(s: S) -> Vec<u16> {
    OsStr::new(s.as_ref()).encode_wide().chain(Some(0).into_iter()).collect()
}

// a few utils
impl From<RECT> for IRect {
    fn from(r: RECT) -> IRect {
        IRect::new(r.left, r.top, r.right-r.left, r.bottom-r.top)
    }
}
impl From<IRect> for RECT {
    fn from(r: IRect) -> RECT {
        RECT {
            left: r.x, top: r.y, right: r.x + r.w, bottom: r.y + r.h,
        }
    }
}

impl Area for RECT {
    type Output = i32;
    fn area(&self) -> i32 {
        (self.right-self.left) * (self.bottom-self.top)
    }
}


fn hwnd_to_wid(hwnd: HWND) -> WinId {
    unsafe { mem::transmute(hwnd) }
}

fn wid_to_hwnd(wid: WinId) -> HWND {
    unsafe { mem::transmute(wid) }
}


struct Win32SharedPlatform {
    windows: HashMap<HWND, Win32Window>,
    exit_code: Option<i32>,
}

pub struct Win32Platform {
    shared_platform: Box<Win32SharedPlatform>,
    registered_cls: HashSet<Vec<u16>>,
}

impl Win32Platform {
    pub fn new() -> Win32Platform {
        Win32Platform {
            shared_platform: Box::new(Win32SharedPlatform {
                windows: HashMap::new(),
                exit_code: None,
            }),
            registered_cls: HashSet::new(),
        }
    }

    // TODO: WindowSettings (popup, frameless, modal...)
    fn register_wnd_cls(&mut self) -> Vec<u16> {
        let cn_u8 = "OwsWindowClassName";
        let cls_name = to_u16(&cn_u8);
        if self.registered_cls.contains(&cls_name) {
            cls_name.clone()
        }
        else { unsafe {
            let hinstance = GetModuleHandleW(ptr::null());
            let wc = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
                style: 0,
                lpfnWndProc: Some(win32_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: mem::size_of::<usize>() as c_int, // size of pointer
                hInstance: hinstance,
                hIcon: LoadIconW(ptr::null_mut() as HINSTANCE, IDI_APPLICATION),
                hCursor: LoadCursorW(ptr::null_mut() as HINSTANCE, IDC_ARROW),
                hbrBackground: ptr::null_mut() as HBRUSH,
                lpszMenuName: ptr::null() as *const u16,
                lpszClassName: cls_name.as_ptr(),
                hIconSm: LoadIconW(ptr::null_mut() as HINSTANCE, IDI_APPLICATION),
            };
            if RegisterClassExW(&wc) == 0 {
                panic!("could not register class {}", &cn_u8);
            }
            self.registered_cls.insert(cls_name.clone());
            cls_name
        }}
    }
}

impl Platform for Win32Platform {
    fn create_window(&mut self) -> WinId {
        let cls_name = self.register_wnd_cls();
        let title = to_u16("");
        let hwnd = unsafe {
            let hinstance = GetModuleHandleW(ptr::null());
            let hwnd = CreateWindowExW(
                    WS_EX_CLIENTEDGE,
                    cls_name.as_ptr(),
                    title.as_ptr(),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    ptr::null_mut(), ptr::null_mut(),
                    hinstance, ptr::null_mut());

            assert!(!hwnd.is_null());
            use std::ops::Deref;
            set_window_long_ptr(hwnd, 0, mem::transmute(self.shared_platform.deref()));
            hwnd
        };
        let w = Win32Window::new(hwnd);
        self.shared_platform.windows.insert(hwnd, w);
        hwnd_to_wid(hwnd)
    }
    fn window(&self, id: WinId) -> &Window {
        self.shared_platform.windows.get(&wid_to_hwnd(id)).unwrap()
    }
    fn window_mut(&mut self, id: WinId) -> &mut Window {
        self.shared_platform.windows.get_mut(&wid_to_hwnd(id)).unwrap()
    }
}

impl EventLoop for Win32Platform {
    fn loop_events(&mut self) -> i32 {
        loop { unsafe {
            let mut msg = mem::uninitialized::<MSG>();
            match GetMessageW(&mut msg, ptr::null_mut(), 0, 0) {
                code @ -1 | code @ 0 => {
                    self.shared_platform.exit_code = Some(code);
                },
                _ => {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                },
            }
            if self.shared_platform.exit_code.is_some() { break; }
        }}
        self.shared_platform.exit_code.unwrap()
    }
    fn exit(&mut self, code: i32) {
        self.shared_platform.exit_code = Some(code);
    }
}

unsafe extern "system"
fn win32_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM)
        -> LRESULT {

    let shared_platform: Option<&mut Win32SharedPlatform> = mem::transmute(get_window_long_ptr(hwnd, 0));

    if let Some(sp) = shared_platform {
        if sp.wnd_proc(hwnd, msg, wparam, lparam) {
            return 0;
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl Win32SharedPlatform {
    fn wnd_proc(&mut self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {
        match msg {
            WM_CLOSE => {
                let close_window = {
                    let w = self.windows.get_mut(&hwnd).unwrap();
                    fire_or!(w.on_close(), true, w)
                };
                if close_window {
                    // on_close was not set or returned true
                    // dropping the window will close it
                    self.windows.remove(&hwnd);
                    if self.windows.is_empty() && self.exit_code.is_none() {
                        self.exit_code = Some(0);
                    }
                }
            },
            _ => {}
        }
        false
    }
}


pub struct Win32Window {
    base: WindowBase,
    hwnd: HWND,
    title: String,

}

impl Win32Window {
    fn new(hwnd: HWND) -> Win32Window {
        Win32Window {
            base: WindowBase::new(),
            hwnd: hwnd,
            title: "".to_string(),
        }
    }
}

impl Window for Win32Window {
    fn base(&self) -> &WindowBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut WindowBase {
        &mut self.base
    }

    fn title(&self) -> String {
        self.title.clone()
    }
    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn state(&self) -> State {
        State::Hidden
    }
    fn set_state(&mut self, state: State) {
        unsafe { ShowWindow(self.hwnd, SW_SHOWNORMAL); }
    }
}
