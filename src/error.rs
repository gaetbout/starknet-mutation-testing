use derive_more::From;
// use derive_more::{From, Display};

// TODO Should do the Display stuff?
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, PartialEq)] // Display
pub enum Error {
    // -- fs errors
    // Display => Invalid path '{}'
    FsInvalidPath {
        path: String,
    },
    // -- cli errors
    // -- mutation errors
    #[from]
    // #[display("Failed to compile the mutated code")]
    Custom(String),
    Example {
        name: String,
    },
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
