use ratatui::style::{Color, Modifier, Style};

pub const BG:      Color = Color::Rgb(12,  12,  12);
pub const SURFACE: Color = Color::Rgb(22,  22,  22);
pub const BORDER:  Color = Color::Rgb(48,  48,  48);
pub const FOCUS:   Color = Color::Rgb(232, 168, 56);
pub const TEXT:    Color = Color::Rgb(210, 210, 210);
pub const DIM:     Color = Color::Rgb(88,  88,  88);
pub const ACCENT:  Color = Color::Rgb(232, 168, 56);
pub const GREEN:   Color = Color::Rgb(120, 190, 120);
pub const RED:     Color = Color::Rgb(220, 88,  88);
pub const YELLOW:  Color = Color::Rgb(220, 190, 70);
pub const BLUE:    Color = Color::Rgb(100, 155, 215);

pub fn normal()   -> Style { Style::default().fg(TEXT).bg(SURFACE) }
pub fn dim()      -> Style { Style::default().fg(DIM) }
pub fn accent()   -> Style { Style::default().fg(ACCENT).add_modifier(Modifier::BOLD) }
pub fn selected() -> Style { Style::default().fg(ACCENT).bg(Color::Rgb(36,36,36)).add_modifier(Modifier::BOLD) }
pub fn ok()       -> Style { Style::default().fg(GREEN) }
pub fn err()      -> Style { Style::default().fg(RED) }

pub fn border(focused: bool) -> Style {
    if focused { Style::default().fg(FOCUS) } else { Style::default().fg(BORDER) }
}
