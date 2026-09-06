use kvlint::analyzer::{Analyzer, IssueSeverity};
use std::path::PathBuf;

#[test]
fn test_detects_dynamic_timestamp_at_prefix() {
    let analyzer = Analyzer::new();
    let bad_prompt = r#"
system_prompt = f"""
Current time: {datetime.now()}
You are a helpful customer support agent.
Instructions:
1. Always be polite.
2. Answer queries accurately.
"""
"#;
    let issues = analyzer.analyze_file(&PathBuf::from("agent.py"), bad_prompt);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "KV001_DYNAMIC_TIMESTAMP_PREFIX");
    assert_eq!(issues[0].severity, IssueSeverity::Critical);
}

#[test]
fn test_detects_dynamic_uuid_header() {
    let analyzer = Analyzer::new();
    let bad_prompt = r#"
const systemPrompt = `
Session ID: ${crypto.randomUUID()}
User role: Editor
Tool definitions:
- fetch_data
- update_record
`;
"#;
    let issues = analyzer.analyze_file(&PathBuf::from("index.ts"), bad_prompt);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule_id, "KV002_DYNAMIC_UUID_AT_HEADER");
    assert_eq!(issues[0].severity, IssueSeverity::Critical);
}

#[test]
fn test_clean_prompt_passes_with_100_score() {
    let analyzer = Analyzer::new();
    let clean_prompt = r#"
SYSTEM_PROMPT = """
You are a senior Rust systems engineer.
Follow these rules:
1. Prefer zero-allocation parsing.
2. Maintain static prefix cache alignment.
"""
"#;
    let issues = analyzer.analyze_file(&PathBuf::from("clean_agent.py"), clean_prompt);
    assert!(issues.is_empty());
    assert_eq!(analyzer.compute_score(&issues), 100);
}
