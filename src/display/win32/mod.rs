
use crate::window;
use libc::c_int;
// use winapi::shared::basetsd::{LONG_PTR};
use winapi::shared::windef::{HBRUSH, HWND};
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::um::winuser::{
    DefWindowProcW, LoadCursorW, LoadIconW, SetWindowTextW,
    RegisterClassExW, CreateWindowExW, ShowWindow,
    WNDCLASSEXW, CS_OWNDC,
    IDI_APPLICATION, IDC_ARROW, CW_USEDEFAULT, WS_EX_OVERLAPPEDWINDOW, 
    WS_OVERLAPPEDWINDOW, WS_MAXIMIZE, WS_MINIMIZE, SW_SHOWNORMAL,
    SW_MINIMIZE, SW_MAXIMIZE,
};
use winapi::um::libloaderapi::GetModuleHandleW;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::mem;
use std::ptr;

const WINDOW_CLASS: &'static str = "ows-rs_window_class";

// 32/64 bits compatibility
// #[cfg(target_arch = "x86_64")]
// unsafe fn get_window_long_ptr(hwnd: HWND, index: c_int) -> LONG_PTR {
//     use winapi::um::winuser::GetWindowLongPtrW;
//     GetWindowLongPtrW(hwnd, index)
// }
// #[cfg(not(target_arch = "x86_64"))]
// unsafe fn get_window_long_ptr(hwnd: HWND, index: c_int) -> LONG_PTR {
//     use winapi::um::winuser::GetWindowLongW;
//     GetWindowLongW(hwnd, index)
// }
// #[cfg(target_arch = "x86_64")]
// unsafe fn set_window_long_ptr(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
//     use winapi::um::winuser::SetWindowLongPtrW;
//     SetWindowLongPtrW(hwnd, index, value)
// }
// #[cfg(not(target_arch = "x86_64"))]
// unsafe fn set_window_long_ptr(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
//     use winapi::um::winuser::SetWindowLongW;
//     SetWindowLongW(hwnd, index, value)
// }

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
}

impl Window
{
    fn new() -> Window
    {
        Window {
            hwnd: ptr::null_mut(), title: String::new()
        }
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
        unsafe {
            let hinstance = GetModuleHandleW(ptr::null());
            let cls_name = to_u16(WINDOW_CLASS);
            let title = to_u16(&self.title);

            let (w, h) = match state {
                window::State::Normal(Some((w, h))) => (w as c_int, h as c_int),
                _ => (CW_USEDEFAULT, CW_USEDEFAULT)
            };

            let (s_ex, s) = match state {
                window::State::Normal(_) => (WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW),
                window::State::Maximized => (WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW | WS_MAXIMIZE),
                window::State::Minimized => (WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW | WS_MINIMIZE),
                window::State::Fullscreen => panic!("fullscreen unsupported at the moment"),
            };

            self.hwnd = CreateWindowExW(
                s_ex, cls_name.as_ptr(), title.as_ptr(), s,
                CW_USEDEFAULT, CW_USEDEFAULT, w, h,
                ptr::null_mut(), ptr::null_mut(), hinstance, ptr::null_mut()
            );

            let show = match state {
                window::State::Normal(_) => SW_SHOWNORMAL,
                window::State::Maximized => SW_MAXIMIZE,
                window::State::Minimized => SW_MINIMIZE,
                _ => 0,
            };

            ShowWindow(self.hwnd, show);
        }
    }

    fn close(&mut self)
    {}

}

unsafe extern "system"
fn win32_wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT 
{
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
