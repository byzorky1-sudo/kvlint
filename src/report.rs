use crate::analyzer::{AnalysisResult, IssueSeverity};

pub fn print_terminal_report(result: &AnalysisResult) {
    println!("\n⚡ kvlint v0.1.0 — LLM Prefix KV-Cache Linter & Optimizer");
    println!("============================================================");
    println!(
        "📊 Files Scanned: {} | Issues Detected: {}",
        result.files_scanned,
        result.issues.len()
    );

    if result.issues.is_empty() {
        println!("\n✅ 100/100 Cache Efficiency Score. Zero prefix KV-cache invalidation hazards detected!\n");
        return;
    }

    println!(
        "🎯 KV-Cache Efficiency Score: {} / 100\n",
        result.cache_efficiency_score
    );

    for (idx, issue) in result.issues.iter().enumerate() {
        let (icon, severity_str) = match issue.severity {
            IssueSeverity::Critical => ("🚨", "CRITICAL [Potential 90%+ Prefix Cache Kill]"),
            IssueSeverity::Warning => ("⚠️ ", "WARNING  [75% Prefix Invalidation]"),
            IssueSeverity::Optimization => ("💡", "OPTIMIZE [40% Prefix Drift]"),
        };

        println!("{}. {} {}: {}", idx + 1, icon, severity_str, issue.title);
        println!("   📍 Location: {}:{}", issue.file.display(), issue.line);
        println!("   🔍 Snippet:  `{}`", issue.snippet);
        println!("   ⚠️  Hazard:   {}", issue.explanation);
        println!("   🛠️  Fix:      {}\n", issue.recommendation);
    }
}
