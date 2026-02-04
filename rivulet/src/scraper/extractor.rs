use crate::scraper::ScraperConfig;

/// Content extractor for cleaning and extracting article content from HTML
pub struct ContentExtractor {
    config: ScraperConfig,
}

impl ContentExtractor {
    pub fn new(config: ScraperConfig) -> Self {
        Self { config }
    }

    /// Generate JavaScript to extract content from the page
    ///
    /// This JS runs in the browser context and:
    /// 1. Removes unwanted elements (ads, nav, etc.)
    /// 2. Finds the main content using configured selectors
    /// 3. Returns the cleaned HTML content
    pub fn extraction_script(&self) -> String {
        let remove_selectors = self
            .config
            .remove_selectors
            .iter()
            .map(|s| format!("'{}'", s.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ");

        let content_selectors = self
            .config
            .content_selectors
            .iter()
            .map(|s| format!("'{}'", s.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"
            (() => {{
                // Remove unwanted elements
                const removeSelectors = [{remove_selectors}];
                for (const selector of removeSelectors) {{
                    document.querySelectorAll(selector).forEach(el => el.remove());
                }}

                // Try content selectors in order
                const contentSelectors = [{content_selectors}];
                for (const selector of contentSelectors) {{
                    const element = document.querySelector(selector);
                    if (element && element.innerText.trim().length > 100) {{
                        return {{
                            html: element.innerHTML,
                            text: element.innerText,
                            selector: selector
                        }};
                    }}
                }}

                // Fallback to body
                const body = document.body;
                if (body) {{
                    return {{
                        html: body.innerHTML,
                        text: body.innerText,
                        selector: 'body'
                    }};
                }}

                return {{ html: '', text: '', selector: null }};
            }})()
            "#
        )
    }

    /// Generate JavaScript to block resources for faster loading
    pub fn resource_blocking_script(&self) -> Option<String> {
        if !self.config.block_images && !self.config.block_stylesheets {
            return None;
        }

        let blocked_types = [
            self.config.block_images.then_some("'image'"),
            self.config.block_stylesheets.then_some("'stylesheet'"),
            self.config.block_stylesheets.then_some("'font'"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");

        Some(format!(
            r#"
            const blockedTypes = [{blocked_types}];
            "#
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_script_generation() {
        let config = ScraperConfig::default();
        let extractor = ContentExtractor::new(config);
        let script = extractor.extraction_script();

        assert!(script.contains("removeSelectors"));
        assert!(script.contains("contentSelectors"));
        assert!(script.contains("article"));
    }
}
