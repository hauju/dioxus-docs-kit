//! Card and CardGroup parser.

use regex::Regex;

use super::utils::{extract_attr, find_closing_tag};
use crate::parser::types::*;

/// Try to parse a CardGroup.
pub(super) fn try_parse_card_group(content: &str) -> Option<(DocNode, &str)> {
    // Match opening tag with optional cols attribute
    let open_re = Regex::new(r"^<CardGroup(?:\s+cols=\{?(\d+)\}?)?\s*>").unwrap();

    let open_caps = open_re.captures(content)?;
    let open_match = open_caps.get(0).expect("regex group 0");
    let cols: u8 = open_caps
        .get(1)
        .map(|m| m.as_str().parse().unwrap_or(2))
        .unwrap_or(2);

    let after_open = &content[open_match.end()..];

    // Find closing tag
    let close_idx = find_closing_tag(after_open, "CardGroup")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</CardGroup>".len()..];

    // Parse inner cards
    let cards = parse_cards(inner);

    Some((DocNode::CardGroup(CardGroupNode { cols, cards }), rest))
}

/// Try to parse a Columns element (treated same as CardGroup).
pub(super) fn try_parse_columns(content: &str) -> Option<(DocNode, &str)> {
    let open_re = Regex::new(r"^<Columns(?:\s+cols=\{?(\d+)\}?)?\s*>").unwrap();

    let open_caps = open_re.captures(content)?;
    let open_match = open_caps.get(0).expect("regex group 0");
    let cols: u8 = open_caps
        .get(1)
        .map(|m| m.as_str().parse().unwrap_or(3))
        .unwrap_or(3);

    let after_open = &content[open_match.end()..];
    let close_idx = find_closing_tag(after_open, "Columns")?;
    let inner = &after_open[..close_idx];
    let rest = &after_open[close_idx + "</Columns>".len()..];

    let cards = parse_cards(inner);

    Some((DocNode::CardGroup(CardGroupNode { cols, cards }), rest))
}

/// Try to parse a standalone Card (not in a group).
pub(super) fn try_parse_standalone_card(content: &str) -> Option<(DocNode, &str)> {
    if !content.starts_with("<Card") {
        return None;
    }

    let card = parse_single_card(content)?;
    let card_end = find_card_end(content)?;

    Some((
        DocNode::CardGroup(CardGroupNode {
            cols: 1,
            cards: vec![card],
        }),
        &content[card_end..],
    ))
}

/// Parse Card elements from content.
fn parse_cards(content: &str) -> Vec<CardNode> {
    let mut cards = Vec::new();
    let mut remaining = content.trim();

    // Pattern for self-closing cards: <Card ... />
    let self_closing_re = Regex::new(
        r#"(?s)^<Card\s+(?:title="([^"]*)")?\s*(?:icon="([^"]*)")?\s*(?:href="([^"]*)")?\s*/>"#,
    )
    .unwrap();

    while !remaining.is_empty() {
        remaining = remaining.trim();

        // Try self-closing first
        if let Some(caps) = self_closing_re.captures(remaining) {
            let full_match = caps.get(0).expect("regex group 0");
            cards.push(CardNode {
                title: caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
                icon: caps.get(2).map(|m| m.as_str().to_string()),
                href: caps.get(3).map(|m| m.as_str().to_string()),
                content: String::new(),
            });
            remaining = &remaining[full_match.end()..];
            continue;
        }

        // Try card with content - more flexible parsing
        if remaining.starts_with("<Card")
            && let Some(card) = parse_single_card(remaining)
        {
            let card_end = find_card_end(remaining).unwrap_or(remaining.len());
            cards.push(card);
            remaining = &remaining[card_end..];
            continue;
        }

        // Skip non-card content
        if let Some(idx) = remaining.find("<Card") {
            remaining = &remaining[idx..];
        } else {
            break;
        }
    }

    cards
}

/// Parse a single Card element with flexible attribute handling.
fn parse_single_card(content: &str) -> Option<CardNode> {
    // Find the end of the opening tag
    let tag_end = content.find('>')?;
    let tag_content = &content[5..tag_end]; // Skip "<Card"

    // Check if self-closing
    let is_self_closing = tag_content.trim().ends_with('/');

    // Extract attributes
    let title = extract_attr(tag_content, "title");
    let icon = extract_attr(tag_content, "icon");
    let href = extract_attr(tag_content, "href");

    let inner_content = if is_self_closing {
        String::new()
    } else {
        // Find closing </Card>
        let after_open = &content[tag_end + 1..];
        if let Some(close_idx) = find_closing_tag(after_open, "Card") {
            after_open[..close_idx].trim().to_string()
        } else {
            String::new()
        }
    };

    Some(CardNode {
        title: title.unwrap_or_default(),
        icon,
        href,
        content: inner_content,
    })
}

/// Find where a Card element ends (including closing tag).
fn find_card_end(content: &str) -> Option<usize> {
    let tag_end = content.find('>')?;
    let tag_content = &content[5..tag_end];

    if tag_content.trim().ends_with('/') {
        // Self-closing
        Some(tag_end + 1)
    } else {
        // Find closing tag
        let after_open = &content[tag_end + 1..];
        let close_idx = find_closing_tag(after_open, "Card")?;
        Some(tag_end + 1 + close_idx + "</Card>".len())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::content::parse_mdx;
    use crate::parser::types::*;

    #[test]
    fn test_parse_card_group() {
        let content = r#"<CardGroup cols={2}>
  <Card title="First" icon="star">Content 1</Card>
  <Card title="Second" href="/link">Content 2</Card>
</CardGroup>"#;

        let nodes = parse_mdx(content);
        assert_eq!(nodes.len(), 1);
        if let DocNode::CardGroup(cg) = &nodes[0] {
            assert_eq!(cg.cols, 2);
            assert_eq!(cg.cards.len(), 2);
            assert_eq!(cg.cards[0].title, "First");
            assert_eq!(cg.cards[1].href, Some("/link".to_string()));
        } else {
            panic!("Expected CardGroup node");
        }
    }
}
