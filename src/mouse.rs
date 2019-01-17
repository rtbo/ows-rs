
bitflags!{
    pub struct Buts : u8 {
        const LEFT      = 0x01;
        const MIDDLE    = 0x02;
        const RIGHT     = 0x04;
        const MASK      = 0x07;
    }
}

impl Default for Buts {
    fn default() -> Buts {
        Buts { bits: 0 }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum But
{
    Left, Middle, Right
}
