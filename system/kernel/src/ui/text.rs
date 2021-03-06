// Text Drawing

use super::font::*;
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use megstd::drawing::*;

pub struct AttributedString<'a> {
    text: &'a str,
    font: FontDescriptor,
    color: SomeColor,
    line_break_mode: LineBreakMode,
    align: TextAlignment,
    valign: VerticalAlignment,
}

impl AttributedString<'_> {
    pub fn new() -> AttributedStringBuilder {
        AttributedStringBuilder::new()
    }

    #[inline]
    pub const fn text(&self) -> &str {
        self.text
    }

    #[inline]
    pub const fn font(&self) -> &FontDescriptor {
        &self.font
    }

    #[inline]
    pub const fn color(&self) -> SomeColor {
        self.color
    }

    #[inline]
    pub const fn line_break_mode(&self) -> LineBreakMode {
        self.line_break_mode
    }

    #[inline]
    pub const fn align(&self) -> TextAlignment {
        self.align
    }

    #[inline]
    pub const fn valign(&self) -> VerticalAlignment {
        self.valign
    }

    #[inline]
    pub fn bounding_size(&self, size: Size, max_lines: usize) -> Size {
        TextProcessing::bounding_size(self.font, self.text, size, max_lines, self.line_break_mode)
    }

    #[inline]
    pub fn draw_text(&self, bitmap: &mut Bitmap, rect: Rect, max_lines: usize) {
        TextProcessing::draw_text(
            bitmap,
            self.text,
            self.font,
            rect,
            self.color,
            max_lines,
            self.line_break_mode,
            self.align,
            self.valign,
        );
    }
}

pub struct AttributedStringBuilder {
    font: FontDescriptor,
    color: SomeColor,
    line_break_mode: LineBreakMode,
    align: TextAlignment,
    valign: VerticalAlignment,
}

impl AttributedStringBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            font: FontManager::ui_font(),
            color: SomeColor::BLACK,
            line_break_mode: LineBreakMode::default(),
            align: TextAlignment::Leading,
            valign: VerticalAlignment::Center,
        }
    }

    #[inline]
    pub fn text(self, text: &str) -> AttributedString {
        AttributedString {
            text,
            font: self.font,
            color: self.color,
            line_break_mode: self.line_break_mode,
            align: self.align,
            valign: self.valign,
        }
    }

    #[inline]
    pub fn font(mut self, font: FontDescriptor) -> Self {
        self.font = font;
        self
    }

    #[inline]
    pub fn color(mut self, color: SomeColor) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn line_break_mode(mut self, line_break_mode: LineBreakMode) -> Self {
        self.line_break_mode = line_break_mode;
        self
    }

    #[inline]
    pub fn align(mut self, align: TextAlignment) -> Self {
        self.align = align;
        self
    }

    #[inline]
    pub fn center(mut self) -> Self {
        self.align = TextAlignment::Center;
        self
    }

    #[inline]
    pub fn trailing(mut self) -> Self {
        self.align = TextAlignment::Trailing;
        self
    }

    #[inline]
    pub fn valign(mut self, valign: VerticalAlignment) -> Self {
        self.valign = valign;
        self
    }

    #[inline]
    pub fn top_left(mut self) -> Self {
        self.valign = VerticalAlignment::Top;
        self.align = TextAlignment::Left;
        self
    }

    #[inline]
    pub fn top_center(mut self) -> Self {
        self.valign = VerticalAlignment::Top;
        self.align = TextAlignment::Center;
        self
    }

    #[inline]
    pub fn top_right(mut self) -> Self {
        self.valign = VerticalAlignment::Top;
        self.align = TextAlignment::Right;
        self
    }

    #[inline]
    pub fn middle_left(mut self) -> Self {
        self.valign = VerticalAlignment::Center;
        self.align = TextAlignment::Left;
        self
    }

    #[inline]
    pub fn middle_center(mut self) -> Self {
        self.valign = VerticalAlignment::Center;
        self.align = TextAlignment::Center;
        self
    }

    #[inline]
    pub fn middle_right(mut self) -> Self {
        self.valign = VerticalAlignment::Center;
        self.align = TextAlignment::Right;
        self
    }

    #[inline]
    pub fn bottom_left(mut self) -> Self {
        self.valign = VerticalAlignment::Bottom;
        self.align = TextAlignment::Left;
        self
    }

    #[inline]
    pub fn bottom_center(mut self) -> Self {
        self.valign = VerticalAlignment::Bottom;
        self.align = TextAlignment::Center;
        self
    }

    #[inline]
    pub fn bottom_right(mut self) -> Self {
        self.valign = VerticalAlignment::Bottom;
        self.align = TextAlignment::Right;
        self
    }
}

pub struct TextProcessing {
    //
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineBreakMode {
    CharWrapping,
    WordWrapping,
    TrancatingTail,
}

impl Default for LineBreakMode {
    fn default() -> Self {
        Self::CharWrapping
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Leading,
    Trailing,
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::Leading
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Bottom,
    Center,
}

impl Default for VerticalAlignment {
    fn default() -> Self {
        Self::Top
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineStatus {
    pub start_position: usize,
    pub end_position: usize,
    pub width: isize,
    pub height: isize,
}

impl LineStatus {
    const fn empty() -> Self {
        Self {
            start_position: 0,
            end_position: 0,
            width: 0,
            height: 0,
        }
    }
}

impl TextProcessing {
    pub fn line_statuses(
        font: FontDescriptor,
        s: &str,
        size: Size,
        max_lines: usize,
        line_break: LineBreakMode,
    ) -> Vec<LineStatus> {
        let max_lines = NonZeroUsize::new(max_lines)
            .map(|v| v.get())
            .unwrap_or(usize::MAX);
        let limit_max_lines = 64;
        let mut vec = Vec::with_capacity(usize::min(max_lines, limit_max_lines));

        // TODO: Line Breaking
        let _ = line_break;

        let mut current_line = LineStatus::empty();
        current_line.height = font.line_height();
        let mut current_height = current_line.height;
        for (index, c) in s.chars().enumerate() {
            match c {
                '\n' => {
                    current_line.end_position = index;
                    current_height += current_line.height;
                    vec.push(current_line);
                    current_line = LineStatus::empty();
                    if vec.len() >= max_lines || current_height >= size.height() {
                        break;
                    }
                    current_line.start_position = index + 1;
                    current_line.height = font.line_height();
                }
                _ => {
                    current_line.end_position = index;
                    let current_width = font.width_of(c);
                    let new_width = current_line.width + current_width;
                    if current_line.width > 0 && new_width > size.width {
                        current_height += current_line.height;
                        vec.push(current_line);
                        current_line = LineStatus::empty();
                        if vec.len() >= max_lines || current_height >= size.height() {
                            break;
                        }
                        current_line.start_position = index;
                        current_line.width = current_width;
                        current_line.height = font.line_height();
                    } else {
                        current_line.width = new_width;
                    }
                }
            }
        }
        if vec.len() < max_lines && current_line.width > 0 {
            current_line.end_position += 1;
            vec.push(current_line);
        }

        vec
    }

    pub fn bounding_size(
        font: FontDescriptor,
        s: &str,
        size: Size,
        max_lines: usize,
        line_break: LineBreakMode,
    ) -> Size {
        let lines = Self::line_statuses(font, s, size, max_lines, line_break);
        Size::new(
            lines.iter().fold(0, |v, i| isize::max(v, i.width)),
            lines.iter().fold(0, |v, i| v + i.height),
        )
    }

    /// Write string to bitmap
    pub fn write_str(
        to: &mut Bitmap,
        s: &str,
        font: FontDescriptor,
        origin: Point,
        color: SomeColor,
    ) {
        Self::draw_text(
            to,
            s,
            font,
            Coordinates::new(
                origin.x,
                origin.y,
                to.width() as isize,
                to.height() as isize,
            )
            .into(),
            color,
            1,
            LineBreakMode::default(),
            TextAlignment::default(),
            VerticalAlignment::default(),
        )
    }

    /// Write text to bitmap
    pub fn draw_text(
        to: &mut Bitmap,
        s: &str,
        font: FontDescriptor,
        rect: Rect,
        color: SomeColor,
        max_lines: usize,
        line_break: LineBreakMode,
        align: TextAlignment,
        valign: VerticalAlignment,
    ) {
        let coords = match Coordinates::from_rect(rect) {
            Ok(v) => v,
            Err(_) => return,
        };

        let lines = Self::line_statuses(font, s, rect.size(), max_lines, line_break);
        let mut chars = s.chars();
        let mut cursor = Point::default();
        let mut prev_position = 0;

        let perferred_height = lines.iter().fold(0, |v, i| v + i.height);
        // let preferred_width = lines.iter().fold(0, |v, i| isize::max(v, i.width));
        cursor.y = match valign {
            VerticalAlignment::Top => coords.top,
            VerticalAlignment::Center => isize::max(
                coords.top,
                coords.top + (rect.height() - perferred_height) / 2,
            ),
            VerticalAlignment::Bottom => isize::max(coords.top, coords.bottom - perferred_height),
        };
        for line in lines {
            for _ in prev_position..line.start_position {
                let _ = chars.next();
            }

            if line.start_position < line.end_position {
                cursor.x = match align {
                    TextAlignment::Leading | TextAlignment::Left => coords.left,
                    TextAlignment::Trailing | TextAlignment::Right => coords.right - line.width,
                    TextAlignment::Center => coords.left + (rect.width() - line.width) / 2,
                };
                for _ in line.start_position..line.end_position {
                    let c = chars.next().unwrap();
                    font.draw_char(c, to, cursor, color);
                    cursor.x += font.width_of(c);
                }
            }

            prev_position = line.end_position;
            cursor.y += line.height;
        }
    }
}
