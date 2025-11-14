use clap::{Parser, Subcommand};
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
            let end_num = parts[parts.len() - 1]
                .parse::<usize>()
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
            let start_num = parts[0]
                .parse::<usize>()
                .map_err(|_| "Invalid start number".to_string())?;

            return Ok(DepthRange {
                start: Endpoint::Inclusive(start_num),
                end: Endpoint::Inclusive(usize::MAX),
            });
        }

        // n..m または n..=m の形式
        if parts.len() == 2 {
            let start_num = parts[0]
                .parse::<usize>()
                .map_err(|_| "Invalid start number".to_string())?;
            let end_num = parts[1]
                .parse::<usize>()
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

impl DepthRange {
    /// 指定された深さが範囲内かどうかを判定する
    pub fn contains(&self, depth: usize) -> bool {
        let start_ok = match self.start {
            Endpoint::Inclusive(start) => depth >= start,
            Endpoint::Exclusive(start) => depth > start,
        };
        let end_ok = match self.end {
            Endpoint::Inclusive(end) => depth <= end,
            Endpoint::Exclusive(end) => depth < end,
        };
        start_ok && end_ok
    }
}

/// 表示設定を管理する構造体
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    /// ノードの範囲情報を表示するかどうか
    pub show_range: bool,
    /// すべてのノードのテキストを表示するかどうか
    pub show_all_text: bool,
    /// 非トークンノードのテキストを表示するかどうか
    pub show_node_text: bool,
    /// トークンのテキストを表示するかどうか
    pub show_token_text: bool,
    /// ノードの種類（NodeまたはToken）を表示するかどうか
    pub show_node_type: bool,
    /// SQL文の間に空行を表示するかどうか
    pub show_sql_separator: bool,
    /// 各ツリーの前にSQL文を表示するかどうか
    pub show_sql: bool,
}

impl DisplayConfig {
    /// 指定されたノードのテキストを表示するかどうかを判定する
    pub fn should_show_text(&self, is_token: bool) -> bool {
        if is_token {
            self.show_token_text
        } else {
            self.show_all_text || self.show_node_text
        }
    }
}

impl From<&Commands> for DisplayConfig {
    fn from(cmd: &Commands) -> Self {
        match cmd {
            Commands::Tree {
                hide_range,
                show_text,
                show_node_text,
                hide_token_text,
                show_node_type,
                show_sql_separator,
                show_sql,
                ..
            } => DisplayConfig {
                show_range: !hide_range,
                show_all_text: *show_text,
                show_node_text: *show_node_text,
                show_token_text: !hide_token_text,
                show_node_type: *show_node_type,
                show_sql_separator: *show_sql_separator,
                show_sql: *show_sql,
            },
            Commands::Tokens { .. } => unreachable!(),
        }
    }
}

/// SQLのCST（具象構文木）を表示するツール
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// 解析するSQLファイルのパス（未指定、または「-」の場合は標準入力を使用）
    #[arg(value_name = "FILE")]
    pub sql_file: Option<PathBuf>,

    /// エラー回復機能を有効にする（エラーがあっても可能な限りパースを続行）
    #[arg(short = 'e', long, default_value = "false")]
    pub error_recovery: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// CST（具象構文木）を表示
    Tree {
        /// 表示する木の深さ範囲（例: 3, 1..3, 1..=3, ..3, ..=3, 3..）
        #[arg(short, long, value_name = "DEPTH")]
        depth: Option<DepthRange>,

        /// ノードの範囲情報を表示しない
        #[arg(long, default_value = "false")]
        hide_range: bool,

        /// すべてのノードのテキストを表示する
        #[arg(long, default_value = "false")]
        show_text: bool,

        /// 非トークンノードのテキストを表示する
        #[arg(long, default_value = "false")]
        show_node_text: bool,

        /// トークンのテキストを表示しない
        #[arg(long, default_value = "false")]
        hide_token_text: bool,

        /// ノードの種類（NodeまたはToken）を表示する
        #[arg(long, default_value = "false")]
        show_node_type: bool,

        /// SQL文の間に空行を表示する
        #[arg(long, default_value = "false")]
        show_sql_separator: bool,

        /// 各ツリーの前にSQL文を表示する
        #[arg(long, default_value = "false")]
        show_sql: bool,
    },

    /// トークン列を表示
    Tokens {
        /// トークンの範囲情報を表示しない
        #[arg(long, default_value = "false")]
        hide_range: bool,

        /// トークンのテキストを表示しない
        #[arg(long, default_value = "false")]
        hide_text: bool,
    },
}

#[cfg(test)]
mod tests {
    use crate::cli::{Cli, DepthRange, Endpoint};

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }

    mod depth_range_parse {
        use super::*;

        #[test]
        fn test_single_number() {
            let range: DepthRange = "3".parse().unwrap();
            assert!(matches!(range.start, Endpoint::Inclusive(3)));
            assert!(matches!(range.end, Endpoint::Inclusive(3)));
        }

        #[test]
        fn test_exclusive_range() {
            let range: DepthRange = "1..3".parse().unwrap();
            assert!(matches!(range.start, Endpoint::Inclusive(1)));
            assert!(matches!(range.end, Endpoint::Exclusive(3)));
        }

        #[test]
        fn test_inclusive_range() {
            let range: DepthRange = "1..=3".parse().unwrap();
            assert!(matches!(range.start, Endpoint::Inclusive(1)));
            assert!(matches!(range.end, Endpoint::Inclusive(3)));
        }

        #[test]
        fn test_from_zero() {
            let range: DepthRange = "..3".parse().unwrap();
            assert!(matches!(range.start, Endpoint::Inclusive(0)));
            assert!(matches!(range.end, Endpoint::Exclusive(3)));
        }

        #[test]
        fn test_from_zero_inclusive() {
            let range: DepthRange = "..=3".parse().unwrap();
            assert!(matches!(range.start, Endpoint::Inclusive(0)));
            assert!(matches!(range.end, Endpoint::Inclusive(3)));
        }

        #[test]
        fn test_to_max() {
            let range: DepthRange = "3..".parse().unwrap();
            assert!(matches!(range.start, Endpoint::Inclusive(3)));
            assert!(matches!(range.end, Endpoint::Inclusive(usize::MAX)));
        }

        #[test]
        fn test_invalid_range() {
            assert!("3...5".parse::<DepthRange>().is_err());
            assert!("a".parse::<DepthRange>().is_err());
            assert!("1,2".parse::<DepthRange>().is_err());
        }

        #[test]
        fn test_invalid_start_greater_than_end() {
            assert!("5..3".parse::<DepthRange>().is_err());
            assert!("5..=3".parse::<DepthRange>().is_err());
        }
    }

    mod depth_range_contains {
        use super::*;

        #[test]
        fn test_single_number() {
            let range: DepthRange = "3".parse().unwrap();
            assert!(!range.contains(2));
            assert!(range.contains(3));
            assert!(!range.contains(4));
        }

        #[test]
        fn test_exclusive_range() {
            let range: DepthRange = "1..3".parse().unwrap();
            assert!(!range.contains(0));
            assert!(range.contains(1));
            assert!(range.contains(2));
            assert!(!range.contains(3));
        }

        #[test]
        fn test_inclusive_range() {
            let range: DepthRange = "1..=3".parse().unwrap();
            assert!(!range.contains(0));
            assert!(range.contains(1));
            assert!(range.contains(2));
            assert!(range.contains(3));
            assert!(!range.contains(4));
        }

        #[test]
        fn test_from_zero() {
            let range: DepthRange = "..3".parse().unwrap();
            assert!(range.contains(0));
            assert!(range.contains(1));
            assert!(range.contains(2));
            assert!(!range.contains(3));
        }

        #[test]
        fn test_to_max() {
            let range: DepthRange = "3..".parse().unwrap();
            assert!(!range.contains(2));
            assert!(range.contains(3));
            assert!(range.contains(100));
            assert!(range.contains(usize::MAX));
        }
    }
}
