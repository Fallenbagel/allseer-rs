pub enum SearchType {
    Issues,
    PullRequests,
}

pub enum SearchStatus {
    Open,
    Closed,
}

impl SearchType {
    pub fn as_str(&self) -> &str {
        match self {
            SearchType::Issues => "issues",
            SearchType::PullRequests => "pull requests",
        }
    }
}

impl SearchStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SearchStatus::Open => "open",
            SearchStatus::Closed => "closed",
        }
    }
}
