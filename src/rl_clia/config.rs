use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    #[serde(rename = "projectIDList")]
    pub project_id_list: Vec<String>,
    #[serde(rename = "projectNameList")]
    pub project_name_list: Vec<String>,
}

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
