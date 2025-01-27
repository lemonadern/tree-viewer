mod cli;

use cli::{Cli, DepthRange, Endpoint};
use clap::Parser;
use std::fs;
use std::process;
use postgresql_cst_parser::tree_sitter::{parse, Tree, TreeCursor, Node};

const INDENT_SIZE: usize = 2;

fn should_print(depth: usize, range: &Option<DepthRange>) -> bool {
    match range {
        None => true,
        Some(range) => range.contains(depth)
    }
}

fn print_tree(node: Node, depth: usize, range: &Option<DepthRange>) {
    let should_display = should_print(depth, range);

    if should_display {
        let indent = if depth == 0 {
            String::new()
        } else {
            format!("{}-+", "-".repeat((depth - 1) * INDENT_SIZE))
        };
        
        if node.child_count() == 0 {
            let text = node.text().escape_debug();
            println!("{}{} {} \"{}\"", 
                indent, node.kind(), node.range(), text);
        } else {
            println!("{}{} {}", 
                indent, node.kind(), node.range());
        }
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            print_tree(cursor.node(), depth + 1, range);
            if !cursor.goto_next_sibling() {
                break;
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

    // ルートノードから木を表示
    let root_node = tree.root_node();
    print_tree(root_node, 0, &cli.depth);
}
