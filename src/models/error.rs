use std::{error, fmt::Debug, io::Error as ioError, process::exit, str::Utf8Error};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    context: Option<Context>,
}

// 1 => Program failed to correctly execute
// 2 => Thread panicked, potentially leaving OS resources in a dirty state
// 3 => Program partially parsed data but was closed unexpectedly
impl From<Error> for i32 {
    fn from(err: Error) -> Self {
        match err.kind {
            ErrorKind::ThreadFailed(_) => 2,
            ErrorKind::UnexpectedChannelClose(_) => 3,
            _ => 1,
        }
    }
}

impl Error {
    fn display_with_context(
        err: &ErrorKind,
        con: &Context,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        type E = ErrorKind;
        type C = Context;
        match (err, con) {
            (E::Io(e), C::DataLenEqualLineBufferLen(s)) => {
                write!(f, "Input data size is >= input buffer, which likely lead to this error. Input buffer size is adjustable, try adding 'config --buf_in {}' (error: {})", s, e)
            }
            (e, _) => write!(f, "{}", e)
        }
    }
}

impl<E, C> From<(E, C)> for Error
where
    E: Into<ErrorKind>,
    C: Into<Context>,
{
    fn from((err, context): (E, C)) -> Self {
        Error {
            kind: err.into(),
            context: Some(context.into()),
        }
    }
}

impl<E> From<E> for Error
where
    E: Into<ErrorKind>,
{
    fn from(err: E) -> Self {
        Error {
            kind: err.into(),
            context: None,
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.kind.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.context
            .as_ref()
            .map_or(write!(f, "{}", self.kind), |con| {
                Error::display_with_context(&self.kind, con, f)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    Overide(String),
    DataLenEqualLineBufferLen(usize),
}

impl<T: AsRef<str>> From<T> for Context {
    fn from(s: T) -> Self {
        Context::Overide(s.as_ref().to_string())
    }
}

/// Contains any error emitted by this program
#[derive(Debug)]
pub enum ErrorKind {
    // Catch all
    Generic,
    Message(String),
    // Handles in-thread panics
    ThreadFailed(String),
    // Handles fatal channel closes
    UnexpectedChannelClose(String),
    // Wrapper for any IO / Json serde errors
    Io(ioError),
    // For byte to str casts
    UTF8(Utf8Error),
    // Handles missing fields during output streaming
    MissingField(String),
}

// IO Error => ErrorKind
impl From<ioError> for ErrorKind {
    fn from(err: ioError) -> Self {
        ErrorKind::Io(err)
    }
}

// json Error => IO error => ErrorKind
impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> Self {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io | Category::Data | Category::Syntax | Category::Eof => {
                ErrorKind::Io(err.into())
            }
        }
    }
}

impl From<Utf8Error> for ErrorKind {
    fn from(err: Utf8Error) -> Self {
        ErrorKind::UTF8(err)
    }
}

// E => ErrorKind, where E implements Error
impl From<Box<dyn error::Error>> for ErrorKind {
    fn from(e: Box<dyn error::Error>) -> Self {
        ErrorKind::Message(format!("{}", e))
    }
}

impl From<std::fmt::Error> for ErrorKind {
    fn from(e: std::fmt::Error) -> Self {
        ErrorKind::Message(format!("{}", e))
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorKind::Generic => write!(f, "Generic Error"),
            ErrorKind::Message(m) => write!(f, "{}", m),
            ErrorKind::ThreadFailed(e) => write!(f, "Thread: {} failed to return", e),
            ErrorKind::UnexpectedChannelClose(e) => write!(f, "A channel quit unexpectedly: {}", e),
            ErrorKind::Io(e) => write!(f, "An underlying IO error occurred: {}", e),
            ErrorKind::UTF8(e) => write!(f, "Invalid or incomplete UTF-8: {}", e),
            ErrorKind::MissingField(e) => write!(f, "Missing required field: {}", e),
        }
    }
}

impl error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::UTF8(e) => Some(e),
            _ => None,
        }
    }
}

pub(crate) trait ErrContext<T, E> {
    fn context<C>(self, context: C) -> std::result::Result<T, (E, C)>;
}

impl<T, E> ErrContext<T, E> for std::result::Result<T, E> {
    fn context<C>(self, context: C) -> std::result::Result<T, (E, C)> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err((err, context)),
        }
    }
}

/// Handles program return codes
pub(crate) enum ProgramExit<T>
where
    T: error::Error,
{
    Success,
    Failure(T),
}

impl<T> ProgramExit<T>
where
    T: Into<i32> + Debug + error::Error,
{
    pub fn exit(self) -> ! {
        match self {
            Self::Success => exit(0),
            Self::Failure(err) => {
                error!("Program exited with error: {}", err);
                exit(err.into())
            }
        }
    }
}

impl From<Result<()>> for ProgramExit<Error> {
    fn from(res: Result<()>) -> Self {
        match res {
            Ok(_) => ProgramExit::Success,
            Err(e) => ProgramExit::Failure(e),
        }
    }
}

/*
// Unstable implementation that is much prettier and doesn't require a try_main()
// Once Termination + Try have been stabilized re-enable

// Option::None => ErrorKind
impl From<std::option::NoneError> for ErrorKind {
    fn from(_: std::option::NoneError) -> Self {
        ErrorKind::Generic
    }
}

impl<T: Into<i32> + Debug + Error> Termination for ProgramExit<T> {
    fn report(self) -> i32 {
        match self {
            ProgramExit::Success => 0,
            ProgramExit::Failure(err) => {
                error!("Program exited with error: {}", err);
                err.into()
            }
        }
    }
}

impl<T: Error> Try for ProgramExit<T> {
    type Ok = ();
    type Error = T;

    fn into_result(self) -> std::result::Result<Self::Ok, Self::Error> {
        match self {
            ProgramExit::Success => Ok(()),
            ProgramExit::Failure(err) => Err(err),
        }
    }

    fn from_error(err: Self::Error) -> Self {
        ProgramExit::Failure(err)
    }

    fn from_ok(_: Self::Ok) -> Self {
        ProgramExit::Success
    }
}
*/
