use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Optimization,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptIssue {
    pub rule_id: String,
    pub severity: IssueSeverity,
    pub file: PathBuf,
    pub line: usize,
    pub title: String,
    pub explanation: String,
    pub recommendation: String,
    pub snippet: String,
    pub estimated_token_waste_pct: u8,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisResult {
    pub files_scanned: usize,
    pub prompts_analyzed: usize,
    pub issues: Vec<PromptIssue>,
    pub cache_efficiency_score: u8,
}

pub struct Analyzer {
    rules: Vec<Box<dyn crate::rules::Rule>>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            rules: crate::rules::all_rules(),
        }
    }

    pub fn analyze_file(&self, path: &std::path::Path, content: &str) -> Vec<PromptIssue> {
        let mut issues = Vec::new();
        for rule in &self.rules {
            issues.extend(rule.check(path, content));
        }
        issues
    }

    pub fn compute_score(&self, issues: &[PromptIssue]) -> u8 {
        let mut score = 100i32;
        for issue in issues {
            match issue.severity {
                IssueSeverity::Critical => score -= 25,
                IssueSeverity::Warning => score -= 10,
                IssueSeverity::Optimization => score -= 5,
            }
        }
        score.max(0).min(100) as u8
    }
}
