use crate::analyzer::{IssueSeverity, PromptIssue};
use regex::Regex;
use std::path::Path;

pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue>;
}

pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DynamicTimestampRule::new()),
        Box::new(DynamicUuidRule::new()),
        Box::new(UnstableSystemPromptPrefixRule::new()),
        Box::new(DynamicVariableAtTopRule::new()),
        Box::new(UncachedFewShotOrderRule::new()),
        Box::new(DynamicDateInJinjaRule::new()),
    ]
}

/// Rule 6: Dynamic Date / Random functions inside Jinja2 / Mustache templates
pub struct DynamicDateInJinjaRule {
    pattern: Regex,
}

impl DynamicDateInJinjaRule {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r#"(?i)(\{\{.*(datetime|time|now|random|uuid).*\}\})"#).unwrap(),
        }
    }
}

impl Rule for DynamicDateInJinjaRule {
    fn id(&self) -> &'static str {
        "KV006_DYNAMIC_TEMPLATE_TAG_IN_SYSTEM_PREFIX"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if idx < 20 && self.pattern.is_match(line) {
                issues.push(PromptIssue {
                    rule_id: self.id().to_string(),
                    severity: IssueSeverity::Warning,
                    file: path.to_path_buf(),
                    line: idx + 1,
                    title: "Dynamic template tag (datetime/now/random) in system prompt".to_string(),
                    explanation: "Evaluating dynamic timestamp or random template filters inside prompt headers mutates prefix tokens per rendering, breaking prefill caching.".to_string(),
                    recommendation: "Separate static system instructions from dynamic template variables or place variables at the tail of the message.".to_string(),
                    snippet: line.trim().to_string(),
                    estimated_token_waste_pct: 80,
                });
            }
        }
        issues
    }
}

/// Rule 1: Dynamic Timestamp in System Prompt Prefix
/// e.g., datetime.now(), time.time(), new Date(), $(date) at the beginning of prompt
pub struct DynamicTimestampRule {
    pattern: Regex,
}

impl DynamicTimestampRule {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r#"(?i)(datetime\.now\(\)|time\.time\(\)|new Date\(\)|Date\.now\(\)|\bnow\(\)|\bcurrent_time\b|strftime|\bcurrent_date\b|\b\{\{timestamp\}\}|\b\{timestamp\}\})"#).unwrap(),
        }
    }
}

impl Rule for DynamicTimestampRule {
    fn id(&self) -> &'static str {
        "KV001_DYNAMIC_TIMESTAMP_PREFIX"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            // Check top 25 lines of prompt / system message
            if idx < 25 && self.pattern.is_match(line) {
                issues.push(PromptIssue {
                    rule_id: self.id().to_string(),
                    severity: IssueSeverity::Critical,
                    file: path.to_path_buf(),
                    line: idx + 1,
                    title: "Dynamic Timestamp invalidates prefix KV-cache".to_string(),
                    explanation: "Placing dynamic timestamps at the beginning of system prompts changes the token prefix on every request, completely invalidating GPU prefix caching across vLLM, SGLang, and Anthropic/OpenAI.".to_string(),
                    recommendation: "Move dynamic timestamps to the end of the prompt or into the user message, or quantize timestamps to 1-hour / 1-day granularity.".to_string(),
                    snippet: line.trim().to_string(),
                    estimated_token_waste_pct: 95,
                });
            }
        }
        issues
    }
}

/// Rule 2: Dynamic UUID / Session ID at System Prompt Header
pub struct DynamicUuidRule {
    pattern: Regex,
}

impl DynamicUuidRule {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r#"(?i)(uuid\.uuid4\(\)|crypto\.randomUUID\(\)|\bnonce\b|\bsession_id\b|\brequest_id\b|\btrace_id\b)"#).unwrap(),
        }
    }
}

impl Rule for DynamicUuidRule {
    fn id(&self) -> &'static str {
        "KV002_DYNAMIC_UUID_AT_HEADER"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if idx < 20 && self.pattern.is_match(line) {
                issues.push(PromptIssue {
                    rule_id: self.id().to_string(),
                    severity: IssueSeverity::Critical,
                    file: path.to_path_buf(),
                    line: idx + 1,
                    title: "Unique session/request ID at prompt start destroys KV reuse".to_string(),
                    explanation: "Generating random session_id, request_id, or UUIDs at the top of system instructions guarantees 0% prefix cache hits for all subsequent tokens.".to_string(),
                    recommendation: "Pass session/trace IDs as metadata parameters or append them to the tail of the context window.".to_string(),
                    snippet: line.trim().to_string(),
                    estimated_token_waste_pct: 100,
                });
            }
        }
        issues
    }
}

/// Rule 3: Dynamic Variable at Top of System Prompt (Template variables before static system instructions)
pub struct DynamicVariableAtTopRule {
    pattern: Regex,
}

impl DynamicVariableAtTopRule {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r#"(\{\{\s*(user_input|query|input|prompt|user_query|context|user_data)\s*\}\}|\{\s*(user_input|query|input|user_query|context)\s*\})"#).unwrap(),
        }
    }
}

impl Rule for DynamicVariableAtTopRule {
    fn id(&self) -> &'static str {
        "KV003_USER_VARIABLE_BEFORE_SYSTEM_PREFIX"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if idx < 15 && self.pattern.is_match(line) {
                issues.push(PromptIssue {
                    rule_id: self.id().to_string(),
                    severity: IssueSeverity::Warning,
                    file: path.to_path_buf(),
                    line: idx + 1,
                    title: "Dynamic user variable injected before static system prompt".to_string(),
                    explanation: "Placing dynamic variables (like user queries or rag context) before static tool definitions and system rules breaks the common prefix tree.".to_string(),
                    recommendation: "Structure prompts with fixed static blocks first (System Instructions -> Tools -> Pinned Guidelines) and dynamic user variables strictly at the end.".to_string(),
                    snippet: line.trim().to_string(),
                    estimated_token_waste_pct: 75,
                });
            }
        }
        issues
    }
}

/// Rule 4: Unstable System Prompt Prefix (Tools / Docs shuffled)
pub struct UnstableSystemPromptPrefixRule {
    pattern: Regex,
}

impl UnstableSystemPromptPrefixRule {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(
                r#"(?i)(random\.shuffle|Math\.random\(\)|\.sort\(\s*=>\s*Math\.random)"#,
            )
            .unwrap(),
        }
    }
}

impl Rule for UnstableSystemPromptPrefixRule {
    fn id(&self) -> &'static str {
        "KV004_SHUFFLED_PROMPT_COMPONENTS"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if self.pattern.is_match(line) {
                issues.push(PromptIssue {
                    rule_id: self.id().to_string(),
                    severity: IssueSeverity::Warning,
                    file: path.to_path_buf(),
                    line: idx + 1,
                    title: "Randomized ordering of prompt components detected".to_string(),
                    explanation: "Randomly shuffling tools, few-shot examples, or documentation fragments causes RadixAttention and PagedAttention prefix trees to diverge on every call.".to_string(),
                    recommendation: "Enforce deterministic alphabetical or key-based sorting for all prompt components to guarantee prefix cache alignment.".to_string(),
                    snippet: line.trim().to_string(),
                    estimated_token_waste_pct: 60,
                });
            }
        }
        issues
    }
}

/// Rule 5: Few-Shot Order Optimization
pub struct UncachedFewShotOrderRule {
    pattern: Regex,
}

impl UncachedFewShotOrderRule {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r#"(?i)(\bexamples\s*=\s*\[|\bfew_shots\s*=\s*\[)"#).unwrap(),
        }
    }
}

impl Rule for UncachedFewShotOrderRule {
    fn id(&self) -> &'static str {
        "KV005_DYNAMIC_FEW_SHOT_ALIGNMENT"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if self.pattern.is_match(line) {
                issues.push(PromptIssue {
                    rule_id: self.id().to_string(),
                    severity: IssueSeverity::Optimization,
                    file: path.to_path_buf(),
                    line: idx + 1,
                    title: "Verify static alignment of few-shot example prefixes".to_string(),
                    explanation: "Few-shot examples should maintain an identical static header and order across calls to enable cross-request KV reuse.".to_string(),
                    recommendation: "Pin shared few-shot examples into the system message prefix rather than re-creating them inside dynamic user prompts.".to_string(),
                    snippet: line.trim().to_string(),
                    estimated_token_waste_pct: 40,
                });
            }
        }
        issues
    }
}
