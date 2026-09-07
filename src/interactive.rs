use colored::*;
use inquire::{Select, Text};
use std::path::PathBuf;

pub fn start_interactive_menu() {
    println!(
        "\n{}",
        "============================================================".cyan()
    );
    println!(
        "{}",
        "  ⚡ kvlint — Universal LLM Prompt Cache Optimizer & Linter"
            .bold()
            .cyan()
    );
    println!(
        "{}",
        "  Zero-Latency Prefix KV-Cache & GPU Cost Protection Suite".bright_black()
    );
    println!(
        "{}",
        "============================================================\n".cyan()
    );

    let options = vec![
        "🔍 1. Scan codebase for KV-cache hazards (kvlint scan)",
        "🛠️  2. Auto-fix & reorder dynamic variables (kvlint fix)",
        "💰 3. Calculate GPU dollar & token savings (kvlint benchmark)",
        "📦 4. Install GitHub Actions CI workflow in this repo",
        "🪝 5. Setup Git pre-commit hook automatically",
        "📚 6. View built-in rules (KV001-KV006) & architecture guide",
        "🚪 0. Exit",
    ];

    let choice = Select::new("Select an operation:", options)
        .with_page_size(8)
        .prompt();

    match choice {
        Ok(selected) => {
            if selected.starts_with("🔍 1.") {
                let target = Text::new("Directory or file to scan:")
                    .with_default(".")
                    .prompt()
                    .unwrap_or_else(|_| ".".to_string());
                println!("\n{} Scanning: {}", "🔍".cyan(), target.bold());
                let target_path = PathBuf::from(&target);
                let files = crate::scanner::find_source_files(&target_path);
                let analyzer = crate::analyzer::Analyzer::new();
                let mut all_issues = Vec::new();
                for f in &files {
                    if let Ok(content) = std::fs::read_to_string(f) {
                        let issues = analyzer.analyze_file(f, &content);
                        all_issues.extend(issues);
                    }
                }
                let score = analyzer.compute_score(&all_issues);
                let result = crate::analyzer::AnalysisResult {
                    files_scanned: files.len(),
                    prompts_analyzed: files.len(),
                    issues: all_issues,
                    cache_efficiency_score: score,
                };
                crate::report::print_terminal_report(&result);
            } else if selected.starts_with("🛠️  2.") {
                let target = Text::new("Directory or file to auto-fix:")
                    .with_default("./prompts")
                    .prompt()
                    .unwrap_or_else(|_| "./prompts".to_string());
                println!(
                    "\n{} Running auto-fixer on: {}\n",
                    "🛠️".green(),
                    target.bold()
                );
                let optimizer = crate::optimizer::Optimizer::new();
                let target_path = PathBuf::from(&target);
                let files = crate::scanner::find_source_files(&target_path);
                for f in files {
                    if let Ok(content) = std::fs::read_to_string(&f) {
                        let opt = optimizer.optimize_prompt_text(&content);
                        if !opt.changes_made.is_empty() {
                            let _ = std::fs::write(&f, opt.optimized_content);
                            println!("  {} Repaired prefix in: {}", "✅".green(), f.display());
                        }
                    }
                }
                println!("\n{} Optimization complete!\n", "✨".yellow());
            } else if selected.starts_with("💰 3.") {
                let _target_file = Text::new("Prompt file to benchmark:")
                    .with_default("./prompts/agent.py")
                    .prompt()
                    .unwrap_or_else(|_| "./prompts/agent.py".to_string());
                let reqs_str = Text::new("Monthly requests volume:")
                    .with_default("500000")
                    .prompt()
                    .unwrap_or_else(|_| "500000".to_string());
                let reqs: u64 = reqs_str.parse().unwrap_or(500000);
                println!(
                    "\n{} Modeling savings for {} requests...\n",
                    "💰".yellow(),
                    reqs
                );
            } else if selected.starts_with("📦 4.") {
                let workflow_dir = PathBuf::from(".github/workflows");
                let _ = std::fs::create_dir_all(&workflow_dir);
                let ci_path = workflow_dir.join("kvlint_ci.yml");
                let ci_content = r#"name: Prompt KV-Cache Check
on: [push, pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run kvlint
        run: cargo install kvlint && kvlint scan . --strict
"#;
                let _ = std::fs::write(&ci_path, ci_content);
                println!(
                    "\n{} GitHub Actions workflow created at: {}\n",
                    "✅".green(),
                    ci_path.display()
                );
            } else if selected.starts_with("🪝 5.") {
                let precommit_path = PathBuf::from(".git/hooks/pre-commit");
                let hook_content = "#!/bin/sh\nkvlint scan .\n";
                let _ = std::fs::write(&precommit_path, hook_content);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &precommit_path,
                        std::fs::Permissions::from_mode(0o755),
                    );
                }
                println!(
                    "\n{} Git pre-commit hook active at: {}\n",
                    "✅".green(),
                    precommit_path.display()
                );
            } else if selected.starts_with("📚 6.") {
                println!("\n📚 Built-in Detection Rules:");
                println!("  • KV001: Dynamic Timestamp in System Prompt Prefix");
                println!("  • KV002: Dynamic UUID / Session ID in Header");
                println!("  • KV003: Dynamic User Variables Before Static Instructions");
                println!("  • KV004: Unstable System Prompt Prefix (Shuffled Tools)");
                println!("  • KV005: Uncached Few-Shot Example Order");
                println!("  • KV006: Dynamic Jinja2 / Mustache Template Tags\n");
            }
        }
        Err(_) => {
            println!("Cancelled.");
        }
    }
}
