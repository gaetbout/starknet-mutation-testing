use crate::file_manager::{change_line_content, copy_dir_all};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum MutationType {
    Equal,
    NotEqual,
    GreaterThan, // TODO Should I just add all 3 other mutations for each greaterThan, greaterThanOrEqual, lessThan, lessThanOrEqual and even ==?
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(Debug)]
pub enum MutationResult {
    Success(Mutation),
    BuildFailure(Mutation),
    Failure(Mutation),
}

#[derive(Debug)]
pub struct Mutation {
    from: MutationType,
    to: MutationType,
    file_name: PathBuf,
    line: String,
    pos: usize,
}

impl fmt::Display for Mutation {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}\n\t\"{}\" updated to \"{}\" ",
            self.file_name.to_str().unwrap(),
            self.pos + 1,
            self.from.as_str(),
            self.to.as_str()
        )
    }
}

impl fmt::Display for MutationResult {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MutationResult::Success(mutation) => panic!("Success should not be printed"),
            MutationResult::BuildFailure(mutation) => write!(f, "Build failure: {}", mutation),
            MutationResult::Failure(mutation) => write!(f, "{}", mutation),
        }
    }
}

impl MutationType {
    fn as_str(&self) -> &str {
        match self {
            MutationType::Equal => "==",
            MutationType::NotEqual => "!=",
            MutationType::GreaterThan => " > ", // TODO fix: should ignore if it is "->"
            MutationType::GreaterThanOrEqual => ">=",
            MutationType::LessThan => " < ",
            MutationType::LessThanOrEqual => "<=",
        }
    }

    pub fn others(&self, file_name: PathBuf, line: String, pos: usize) -> Vec<Mutation> {
        if !line.contains(self.as_str()) {
            return vec![];
        }
        match self {
            MutationType::Equal => vec![Mutation {
                from: self.clone(),
                to: MutationType::NotEqual,
                file_name,
                line,
                pos,
            }],
            MutationType::NotEqual => vec![Mutation {
                from: self.clone(),
                to: MutationType::Equal,
                file_name,
                line,
                pos,
            }],
            MutationType::GreaterThan => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::GreaterThanOrEqual,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::LessThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
            MutationType::GreaterThanOrEqual => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::Equal,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::GreaterThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
            MutationType::LessThan => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::LessThanOrEqual,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::GreaterThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
            MutationType::LessThanOrEqual => {
                vec![
                    Mutation {
                        from: self.clone(),
                        to: MutationType::Equal,
                        file_name: file_name.clone(),
                        line: line.clone(),
                        pos,
                    },
                    Mutation {
                        from: self.clone(),
                        to: MutationType::LessThan,
                        file_name,
                        line,
                        pos,
                    },
                ]
            }
        }
    }
}

impl Mutation {
    pub fn apply_mutation(&self, path_src: &Path, path_dst: &Path) {
        copy_dir_all(path_src, path_dst, &["cairo", "toml", "lock"])
            .expect("Couldn't copy test data");

        // Mutation from as fn
        let new_line = self.line.replace(self.from.as_str(), self.to.as_str());

        let file_dst = path_dst.join(self.file_name.clone());
        change_line_content(&file_dst, self.pos + 1, &new_line).expect("Error applying mutation");
    }
}
