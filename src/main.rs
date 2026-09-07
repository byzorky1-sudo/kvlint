use clap::{Parser, Subcommand};
use kvlint::{
    analyzer::{AnalysisResult, Analyzer},
    interactive::start_interactive_menu,
    optimizer::Optimizer,
    report::print_terminal_report,
    scanner::find_source_files,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "kvlint",
    about = "⚡ Blazing-fast static analyzer & linter for LLM prefix KV-caching optimization.",
    version = "0.2.0",
    author = "byzorky1-sudo (Ali)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Target directory or file to scan directly
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Fail with exit code 1 if any Critical issues are detected (CI/CD mode)
    #[arg(long, default_value_t = false)]
    strict: bool,

    /// Automatically repair prompts by re-ordering dynamic variables to the end
    #[arg(long, default_value_t = false)]
    fix: bool,

    /// Preview fixes without modifying files on disk
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Output results as clean JSON
    #[arg(long, default_value_t = false)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan files for prefix KV-cache invalidation hazards
    Scan {
        /// Target path (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output results as JSON
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Strict mode for CI/CD
        #[arg(long, default_value_t = false)]
        strict: bool,
    },
    /// Automatically fix prompt files to restore static prefix alignment
    Fix {
        /// Target path
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Preview changes without modifying files on disk
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Model GPU cost & latency savings for your monthly traffic volume
    Benchmark {
        /// Target prompt file
        path: PathBuf,

        /// Estimated monthly requests / turn volume
        #[arg(long, default_value_t = 500000)]
        requests: u64,

        /// Input token price per 1M tokens in USD
        #[arg(long, default_value_t = 3.0)]
        price_per_m: f64,
    },
    /// Explain all built-in linting rules
    Rules,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan { path, json, strict }) => {
            run_scan(&path, json, strict);
        }
        Some(Commands::Fix { path, dry_run }) => {
            run_fix(&path, dry_run);
        }
        Some(Commands::Benchmark {
            path,
            requests,
            price_per_m,
        }) => {
            run_benchmark(&path, requests, price_per_m);
        }
        Some(Commands::Rules) => {
            print_rules_explanation();
        }
        None => {
            if let Some(path) = cli.path {
                if cli.fix {
                    run_fix(&path, cli.dry_run);
                } else {
                    run_scan(&path, cli.json, cli.strict);
                }
            } else {
                // Launch beautiful interactive CLI Menu when run with bare `kvlint`
                start_interactive_menu();
            }
        }
    }
}

fn run_scan(target_path: &Path, json_output: bool, strict_mode: bool) {
    if !target_path.exists() {
        eprintln!("❌ Target path does not exist: {}", target_path.display());
        std::process::exit(1);
    }

    let files = find_source_files(target_path);
    let analyzer = Analyzer::new();

    let mut all_issues = Vec::new();

    for file in &files {
        if let Ok(content) = fs::read_to_string(file) {
            let issues = analyzer.analyze_file(file, &content);
            all_issues.extend(issues);
        }
    }

    let score = analyzer.compute_score(&all_issues);

    let result = AnalysisResult {
        files_scanned: files.len(),
        prompts_analyzed: files.len(),
        issues: all_issues,
        cache_efficiency_score: score,
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        print_terminal_report(&result);
    }

    if strict_mode
        && result
            .issues
            .iter()
            .any(|i| i.severity == kvlint::analyzer::IssueSeverity::Critical)
    {
        std::process::exit(1);
    }
}

fn run_fix(target_path: &Path, dry_run: bool) {
    let files = find_source_files(target_path);
    let optimizer = Optimizer::new();
    let mut total_repaired = 0;

    println!("\n🛠️ kvlint --fix — Automated Prefix Cache Re-Architecting:\n");

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            let res = optimizer.optimize_prompt_text(&content);
            if !res.changes_made.is_empty() {
                total_repaired += 1;
                if dry_run {
                    println!("🔍 [DRY-RUN] Would optimize: {}", file.display());
                } else {
                    fs::write(&file, res.optimized_content).expect("Failed to write fixed file");
                    println!("✅ Successfully repaired: {}", file.display());
                }
                for change in res.changes_made {
                    println!("   • {}", change);
                }
                println!();
            }
        }
    }

    if total_repaired == 0 {
        println!("✨ All prompts already maintain optimal static prefix alignment! Zero changes required.\n");
    } else {
        println!(
            "🎉 Successfully optimized {} file(s) for 100% prefix cache reuse!\n",
            total_repaired
        );
    }
}

fn run_benchmark(path: &PathBuf, requests: u64, price_per_m: f64) {
    if !path.exists() {
        eprintln!("❌ Error: File not found at {}", path.display());
        std::process::exit(1);
    }

    let content = fs::read_to_string(path).expect("Failed to read file");
    let prompt_tokens = content.len().div_ceil(4); // Estimate 4 chars per token

    let tokens_per_month_uncached = (prompt_tokens as u64) * requests;
    let tokens_per_month_cached = (prompt_tokens as u64 / 10) * requests; // 90% savings on cache hits

    let cost_uncached = (tokens_per_month_uncached as f64 / 1_000_000.0) * price_per_m;
    let cost_cached = (tokens_per_month_cached as f64 / 1_000_000.0) * price_per_m;
    let net_savings = cost_uncached - cost_cached;

    println!("\n💰 kvlint Benchmark — GPU Token Cost & Latency Model:\n");
    println!("📊 Target File:        {}", path.display());
    println!("📏 Prompt Length:      ~{} input tokens", prompt_tokens);
    println!("📈 Monthly Requests:   {}", requests);
    println!(
        "🔥 Tokens (0% Cache):  {:.1}M tokens/mo -> ${:.2}/mo",
        tokens_per_month_uncached as f64 / 1_000_000.0,
        cost_uncached
    );
    println!(
        "⚡ Tokens (90% Cache): {:.1}M tokens/mo -> ${:.2}/mo",
        tokens_per_month_cached as f64 / 1_000_000.0,
        cost_cached
    );
    println!(
        "🎯 NET MONTHLY SAVING: ${:.2} / month (~{:.1}% cost cut)\n",
        net_savings,
        (net_savings / cost_uncached) * 100.0
    );
}

fn print_rules_explanation() {
    println!("\n📚 kvlint Built-in Linting Rules & GPU Cache Mechanics:\n");
    println!("1. KV001_DYNAMIC_TIMESTAMP_PREFIX (Critical)");
    println!("   Detects datetime.now(), new Date(), $(date) at prompt start.");
    println!("   Fix: Move timestamp to end of prompt or dynamic user turn.\n");

    println!("2. KV002_DYNAMIC_UUID_IN_HEADER (Critical)");
    println!("   Detects random session UUIDs generated before static instructions.");
    println!("   Fix: Keep session UUID in API request metadata or prompt footer.\n");

    println!("3. KV003_DYNAMIC_VARIABLE_BEFORE_STATIC_INSTRUCTIONS (High)");
    println!("   Detects user variables injected before general system rules.");
    println!("   Fix: Reorder so static rules come first, dynamic data second.\n");

    println!("4. KV004_UNSTABLE_SYSTEM_PROMPT_PREFIX (High)");
    println!("   Detects shuffled tool descriptions or non-deterministic ordering.");
    println!("   Fix: Sort tools and document sections deterministically.\n");

    println!("5. KV005_UNCACHED_FEW_SHOT_ORDER (Medium)");
    println!("   Detects random few-shot examples ordering across requests.");
    println!("   Fix: Freeze example ordering in system prompt prefix.\n");

    println!("6. KV006_DYNAMIC_TEMPLATE_TAG_IN_SYSTEM_PREFIX (Warning)");
    println!("   Detects Jinja2 / Mustache dynamic tags {{ now }} / {{ random }} in header.");
    println!("   Fix: Move Jinja2 runtime variables to user message body.\n");
}
