//! Accordion and AccordionGroup parser.

use super::content::parse_content;
use super::utils::{extract_attr, find_closing_tag, skip_to_next_tag};
use crate::parser::types::*;

/// Try to parse an AccordionGroup component.
pub(super) fn try_parse_accordion_group(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<AccordionGroup>") {
        return None;
    }

    let after_open = &content["<AccordionGroup>".len()..];
    let close_idx = find_closing_tag(after_open, "AccordionGroup")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</AccordionGroup>".len()..];

    let items = parse_accordions(inner);

    Some((DocNode::AccordionGroup(AccordionGroupNode { items }), rest))
}

/// Try to parse a standalone Accordion (not in a group).
pub(super) fn try_parse_standalone_accordion(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<Accordion") {
        return None;
    }

    let accordion = parse_single_accordion(content)?;
    let acc_end = find_accordion_end(content)?;

    Some((
        DocNode::AccordionGroup(AccordionGroupNode {
            items: vec![accordion],
        }),
        &content[acc_end..],
    ))
}

/// Parse Accordion elements from content.
fn parse_accordions(content: &str) -> Vec<AccordionNode> {
    let mut items = Vec::new();
    let mut remaining = content.trim();

    while !remaining.is_empty() {
        remaining = remaining.trim();

        if remaining.starts_with("<Accordion")
            && let Some(acc) = parse_single_accordion(remaining)
        {
            let acc_end = find_accordion_end(remaining).unwrap_or(remaining.len());
            items.push(acc);
            remaining = &remaining[acc_end..];
            continue;
        }

        // Skip to next Accordion
        match skip_to_next_tag(remaining, "<Accordion") {
            Some(next) => remaining = next,
            None => break,
        }
    }

    items
}

/// Parse a single Accordion element.
fn parse_single_accordion(content: &str) -> Option<AccordionNode> {
    let tag_end = content.find('>')?;
    let tag_content = &content[10..tag_end]; // Skip "<Accordion"

    let title = extract_attr(tag_content, "title")?;
    let icon = extract_attr(tag_content, "icon");

    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "Accordion")?;
    let inner = after_open[..close_idx].trim();
    // Parse inner content recursively
    let parsed_content = parse_content(inner);

    Some(AccordionNode {
        title,
        icon,
        content: parsed_content,
    })
}

/// Find where an Accordion element ends.
fn find_accordion_end(content: &str) -> Option<usize> {
    let tag_end = content.find('>')?;
    let after_open = &content[tag_end + 1..];
    let close_idx = find_closing_tag(after_open, "Accordion")?;
    Some(tag_end + 1 + close_idx + "</Accordion>".len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_accordion_group_with_items() {
        let content = "<AccordionGroup>\n<Accordion title=\"One\" icon=\"star\">\nBody one\n</Accordion>\n<Accordion title=\"Two\">\nBody two\n</Accordion>\n</AccordionGroup>rest";
        let (node, rest) = try_parse_accordion_group(content).unwrap();
        match node {
            DocNode::AccordionGroup(group) => {
                assert_eq!(group.items.len(), 2);
                assert_eq!(group.items[0].title, "One");
                assert_eq!(group.items[0].icon.as_deref(), Some("star"));
                assert_eq!(group.items[1].title, "Two");
                assert_eq!(group.items[1].icon, None);
                assert!(!group.items[0].content.is_empty());
            }
            other => panic!("expected AccordionGroup, got {other:?}"),
        }
        assert_eq!(rest, "rest");
    }

    #[test]
    fn standalone_accordion_wraps_into_group() {
        let content = "<Accordion title=\"Solo\">\nInner text\n</Accordion>after";
        let (node, rest) = try_parse_standalone_accordion(content).unwrap();
        match node {
            DocNode::AccordionGroup(group) => {
                assert_eq!(group.items.len(), 1);
                assert_eq!(group.items[0].title, "Solo");
            }
            other => panic!("expected AccordionGroup, got {other:?}"),
        }
        assert_eq!(rest, "after");
    }

    #[test]
    fn accordion_without_title_is_rejected() {
        assert!(try_parse_standalone_accordion("<Accordion>\nBody\n</Accordion>").is_none());
    }

    #[test]
    fn non_accordion_input_returns_none() {
        assert!(try_parse_accordion_group("plain text").is_none());
        assert!(try_parse_standalone_accordion("plain text").is_none());
    }

    #[test]
    fn non_ascii_prose_between_accordions_does_not_panic() {
        let content = "<AccordionGroup>\n\u{dc}berblick\n<Accordion title=\"A\">body</Accordion>\n</AccordionGroup>";
        let nodes = crate::parser::content::parse_mdx(content);
        let group = nodes.iter().find_map(|n| match n {
            DocNode::AccordionGroup(g) => Some(g),
            _ => None,
        });
        assert_eq!(group.expect("AccordionGroup node").items.len(), 1);
    }
}
