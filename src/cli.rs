use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;

/// 深さの境界点を表す列挙型
#[derive(Debug, Clone)]
pub enum Endpoint {
    Inclusive(usize),
    Exclusive(usize),
}

/// 深さの範囲を表す構造体
#[derive(Debug, Clone)]
pub struct DepthRange {
    pub start: Endpoint,
    pub end: Endpoint,
}

impl FromStr for DepthRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 単一の数値 `n` の場合
        if let Ok(single) = s.parse::<usize>() {
            return Ok(DepthRange {
                start: Endpoint::Inclusive(single),
                end: Endpoint::Inclusive(single),
            });
        }

        // 範囲指定の場合
        let (parts, end_inclusive) = if s.contains("..=") {
            (s.split("..=").collect::<Vec<_>>(), true)
        } else if s.contains("..") {
            (s.split("..").collect::<Vec<_>>(), false)
        } else {
            return Err("Invalid range format".to_string());
        };

        // ..n または ..=n の形式
        if s.starts_with("..") {
            let end_num = parts[parts.len() - 1].parse::<usize>()
                .map_err(|_| "Invalid end number".to_string())?;
            
            return Ok(DepthRange {
                start: Endpoint::Inclusive(0),
                end: if end_inclusive {
                    Endpoint::Inclusive(end_num)
                } else {
                    Endpoint::Exclusive(end_num)
                },
            });
        }

        // n.. の形式（終端なし）
        if s.ends_with("..") {
            let start_num = parts[0].parse::<usize>()
                .map_err(|_| "Invalid start number".to_string())?;
            
            return Ok(DepthRange {
                start: Endpoint::Inclusive(start_num),
                end: Endpoint::Inclusive(usize::MAX),
            });
        }

        // n..m または n..=m の形式
        if parts.len() == 2 {
            let start_num = parts[0].parse::<usize>()
                .map_err(|_| "Invalid start number".to_string())?;
            let end_num = parts[1].parse::<usize>()
                .map_err(|_| "Invalid end number".to_string())?;

            if start_num > end_num {
                return Err("Start must be less than or equal to end".to_string());
            }

            return Ok(DepthRange {
                start: Endpoint::Inclusive(start_num),
                end: if end_inclusive {
                    Endpoint::Inclusive(end_num)
                } else {
                    Endpoint::Exclusive(end_num)
                },
            });
        }

        Err("Invalid range format".to_string())
    }
}

/// SQLのCST（具象構文木）を表示するツール
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 解析するSQLファイルのパス
    #[arg(value_name = "FILE")]
    pub sql_file: PathBuf,

    /// 表示する木の深さ範囲（例: 3, 1..3, 1..=3, ..3, ..=3, 3..）
    #[arg(short, long, value_name = "DEPTH")]
    pub depth: Option<DepthRange>,
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
