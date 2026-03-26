mod db;

use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use std::{fs, process};

#[derive(Parser)]
#[command(
    name = "zr",
    about = "Relocate directories while preserving zoxide scores"
)]
struct Cli {
    /// Source directory to move
    source: Option<String>,

    /// Destination path
    destination: Option<String>,

    /// Preview changes without executing
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Show each zoxide entry being updated
    #[arg(short, long)]
    verbose: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL")]
    completions: Option<Shell>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        clap_complete::generate(shell, &mut Cli::command(), "zr", &mut std::io::stdout());
        return;
    }

    let source = match &cli.source {
        Some(s) => s,
        None => {
            Cli::command().print_help().ok();
            process::exit(1);
        }
    };
    let destination = match &cli.destination {
        Some(d) => d,
        None => {
            eprintln!("error: missing <DESTINATION> argument");
            process::exit(1);
        }
    };

    if let Err(e) = run(source, destination, cli.dry_run, cli.verbose) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(source: &str, destination: &str, dry_run: bool, verbose: bool) -> Result<(), String> {
    let src = std::path::absolute(source).map_err(|e| format!("resolving source: {e}"))?;
    let dst =
        std::path::absolute(destination).map_err(|e| format!("resolving destination: {e}"))?;

    let meta = fs::metadata(&src).map_err(|e| format!("source {}: {e}", src.display()))?;
    if !meta.is_dir() {
        return Err(format!("{} is not a directory", src.display()));
    }

    if dst.exists() {
        return Err(format!("destination {} already exists", dst.display()));
    }

    let dst_parent = dst.parent().ok_or("destination has no parent")?;
    if !dst_parent.exists() {
        return Err(format!(
            "destination parent {} does not exist",
            dst_parent.display()
        ));
    }

    let src_str = src.to_string_lossy();
    let dst_str = dst.to_string_lossy();

    let mut database = db::Database::open().map_err(|e| format!("reading zoxide database: {e}"))?;

    if dry_run {
        println!("dry run — no changes will be made\n");
        println!("move: {} → {}\n", src_str, dst_str);

        let matched = database.matching_paths(&src_str);
        if matched.is_empty() {
            println!("no zoxide entries to update");
        } else {
            println!("zoxide entries to update ({}):", matched.len());
            for d in &matched {
                let new_path = db::rewrite_path(&d.path, &src_str, &dst_str);
                println!("  rank:{:.1}  {} → {}", d.rank, d.path, new_path);
            }
        }
        return Ok(());
    }

    fs::rename(&src, &dst).map_err(|e| format!("moving directory: {e}"))?;

    let relocated = database.relocate_paths(&src_str, &dst_str);

    if !relocated.is_empty() {
        if let Err(e) = database.save() {
            eprintln!("warning: directory moved but zoxide db update failed: {e}");
            eprintln!("you may need to manually update zoxide entries");
            return Err(e.to_string());
        }
    }

    println!("moved {} → {}", src_str, dst_str);
    if verbose {
        for r in &relocated {
            println!(
                "  zoxide: {} → {} (rank: {:.1})",
                r.old_path, r.new_path, r.rank
            );
        }
    }
    if !relocated.is_empty() {
        println!("updated {} zoxide entries", relocated.len());
    }
    Ok(())
}
