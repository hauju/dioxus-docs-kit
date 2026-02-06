//! Steps component parser.

use regex::Regex;

use super::content::parse_content;
use super::utils::find_closing_tag;
use crate::parser::types::*;

/// Try to parse a Steps component.
pub(super) fn try_parse_steps(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<Steps>") {
        return None;
    }

    let after_open = &content["<Steps>".len()..];
    let close_idx = find_closing_tag(after_open, "Steps")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</Steps>".len()..];

    // Parse steps - they use ### headings or <Step> tags
    let steps = parse_steps(inner);

    Some((DocNode::Steps(StepsNode { steps }), rest))
}

/// Parse Step elements from content.
fn parse_steps(content: &str) -> Vec<StepNode> {
    let mut steps = Vec::new();

    // First try <Step title="..."> format
    let step_re = Regex::new(r#"(?s)<Step\s+title="([^"]*)">(.*?)</Step>"#).unwrap();
    for caps in step_re.captures_iter(content) {
        let inner = caps.get(2).map(|m| m.as_str()).unwrap_or_default().trim();
        // Parse inner content recursively
        let parsed_content = parse_content(inner);
        steps.push(StepNode {
            title: caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or_default()
                .to_string(),
            content: parsed_content,
        });
    }

    if !steps.is_empty() {
        return steps;
    }

    // Fall back to ### heading format
    let heading_re = Regex::new(r"(?m)^###\s+(.+)$").unwrap();
    let headings: Vec<_> = heading_re.captures_iter(content).collect();

    for (i, caps) in headings.iter().enumerate() {
        let title = caps
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .trim()
            .to_string();
        let full_match = caps.get(0).expect("regex group 0");

        let start = full_match.end();
        let end = if i + 1 < headings.len() {
            headings[i + 1].get(0).expect("regex group 0").start()
        } else {
            content.len()
        };

        let step_content = content[start..end].trim();
        // Parse inner content recursively
        let parsed_content = parse_content(step_content);
        steps.push(StepNode {
            title,
            content: parsed_content,
        });
    }

    steps
}

#[cfg(test)]
mod tests {
    use crate::parser::content::parse_mdx;
    use crate::parser::types::*;

    #[test]
    fn test_parse_steps_with_headings() {
        let content = r#"<Steps>
### Step 1: Do something

First instruction.

### Step 2: Do another thing

Second instruction.
</Steps>"#;

        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::Steps(s) = &nodes[0] {
            assert_eq!(s.steps.len(), 2);
            assert!(s.steps[0].title.contains("Step 1"));
            assert!(s.steps[1].title.contains("Step 2"));
        } else {
            panic!("Expected Steps node");
        }
    }
}
