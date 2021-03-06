//! Human Interface Device Manager

use crate::ui::window::*;
use crate::*;
use bitflags::*;
use core::cell::UnsafeCell;
use core::num::*;
use megstd::drawing::*;
use megstd::io::hid::*;

const INVALID_UNICHAR: char = '\u{FEFF}';

bitflags! {
    pub struct KeyEventFlags: u8 {
        const BREAK = 0b1000_0000;
    }
}

/// USB HID BIOS Keyboard Raw Report
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Default)]
pub struct KeyReportRaw {
    pub modifier: Modifier,
    _reserved_1: u8,
    pub keydata: [Usage; 6],
}

#[derive(Debug)]
pub struct KeyboardState {
    pub current: KeyReportRaw,
    pub prev: KeyReportRaw,
}

impl KeyboardState {
    pub fn process_key_report(&mut self, report: KeyReportRaw) {
        let modifier = report.modifier;
        self.prev = self.current;
        self.current = report;
        for usage in &self.prev.keydata {
            let usage = *usage;
            if usage != Usage::NONE
                && usage < Usage::MOD_MIN
                && usage > Usage::MOD_MAX
                && !self.current.keydata.contains(&usage)
            {
                KeyEvent::new(usage, modifier, KeyEventFlags::BREAK).post();
            }
        }
        for usage in &self.current.keydata {
            let usage = *usage;
            if usage != Usage::NONE
                && usage < Usage::MOD_MIN
                && usage > Usage::MOD_MAX
                && !self.prev.keydata.contains(&usage)
            {
                KeyEvent::new(usage, modifier, KeyEventFlags::empty()).post();
            }
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct KeyEvent(NonZeroU32);

impl KeyEvent {
    #[inline]
    pub const fn new(usage: Usage, modifier: Modifier, flags: KeyEventFlags) -> Self {
        unsafe {
            Self(NonZeroU32::new_unchecked(
                usage.0 as u32 | ((modifier.bits() as u32) << 16) | ((flags.bits() as u32) << 24),
            ))
        }
    }

    #[inline]
    pub fn into_char(self) -> char {
        HidManager::key_event_to_char(self)
    }

    #[inline]
    pub const fn usage(self) -> Usage {
        Usage((self.0.get() & 0xFF) as u8)
    }

    #[inline]
    pub const fn modifier(self) -> Modifier {
        unsafe { Modifier::from_bits_unchecked(((self.0.get() >> 16) & 0xFF) as u8) }
    }

    #[inline]
    pub const fn flags(self) -> KeyEventFlags {
        unsafe { KeyEventFlags::from_bits_unchecked(((self.0.get() >> 24) & 0xFF) as u8) }
    }

    /// Returns the data for which a valid key was pressed. Otherwise, it is None.
    #[inline]
    pub fn key_data(self) -> Option<Self> {
        if self.usage() != Usage::NONE && !self.flags().contains(KeyEventFlags::BREAK) {
            Some(self)
        } else {
            None
        }
    }

    #[inline]
    pub fn post(self) {
        WindowManager::post_key_event(self);
    }
}

impl Into<char> for KeyEvent {
    #[inline]
    fn into(self) -> char {
        self.into_char()
    }
}

/// USB HID BIOS Mouse Raw Report
pub type MouseReportRaw = MouseReport<i8>;

#[derive(Debug, Copy, Clone, Default)]
pub struct MouseState {
    pub current_buttons: MouseButton,
    pub prev_buttons: MouseButton,
    pub x: isize,
    pub y: isize,
}

impl MouseState {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            current_buttons: MouseButton::empty(),
            prev_buttons: MouseButton::empty(),
            x: 0,
            y: 0,
        }
    }

    #[inline]
    pub fn process_mouse_report<T>(&mut self, report: MouseReport<T>)
    where
        T: Into<isize> + Copy,
    {
        self.prev_buttons = self.current_buttons;
        self.current_buttons = report.buttons;
        self.x += report.x.into();
        self.y += report.y.into();
        WindowManager::post_mouse_event(self);
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct MouseEvent {
    pub x: i16,
    pub y: i16,
    pub buttons: MouseButton,
    pub event_buttons: MouseButton,
}

impl MouseEvent {
    #[inline]
    pub const fn new(point: Point, buttons: MouseButton, event_buttons: MouseButton) -> Self {
        Self {
            x: point.x as i16,
            y: point.y as i16,
            buttons,
            event_buttons,
        }
    }

    #[inline]
    pub const fn point(&self) -> Point {
        Point {
            x: self.x as isize,
            y: self.y as isize,
        }
    }

    #[inline]
    pub const fn buttons(&self) -> MouseButton {
        self.buttons
    }

    #[inline]
    pub const fn event_buttons(&self) -> MouseButton {
        self.event_buttons
    }
}

/// HidManager relays between human interface devices and the window event subsystem.
///
/// Keyboard scancodes will be converted to the Usage specified by the USB-HID specification on all platforms.
pub struct HidManager;

static mut HID_MANAGER: UnsafeCell<HidManager> = UnsafeCell::new(HidManager::new());

impl HidManager {
    #[inline]
    const fn new() -> Self {
        HidManager {}
    }

    #[inline]
    pub unsafe fn init() {
        //
    }

    #[inline]
    #[allow(dead_code)]
    fn shared<'a>() -> &'a HidManager {
        unsafe { &*HID_MANAGER.get() }
    }

    #[inline]
    fn key_event_to_char(event: KeyEvent) -> char {
        if event.flags().contains(KeyEventFlags::BREAK) || event.usage() == Usage::NONE {
            '\0'
        } else {
            Self::usage_to_char_109(event.usage(), event.modifier())
        }
    }

    fn usage_to_char_109(usage: Usage, modifier: Modifier) -> char {
        let mut uni: char = INVALID_UNICHAR;

        if usage >= Usage::ALPHABET_MIN && usage <= Usage::ALPHABET_MAX {
            uni = (usage.0 - Usage::KEY_A.0 + 0x61) as char;
        } else if usage >= Usage::NUMBER_MIN && usage <= Usage::NON_ALPHABET_MAX {
            uni = USAGE_TO_CHAR_NON_ALPLABET_109[(usage.0 - Usage::NUMBER_MIN.0) as usize];
            if uni > ' ' && uni < '\x40' && uni != '0' && modifier.has_shift() {
                uni = (uni as u8 ^ 0x10) as char;
            }
        } else if usage == Usage::DELETE {
            uni = '\x7F';
        } else if usage >= Usage::NUMPAD_MIN && usage <= Usage::NUMPAD_MAX {
            uni = USAGE_TO_CHAR_NUMPAD[(usage.0 - Usage::NUMPAD_MIN.0) as usize];
        } else if usage == Usage::INTERNATIONAL_3 {
            // '\|'
            uni = '\\';
        }

        if uni >= '\x40' && uni < '\x7F' {
            if modifier.has_ctrl() {
                uni = (uni as u8 & 0x1F) as char;
            } else if modifier.has_shift() {
                uni = (uni as u8 ^ 0x20) as char;
            }
        }

        if usage == Usage::INTERNATIONAL_1 {
            if modifier.has_shift() {
                uni = '_';
            } else {
                uni = '\\';
            }
        }

        uni
    }
}

// Non Alphabet
static USAGE_TO_CHAR_NON_ALPLABET_109: [char; 27] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '\x0D', '\x1B', '\x08', '\x09', ' ', '-',
    '^', '@', '[', ']', ']', ';', ':', '`', ',', '.', '/',
];

// Numpads
static USAGE_TO_CHAR_NUMPAD: [char; 16] = [
    '/', '*', '-', '+', '\x0D', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.',
];
