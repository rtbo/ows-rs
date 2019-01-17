use crate::key;

pub fn code(scancode: u32) -> key::Code {
    debug_assert!(scancode < 256);
    CODES[scancode as usize]
}

pub fn sym(vkey: u32) -> key::Sym {
    debug_assert!(vkey < 256);
    SYMS[vkey as usize]
}

pub const CODES: [key::Code; 256] = [
    // 0x00     0
    key::Code::Unknown,
    key::Code::Escape,
    key::Code::N1,
    key::Code::N2,
    key::Code::N3,
    key::Code::N4,
    key::Code::N5,
    key::Code::N6,
    key::Code::N7,
    key::Code::N8,
    key::Code::N9,
    key::Code::N0,
    key::Code::Minus,
    key::Code::Equals,
    key::Code::Backspace,
    key::Code::Tab,
    // 0x10     16
    key::Code::Q,
    key::Code::W,
    key::Code::E,
    key::Code::R,
    key::Code::T,
    key::Code::Y,
    key::Code::U,
    key::Code::I,
    key::Code::O,
    key::Code::P,
    key::Code::LeftBracket,
    key::Code::RightBracket,
    key::Code::Enter,
    key::Code::LeftCtrl,
    key::Code::A,
    key::Code::S,
    // 0x20     32
    key::Code::D,
    key::Code::F,
    key::Code::G,
    key::Code::H,
    key::Code::J,
    key::Code::K,
    key::Code::L,
    key::Code::Semicolon,
    key::Code::Quote,
    key::Code::Grave,
    key::Code::LeftShift,
    key::Code::UK_Hash,
    key::Code::Z,
    key::Code::X,
    key::Code::C,
    key::Code::V,
    // 0x30     48
    key::Code::B,
    key::Code::N,
    key::Code::M,
    key::Code::Comma,
    key::Code::Period,
    key::Code::Slash,
    key::Code::RightShift,
    key::Code::PrintScreen,
    key::Code::LeftAlt,
    key::Code::Space,
    key::Code::CapsLock,
    key::Code::F1,
    key::Code::F2,
    key::Code::F3,
    key::Code::F4,
    key::Code::F5,
    // 0x40     64
    key::Code::F6,
    key::Code::F7,
    key::Code::F8,
    key::Code::F9,
    key::Code::F10,
    key::Code::Kp_NumLock,
    key::Code::ScrollLock,
    key::Code::Home,
    key::Code::Up,
    key::Code::PageUp,
    key::Code::Kp_Subtract,
    key::Code::Left,
    key::Code::Kp_5,
    key::Code::Right,
    key::Code::Kp_Add,
    key::Code::End,
    // 0x50     80
    key::Code::Down,
    key::Code::PageDown,
    key::Code::Insert,
    key::Code::Delete,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Kp_Add,
    key::Code::F11,
    key::Code::F12,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0x60     96
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown, // line feed
    key::Code::Unknown,
    key::Code::Unknown,
    // 0x70     112
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0x80     128
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0x90     144
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0xA0     160
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0xB0     176
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0xC0     192
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0xD0     208
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0xE0     224
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    // 0xF0     240
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
    key::Code::Unknown,
];

// a little help from Qt for that one
pub const SYMS: [key::Sym; 256] = [
    // Dec |  Hex | Windows Virtual key
    key::Sym::Unknown,      //   0   0x00
    key::Sym::Unknown,      //   1   0x01   VK_LBUTTON          | Left mouse button
    key::Sym::Unknown,      //   2   0x02   VK_RBUTTON          | Right mouse button
    key::Sym::Cancel,       //   3   0x03   VK_CANCEL           | Control-Break processing
    key::Sym::Unknown,      //   4   0x04   VK_MBUTTON          | Middle mouse button
    key::Sym::Unknown,      //   5   0x05   VK_XBUTTON1         | X1 mouse button
    key::Sym::Unknown,      //   6   0x06   VK_XBUTTON2         | X2 mouse button
    key::Sym::Unknown,      //   7   0x07   -- unassigned --
    key::Sym::Backspace,    //   8   0x08   VK_BACK             | BackSpace key
    key::Sym::Tab,          //   9   0x09   VK_TAB              | Tab key
    key::Sym::Unknown,      //  10   0x0A   -- reserved --
    key::Sym::Unknown,      //  11   0x0B   -- reserved --
    key::Sym::Clear,        //  12   0x0C   VK_CLEAR            | Clear key
    key::Sym::Return,      //  13   0x0D   VK_RETURN           | Enter key
    key::Sym::Unknown,      //  14   0x0E   -- unassigned --
    key::Sym::Unknown,      //  15   0x0F   -- unassigned --
    key::Sym::Shift,        //  16   0x10   VK_SHIFT            | Shift key
    key::Sym::Ctrl,         //  17   0x11   VK_CONTROL          | Ctrl key
    key::Sym::Alt,          //  18   0x12   VK_MENU             | Alt key
    key::Sym::Pause,        //  19   0x13   VK_PAUSE            | Pause key
    key::Sym::CapsLock,     //  20   0x14   VK_CAPITAL          | Caps-Lock
    key::Sym::Unknown,      //  21   0x15   VK_KANA / VK_HANGUL | IME Kana or Hangul mode
    key::Sym::Unknown,      //  22   0x16   -- unassigned --
    key::Sym::Unknown,       //  23   0x17   VK_JUNJA            | IME Junja mode
    key::Sym::Unknown,       //  24   0x18   VK_FINAL            | IME final mode
    key::Sym::Unknown,        //  25   0x19   VK_HANJA / VK_KANJI | IME Hanja or Kanji mode
    key::Sym::Unknown,      //  26   0x1A   -- unassigned --
    key::Sym::Escape,       //  27   0x1B   VK_ESCAPE           | Esc key
    key::Sym::Unknown,      //  28   0x1C   VK_CONVERT          | IME convert
    key::Sym::Unknown,      //  29   0x1D   VK_NONCONVERT       | IME non-convert
    key::Sym::Unknown,      //  30   0x1E   VK_ACCEPT           | IME accept
    key::Sym::ModeSwitch,   //  31   0x1F   VK_MODECHANGE       | IME mode change request
    key::Sym::Space,        //  32   0x20   VK_SPACE            | Spacebar
    key::Sym::PageUp,       //  33   0x21   VK_PRIOR            | Page Up key
    key::Sym::PageDown,     //  34   0x22   VK_NEXT             | Page Down key
    key::Sym::End,          //  35   0x23   VK_END              | End key
    key::Sym::Home,         //  36   0x24   VK_HOME             | Home key
    key::Sym::Left,         //  37   0x25   VK_LEFT             | Left arrow key
    key::Sym::Up,           //  38   0x26   VK_UP               | Up arrow key
    key::Sym::Right,        //  39   0x27   VK_RIGHT            | Right arrow key
    key::Sym::Down,         //  40   0x28   VK_DOWN             | Down arrow key
    key::Sym::Select,       //  41   0x29   VK_SELECT           | Select key
    key::Sym::Printer,      //  42   0x2A   VK_PRINT            | Print key
    key::Sym::Execute,      //  43   0x2B   VK_EXECUTE          | Execute key
    key::Sym::Print,        //  44   0x2C   VK_SNAPSHOT         | Print Screen key
    key::Sym::Insert,       //  45   0x2D   VK_INSERT           | Ins key
    key::Sym::Delete,      //  46   0x2E   VK_DELETE           | Del key
    key::Sym::Help,         //  47   0x2F   VK_HELP             | Help key
    key::Sym::D0,         //  48   0x30   (VK_0)              | 0 key
    key::Sym::D1,         //  49   0x31   (VK_1)              | 1 key
    key::Sym::D2,         //  50   0x32   (VK_2)              | 2 key
    key::Sym::D3,         //  51   0x33   (VK_3)              | 3 key
    key::Sym::D4,         //  52   0x34   (VK_4)              | 4 key
    key::Sym::D5,         //  53   0x35   (VK_5)              | 5 key
    key::Sym::D6,         //  54   0x36   (VK_6)              | 6 key
    key::Sym::D7,         //  55   0x37   (VK_7)              | 7 key
    key::Sym::D8,         //  56   0x38   (VK_8)              | 8 key
    key::Sym::D9,         //  57   0x39   (VK_9)              | 9 key
    key::Sym::Unknown,      //  58   0x3A   -- unassigned --
    key::Sym::Unknown,      //  59   0x3B   -- unassigned --
    key::Sym::Unknown,      //  60   0x3C   -- unassigned --
    key::Sym::Unknown,      //  61   0x3D   -- unassigned --
    key::Sym::Unknown,      //  62   0x3E   -- unassigned --
    key::Sym::Unknown,      //  63   0x3F   -- unassigned --
    key::Sym::Unknown,      //  64   0x40   -- unassigned --
    key::Sym::A,         //  65   0x41   (VK_A)              | A key
    key::Sym::B,         //  66   0x42   (VK_B)              | B key
    key::Sym::C,         //  67   0x43   (VK_C)              | C key
    key::Sym::D,         //  68   0x44   (VK_D)              | D key
    key::Sym::E,         //  69   0x45   (VK_E)              | E key
    key::Sym::F,         //  70   0x46   (VK_F)              | F key
    key::Sym::G,         //  71   0x47   (VK_G)              | G key
    key::Sym::H,         //  72   0x48   (VK_H)              | H key
    key::Sym::I,         //  73   0x49   (VK_I)              | I key
    key::Sym::J,         //  74   0x4A   (VK_J)              | J key
    key::Sym::K,         //  75   0x4B   (VK_K)              | K key
    key::Sym::L,         //  76   0x4C   (VK_L)              | L key
    key::Sym::M,         //  77   0x4D   (VK_M)              | M key
    key::Sym::N,         //  78   0x4E   (VK_N)              | N key
    key::Sym::O,         //  79   0x4F   (VK_O)              | O key
    key::Sym::P,         //  80   0x50   (VK_P)              | P key
    key::Sym::Q,         //  81   0x51   (VK_Q)              | Q key
    key::Sym::R,         //  82   0x52   (VK_R)              | R key
    key::Sym::S,         //  83   0x53   (VK_S)              | S key
    key::Sym::T,         //  84   0x54   (VK_T)              | T key
    key::Sym::U,         //  85   0x55   (VK_U)              | U key
    key::Sym::V,         //  86   0x56   (VK_V)              | V key
    key::Sym::W,         //  87   0x57   (VK_W)              | W key
    key::Sym::X,         //  88   0x58   (VK_X)              | X key
    key::Sym::Y,         //  89   0x59   (VK_Y)              | Y key
    key::Sym::Z,         //  90   0x5A   (VK_Z)              | Z key
    key::Sym::LeftSuper,    //  91   0x5B   VK_LWIN             | Left Windows  - MS Natural kbd
    key::Sym::RightSuper,   //  92   0x5C   VK_RWIN             | Right Windows - MS Natural kbd
    key::Sym::Menu,         //  93   0x5D   VK_APPS             | Application key-MS Natural kbd
    key::Sym::Unknown,      //  94   0x5E   -- reserved --
    key::Sym::Sleep,        //  95   0x5F   VK_SLEEP
    key::Sym::Kp_0,         //  96   0x60   VK_NUMPAD0          | Numeric keypad 0 key
    key::Sym::Kp_1,         //  97   0x61   VK_NUMPAD1          | Numeric keypad 1 key
    key::Sym::Kp_2,         //  98   0x62   VK_NUMPAD2          | Numeric keypad 2 key
    key::Sym::Kp_3,         //  99   0x63   VK_NUMPAD3          | Numeric keypad 3 key
    key::Sym::Kp_4,         // 100   0x64   VK_NUMPAD4          | Numeric keypad 4 key
    key::Sym::Kp_5,         // 101   0x65   VK_NUMPAD5          | Numeric keypad 5 key
    key::Sym::Kp_6,         // 102   0x66   VK_NUMPAD6          | Numeric keypad 6 key
    key::Sym::Kp_7,         // 103   0x67   VK_NUMPAD7          | Numeric keypad 7 key
    key::Sym::Kp_8,         // 104   0x68   VK_NUMPAD8          | Numeric keypad 8 key
    key::Sym::Kp_9,         // 105   0x69   VK_NUMPAD9          | Numeric keypad 9 key
    key::Sym::Kp_Multiply,  // 106   0x6A   VK_MULTIPLY         | Multiply key
    key::Sym::Kp_Add,       // 107   0x6B   VK_ADD              | Add key
    key::Sym::Kp_Separator, // 108   0x6C   VK_SEPARATOR        | Separator key
    key::Sym::Kp_Subtract,  // 109   0x6D   VK_SUBTRACT         | Subtract key
    key::Sym::Kp_Decimal,   // 110   0x6E   VK_DECIMAL          | Decimal key
    key::Sym::Kp_Divide,    // 111   0x6F   VK_DIVIDE           | Divide key
    key::Sym::F1,           // 112   0x70   VK_F1               | F1 key
    key::Sym::F2,           // 113   0x71   VK_F2               | F2 key
    key::Sym::F3,           // 114   0x72   VK_F3               | F3 key
    key::Sym::F4,           // 115   0x73   VK_F4               | F4 key
    key::Sym::F5,           // 116   0x74   VK_F5               | F5 key
    key::Sym::F6,           // 117   0x75   VK_F6               | F6 key
    key::Sym::F7,           // 118   0x76   VK_F7               | F7 key
    key::Sym::F8,           // 119   0x77   VK_F8               | F8 key
    key::Sym::F9,           // 120   0x78   VK_F9               | F9 key
    key::Sym::F10,          // 121   0x79   VK_F10              | F10 key
    key::Sym::F11,          // 122   0x7A   VK_F11              | F11 key
    key::Sym::F12,          // 123   0x7B   VK_F12              | F12 key
    key::Sym::F13,          // 124   0x7C   VK_F13              | F13 key
    key::Sym::F14,          // 125   0x7D   VK_F14              | F14 key
    key::Sym::F15,          // 126   0x7E   VK_F15              | F15 key
    key::Sym::F16,          // 127   0x7F   VK_F16              | F16 key
    key::Sym::F17,          // 128   0x80   VK_F17              | F17 key
    key::Sym::F18,          // 129   0x81   VK_F18              | F18 key
    key::Sym::F19,          // 130   0x82   VK_F19              | F19 key
    key::Sym::F20,          // 131   0x83   VK_F20              | F20 key
    key::Sym::F21,          // 132   0x84   VK_F21              | F21 key
    key::Sym::F22,          // 133   0x85   VK_F22              | F22 key
    key::Sym::F23,          // 134   0x86   VK_F23              | F23 key
    key::Sym::F24,          // 135   0x87   VK_F24              | F24 key
    key::Sym::Unknown,      // 136   0x88   -- unassigned --
    key::Sym::Unknown,      // 137   0x89   -- unassigned --
    key::Sym::Unknown,      // 138   0x8A   -- unassigned --
    key::Sym::Unknown,      // 139   0x8B   -- unassigned --
    key::Sym::Unknown,      // 140   0x8C   -- unassigned --
    key::Sym::Unknown,      // 141   0x8D   -- unassigned --
    key::Sym::Unknown,      // 142   0x8E   -- unassigned --
    key::Sym::Unknown,      // 143   0x8F   -- unassigned --
    key::Sym::NumLock,      // 144   0x90   VK_NUMLOCK          | Num Lock key
    key::Sym::ScrollLock,   // 145   0x91   VK_SCROLL           | Scroll Lock key
    // Fujitsu/OASYS kbd --------------------
    key::Sym::Unknown, // 146   0x92   VK_OEM_FJ_JISHO     | 'Dictionary' key /
    //              VK_OEM_NEC_EQUAL  = key on numpad on NEC PC-9800 kbd
    key::Sym::Unknown, // 147   0x93   VK_OEM_FJ_MASSHOU   | 'Unregister word' key
    key::Sym::Unknown, // 148   0x94   VK_OEM_FJ_TOUROKU   | 'Register word' key
    key::Sym::Unknown, // 149   0x95   VK_OEM_FJ_LOYA      | 'Left OYAYUBI' key
    key::Sym::Unknown, // 150   0x96   VK_OEM_FJ_ROYA      | 'Right OYAYUBI' key
    key::Sym::Unknown, // 151   0x97   -- unassigned --
    key::Sym::Unknown, // 152   0x98   -- unassigned --
    key::Sym::Unknown, // 153   0x99   -- unassigned --
    key::Sym::Unknown, // 154   0x9A   -- unassigned --
    key::Sym::Unknown, // 155   0x9B   -- unassigned --
    key::Sym::Unknown, // 156   0x9C   -- unassigned --
    key::Sym::Unknown, // 157   0x9D   -- unassigned --
    key::Sym::Unknown, // 158   0x9E   -- unassigned --
    key::Sym::Unknown, // 159   0x9F   -- unassigned --
    key::Sym::LeftShift, // 160   0xA0   VK_LSHIFT           | Left Shift key
    key::Sym::RightShift, // 161   0xA1   VK_RSHIFT           | Right Shift key
    key::Sym::LeftCtrl, // 162   0xA2   VK_LCONTROL         | Left Ctrl key
    key::Sym::RightCtrl, // 163   0xA3   VK_RCONTROL         | Right Ctrl key
    key::Sym::LeftAlt, // 164   0xA4   VK_LMENU            | Left Menu key
    key::Sym::RightAlt, // 165   0xA5   VK_RMENU            | Right Menu key
    key::Sym::Unknown, // 166   0xA6   VK_BROWSER_BACK     | Browser Back key
    key::Sym::Unknown, // 167   0xA7   VK_BROWSER_FORWARD  | Browser Forward key
    key::Sym::Unknown, // 168   0xA8   VK_BROWSER_REFRESH  | Browser Refresh key
    key::Sym::Unknown, // 169   0xA9   VK_BROWSER_STOP     | Browser Stop key
    key::Sym::Unknown, // 170   0xAA   VK_BROWSER_SEARCH   | Browser Search key
    key::Sym::Unknown, // 171   0xAB   VK_BROWSER_FAVORITES| Browser Favorites key
    key::Sym::Unknown, // 172   0xAC   VK_BROWSER_HOME     | Browser Start and Home key
    key::Sym::VolumeMute, // 173   0xAD   VK_VOLUME_MUTE      | Volume Mute key
    key::Sym::VolumeDown, // 174   0xAE   VK_VOLUME_DOWN      | Volume Down key
    key::Sym::VolumeUp, // 175   0xAF   VK_VOLUME_UP        | Volume Up key
    key::Sym::MediaNext, // 176   0xB0   VK_MEDIA_NEXT_TRACK | Next Track key
    key::Sym::MediaPrevious, // 177   0xB1   VK_MEDIA_PREV_TRACK | Previous Track key
    key::Sym::MediaStop, // 178   0xB2   VK_MEDIA_STOP       | Stop Media key
    key::Sym::MediaPlay, // 179   0xB3   VK_MEDIA_PLAY_PAUSE | Play/Pause Media key
    key::Sym::LaunchMail, // 180   0xB4   VK_LAUNCH_MAIL      | Start Mail key
    key::Sym::LaunchMedia, // 181   0xB5   VK_LAUNCH_MEDIA_SELECT Select Media key
    key::Sym::Launch0, // 182   0xB6   VK_LAUNCH_APP1      | Start Application 1 key
    key::Sym::Launch1, // 183   0xB7   VK_LAUNCH_APP2      | Start Application 2 key
    key::Sym::Unknown, // 184   0xB8   -- reserved --
    key::Sym::Unknown, // 185   0xB9   -- reserved --
    key::Sym::Semicolon, // 186   0xBA   VK_OEM_1            | ';:' for US
    key::Sym::Plus,    // 187   0xBB   VK_OEM_PLUS         | '+' any country
    key::Sym::Comma,   // 188   0xBC   VK_OEM_COMMA        | ',' any country
    key::Sym::Minus,   // 189   0xBD   VK_OEM_MINUS        | '-' any country
    key::Sym::Period,  // 190   0xBE   VK_OEM_PERIOD       | '.' any country
    key::Sym::Slash,   // 191   0xBF   VK_OEM_2            | '/?' for US
    key::Sym::AsciiTilde, // 192   0xC0   VK_OEM_3            | '`~' for US
    key::Sym::Unknown, // 193   0xC1   -- reserved --
    key::Sym::Unknown, // 194   0xC2   -- reserved --
    key::Sym::Unknown, // 195   0xC3   -- reserved --
    key::Sym::Unknown, // 196   0xC4   -- reserved --
    key::Sym::Unknown, // 197   0xC5   -- reserved --
    key::Sym::Unknown, // 198   0xC6   -- reserved --
    key::Sym::Unknown, // 199   0xC7   -- reserved --
    key::Sym::Unknown, // 200   0xC8   -- reserved --
    key::Sym::Unknown, // 201   0xC9   -- reserved --
    key::Sym::Unknown, // 202   0xCA   -- reserved --
    key::Sym::Unknown, // 203   0xCB   -- reserved --
    key::Sym::Unknown, // 204   0xCC   -- reserved --
    key::Sym::Unknown, // 205   0xCD   -- reserved --
    key::Sym::Unknown, // 206   0xCE   -- reserved --
    key::Sym::Unknown, // 207   0xCF   -- reserved --
    key::Sym::Unknown, // 208   0xD0   -- reserved --
    key::Sym::Unknown, // 209   0xD1   -- reserved --
    key::Sym::Unknown, // 210   0xD2   -- reserved --
    key::Sym::Unknown, // 211   0xD3   -- reserved --
    key::Sym::Unknown, // 212   0xD4   -- reserved --
    key::Sym::Unknown, // 213   0xD5   -- reserved --
    key::Sym::Unknown, // 214   0xD6   -- reserved --
    key::Sym::Unknown, // 215   0xD7   -- reserved --
    key::Sym::Unknown, // 216   0xD8   -- unassigned --
    key::Sym::Unknown, // 217   0xD9   -- unassigned --
    key::Sym::Unknown, // 218   0xDA   -- unassigned --
    key::Sym::BracketLeft, // 219   0xDB   VK_OEM_4            | '[{' for US
    key::Sym::Bar,     // 220   0xDC   VK_OEM_5            | '\|' for US
    key::Sym::BracketRight, // 221   0xDD   VK_OEM_6            | ']}' for US
    key::Sym::Unknown, // 222   0xDE   VK_OEM_7            | ''"' for US
    key::Sym::Unknown, // 223   0xDF   VK_OEM_8
    key::Sym::Unknown, // 224   0xE0   -- reserved --
    key::Sym::Unknown, // 225   0xE1   VK_OEM_AX           | 'AX' key on Japanese AX kbd
    key::Sym::Unknown, // 226   0xE2   VK_OEM_102          | "<>" or "\|" on RT 102-key kbd
    key::Sym::Unknown, // 227   0xE3   VK_ICO_HELP         | Help key on ICO
    key::Sym::Unknown, // 228   0xE4   VK_ICO_00           | 00 key on ICO
    key::Sym::Unknown, // 229   0xE5   VK_PROCESSKEY       | IME Process key
    key::Sym::Unknown, // 230   0xE6   VK_ICO_CLEAR        |
    key::Sym::Unknown, // 231   0xE7   VK_PACKET           | Unicode char as keystrokes
    key::Sym::Unknown, // 232   0xE8   -- unassigned --
    // Nokia/Ericsson definitions ---------------
    key::Sym::Unknown, // 233   0xE9   VK_OEM_RESET
    key::Sym::Unknown, // 234   0xEA   VK_OEM_JUMP
    key::Sym::Unknown, // 235   0xEB   VK_OEM_PA1
    key::Sym::Unknown, // 236   0xEC   VK_OEM_PA2
    key::Sym::Unknown, // 237   0xED   VK_OEM_PA3
    key::Sym::Unknown, // 238   0xEE   VK_OEM_WSCTRL
    key::Sym::Unknown, // 239   0xEF   VK_OEM_CUSEL
    key::Sym::Unknown, // 240   0xF0   VK_OEM_ATTN
    key::Sym::Unknown, // 241   0xF1   VK_OEM_FINISH
    key::Sym::Unknown, // 242   0xF2   VK_OEM_COPY
    key::Sym::Unknown, // 243   0xF3   VK_OEM_AUTO
    key::Sym::Unknown, // 244   0xF4   VK_OEM_ENLW
    key::Sym::Unknown, // 245   0xF5   VK_OEM_BACKTAB
    key::Sym::Unknown, // 246   0xF6   VK_ATTN             | Attn key
    key::Sym::Unknown, // 247   0xF7   VK_CRSEL            | CrSel key
    key::Sym::Unknown, // 248   0xF8   VK_EXSEL            | ExSel key
    key::Sym::Unknown, // 249   0xF9   VK_EREOF            | Erase EOF key
    key::Sym::Play,    // 250   0xFA   VK_PLAY             | Play key
    key::Sym::Zoom,    // 251   0xFB   VK_ZOOM             | Zoom key
    key::Sym::Unknown, // 252   0xFC   VK_NONAME           | Reserved
    key::Sym::Unknown, // 253   0xFD   VK_PA1              | PA1 key
    key::Sym::Clear,   // 254   0xFE   VK_OEM_CLEAR        | Clear key
    key::Sym::Unknown,
];
