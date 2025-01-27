mod cli;

use clap::Parser;
use cli::{Cli, DepthRange, DisplayConfig};
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
        // インデント
        if depth > 0 {
            write!(output, "{}-+", "-".repeat((depth - 1) * INDENT_SIZE))?;
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

fn main() {
    let cli = Cli::parse();
    let config = DisplayConfig::from(&cli);

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

    // ルートノードから木を表示
    let root_node = tree.root_node();
    print_tree(root_node, 0, &cli.depth, &config);
}
