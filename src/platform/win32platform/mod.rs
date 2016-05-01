
use ::{RcCell, WeakCell};
use platform::{Platform, PlatformWindow, EventLoop};
use window::{self, Window, WindowBase};
use geometry::{Area, IRect};

use winapi::*;
use kernel32::*;
use user32::*;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};
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
    windows: RefCell<HashMap<HWND, Weak<Win32Window>>>,
    registered_cls: RefCell<HashSet<Vec<u16>>>,
    exit_code: Cell<Option<i32>>,
}

pub struct Win32Platform {
    shared_platform: Rc<Win32SharedPlatform>,
}

impl Win32Platform {
    pub fn new() -> Win32Platform {
        Win32Platform {
            shared_platform: Rc::new(Win32SharedPlatform {
                windows: RefCell::new(HashMap::new()),
                registered_cls: RefCell::new(HashSet::new()),
                exit_code: Cell::new(None),
            }),
        }
    }
}

impl Platform for Win32Platform {
    fn create_window(&self, base: RcCell<WindowBase>) -> Rc<PlatformWindow> {
        Win32Window::new(base, self.shared_platform.clone())
    }
}

impl EventLoop for Win32Platform {
    fn loop_events(&self) -> i32 {
        loop { unsafe {
            let mut msg = mem::uninitialized::<MSG>();
            match GetMessageW(&mut msg, ptr::null_mut(), 0, 0) {
                code @ -1 | code @ 0 => {
                    self.shared_platform.exit_code.set(Some(code));
                },
                _ => {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                },
            }
            if self.shared_platform.exit_code.get().is_some() { break; }
        }}
        self.shared_platform.exit_code.get().unwrap()
    }
    fn exit(&self, code: i32) {
        self.shared_platform.exit_code.set(Some(code));
    }
}

unsafe extern "system"
fn win32_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM)
        -> LRESULT {

    let shared_platform: Option<&mut Weak<Win32SharedPlatform>>
            = mem::transmute(get_window_long_ptr(hwnd, 0));

    if let Some(sp) = shared_platform {
        if let Some(sp) = Weak::upgrade(&sp) {
            if sp.wnd_proc(hwnd, msg, wparam, lparam) {
                return 0;
            }
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl Win32SharedPlatform {
    fn wnd_proc(&self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {

        if let Some(w) = self.window(hwnd) {
            match msg {
                WM_SIZE => {
                    w.handle_wm_size(wparam, lparam)
                },
                WM_MOVE => {
                    w.handle_wm_move(wparam, lparam)
                },
                WM_CLOSE => {
                    if handler_fire_or!(w.base.borrow().on_close.clone(),
                            true, make_window(w.clone())) {
                        w.close();
                        if self.windows.borrow().is_empty()
                                && self.exit_code.get().is_none() {
                            self.exit_code.set(Some(0));
                        }
                    }
                    true
                },
                _ => { false }
            }
        }
        else {
            warn!("Ows-Win32: message from unregistered window");
            false
        }
    }

    fn window(&self, hwnd: HWND) -> Option<Rc<Win32Window>> {
        self.windows.borrow().get(&hwnd).and_then(|ww| Weak::upgrade(&ww))
    }

    // TODO: WindowSettings (popup, frameless, modal...)
    fn register_wnd_cls(&self) -> Vec<u16> {
        let cn_u8 = "OwsWindowClassName";
        let cls_name = to_u16(&cn_u8);
        {
            if self.registered_cls.borrow().contains(&cls_name) {
                return cls_name.clone();
            }
        }
        unsafe {
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
        }
    }
}


fn make_window(pw: Rc<Win32Window>) -> Window {
    let base = pw.base.clone();
    Window::make(base, pw)
}



pub struct Win32Window {
    base: RcCell<WindowBase>,
    weak_me: RefCell<Weak<Win32Window>>,
    shared_platform: Rc<Win32SharedPlatform>,
    hwnd: Cell<HWND>,
    rect: Cell<IRect>,
}

impl Win32Window {
    fn new(base: RcCell<WindowBase>, shared_platform: Rc<Win32SharedPlatform>) -> Rc<Win32Window> {
        let w = Rc::new(Win32Window {
            base: base,
            weak_me: RefCell::new(Weak::new()),
            shared_platform: shared_platform,
            hwnd: Cell::new(ptr::null_mut()),
            rect: Cell::new(IRect::new(0, 0, 0, 0)),
        });
        (*w.weak_me.borrow_mut()) = Rc::downgrade(&w);
        w
    }

    fn rc_me(&self) -> Rc<Win32Window> {
        // self is not dropped, so unwrapping should be safe
        Weak::upgrade(&self.weak_me.borrow()).unwrap()
    }

    fn created(&self) -> bool {
        !self.hwnd.get().is_null()
    }

    fn handle_wm_size(&self, wparam: WPARAM, _: LPARAM) -> bool {
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

    fn handle_wm_move(&self, _: WPARAM, _: LPARAM) -> bool {
        if unsafe { IsIconic(self.hwnd.get()) } == 0 {
            self.handle_rect_change();
            true
        }
        else {
            false
        }
    }

    fn handle_rect_change(&self) {
        let old_r = self.rect.get();
        let r = self.rect_sys();

        self.rect.set(r);

        if old_r.size() != r.size() {
            event_fire!(self.base.borrow().on_resize.clone(),
                make_window(self.rc_me()), r.size());
        }
        if old_r.point() != r.point() {
            event_fire!(self.base.borrow().on_move.clone(),
                make_window(self.rc_me()), r.point());
        }
    }

    fn rect_sys(&self) -> IRect {
        unsafe {
			let mut wr = mem::uninitialized::<RECT>();
			GetWindowRect(self.hwnd.get(), &mut wr);

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
        unsafe { GetWindowLongW(self.hwnd.get(), GWL_STYLE) as DWORD }
    }
    fn ex_style(&self) -> DWORD {
        unsafe { GetWindowLongW(self.hwnd.get(), GWL_EXSTYLE) as DWORD }
    }
}

impl PlatformWindow for Win32Window {

    fn create(&self) {
        let cls_name = self.shared_platform.register_wnd_cls();
        let title = to_u16(&self.base.borrow().title);
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
            let spbox = Box::new(Rc::downgrade(&self.shared_platform));
            let spbox = Box::into_raw(spbox);
            set_window_long_ptr(hwnd, 0, mem::transmute(spbox));
            hwnd
        };
        self.hwnd.set(hwnd);
        self.shared_platform.windows.borrow_mut()
                .insert(hwnd, self.weak_me.borrow().clone());
        self.update_state();
    }

    fn check_base(&self, base: &WindowBase) -> bool {
        let mine = &(*self.base.borrow()) as *const WindowBase;
        let foreign = base as *const WindowBase;
        mine == foreign
    }

    fn update_title(&self) {
        let title = to_u16(&self.base.borrow().title);
        unsafe { SetWindowTextW(self.hwnd.get(), title.as_ptr()); }
    }

    fn update_state(&self) {
        if !self.created() { self.create(); }
        unsafe {
            ShowWindow(self.hwnd.get(), SW_NORMAL);
        }
    }

    fn close(&self) {
        if self.created() {
            {
                let mut windows = self.shared_platform.windows.borrow_mut();
                windows.remove(&self.hwnd.get());
            }
            unsafe {
                let spbox = get_window_long_ptr(self.hwnd.get(), 0);
                let spbox: *mut Weak<Win32SharedPlatform> = mem::transmute(spbox);
                assert!(!spbox.is_null());
                let _ = Box::from_raw(spbox);
                set_window_long_ptr(self.hwnd.get(), 0, 0);
                DestroyWindow(self.hwnd.get());
            }
            self.hwnd.set(ptr::null_mut());
        }
    }
}

impl Drop for Win32Window {
    fn drop (&mut self) {
        self.close();
    }
}
