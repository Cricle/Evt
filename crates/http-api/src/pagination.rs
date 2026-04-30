use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

impl PageQuery {
    pub fn normalized(&self) -> (u64, u64) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(20).clamp(1, 100);
        (page, page_size)
    }
}

#[cfg(test)]
mod tests {
    use super::PageQuery;

    #[test]
    fn normalized_applies_defaults() {
        assert_eq!(
            PageQuery {
                page: None,
                page_size: None,
            }
            .normalized(),
            (1, 20)
        );
    }

    #[test]
    fn normalized_clamps_invalid_or_excessive_values() {
        assert_eq!(
            PageQuery {
                page: Some(0),
                page_size: Some(0),
            }
            .normalized(),
            (1, 1)
        );

        assert_eq!(
            PageQuery {
                page: Some(3),
                page_size: Some(500),
            }
            .normalized(),
            (3, 100)
        );
    }
}
