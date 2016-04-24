
use ::{RcCell, WeakCell};
use platform::{Platform, PlatformWindow, EventLoop};
use window::{State, WindowBase};
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


struct Win32SharedPlatform {
    windows: HashMap<HWND, WeakCell<Win32Window>>,
    registered_cls: HashSet<Vec<u16>>,
    exit_code: Option<i32>
}

pub struct Win32Platform {
    shared_platform: RcCell<Win32SharedPlatform>,
}

impl Win32Platform {
    pub fn new() -> Win32Platform {
        Win32Platform {
            shared_platform: Rc::new(RefCell::new(Win32SharedPlatform {
                windows: HashMap::new(),
                registered_cls: HashSet::new(),
                exit_code: None,
            })),
        }
    }
}

impl Platform for Win32Platform {
    fn create_window(&self, base: WindowBase) -> RcCell<PlatformWindow> {
        Win32Window::new(base, Rc::downgrade(&self.shared_platform))
    }
}

impl EventLoop for Win32Platform {
    fn loop_events(&self) -> i32 {
        loop { unsafe {
            let mut msg = mem::uninitialized::<MSG>();
            match GetMessageW(&mut msg, ptr::null_mut(), 0, 0) {
                code @ -1 | code @ 0 => {
                    let sp = self.shared_platform.borrow_mut();
                    sp.exit_code = Some(code);
                },
                _ => {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                },
            }
            let sp = self.shared_platform.borrow();
            if sp.exit_code.is_some() { break; }
        }}
        let sp = self.shared_platform.borrow();
        sp.exit_code.unwrap()
    }
    fn exit(&self, code: i32) {
        self.shared_platform.borrow_mut().exit_code = Some(code);
    }
}

unsafe extern "system"
fn win32_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM)
        -> LRESULT {

    let shared_platform: Option<&mut WeakCell<Win32SharedPlatform>>
            = mem::transmute(get_window_long_ptr(hwnd, 0));

    if let Some(sp) = shared_platform {
        if let Some(sp) = Weak::upgrade(&sp) {
            if sp.borrow_mut().wnd_proc(hwnd, msg, wparam, lparam) {
                return 0;
            }
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl Win32SharedPlatform {
    fn wnd_proc(&mut self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {

        let handle_msg = |w, f| {
            if let Some(w) = w { f(w) }
            else {
                warn!("Ows-Win32: message from unregistered window");
            }
        };

        if let Some(w) = self.window(hwnd) {
            match msg {
                WM_SIZE => {
                    w.borrow_mut().handle_wm_size(wparam, lparam);
                },
                WM_CLOSE => {
                    if fire_or!(w.borrow().on_close.clone(), true) {
                        w.borrow_mut().close();
                        if self.windows.is_empty() && self.exit_code.is_none() {
                            self.exit_code = Some(0);
                        }
                    }
                    true
                },
                _ => { false }
            }
        }
    }

    fn window(&self, hwnd: HWND) -> Option<RcCell<Win32Window>> {
        self.windows.get(&hwnd).and_then(|wwc| Weak::upgrade(&wwc))
    }

    // TODO: WindowSettings (popup, frameless, modal...)
    fn register_wnd_cls(&self) -> Vec<u16> {
        let cn_u8 = "OwsWindowClassName";
        let cls_name = to_u16(&cn_u8);
        if self.registered_cls.borrow().contains(&cls_name) {
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
            self.registered_cls.borrow_mut().insert(cls_name.clone());
            cls_name
        }}
    }
}


pub struct Win32Window {
    base: WindowBase,
    weak_me: WeakCell<Win32Window>,
    shared_platform: WeakCell<Win32SharedPlatform>,
    hwnd: HWND,
    title: String,
    rect: IRect,
}

impl Win32Window {
    fn new(base: WindowBase, shared_platform: WeakCell<Win32SharedPlatform>) -> RcCell<Win32Window> {
        let w = Rc::new(RefCell::new(Win32Window {
            base: base,
            weak_me: Weak::new(),
            shared_platform: shared_platform,
            hwnd: ptr::null_mut(),
            title: String::new(),
            rect: IRect::new(0, 0, 0, 0),
        }));
        w.borrow_mut().weak_me = Rc::downgrade(&w);
        w
    }
    fn create(&mut self) {
        let sp = Weak::upgrade(self.shared_platform)
            .expect("Win32: PlatformWindow outlived Platform");
        let cls_name = {
            let sp = sp.borrow_mut();
            sp.register_wnd_cls()
        };
        let title = to_u16(&self.title);
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
            // shared_platform must be found by WindowProc through HWND
            let spbox = Box::new(self.shared_platform.clone());
            let spbox = Box::into_raw(spbox);
            set_window_long_ptr(hwnd, 0, mem::transmute(spbox));
            hwnd
        };
        let mut sp = sp.borrow_mut();
        sp.windows.insert(&hwnd, self.weak_me.clone());
    }
    fn handle_wm_size(&mut self, wparam: WPARAM, lparam: LPARAM) -> bool {
        match wparam as u32 {
            SIZE_MAXSHOW | SIZE_MAXHIDE => { false },
            SIZE_MINIMIZED => {
                // state change
                true
            },
            SIZE_MAXIMIZED => {
                // state change
                self.handle_rect_change();
                true
            },
            SIZE_RESTORED => {
                self.handle_rect_change();
                true
            },
            _ => { false },
        }
    }

    fn handle_rect_change(&mut self) {
        let old_r = self.rect;
        let r = self.rect_sys();

        self.rect = r;

        if old_r.size() != r.size() {
            fire!(self.base.on_resize.clone(), r.size());
        }
        if old_r.point() != r.point() {
            // move
        }
    }

    fn rect_sys(&self) -> IRect {
        unsafe {
			let mut wr = mem::uninitialized::<RECT>();
			GetWindowRect(self.hwnd, &mut wr);

			let mut ar = RECT { left: 0, top: 0, right: 0, bottom: 0};
			AdjustWindowRectEx(&mut ar, self.style(), 0, self.ex_style());

			wr.left -= ar.left;
			wr.top -= ar.top;
			wr.right -= ar.right;
			wr.bottom -= ar.bottom;

            IRect::from(wr)
        }
    }

    fn style(&self) -> DWORD {
        unsafe { GetWindowLongW(self.hwnd, GWL_STYLE) as DWORD }
    }
    fn ex_style(&self) -> DWORD {
        unsafe { GetWindowLongW(self.hwnd, GWL_EXSTYLE) as DWORD }
    }
}

impl PlatformWindow for Win32Window {
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
        let title = to_u16(&self.title);
        unsafe { SetWindowTextW(self.hwnd, title.as_ptr()); }
    }

    fn state(&self) -> State {
        State::Hidden
    }
    fn set_state(&mut self, state: State) {
        unsafe { ShowWindow(self.hwnd, SW_SHOWNORMAL); }
    }

    fn close(&mut self) {
        if self.created() {
            if let Some(sp) = Weak::upgrade(&self.shared_platform) {
                let sp = sp.borrow_mut();
                sp.windows.remove(&self.hwnd);
            }
            unsafe {
                let spbox = get_window_long_ptr(self.hwnd, 0);
                let spbox: *mut WeakCell<Win32SharedPlatform> = spbox;
                assert!(!spbox.is_null());
                let spbox = Box::from_raw(spbox);
                set_window_long_ptr(self.hwnd, 0, 0);
                DestroyWindow(self.hwnd);
            }
            self.hwnd = ptr::null_mut();
        }
    }
}

impl Drop for Win32Window {
    fn drop (&mut self) {
        self.close();
    }
}
