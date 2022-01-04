use eframe::egui::{ComboBox, Response, Ui, Widget};

use bin2json::secure::{Hasher, SecureKey};

pub struct SecureKeyUi<'a>(pub &'a mut SecureKey, pub String, pub bool);

impl Widget for SecureKeyUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let sk = self.0;
            let id = self.1;
            let show_hasher = self.2;
            if ui.radio(sk == &SecureKey::None, "不加密").clicked() {
                *sk = SecureKey::None;
            };

            if let SecureKey::RsaPkcs1Pem { secure_key, key, hasher } = sk {
                let _ = ui.radio(true, "RSA PKCS1 PEM");
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(secure_key, true, "私钥");
                        ui.radio_value(secure_key, false, "公钥");
                    });
                    ui.separator();

                    if show_hasher {
                        ui.label("签名哈希方式");
                        ComboBox::from_id_source(id)
                            .selected_text(hasher_label(*hasher))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(hasher, Hasher::None, hasher_label(Hasher::None));
                                ui.selectable_value(hasher, Hasher::SHA2_256, hasher_label(Hasher::SHA2_256));
                                ui.selectable_value(hasher, Hasher::SHA2_512, hasher_label(Hasher::SHA2_512));
                                ui.selectable_value(hasher, Hasher::SHA3_256, hasher_label(Hasher::SHA3_256));
                                ui.selectable_value(hasher, Hasher::SHA3_512, hasher_label(Hasher::SHA3_512));
                            });
                        ui.separator();
                    }

                    ui.text_edit_multiline(key);
                });
            } else {
                if ui.radio(false, "RSA PKCS1 PEM").clicked() {
                    *sk = SecureKey::RsaPkcs1Pem {
                        secure_key: true,
                        key: Default::default(),
                        hasher: Default::default(),
                    }
                };
            }
        }).response
    }
}

const fn hasher_label(hasher: Hasher) -> &'static str {
    match hasher {
        Hasher::None => "不进行哈希",
        Hasher::SHA2_256 => "SHA2_256",
        Hasher::SHA2_512 => "SHA2_512",
        Hasher::SHA3_256 => "SHA3_256",
        Hasher::SHA3_512 => "SHA3_512",
    }
}
