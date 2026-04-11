use bitflags::bitflags;

#[derive(Debug, Clone, Default)]
pub struct CommitFilter {
    pub max_count: Option<usize>,
    pub since_sha: Option<String>,
    pub until_sha: Option<String>,
    pub reverse: bool,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CommitSortStrategies: u32 {
        const TIME = 1;
        const TOPOLOGICAL = 2;
        const REVERSE = 4;
    }
}
