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
fn test_clean_prompt_passes_with_100_score() {
    let analyzer = Analyzer::new();
    let prompt = "# Static instruction block\nSYSTEM_PROMPT = \"You are an expert compiler engineer. Always return valid ASTs.\"\n\ndef execute_task(task_payload):\n    return f\"{SYSTEM_PROMPT}\\nTask: {task_payload}\"\n";
    let issues = analyzer.analyze_file(&PathBuf::from("clean.py"), prompt);
    assert!(issues.is_empty());
    assert_eq!(analyzer.compute_score(&issues), 100);
}
