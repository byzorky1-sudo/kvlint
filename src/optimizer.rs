use regex::Regex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptimizationResult {
    pub original_content: String,
    pub optimized_content: String,
    pub changes_made: Vec<String>,
    pub estimated_savings_pct: u8,
}

pub struct Optimizer {
    timestamp_regex: Regex,
    uuid_regex: Regex,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            timestamp_regex: Regex::new(r#"(?i)^.*(datetime\.now\(\)|time\.time\(\)|new Date\(\)|Date\.now\(\)|\bnow\(\)|\bcurrent_time\b|\b\{\{timestamp\}\}|\b\{timestamp\}\}).*$"#).unwrap(),
            uuid_regex: Regex::new(r#"(?i)^.*(uuid\.uuid4\(\)|crypto\.randomUUID\(\)|\bsession_id\b|\brequest_id\b|\btrace_id\b).*$"#).unwrap(),
        }
    }

    pub fn optimize_prompt_text(&self, content: &str) -> OptimizationResult {
        let mut static_lines = Vec::new();
        let mut dynamic_lines = Vec::new();
        let mut changes = Vec::new();

        for line in content.lines() {
            if self.timestamp_regex.is_match(line) {
                dynamic_lines.push(line.to_string());
                changes.push(format!(
                    "Moved dynamic timestamp line to tail: `{}`",
                    line.trim()
                ));
            } else if self.uuid_regex.is_match(line) {
                dynamic_lines.push(line.to_string());
                changes.push(format!(
                    "Moved dynamic UUID/session variable to tail: `{}`",
                    line.trim()
                ));
            } else {
                static_lines.push(line.to_string());
            }
        }

        let mut optimized = static_lines.join("\n");
        if !dynamic_lines.is_empty() {
            optimized.push_str("\n\n# --- Dynamic Runtime Tail (Auto-optimized by kvlint for Prefix Caching) ---\n");
            optimized.push_str(&dynamic_lines.join("\n"));
        }

        let savings = if !changes.is_empty() { 65 } else { 0 };

        OptimizationResult {
            original_content: content.to_string(),
            optimized_content: optimized,
            changes_made: changes,
            estimated_savings_pct: savings,
        }
    }
}
