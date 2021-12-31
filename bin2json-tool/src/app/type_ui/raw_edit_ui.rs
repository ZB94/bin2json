use eframe::egui;
use eframe::egui::{Response, Ui, Widget};

pub struct RawEditUi<'a> {
    raw: &'a mut Vec<u8>,
    multiline: bool,
}

impl<'a> RawEditUi<'a> {
    pub fn new(raw: &'a mut Vec<u8>, multiline: bool) -> Self {
        Self {
            raw,
            multiline,
        }
    }
}

impl Widget for RawEditUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let raw = self.raw;
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

        let edit = if self.multiline {
            egui::TextEdit::multiline(&mut line)
        } else {
            egui::TextEdit::singleline(&mut line)
        };

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
