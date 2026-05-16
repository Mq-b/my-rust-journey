#![windows_subsystem = "windows"]

mod barcode;

use slint::Model;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

slint::include_modules!();

/// 获取配置文件路径 (assets/projects.json)
fn config_path() -> PathBuf {
    let exe_path = std::env::current_exe().expect("无法获取可执行文件路径");
    let exe_dir = exe_path.parent().expect("无法获取可执行文件目录");
    exe_dir.join("assets").join("projects.json")
}

/// 加载项目配置
fn load_projects() -> HashMap<u8, String> {
    let path = config_path();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

/// 保存项目配置
fn save_projects(projects: &HashMap<u8, String>) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let data = serde_json::to_string_pretty(projects).unwrap_or_default();
    let _ = std::fs::write(&path, data);
}

/// 转换为 UI 模型
fn to_model(projects: &HashMap<u8, String>) -> Vec<ProjectEntry> {
    let mut entries: Vec<_> = projects
        .iter()
        .map(|(id, name)| ProjectEntry {
            id: id.to_string().into(),
            name: name.clone().into(),
        })
        .collect();
    entries.sort_by(|a, b| a.id.to_string().cmp(&b.id.to_string()));
    entries
}

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    let projects = Arc::new(Mutex::new(load_projects()));

    // 初始化项目列表
    {
        let projs = projects.lock().unwrap();
        let model = to_model(&projs);
        app.set_project_list(std::rc::Rc::new(slint::VecModel::from(model)).into());
    }

    // 打开配置回调
    {
        let weak = app.as_weak();
        let projs = projects.clone();
        app.on_open_config(move || {
            let app = weak.unwrap();
            let projs = projs.lock().unwrap();
            let model = to_model(&projs);
            app.set_project_list(std::rc::Rc::new(slint::VecModel::from(model)).into());
        });
    }

    // 新增项目回调
    {
        let weak = app.as_weak();
        let projs = projects.clone();
        app.on_add_project(move || {
            let app = weak.unwrap();
            let mut projs = projs.lock().unwrap();
            // 找一个未使用的ID
            let mut new_id = 1u8;
            while projs.contains_key(&new_id) {
                new_id = new_id.wrapping_add(1);
                if new_id == 0 {
                    new_id = 1;
                }
            }
            projs.insert(new_id, "新项目".to_string());
            let model = to_model(&projs);
            app.set_project_list(std::rc::Rc::new(slint::VecModel::from(model)).into());
        });
    }

    // 保存配置回调
    {
        let weak = app.as_weak();
        let projs = projects.clone();
        app.on_save_config(move || {
            let app = weak.unwrap();
            let list = app.get_project_list();
            let mut new_projects = HashMap::new();
            for i in 0..list.row_count() {
                if let Some(entry) = list.row_data(i) {
                    if let Ok(id) = entry.id.to_string().parse::<u8>() {
                        new_projects.insert(id, entry.name.to_string());
                    }
                }
            }
            save_projects(&new_projects);
            *projs.lock().unwrap() = new_projects;
            app.set_is_error(false);
            app.set_error_message("配置已保存".into());
        });
    }

    // 删除项目回调
    {
        let weak = app.as_weak();
        let projs = projects.clone();
        app.on_remove_project(move |index| {
            let app = weak.unwrap();
            let mut projs = projs.lock().unwrap();
            let keys: Vec<u8> = projs.keys().copied().collect();
            if (index as usize) < keys.len() {
                projs.remove(&keys[index as usize]);
                let model = to_model(&projs);
                app.set_project_list(std::rc::Rc::new(slint::VecModel::from(model)).into());
            }
        });
    }

    // 解码回调
    {
        let weak = app.as_weak();
        let projs = projects.clone();
        app.on_decode_input(move |input| {
            let app = weak.unwrap();
            let projs = projs.lock().unwrap();
            match barcode::QcBarcode::parse(input.as_str()) {
                Ok(qc) => {
                    app.set_raw_barcode(input);
                    app.set_project_id(qc.project_id.to_string().into());
                    app.set_lot_number(qc.lot_number.to_string().into());
                    app.set_level(format!("Level {}", qc.level).into());
                    app.set_is_error(false);
                    app.set_error_message("".into());
                    let name: slint::SharedString = projs
                        .get(&qc.project_id)
                        .map(|s| s.clone().into())
                        .unwrap_or_else(|| "未配置".into());
                    app.set_project_name(name);
                }
                Err(e) => {
                    app.set_is_error(true);
                    app.set_error_message(e.to_string().into());
                    app.set_raw_barcode("--".into());
                    app.set_project_id("--".into());
                    app.set_project_name("--".into());
                    app.set_lot_number("--".into());
                    app.set_level("--".into());
                }
            }
        });
    }

    app.run()
}
