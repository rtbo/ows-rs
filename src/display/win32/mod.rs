
use crate::window;
use winapi::shared::windef::HWND;

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
        Ok(Display{})
    }

    fn create_window(&self) -> Window
    {
        Window{ _hwnd: std::ptr::null_mut(), title: String::new() }
    }
}


pub struct Window
{
    _hwnd: HWND,
    title: String,
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
    }

    fn show (&mut self, _state: window::State)
    {}

    fn close(&mut self)
    {}

}
