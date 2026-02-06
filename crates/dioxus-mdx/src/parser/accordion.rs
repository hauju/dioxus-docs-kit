//! Accordion and AccordionGroup parser.

use super::content::parse_content;
use super::utils::{extract_attr, find_closing_tag};
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
        if let Some(idx) = remaining[1..].find("<Accordion") {
            remaining = &remaining[idx + 1..];
        } else {
            break;
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
