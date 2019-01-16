
use crate::window;
use libc::c_int;
use winapi::shared::basetsd::{LONG_PTR};
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

const WINDOW_CLASS: &'static str = "ows-rs_window_class";

// 32/64 bits compatibility
#[cfg(target_arch = "x86_64")]
unsafe fn get_window_long_ptr(hwnd: HWND, index: c_int) -> LONG_PTR {
    use winapi::um::winuser::GetWindowLongPtrW;
    GetWindowLongPtrW(hwnd, index)
}
#[cfg(not(target_arch = "x86_64"))]
unsafe fn get_window_long_ptr(hwnd: HWND, index: c_int) -> LONG_PTR {
    use winapi::um::winuser::GetWindowLongW;
    GetWindowLongW(hwnd, index)
}
#[cfg(target_arch = "x86_64")]
unsafe fn set_window_long_ptr(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    use winapi::um::winuser::SetWindowLongPtrW;
    SetWindowLongPtrW(hwnd, index, value)
}
#[cfg(not(target_arch = "x86_64"))]
unsafe fn set_window_long_ptr(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    use winapi::um::winuser::SetWindowLongW;
    SetWindowLongW(hwnd, index, value)
}

/// convert a String or &str into utf16 string usable in Windows Unicode API
fn to_u16<S : AsRef<str>>(s: S) -> Vec<u16> {
    OsStr::new(s.as_ref()).encode_wide().chain(Some(0).into_iter()).collect()
}

pub struct Display
{

}

impl Drop for Display
{
    fn drop(&mut self) {}
}

impl super::Display for Display
{
    type Window = Window;
    type OpenError = ();

    fn open() -> Result<Display, ()>
    {
        Ok(Display::new())
    }

    fn create_window(&self) -> Window
    {
        Window::new()
    }
}

impl Display
{
    fn new() -> Display
    {
        let display = Display{};
        unsafe { display.register_window_class(); }
        display
    }

    unsafe fn register_window_class(&self)
    {
        let instance = GetModuleHandleW(ptr::null());
        let cls_name = to_u16(WINDOW_CLASS);

        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_OWNDC,
            lpfnWndProc: Some(win32_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: mem::size_of::<usize>() as c_int,
            hInstance: instance,
            hIcon: LoadIconW(ptr::null_mut() as HINSTANCE, IDI_APPLICATION),
            hCursor: LoadCursorW(ptr::null_mut() as HINSTANCE, IDC_ARROW),
            hbrBackground: ptr::null_mut() as HBRUSH,
            lpszMenuName: ptr::null() as *const u16,
            lpszClassName: cls_name.as_ptr(),
            hIconSm: LoadIconW(ptr::null_mut() as HINSTANCE, IDI_APPLICATION),
        };

        if RegisterClassExW(&wc) == 0 {
            panic!("Could not register window class {}", WINDOW_CLASS);
        }
    }
}

pub struct Window
{
    hwnd: HWND,
    title: String,
    state: window::State,
    saved_info: SavedInfo,
    shared: Option<Box<WindowShared>>,
}

struct SavedInfo {
    rect: RECT,
    maximized: bool,
    style: DWORD,
    ex_style: DWORD,
}

struct WindowShared {
    event_buf: Vec<window::Event>,
}

impl Window
{
    fn new() -> Window
    {
        Window {
            hwnd: ptr::null_mut(), 
            title: String::new(),
            state: window::State::Normal(None),
            saved_info: SavedInfo {
                rect: RECT {left: 0, top: 0, right: 0, bottom: 0},
                maximized: false,
                style: 0, ex_style: 0
            },
            shared: None,
        }
    }

    fn create(&mut self, state: window::State)
    {
        let (s_ex, mut s) = (WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW);
        match state {
            window::State::Maximized => { s |= WS_MAXIMIZE; },
            window::State::Minimized => { s |= WS_MINIMIZE; },
            _ => {},
        }
        let (w, h) = match state {
            window::State::Normal(Some((w, h))) => (w as c_int, h as c_int),
            _ => (CW_USEDEFAULT, CW_USEDEFAULT),
        };
        let cls_name = to_u16(WINDOW_CLASS);
        let title = to_u16(&self.title);


        self.hwnd = unsafe {
            let hinstance = GetModuleHandleW(ptr::null());
            let hwnd = CreateWindowExW(
                s_ex, cls_name.as_ptr(), title.as_ptr(), s,
                CW_USEDEFAULT, CW_USEDEFAULT, w, h,
                ptr::null_mut(), ptr::null_mut(), hinstance, ptr::null_mut()
            );
            let mut shared = Box::new(WindowShared { event_buf: Vec::new() });
            let shared_ptr = &mut *shared as *mut _; 
            self.shared = Some(shared);
            set_window_long_ptr(hwnd, 0, mem::transmute(shared_ptr));
            hwnd
        };
        self.state = match state {
            window::State::Fullscreen => window::State::Normal(None),
            s @ _ => s,
        };
    }
}

impl window::Window<Display> for Window
{
    fn title(&self) -> &str
    {
        &self.title
    }

    fn set_title(&mut self, val: String)
    {
        self.title = val;
        if !self.hwnd.is_null() {
            let tit = to_u16(&self.title);
            unsafe { SetWindowTextW(self.hwnd, tit.as_ptr()); }
        }
    }

    fn show (&mut self, state: window::State)
    {
        if self.hwnd.is_null() {
            self.create(state);
            unsafe { ShowWindow(self.hwnd, SW_SHOWNORMAL); }
        }

        if state == self.state { return; }

        match (self.state, state) {
            (_, window::State::Fullscreen) => {
                unsafe {
                    // save some info for next time mode changes
                    self.saved_info.style = get_window_long_ptr(self.hwnd, GWL_STYLE) as DWORD;
                    self.saved_info.ex_style = get_window_long_ptr(self.hwnd, GWL_EXSTYLE) as DWORD;
                    GetWindowRect(self.hwnd, &mut self.saved_info.rect as *mut _);
                    self.saved_info.maximized = IsZoomed(self.hwnd) != 0;
                    if self.saved_info.maximized {
                        SendMessageW(self.hwnd, WM_SYSCOMMAND, SC_RESTORE, 0);
                    } 

                    // Set new window style and size.
                    let style = self.saved_info.style & !(WS_CAPTION | WS_THICKFRAME);
                    let ex_style = self.saved_info.ex_style & !(
                        WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE
                    );
                    set_window_long_ptr(self.hwnd, GWL_STYLE, style as LONG_PTR);
                    set_window_long_ptr(self.hwnd, GWL_EXSTYLE, ex_style as LONG_PTR);

                    // On expand, if we're given a window_rect, grow to it, otherwise do
                    // not resize.
                    let mut minfo = MONITORINFO {
                        cbSize: mem::size_of::<MONITORINFO>() as DWORD,
                        rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                        rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                        dwFlags: 0,
                    };
                    GetMonitorInfoW(MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST), &mut minfo as *mut _);
                    let r = minfo.rcMonitor;
                    let (w, h) = (r.right - r.left, r.bottom - r.top);
                    SetWindowPos(
                        self.hwnd, ptr::null_mut(), r.left, r.top, w, h,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
                    );
                }
            },
            (window::State::Fullscreen, _) => {
                unsafe {
                    set_window_long_ptr(self.hwnd, GWL_STYLE, self.saved_info.style as LONG_PTR);
                    set_window_long_ptr(self.hwnd, GWL_EXSTYLE, self.saved_info.ex_style as LONG_PTR);
                    let r = self.saved_info.rect;
                    let (w, h) = (r.right - r.left, r.bottom - r.top);
                    SetWindowPos(
                        self.hwnd, ptr::null_mut(), r.left, r.top, w, h,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
                    );
                    if self.saved_info.maximized {
                        SendMessageW(self.hwnd, WM_SYSCOMMAND, SC_MAXIMIZE, 0);
                    }
                }
            },
            _ => {}
        }

        match state {
            window::State::Normal(sz @ _) => {
                unsafe { 
                    ShowWindow(self.hwnd, SW_SHOWNORMAL); 
                    if let Some((w, h)) = sz {
                        let r = self.saved_info.rect;
                        SetWindowPos(
                            self.hwnd, ptr::null_mut(), r.left, r.top, w as c_int, h as c_int,
                            SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED
                        );
                    }
                }
            },
            window::State::Maximized => {
                unsafe { ShowWindow(self.hwnd, SW_MAXIMIZE); }
            },
            window::State::Minimized => {
                unsafe { ShowWindow(self.hwnd, SW_MINIMIZE); }
            },
            _ => {}
        }

        self.state = state;
    }

    fn close(&mut self)
    {
        if !self.hwnd.is_null() {
            unsafe { DestroyWindow(self.hwnd); }
            self.hwnd = ptr::null_mut();
            self.shared = None;
        }
    }
}

unsafe extern "system"
fn win32_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT 
{
    let shared: *mut WindowShared = mem::transmute(get_window_long_ptr(hwnd, 0));
    if shared.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let shared: &mut WindowShared = mem::transmute(shared);

    match msg {
        WM_CLOSE => {
            println!("closing!");
            shared.event_buf.push(window::Event::Close);
            0
        },
        _ => {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}
