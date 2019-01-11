
pub enum State
{
    Normal(Option<(u16, u16)>),
    Maximized,
    Minimized,
    Fullscreen,
}

pub trait Window
{
    fn title(&self) -> String;
    fn set_title(&mut self, val: String);

    fn show (&mut self, state: State);

    fn close(&mut self);
}
