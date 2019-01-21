use crate::geom::{IPoint, IRect, ISize};
use crate::gfx;
use crate::key;
use crate::mouse;
use crate::window;
use libc::c_int;
use std::ffi::OsStr;
use std::iter;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::windowsx::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

mod keymap;

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
fn to_u16<S: AsRef<str>>(s: S) -> Vec<u16> {
    OsStr::new(s.as_ref())
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect()
}

pub struct Display {
    shared: Rc<DisplayShared>,
}

struct DisplayShared {
    instance: Arc<gfx::Instance>,
}

impl Drop for Display {
    fn drop(&mut self) {}
}

impl super::Display for Display {
    type Window = Window;
    type OpenError = ();

    fn open() -> Result<Display, ()> {
        Ok(Display::new())
    }

    fn create_window(&self) -> Window {
        Window::new(self.shared.clone())
    }

    fn collect_events(&self) {
        unsafe {
            let mut msg: MSG = mem::zeroed();
            while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn instance(&self) -> Arc<gfx_back::Instance> {
        self.shared.instance.clone()
    }
}

impl Display {
    fn new() -> Display {
        let display = Display {
            shared: Rc::new(DisplayShared {
                instance: Arc::new(gfx_back::Instance::create("ows-rs app", 0)),
            })
        };
        unsafe {
            display.register_window_class();
        }
        display
    }

    unsafe fn register_window_class(&self) {
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

pub struct Window {
    hwnd: HWND,
    title: String,
    saved_info: SavedInfo,
    disp_shared: Rc<DisplayShared>,
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
    event_comp: u32,
    rect: IRect,
    state: window::State,
    mods: key::Mods,
    mouse_state: mouse::State,
    mouse_pos: IPoint,
    mouse_out: bool,
}

const COMP_RESIZE: u32 = 1;
const COMP_MOUSE_MOVE: u32 = 2;

impl Window {
    fn new(disp_shared: Rc<DisplayShared>) -> Window {
        Window {
            hwnd: ptr::null_mut(),
            title: String::new(),
            saved_info: SavedInfo {
                rect: unsafe { mem::zeroed() },
                maximized: false,
                style: 0,
                ex_style: 0,
            },
            disp_shared,
            shared: None,
        }
    }

    fn create(&mut self, state: window::State) {
        let (s_ex, mut s) = (WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW);
        match state {
            window::State::Maximized => {
                s |= WS_MAXIMIZE;
            }
            window::State::Minimized => {
                s |= WS_MINIMIZE;
            }
            _ => {}
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
                s_ex,
                cls_name.as_ptr(),
                title.as_ptr(),
                s,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                w,
                h,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance,
                ptr::null_mut(),
            );
            let mut shared = Box::new(WindowShared::new());
            let shared_ptr = &mut *shared as *mut _;
            self.shared = Some(shared);
            set_window_long_ptr(hwnd, 0, mem::transmute(shared_ptr));
            hwnd
        };
        let mut shared = self.shared.as_mut().unwrap();
        println!("state = {:?}", state);
        shared.state = match state {
            window::State::Fullscreen => window::State::Normal(None),
            s @ _ => s,
        };
        println!("shared.state = {:?}", shared.state);
    }
}

impl window::Window<Display> for Window {
    fn title(&self) -> &str {
        &self.title
    }

    fn set_title(&mut self, val: String) {
        self.title = val;
        if !self.hwnd.is_null() {
            let tit = to_u16(&self.title);
            unsafe {
                SetWindowTextW(self.hwnd, tit.as_ptr());
            }
        }
    }

    fn size(&self) -> ISize {
        assert!(!self.hwnd.is_null());
        let r = unsafe { window_rect(self.hwnd) };
        r.size()
    }

    fn show(&mut self, state: window::State) {
        if self.hwnd.is_null() {
            assert!(state != window::State::Minimized, "cannot create window in minimized state");
            self.create(state);
            match state {
                window::State::Fullscreen => {}
                window::State::Normal(_) => {
                    unsafe { ShowWindow(self.hwnd, SW_SHOWNORMAL); }
                    return;
                }
                window::State::Maximized => {
                    unsafe { ShowWindow(self.hwnd, SW_SHOWMAXIMIZED); }
                    return;
                }
                _ => {}
            }
        }

        let mut shared = self.shared.as_mut().unwrap();
        if state == shared.state {
            return;
        }

        println!("{:?} != {:?}", state, shared.state);

        match (shared.state, state) {
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
                    let ex_style = self.saved_info.ex_style
                        & !(WS_EX_DLGMODALFRAME
                            | WS_EX_WINDOWEDGE
                            | WS_EX_CLIENTEDGE
                            | WS_EX_STATICEDGE);
                    set_window_long_ptr(self.hwnd, GWL_STYLE, style as LONG_PTR);
                    set_window_long_ptr(self.hwnd, GWL_EXSTYLE, ex_style as LONG_PTR);

                    // On expand, if we're given a window_rect, grow to it, otherwise do
                    // not resize.
                    let mut minfo = MONITORINFO {
                        cbSize: mem::size_of::<MONITORINFO>() as DWORD,
                        rcMonitor: RECT {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        },
                        rcWork: RECT {
                            left: 0,
                            top: 0,
                            right: 0,
                            bottom: 0,
                        },
                        dwFlags: 0,
                    };
                    GetMonitorInfoW(
                        MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST),
                        &mut minfo as *mut _,
                    );
                    let r = minfo.rcMonitor;
                    let (w, h) = (r.right - r.left, r.bottom - r.top);
                    SetWindowPos(
                        self.hwnd,
                        ptr::null_mut(),
                        r.left,
                        r.top,
                        w,
                        h,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                }
            }
            (window::State::Fullscreen, _) => unsafe {
                set_window_long_ptr(self.hwnd, GWL_STYLE, self.saved_info.style as LONG_PTR);
                set_window_long_ptr(self.hwnd, GWL_EXSTYLE, self.saved_info.ex_style as LONG_PTR);
                let r = self.saved_info.rect;
                let (w, h) = (r.right - r.left, r.bottom - r.top);
                SetWindowPos(
                    self.hwnd,
                    ptr::null_mut(),
                    r.left,
                    r.top,
                    w,
                    h,
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
                if self.saved_info.maximized {
                    SendMessageW(self.hwnd, WM_SYSCOMMAND, SC_MAXIMIZE, 0);
                }
            },
            _ => {}
        }

        match state {
            window::State::Normal(sz @ _) => unsafe {
                ShowWindow(self.hwnd, SW_SHOWNORMAL);
                println!("change state normal");
                if let Some((w, h)) = sz {
                    let r = self.saved_info.rect;
                    SetWindowPos(
                        self.hwnd,
                        ptr::null_mut(),
                        r.left,
                        r.top,
                        w as c_int,
                        h as c_int,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                }
            },
            window::State::Maximized => unsafe {
                ShowWindow(self.hwnd, SW_MAXIMIZE);
            },
            window::State::Minimized => unsafe {
                ShowWindow(self.hwnd, SW_MINIMIZE);
            },
            _ => {}
        }

        shared.state = state;
    }

    fn retrieve_events(&mut self) -> Vec<window::Event> {
        let mut evs = Vec::new();
        if let Some(ref mut shared) = self.shared.as_mut() {
            mem::swap(&mut evs, &mut shared.event_buf);
            shared.event_comp = 0;
        }
        evs
    }

    fn close(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                DestroyWindow(self.hwnd);
            }
            self.hwnd = ptr::null_mut();
            self.shared = None;
        }
    }

    fn token(&self) -> window::Token {
        assert!(!self.hwnd.is_null());
        window::Token::new(self.hwnd as _)
    }

    fn create_surface(&self) -> gfx::Surface {
        assert!(!self.hwnd.is_null());
        let hinstance = unsafe { GetModuleHandleW(ptr::null_mut()) };
        self.disp_shared.instance.create_surface_from_hwnd(hinstance as _, self.hwnd as _)
    }
}

impl WindowShared {
    fn new() -> WindowShared {
        WindowShared {
            event_buf: Vec::new(),
            event_comp: 0,
            rect: unsafe { mem::zeroed() },
            state: window::State::Normal(None),
            mods: key::Mods::empty(),
            mouse_state: mouse::State::empty(),
            mouse_pos: IPoint::new(0, 0),
            mouse_out: true,
        }
    }

    fn state_change(&mut self, state: window::State) -> bool {
        if state != self.state {
            self.event_buf.push(window::Event::State(state));
            self.state = state;
        }
        true
    }

    fn geom_change(&mut self, hwnd: HWND) -> bool {
        let new_r = unsafe { window_rect(hwnd) };

        let new_s = new_r.size();
        let old_s = self.rect.size();

        self.rect = new_r;

        if new_s.w != old_s.w || new_s.h != old_s.h {
            self.resize_event(new_s)
        } else {
            false
        }
    }

    fn state_geom_change(&mut self, hwnd: HWND, state: window::State) -> bool {
        let state = self.state_change(state);
        let geom = self.geom_change(hwnd);
        state || geom
    }

    fn mouse_change(&mut self, hwnd: HWND, msg: UINT, _wparam: WPARAM, lparam: LPARAM) -> bool {
        let pos = IPoint::new(GET_X_LPARAM(lparam), GET_Y_LPARAM(lparam));
        let mods = self.mods;

        match msg {
            WM_LBUTTONDOWN => {
                self.mouse_state.insert(mouse::State::LEFT);
                self.event(window::Event::MouseDown(
                    pos,
                    mouse::But::Left,
                    self.mouse_state,
                    mods,
                ))
            }
            WM_MBUTTONDOWN => {
                self.mouse_state.insert(mouse::State::MIDDLE);
                self.event(window::Event::MouseDown(
                    pos,
                    mouse::But::Middle,
                    self.mouse_state,
                    mods,
                ))
            }
            WM_RBUTTONDOWN => {
                self.mouse_state.insert(mouse::State::RIGHT);
                self.event(window::Event::MouseDown(
                    pos,
                    mouse::But::Right,
                    self.mouse_state,
                    mods,
                ))
            }
            WM_LBUTTONUP => {
                self.mouse_state.remove(mouse::State::LEFT);
                self.event(window::Event::MouseUp(
                    pos,
                    mouse::But::Left,
                    self.mouse_state,
                    mods,
                ))
            }
            WM_MBUTTONUP => {
                self.mouse_state.remove(mouse::State::MIDDLE);
                self.event(window::Event::MouseUp(
                    pos,
                    mouse::But::Middle,
                    self.mouse_state,
                    mods,
                ))
            }
            WM_RBUTTONUP => {
                self.mouse_state.remove(mouse::State::RIGHT);
                self.event(window::Event::MouseUp(
                    pos,
                    mouse::But::Right,
                    self.mouse_state,
                    mods,
                ))
            }
            WM_MOUSEMOVE => {
                if self.mouse_out {
                    self.mouse_out = false;

                    // mouse was out: deliver enter event
                    self.event_buf
                        .push(window::Event::MouseEnter(pos, self.mouse_state, mods));
                    // and register for leave event
                    let mut tm = TRACKMOUSEEVENT {
                        cbSize: mem::size_of::<TRACKMOUSEEVENT>() as DWORD,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    unsafe {
                        TrackMouseEvent(&mut tm);
                    }
                }
                self.mouse_pos = pos;
                self.mouse_move_event(window::Event::MouseMove(pos, self.mouse_state, mods))
            }
            WM_MOUSELEAVE => {
                self.mouse_out = true;
                self.event(window::Event::MouseLeave(
                    self.mouse_pos,
                    self.mouse_state,
                    mods,
                ))
            }
            _ => false,
        }
    }

    fn key_change(&mut self, hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> bool {
        debug_assert!(
            msg != WM_CHAR,
            "Char msg must be intercepted before delivery!"
        );
        assert!(wparam < 256);

        let sym = keymap::sym(wparam as u32);
        let scancode = (lparam as u32 & SCANCODE_MASK) >> SCANCODE_SHIFT;
        let code = keymap::code(scancode);
        self.mods = unsafe { key_mods() }; // caching for mouse move. FIXME: mods pressed out of window?

        match msg {
            WM_KEYDOWN => {
                let text = unsafe { peek_char_msg(hwnd) };
                self.event(window::Event::KeyDown(sym, code, self.mods, text))
            }
            WM_KEYUP => self.event(window::Event::KeyUp(sym, code, self.mods)),
            _ => false,
        }
    }

    fn event(&mut self, ev: window::Event) -> bool {
        self.event_buf.push(ev);
        true
    }

    fn resize_event(&mut self, new_size: ISize) -> bool {
        if self.event_comp & COMP_RESIZE == 0 {
            self.event_buf.push(window::Event::Resize(new_size));
            self.event_comp |= COMP_RESIZE;
            true
        } else {
            let mut handled = false;
            for ev in &mut self.event_buf {
                match ev {
                    window::Event::Resize(_) => {
                        *ev = window::Event::Resize(new_size);
                        handled = true;
                        break;
                    }
                    _ => {}
                }
            }
            debug_assert!(handled, "did not find compressed resize event");
            handled
        }
    }

    fn mouse_move_event(&mut self, mm_ev: window::Event) -> bool {
        // assert ev is mouse move?
        if self.event_comp & COMP_MOUSE_MOVE == 0 {
            self.event_buf.push(mm_ev);
            self.event_comp |= COMP_MOUSE_MOVE;
            true
        } else {
            let mut handled = false;
            for ev in &mut self.event_buf {
                match ev {
                    window::Event::MouseMove(_, _, _) => {
                        *ev = mm_ev;
                        handled = true;
                        break;
                    }
                    _ => {}
                }
            }
            debug_assert!(handled, "did not find compressed resize event");
            handled
        }
    }
}

//const PREVIOUS_STATE_MASK: u32 = 0x40000000;
const REPEAT_COUNT_MASK: u32 = 0x0000ffff;
const SCANCODE_MASK: u32 = 0x00ff0000;
const SCANCODE_SHIFT: u32 = 16;

unsafe fn peek_char_msg(hwnd: HWND) -> String {
    let mut msg: MSG = mem::zeroed();
    if PeekMessageW(&mut msg, hwnd, WM_CHAR, WM_CHAR, PM_REMOVE) != 0 {
        let count = msg.lParam as u32 & REPEAT_COUNT_MASK;
        let utf16: Vec<u16> = iter::repeat(msg.wParam as u16)
            .take(count as usize)
            .collect();
        String::from_utf16_lossy(&utf16)
    } else {
        String::new()
    }
}

unsafe fn window_rect(hwnd: HWND) -> IRect {
    let style = get_window_long_ptr(hwnd, GWL_STYLE);
    let ex_style = get_window_long_ptr(hwnd, GWL_EXSTYLE);

    let mut wr: RECT = mem::uninitialized();
    GetWindowRect(hwnd, &mut wr as *mut _);
    let mut ar = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    AdjustWindowRectEx(&mut ar as *mut _, style as _, FALSE, ex_style as _);

    wr.left -= ar.left;
    wr.top -= ar.top;
    wr.right -= ar.right;
    wr.bottom -= ar.bottom;

    IRect::new(wr.left, wr.top, wr.right - wr.left, wr.bottom - wr.top)
}

unsafe fn key_mods() -> key::Mods {
    let mut mods = key::Mods::empty();

    if GetKeyState(VK_LSHIFT) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::LEFT_SHIFT);
    }
    if GetKeyState(VK_LCONTROL) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::LEFT_CTRL);
    }
    if GetKeyState(VK_LMENU) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::LEFT_ALT);
    }
    if GetKeyState(VK_LWIN) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::SUPER);
    }

    if GetKeyState(VK_RSHIFT) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::RIGHT_SHIFT);
    }
    if GetKeyState(VK_RCONTROL) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::RIGHT_CTRL);
    }
    if GetKeyState(VK_RMENU) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::RIGHT_ALT);
    }
    if GetKeyState(VK_RWIN) as u16 & 0x8000 != 0 {
        mods.insert(key::Mods::SUPER);
    }

    mods
}

unsafe extern "system" fn win32_wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let shared: *mut WindowShared = mem::transmute(get_window_long_ptr(hwnd, 0));
    if shared.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let shared: &mut WindowShared = mem::transmute(shared);

    let handled = match msg {
        WM_CLOSE => shared.event(window::Event::Close),
        WM_SIZE => {
            match wparam {
                SIZE_MINIMIZED => shared.state_change(window::State::Minimized), // TODO: state change event
                SIZE_MAXIMIZED => shared.state_geom_change(hwnd, window::State::Maximized),
                SIZE_RESTORED => shared.state_geom_change(hwnd, window::State::Normal(None)),
                _ => false,
            }
        }
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_RBUTTONDOWN
        | WM_RBUTTONUP | WM_MOUSEMOVE | WM_MOUSELEAVE => {
            shared.mouse_change(hwnd, msg, wparam, lparam)
        }
        WM_KEYDOWN | WM_KEYUP | WM_CHAR => shared.key_change(hwnd, msg, wparam, lparam),
        _ => false,
    };

    if handled {
        0
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
