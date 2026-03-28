use calamine::{open_workbook, Reader, Xlsx};
use rust_xlsxwriter::{Format, Workbook};

pub fn save_to_excel(path: &str, encrypted: &str, decrypted: &str) -> Result<(), String> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();

    let header_fmt = Format::new().set_bold().set_font_size(20.0);
    let cell_fmt = Format::new().set_font_size(15.0);

    ws.set_column_width(0, 40).map_err(|e| e.to_string())?;
    ws.set_column_width(1, 40).map_err(|e| e.to_string())?;

    ws.write_string_with_format(0, 0, "加密结果", &header_fmt)
        .map_err(|e| e.to_string())?;
    ws.write_string_with_format(0, 1, "解密结果", &header_fmt)
        .map_err(|e| e.to_string())?;

    for (i, line) in encrypted.lines().enumerate() {
        ws.write_string_with_format((i + 1) as u32, 0, line, &cell_fmt)
            .map_err(|e| e.to_string())?;
    }

    for (i, line) in decrypted.lines().enumerate() {
        ws.write_string_with_format((i + 1) as u32, 1, line, &cell_fmt)
            .map_err(|e| e.to_string())?;
    }

    wb.save(path).map_err(|e| e.to_string())
}

pub fn load_first_column(path: &str) -> Result<String, String> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| format!("打开Excel失败: {e}"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or("Excel文件没有工作表")?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("读取工作表失败: {e}"))?;

    let mut values = Vec::new();
    for row in range.rows() {
        if let Some(cell) = row.first() {
            let s = cell.to_string();
            if !s.is_empty() {
                values.push(s);
            }
        }
    }

    Ok(values.join("\n"))
}
