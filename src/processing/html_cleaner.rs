/// HTML 清理器
pub struct HtmlCleaner;

impl HtmlCleaner {
    /// 清理 HTML 內容
    pub fn clean(html: &str) -> String {
        let text = Self::remove_tags(html);
        let text = Self::remove_noise(&text);
        Self::normalize_whitespace(&text)
    }

    /// 移除 HTML 標籤（狀態機實作）
    fn remove_tags(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut in_tag = false;
        let mut in_script_or_style = false;
        let mut tag_content = String::new();

        for ch in text.chars() {
            if ch == '<' {
                in_tag = true;
                tag_content.clear();
            } else if ch == '>' {
                // 檢查標籤內容
                let tag_lower = tag_content.to_lowercase();
                if tag_lower.starts_with("script") || tag_lower.starts_with("style") {
                    in_script_or_style = true;
                } else if tag_lower.starts_with("/script") || tag_lower.starts_with("/style") {
                    in_script_or_style = false;
                }
                in_tag = false;
                tag_content.clear();
            } else if in_tag {
                tag_content.push(ch);
            } else if !in_script_or_style {
                result.push(ch);
            }
        }

        result
    }

    /// 正規化空白字元
    fn normalize_whitespace(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_was_space = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }

        result.trim().to_string()
    }

    /// 移除常見的雜訊內容
    fn remove_noise(text: &str) -> String {
        let noise_patterns = [
            "cookie",
            "privacy policy",
            "terms of service",
            "subscribe",
            "newsletter",
            "advertisement",
            "sponsored",
            "click here",
            "read more",
            "share on",
            "follow us",
            "copyright ©",
            "all rights reserved",
        ];

        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            let line_lower = line.to_lowercase();
            let mut is_noise = false;

            for pattern in &noise_patterns {
                if line_lower.contains(pattern) && line.len() < 200 {
                    is_noise = true;
                    break;
                }
            }

            if !is_noise && !line.trim().is_empty() {
                result.push(line);
            }
        }

        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_simple_tags() {
        let html = "<p>Hello <b>world</b>!</p>";
        let result = HtmlCleaner::remove_tags(html);
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_remove_script_tags() {
        let html = "<p>Content</p><script>alert('test');</script><p>More</p>";
        let result = HtmlCleaner::remove_tags(html);
        assert!(!result.contains("alert"));
    }

    #[test]
    fn test_normalize_whitespace() {
        let text = "Hello    world\n\n\ntest";
        let result = HtmlCleaner::normalize_whitespace(text);
        assert_eq!(result, "Hello world test");
    }

    #[test]
    fn test_remove_noise() {
        let text = "Important content\nClick here to subscribe\nMore content";
        let result = HtmlCleaner::remove_noise(text);
        assert!(!result.contains("Click here"));
        assert!(result.contains("Important content"));
    }

    #[test]
    fn test_clean_full() {
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <h1>Main Title</h1>
                    <p>This is   important content.</p>
                    <div>Subscribe to our newsletter</div>
                    <script>console.log('test');</script>
                </body>
            </html>
        "#;

        let result = HtmlCleaner::clean(html);
        assert!(result.contains("Main Title"));
        assert!(result.contains("important content"));
        assert!(!result.contains("console.log"));
    }

    #[test]
    fn test_empty_input() {
        let result = HtmlCleaner::clean("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_html_tags() {
        let text = "Plain text without tags";
        let result = HtmlCleaner::clean(text);
        assert_eq!(result, "Plain text without tags");
    }
}
