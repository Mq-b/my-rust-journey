use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// 项目名称与项目编号的配置。
///
/// 数据优先从 `Setting/project.json` 读取；如果读取失败，则回退到内置默认值。
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    #[serde(rename = "projectIDList")]
    pub project_id_list: Vec<String>,
    #[serde(rename = "projectNameList")]
    pub project_name_list: Vec<String>,
}

/// 试剂批号对应的最新已分配 ID（按批号独立递增）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReagentIdState {
    #[serde(default)]
    pub last_id_by_lot: BTreeMap<String, u32>,
}

const REAGENT_ID_STATE_PATH: &str = "Setting/rl-clia-reagent-id-state.json";

/// 读取项目配置，并在失败时返回内置默认值。
pub fn load_project_config() -> ProjectConfig {
    let path = Path::new("Setting/project.json");
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str(&data) {
            return cfg;
        }
    }
    // fallback defaults
    ProjectConfig {
        project_id_list: (1..=24).map(|i| i.to_string()).collect(),
        project_name_list: vec![
            "cTnI",
            "NT-proBNP",
            "Myoglobin",
            "CK-MB",
            "PCT",
            "D-Dimer",
            "cTnT",
            "BNP",
            "IL-6",
            "S100β",
            "SAA",
            "CRP",
            "H-FABP",
            "NGAL",
            "PGI",
            "PGII",
            "HCY",
            "LP-PLA2",
            "ST2",
            "G-17",
            "Aβ1-42",
            "P-Tau181",
            "AD7c-NTP",
            "β-HCG",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    }
}

/// 读取试剂 ID 分配状态，失败时返回空状态。
pub fn load_reagent_id_state() -> ReagentIdState {
    let path = Path::new(REAGENT_ID_STATE_PATH);
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(state) = serde_json::from_str(&data) {
            return state;
        }
    }
    ReagentIdState::default()
}

/// 写入试剂 ID 分配状态。
pub fn save_reagent_id_state(state: &ReagentIdState) -> Result<(), String> {
    if let Some(parent) = Path::new(REAGENT_ID_STATE_PATH).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    let content =
        serde_json::to_string_pretty(state).map_err(|e| format!("序列化试剂ID配置失败: {e}"))?;
    std::fs::write(REAGENT_ID_STATE_PATH, content).map_err(|e| format!("写入试剂ID配置失败: {e}"))
}

/// 预览模式读取即将分配的 ID（不落盘，不改变状态）。
pub fn preview_reagent_ids(lot: &str, count: usize) -> Result<Vec<String>, String> {
    allocate_ids(lot, count, false)
}

/// 正式生成时分配 ID 并落盘，防止重启后重复。
pub fn consume_reagent_ids(lot: &str, count: usize) -> Result<Vec<String>, String> {
    allocate_ids(lot, count, true)
}

fn allocate_ids(lot: &str, count: usize, persist: bool) -> Result<Vec<String>, String> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let lot = lot.trim();
    if lot.is_empty() {
        return Err("试剂批号不能为空，无法分配唯一ID".into());
    }

    let mut state = load_reagent_id_state();
    let last_id = state.last_id_by_lot.get(lot).copied().unwrap_or(0);
    let mut ids = Vec::with_capacity(count);

    for offset in 1..=count {
        let step = u32::try_from(offset).map_err(|_| "数量过大，无法分配ID".to_string())?;
        let next = last_id
            .checked_add(step)
            .ok_or_else(|| "ID已达到上限，无法继续分配".to_string())?;
        ids.push(next.to_string());
    }

    if persist {
        let total = u32::try_from(count).map_err(|_| "数量过大，无法分配ID".to_string())?;
        let final_id = last_id
            .checked_add(total)
            .ok_or_else(|| "ID已达到上限，无法继续分配".to_string())?;
        state.last_id_by_lot.insert(lot.to_string(), final_id);
        save_reagent_id_state(&state)?;
    }

    Ok(ids)
}
