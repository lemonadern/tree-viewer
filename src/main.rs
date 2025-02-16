mod cli;

use clap::Parser;
use cli::{Commands, Cli, DepthRange, DisplayConfig, Endpoint};
use postgresql_cst_parser::tree_sitter::{parse, Node};
use std::fs;
use std::process;
use std::fmt::Write;

const INDENT_SIZE: usize = 2;

fn should_print(depth: usize, range: &Option<DepthRange>) -> bool {
    match range {
        None => true,
        Some(range) => range.contains(depth),
    }
}

fn print_tree(
    node: Node,
    depth: usize,
    range: &Option<DepthRange>,
    config: &DisplayConfig,
) {
    let mut output = String::new();
    write_tree(node, depth, range, config, &mut output).expect("writing to string should not fail");
    print!("{}", output);
}

fn write_tree(
    node: Node,
    depth: usize,
    range: &Option<DepthRange>,
    config: &DisplayConfig,
    output: &mut String,
) -> std::fmt::Result {
    let should_display = should_print(depth, range);

    if should_display {
        // インデントの基準となる深さを取得
        let base_depth = match range {
            Some(range) => match range.start {
                Endpoint::Inclusive(start) => start,
                Endpoint::Exclusive(start) => start + 1,
            },
            None => 0,
        };

        // インデント
        if depth > 0 {
            // 基準深さからの相対的なインデントを計算
            let relative_depth = if depth > base_depth {
                depth - base_depth
            } else {
                0
            };
            if relative_depth > 0 {
                write!(output, "{}-+", "-".repeat((relative_depth - 1) * INDENT_SIZE))?;
            }
        }

        // ノードの種類
        let is_token = node.child_count() == 0;
        write!(output, "{}", node.kind())?;
        if config.show_node_type {
            write!(output, " ({})", if is_token { "Token" } else { "Node" })?;
        }

        // 範囲情報
        if config.show_range {
            write!(output, " {}", node.range())?;
        }

        // テキスト
        if config.should_show_text(is_token) {
            write!(output, " \"{}\"", node.text().escape_debug())?;
        }

        writeln!(output)?;
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            write_tree(cursor.node(), depth + 1, range, config, output)?;
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    Ok(())
}

fn print_tokens(node: Node, hide_range: bool, show_text: bool) {
    let mut cursor = node.walk();
    
    // 深さ優先探索でトークンを列挙
    loop {
        let current_node = cursor.node();
        // トークンの場合は出力
        if current_node.child_count() == 0 {
            if !hide_range {
                print!("{}@{}", current_node.kind(), current_node.range());
            } else {
                print!("{}", current_node.kind());
            }
            if show_text {
                println!(" \"{}\"", current_node.text().escape_debug());
            } else {
                println!();
            }
        }

        // 子ノードがあれば進む
        if cursor.goto_first_child() {
            continue;
        }

        // 次の兄弟ノードがあれば進む
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                // ルートまで戻ってきたら終了
                return;
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // SQLファイルの読み込み
    let sql = match fs::read_to_string(&cli.sql_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("ファイルの読み込みに失敗しました: {}", err);
            process::exit(1);
        }
    };

    // SQLのパース
    let tree = match parse(&sql) {
        Ok(tree) => tree,
        Err(err) => {
            eprintln!("SQLのパースに失敗しました: {:?}", err);
            process::exit(1);
        }
    };

    let root_node = tree.root_node();

    match cli.command.unwrap_or(Commands::Tree {
        depth: None,
        hide_range: false,
        show_text: false,
        show_node_text: false,
        hide_token_text: false,
        show_node_type: false,
    }) {
        Commands::Tree {
            depth,
            hide_range,
            show_text,
            show_node_text,
            hide_token_text,
            show_node_type,
        } => {
            let command = Commands::Tree {
                depth: depth.clone(),
                hide_range,
                show_text,
                show_node_text,
                hide_token_text,
                show_node_type,
            };
            let config = DisplayConfig::from(&command);
            print_tree(root_node, 0, &depth, &config);
        }
        Commands::Tokens { hide_range, hide_text } => {
            print_tokens(root_node, hide_range, !hide_text);
        }
    }
}
