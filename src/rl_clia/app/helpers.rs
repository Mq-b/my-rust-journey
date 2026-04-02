use crate::config;
use chrono::{Duration, Local};

/// 返回当前日期字符串，格式为 `YYYY-MM-DD`。
pub(super) fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// 根据生产日期和有效天数计算失效日期。
pub(super) fn compute_expire(date: &str, days: i64) -> String {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// 批量校验字符串字段不能为空。
pub(super) fn require_fields(fields: &[(&str, &str)]) -> Result<(), String> {
    for (name, val) in fields {
        if val.trim().is_empty() {
            return Err(format!("「{name}」不能为空"));
        }
    }
    Ok(())
}

/// 读取项目名称，索引越界时返回空串。
pub(super) fn project_name_at(proj: &config::ProjectConfig, index: usize) -> String {
    proj.project_name_list
        .get(index)
        .cloned()
        .unwrap_or_default()
}

/// 读取项目编号，索引越界时返回空串。
pub(super) fn project_id_at(proj: &config::ProjectConfig, index: usize) -> String {
    proj.project_id_list.get(index).cloned().unwrap_or_default()
}

/// 如果字符串为空白，返回指定兜底值。
pub(super) fn blank_or(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

/// 还原特殊项目名中的希腊字母展示。
pub(super) fn restore_special_name(name: &str) -> String {
    match name {
        "S100B" => "S100β".into(),
        "AB1-42" => "Aβ1-42".into(),
        "B-HCG" => "β-HCG".into(),
        _ => name.to_string(),
    }
}
