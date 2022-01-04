use eframe::egui;
use eframe::egui::{Response, Ui, Widget};

pub struct RawEditUi<'a> {
    raw: &'a mut Vec<u8>,
    multiline: bool,
    width: f32,
}

impl<'a> RawEditUi<'a> {
    pub fn new(raw: &'a mut Vec<u8>, multiline: bool) -> Self {
        Self {
            raw,
            multiline,
            width: 0.0,
        }
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Widget for RawEditUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let RawEditUi {
            raw,
            multiline,
            width
        } = self;

        let mut line: String = if raw.len() > 0 {
            let bytes = raw.iter()
                .take(raw.len() - 1)
                .map(|b| format!("{:02X} ", b))
                .collect::<String>();
            let last = raw[raw.len() - 1];
            let last = if last < 0x10 {
                format!("{:X}", last)
            } else {
                format!("{:02X} ", last)
            };
            format!("{}{}", bytes, last)
        } else {
            Default::default()
        };

        let mut edit = if multiline {
            egui::TextEdit::multiline(&mut line)
        } else {
            egui::TextEdit::singleline(&mut line)
        };

        if width > 0.0 {
            edit = edit.desired_width(width);
        }

        let res = ui.add(edit);
        if res.changed() {
            if let Ok(bs) = line.split_whitespace()
                .map(|b| {
                    u8::from_str_radix(b, 16)
                })
                .collect::<Result<Vec<_>, _>>()
            {
                *raw = bs;
            }
        }
        res
    }
}
