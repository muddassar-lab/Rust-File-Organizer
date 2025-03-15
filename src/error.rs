#[derive(Debug)]
pub enum OrganizeError {
    NoPathSelected,
    InvalidFolderName,
    NoParentDirectory,
    DirectoryCreationFailed(String),
    FileCopyFailed(String),
    UserInputError(String),
    InvalidOutputPath(String),
}

impl std::fmt::Display for OrganizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPathSelected => write!(f, "No folder selected"),
            Self::InvalidFolderName => write!(f, "Invalid folder name"),
            Self::NoParentDirectory => write!(f, "Could not determine parent directory"),
            Self::DirectoryCreationFailed(e) => write!(f, "Failed to create directory: {}", e),
            Self::FileCopyFailed(e) => write!(f, "Failed to copy file: {}", e),
            Self::UserInputError(e) => write!(f, "User input error: {}", e),
            Self::InvalidOutputPath(e) => write!(f, "Invalid output path: {}", e),
        }
    }
}