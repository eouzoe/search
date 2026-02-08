use std::collections::HashSet;

/// 文本塊類型
#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockType {
    Heading,
    Code,
    Paragraph,
    #[allow(dead_code)]
    Other,
}

/// 文本塊
#[derive(Debug, Clone)]
struct TextBlock {
    content: String,
    #[allow(dead_code)]
    block_type: BlockType,
    priority: u32,
}

impl TextBlock {
    fn new(content: String, block_type: BlockType) -> Self {
        let priority = match block_type {
            BlockType::Heading => 100,
            BlockType::Code => 80,
            BlockType::Paragraph => 50,
            BlockType::Other => 10,
        };
        Self {
            content,
            block_type,
            priority,
        }
    }

    fn estimate_tokens(&self) -> usize {
        self.content.len() / 4
    }
}

/// 上下文裁剪器
pub struct ContextPruner {
    max_tokens: usize,
}

impl ContextPruner {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    /// 裁剪內容到指定的 Token 預算
    pub fn prune(&self, content: &str) -> String {
        let blocks = self.split_into_blocks(content);
        let blocks = self.remove_duplicates(blocks);
        let mut blocks = blocks;
        self.rank_blocks(&mut blocks);
        self.truncate_to_budget(&blocks)
    }

    fn split_into_blocks(&self, text: &str) -> Vec<TextBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut current_paragraph = String::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !current_paragraph.is_empty() {
                    blocks.push(TextBlock::new(
                        current_paragraph.clone(),
                        BlockType::Paragraph,
                    ));
                    current_paragraph.clear();
                }
                continue;
            }

            if trimmed.starts_with('#')
                || (trimmed.len() < 100
                    && trimmed.chars().filter(|c| c.is_uppercase()).count()
                        > trimmed.len() / 2)
            {
                if !current_paragraph.is_empty() {
                    blocks.push(TextBlock::new(
                        current_paragraph.clone(),
                        BlockType::Paragraph,
                    ));
                    current_paragraph.clear();
                }
                blocks.push(TextBlock::new(trimmed.to_string(), BlockType::Heading));
                continue;
            }

            let code_indicators = [
                "{", "}", "()", "=>", "fn ", "def ", "class ", "import ", "const ", "let ",
                "var ",
            ];
            let is_code = code_indicators
                .iter()
                .any(|&indicator| trimmed.contains(indicator));

            if is_code {
                if !current_paragraph.is_empty() {
                    blocks.push(TextBlock::new(
                        current_paragraph.clone(),
                        BlockType::Paragraph,
                    ));
                    current_paragraph.clear();
                }
                blocks.push(TextBlock::new(trimmed.to_string(), BlockType::Code));
                continue;
            }

            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(trimmed);
        }

        if !current_paragraph.is_empty() {
            blocks.push(TextBlock::new(current_paragraph, BlockType::Paragraph));
        }

        blocks
    }

    fn rank_blocks(&self, blocks: &mut [TextBlock]) {
        blocks.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    fn truncate_to_budget(&self, blocks: &[TextBlock]) -> String {
        let mut result = Vec::new();
        let mut total_tokens = 0;

        for block in blocks {
            let block_tokens = block.estimate_tokens();

            if total_tokens + block_tokens <= self.max_tokens {
                result.push(block.content.clone());
                total_tokens += block_tokens;
            } else {
                let remaining_tokens = self.max_tokens.saturating_sub(total_tokens);
                let remaining_chars = remaining_tokens * 4;

                if remaining_chars > 100 {
                    let truncated: String =
                        block.content.chars().take(remaining_chars).collect();
                    result.push(format!("{}...", truncated));
                }
                break;
            }
        }

        result.join("\n\n")
    }

    fn remove_duplicates(&self, blocks: Vec<TextBlock>) -> Vec<TextBlock> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for block in blocks {
            let key: String = block.content.chars().take(100).collect();

            if seen.insert(key) {
                result.push(block);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_blocks() {
        let pruner = ContextPruner::new(1000);
        let text = "# Title\n\nThis is a paragraph.\n\nfn main() { }\n\nAnother paragraph.";
        let blocks = pruner.split_into_blocks(text);

        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].block_type, BlockType::Heading);
        assert_eq!(blocks[1].block_type, BlockType::Paragraph);
        assert_eq!(blocks[2].block_type, BlockType::Code);
        assert_eq!(blocks[3].block_type, BlockType::Paragraph);
    }

    #[test]
    fn test_rank_blocks() {
        let pruner = ContextPruner::new(1000);
        let mut blocks = vec![
            TextBlock::new("paragraph".to_string(), BlockType::Paragraph),
            TextBlock::new("heading".to_string(), BlockType::Heading),
            TextBlock::new("code".to_string(), BlockType::Code),
        ];

        pruner.rank_blocks(&mut blocks);

        assert_eq!(blocks[0].block_type, BlockType::Heading);
        assert_eq!(blocks[1].block_type, BlockType::Code);
        assert_eq!(blocks[2].block_type, BlockType::Paragraph);
    }

    #[test]
    fn test_remove_duplicates() {
        let pruner = ContextPruner::new(1000);
        let blocks = vec![
            TextBlock::new("Same content".to_string(), BlockType::Paragraph),
            TextBlock::new("Same content".to_string(), BlockType::Paragraph),
            TextBlock::new("Different content".to_string(), BlockType::Paragraph),
        ];

        let result = pruner.remove_duplicates(blocks);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_truncate_to_budget() {
        let pruner = ContextPruner::new(50);
        let blocks = vec![
            TextBlock::new("A".repeat(100), BlockType::Heading),
            TextBlock::new("B".repeat(100), BlockType::Paragraph),
            TextBlock::new("C".repeat(100), BlockType::Paragraph),
        ];

        let result = pruner.truncate_to_budget(&blocks);
        assert!(result.contains("AAA"));
        assert!(result.len() <= 250);
    }

    #[test]
    fn test_prune_full() {
        let pruner = ContextPruner::new(100);
        let content = r#"
# Important Title

This is a very important paragraph with lots of information.

fn example_code() {
    println!("Hello");
}

Another paragraph.

This is a very important paragraph with lots of information.
        "#;

        let result = pruner.prune(content);
        assert!(result.contains("Important Title"));
        let count = result.matches("very important paragraph").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_empty_input() {
        let pruner = ContextPruner::new(1000);
        let result = pruner.prune("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_estimate_tokens() {
        let block = TextBlock::new("A".repeat(400), BlockType::Paragraph);
        assert_eq!(block.estimate_tokens(), 100);
    }
}