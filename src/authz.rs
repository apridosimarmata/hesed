/// Evaluate a tool against a pre-resolved list of allowed_tools (from agent key resolution).
/// Supports exact match, full wildcard `"*"`, and prefix wildcards like `"poimen_*"`.
pub fn evaluate_tools(allowed_tools: &[String], tool: &str) -> bool {
    allowed_tools.iter().any(|t| {
        t == "*"
            || t == tool
            || (t.ends_with('*') && tool.starts_with(&t[..t.len() - 1]))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_allows_any_tool() {
        let tools = vec!["*".into()];
        assert!(evaluate_tools(&tools, "anything"));
        assert!(evaluate_tools(&tools, "db_delete"));
    }

    #[test]
    fn exact_match() {
        let tools = vec!["jira_search".into(), "github_pr".into()];
        assert!(evaluate_tools(&tools, "jira_search"));
        assert!(evaluate_tools(&tools, "github_pr"));
        assert!(!evaluate_tools(&tools, "db_delete"));
    }

    #[test]
    fn prefix_wildcard() {
        let tools = vec!["poimen_*".into()];
        assert!(evaluate_tools(&tools, "poimen_list_roles"));
        assert!(evaluate_tools(&tools, "poimen_create_role"));
        assert!(!evaluate_tools(&tools, "db_delete"));
        assert!(!evaluate_tools(&tools, "list_roles"));
    }

    #[test]
    fn empty_list_denies_all() {
        let tools: Vec<String> = vec![];
        assert!(!evaluate_tools(&tools, "anything"));
    }
}
