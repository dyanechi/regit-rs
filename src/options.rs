pub enum GitServer {
    GitHub,
    GitLab,
    BitBucket,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ValidModes {
    #[default]
    Tar,
    Git,
}