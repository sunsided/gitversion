#[derive(Debug, Clone, Default)]
pub struct MainlineCommit {
    pub sha: String,
    pub previous: Option<Box<MainlineCommit>>,
}

#[derive(Debug, Clone, Default)]
pub struct MainlineIteration {
    pub branch_name: String,
    pub commits: Vec<MainlineCommit>,
}
