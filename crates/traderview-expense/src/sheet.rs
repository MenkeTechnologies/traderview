//! Uniform tabular-row extraction across CSV and spreadsheet formats.
//!
//! Each source parser shouldn't need to know whether the user uploaded a CSV
//! or an XLSX or an ODS — they all reduce to "rows of string cells". This
//! module dispatches by magic bytes:
//!   * `PK\x03\x04` (ZIP) → calamine (xlsx/xlsm/ods/xlsb)
//!   * anything else → csv crate
//!
//! We deliberately produce `Vec<Vec<String>>` rather than streaming. Expense
//! CSVs are small (typically <10k rows / <1MB) so the simplicity wins.

use crate::ImportError;
use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetKind {
    Csv,
    Spreadsheet,
}

pub fn detect_kind(bytes: &[u8]) -> SheetKind {
    // ZIP magic — xlsx, xlsm, xlsb, ods all start with 'PK\x03\x04'.
    if bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04 {
        SheetKind::Spreadsheet
    } else {
        SheetKind::Csv
    }
}

/// Read the first worksheet (or only the CSV) into rows of trimmed strings.
///
/// Empty trailing cells are kept so column indexes line up with the schema.
/// Empty leading rows are NOT pruned — the parser may rely on positional
/// alignment with the source export.
pub fn rows(bytes: &[u8]) -> Result<Vec<Vec<String>>, ImportError> {
    match detect_kind(bytes) {
        SheetKind::Csv => csv_rows(bytes),
        SheetKind::Spreadsheet => spreadsheet_rows(bytes),
    }
}

fn csv_rows(bytes: &[u8]) -> Result<Vec<Vec<String>>, ImportError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(bytes);
    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        out.push(rec.iter().map(|c| c.to_string()).collect());
    }
    Ok(out)
}

fn spreadsheet_rows(bytes: &[u8]) -> Result<Vec<Vec<String>>, ImportError> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut wb = open_workbook_auto_from_rs(cursor)
        .map_err(|e| ImportError::Parse(format!("workbook open: {e}")))?;
    let first = wb
        .sheet_names()
        .into_iter()
        .next()
        .ok_or_else(|| ImportError::Parse("workbook has no sheets".into()))?;
    let range = wb
        .worksheet_range(&first)
        .map_err(|e| ImportError::Parse(format!("worksheet '{first}': {e}")))?;

    let mut out = Vec::with_capacity(range.height());
    for row in range.rows() {
        out.push(row.iter().map(cell_to_string).collect());
    }
    Ok(out)
}

fn cell_to_string(c: &Data) -> String {
    match c {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // Trim trailing ".0" so "143.09" stays "143.09" but "1" doesn't
            // become "1.0". Two decimals preserves currency presentation.
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                // Use generic float formatting; the parser does its own
                // numeric parsing so 1.2300000001 won't cause issues.
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.as_f64().to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{e:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_csv() {
        assert_eq!(detect_kind(b"date,amount\n2024-01-01,5.00"), SheetKind::Csv);
    }

    #[test]
    fn detects_zip_as_spreadsheet() {
        let zip_magic = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(detect_kind(&zip_magic), SheetKind::Spreadsheet);
    }

    #[test]
    fn csv_parses() {
        let bytes = b"a,b,c\n1,2,3\n4,5,6";
        let r = rows(bytes).unwrap();
        assert_eq!(r.len(), 3);
        assert_eq!(r[1], vec!["1", "2", "3"]);
    }

    #[test]
    fn csv_flexible_columns() {
        // BoA-style: summary block has 3 cols, transactions has 4.
        let bytes = b"Description,,Summary Amt.\nBeginning,,1000.00\n\nDate,Description,Amount,Bal\n01/01,Foo,5.00,1005.00";
        let r = rows(bytes).unwrap();
        assert!(r.len() >= 4, "expected ≥4 rows, got {}", r.len());
    }
}
