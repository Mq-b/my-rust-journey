#![windows_subsystem = "windows"]

mod crypto;
mod excel;

slint::include_modules!();

fn main() {
    let window = LiteCryptWindow::new().unwrap();

    {
        let w = window.as_weak();
        window.on_encrypt(move || {
            let window = w.unwrap();
            let input = window.get_input_text().to_string();
            let key = window.get_key_input().to_string();

            match crypto::encrypt_lines(&input, &key) {
                Ok(result) => {
                    window.set_encrypted_result(result.clone().into());
                    window.set_status("加密成功".into());
                }
                Err(e) => {
                    window.set_status(format!("加密失败: {e}").into());
                }
            }
        });
    }

    {
        let w = window.as_weak();
        window.on_decrypt(move || {
            let window = w.unwrap();
            let input = window.get_input_text().to_string();
            let key = window.get_key_input().to_string();

            match crypto::decrypt_lines(&input, &key) {
                Ok(result) => {
                    window.set_decrypted_result(result.clone().into());
                    window.set_status("解密成功".into());
                }
                Err(e) => {
                    window.set_status(format!("解密失败: {e}").into());
                }
            }
        });
    }

    {
        let w = window.as_weak();
        window.on_save_results(move || {
            let window = w.unwrap();
            let encrypted = window.get_encrypted_result().to_string();
            let decrypted = window.get_decrypted_result().to_string();

            if encrypted.is_empty() && decrypted.is_empty() {
                window.set_status("没有结果可保存".into());
                return;
            }

            let path = rfd::FileDialog::new()
                .add_filter("Excel Files", &["xlsx"])
                .set_title("保存结果")
                .save_file();

            match path {
                Some(p) => {
                    match excel::save_to_excel(p.to_str().unwrap_or(""), &encrypted, &decrypted) {
                        Ok(_) => {
                            window.set_encrypted_result("".into());
                            window.set_decrypted_result("".into());
                            window.set_status(format!("已保存到: {}", p.display()).into());
                        }
                        Err(e) => {
                            window.set_status(format!("保存失败: {e}").into());
                        }
                    }
                }
                None => {
                    window.set_status("取消保存".into());
                }
            }
        });
    }

    {
        let w = window.as_weak();
        window.on_import_excel(move || {
            let window = w.unwrap();

            let path = rfd::FileDialog::new()
                .add_filter("Excel Files", &["xlsx"])
                .set_title("选择Excel文件")
                .pick_file();

            match path {
                Some(p) => match excel::load_first_column(p.to_str().unwrap_or("")) {
                    Ok(data) => {
                        window.set_input_text(data.into());
                        window.set_status("导入成功".into());
                    }
                    Err(e) => {
                        window.set_status(format!("导入失败: {e}").into());
                    }
                },
                None => {
                    window.set_status("取消导入".into());
                }
            }
        });
    }

    window.run().unwrap();
}
