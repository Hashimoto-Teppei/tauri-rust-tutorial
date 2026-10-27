//! 第4章: エラー処理・コレクション・イテレータ

use std::collections::HashMap;
use std::num::ParseIntError;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ConfigError {
    #[error("'=' がありません: {0}")]
    MissingEquals(String),
    #[error("キーが空です")]
    EmptyKey,
}

/// "key=value" をパースする。前後の空白は除去
pub fn parse_kv(line: &str) -> Result<(String, String), ConfigError> {
    let (k, v) = line
        .split_once('=')
        .ok_or_else(|| ConfigError::MissingEquals(line.to_string()))?;
    let k = k.trim();
    if k.is_empty() {
        return Err(ConfigError::EmptyKey);
    }
    Ok((k.to_string(), v.trim().to_string()))
}

/// 複数行の設定をパースして HashMap にする。空行と '#' で始まる行は無視。
/// 1行でもエラーなら最初のエラーを返す
pub fn parse_config(text: &str) -> Result<HashMap<String, String>, ConfigError> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(parse_kv)
        .collect()
}

/// 数値文字列（前後の空白は無視）をすべてパースして合計。1つでも失敗したらそのエラー
pub fn sum_strings(inputs: &[&str]) -> Result<i64, ParseIntError> {
    let mut total = 0;
    for s in inputs {
        total += s.trim().parse::<i64>()?;
    }
    Ok(total)
}

/// 単語の出現回数（小文字化して数える）
pub fn word_count(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for w in text.split_whitespace() {
        *counts.entry(w.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// n 文字以上の単語を、重複なし・出現順で返す
pub fn long_words(text: &str, n: usize) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    text.split_whitespace()
        .filter(|w| w.chars().count() >= n)
        .filter(|w| seen.insert(w.to_string()))
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kv() {
        assert_eq!(parse_kv(" name = tauri "), Ok(("name".into(), "tauri".into())));
        assert_eq!(parse_kv("novalue"), Err(ConfigError::MissingEquals("novalue".into())));
        assert_eq!(parse_kv(" =x"), Err(ConfigError::EmptyKey));
        assert_eq!(ConfigError::EmptyKey.to_string(), "キーが空です");
    }

    #[test]
    fn test_parse_config() {
        let cfg = parse_config("# comment\nwidth=800\n\nheight = 600\n").unwrap();
        assert_eq!(cfg.len(), 2);
        assert_eq!(cfg["height"], "600");
        assert!(parse_config("a=1\nbroken").is_err());
    }

    #[test]
    fn test_sum_strings() {
        assert_eq!(sum_strings(&["1", " 2 ", "-3"]), Ok(0));
        assert!(sum_strings(&["1", "x"]).is_err());
    }

    #[test]
    fn test_word_count() {
        let c = word_count("Rust rust TAURI");
        assert_eq!(c["rust"], 2);
        assert_eq!(c["tauri"], 1);
        assert_eq!(c.get("electron"), None);
    }

    #[test]
    fn test_long_words() {
        assert_eq!(long_words("tauri is a fast tauri app framework", 4), vec!["tauri", "fast", "framework"]);
    }
}
