use kvlint::analyzer::Analyzer;
use std::path::PathBuf;

#[test]
fn test_detects_dynamic_timestamp_at_prefix() {
    let analyzer = Analyzer::new();
    let prompt = "import datetime\n\nSYSTEM_PROMPT = f'Current time is: {datetime.now()}\\nYou are a helpful assistant.'\n";
    let issues = analyzer.analyze_file(&PathBuf::from("test.py"), prompt);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.rule_id.contains("KV001")));
}

#[test]
fn test_detects_dynamic_uuid_header() {
    let analyzer = Analyzer::new();
    let prompt = "SYSTEM = f'Session: {uuid.uuid4()}\\nInstructions: Be concise.'\n";
    let issues = analyzer.analyze_file(&PathBuf::from("agent.py"), prompt);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.rule_id.contains("KV002")));
}

#[test]
fn test_detects_raw_git_diff_in_prefix() {
    let analyzer = Analyzer::new();
    let prompt = "System: Review the following code diff:\ngit diff --cached\ncommit 40de1fc00cbe764b3b9015a8805277b07096343d\n";
    let issues = analyzer.analyze_file(&PathBuf::from("diff_agent.py"), prompt);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.rule_id.contains("KV007")));
}

#[test]
fn test_detects_thinking_trace_in_history_prefix() {
    let analyzer = Analyzer::new();
    let prompt = "User: Hello\nAssistant: <think>Evaluating best response approach...</think>\nHere is the answer.\n";
    let issues = analyzer.analyze_file(&PathBuf::from("chat_agent.py"), prompt);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.rule_id.contains("KV008")));
}

#[test]
fn test_detects_unsorted_json_in_prefix() {
    let analyzer = Analyzer::new();
    let prompt = "tools_schema = json.dumps(tools_dict)\nsystem_prompt = f\"Tools: {tools_schema}\\nExecute carefully.\"\n";
    let issues = analyzer.analyze_file(&PathBuf::from("agent_tools.py"), prompt);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.rule_id.contains("KV009")));
}

#[test]
fn test_detects_dynamic_tool_schema_mutation_in_prefix() {
    let analyzer = Analyzer::new();
    let prompt = "# System initialization\ntools = [\"read_file\", \"execute_code\"]\nsystem_prompt = \"You are a coding assistant.\"\n";
    let issues = analyzer.analyze_file(&PathBuf::from("tool_agent.py"), prompt);
    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|i| i.rule_id == "KV010_DYNAMIC_TOOL_SCHEMA_MUTATION_IN_PREFIX"));
    let issue = issues.iter().find(|i| i.rule_id.contains("KV010")).unwrap();
    assert_eq!(issue.estimated_token_waste_pct, 80);
}

#[test]
fn test_dynamic_tool_schema_beyond_prefix_line_limit_not_flagged() {
    let analyzer = Analyzer::new();
    // 36 lines of static instructions before tools definition
    let padding = (0..36)
        .map(|i| format!("# static line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!("{}\ntools = [\"read_file\"]\n", padding);
    let issues = analyzer.analyze_file(&PathBuf::from("tool_agent_tail.py"), &prompt);
    assert!(!issues.iter().any(|i| i.rule_id.contains("KV010")));
}

#[test]
fn test_clean_prompt_passes_with_100_score() {
    let analyzer = Analyzer::new();
    let prompt = "# Static instruction block\nSYSTEM_PROMPT = \"You are an expert compiler engineer. Always return valid ASTs.\"\n\ndef execute_task(task_payload):\n    return f\"{SYSTEM_PROMPT}\\nTask: {task_payload}\"\n";
    let issues = analyzer.analyze_file(&PathBuf::from("clean.py"), prompt);
    assert!(issues.is_empty());
    assert_eq!(analyzer.compute_score(&issues), 100);
}
