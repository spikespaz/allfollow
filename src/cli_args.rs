use std::convert::Infallible;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, StdinLock, StdoutLock, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Stdin,
    File(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Output {
    #[default]
    Stdout,
    File(PathBuf),
}

#[derive(Debug)]
pub enum InputReader<'a> {
    Stdin(StdinLock<'a>),
    File(BufReader<File>),
}

#[derive(Debug)]
pub enum OutputWriter<'a> {
    Stdout(StdoutLock<'a>),
    File(BufWriter<File>),
}

impl Input {
    pub fn from_arg(arg: impl Into<PathBuf>) -> Self {
        let path = arg.into();
        if path == Path::new("-") {
            Self::Stdin
        } else {
            Self::File(path)
        }
    }

    pub fn open(&self) -> io::Result<InputReader> {
        match self {
            Self::Stdin => Ok(InputReader::Stdin(io::stdin().lock())),
            Self::File(path) => Ok(InputReader::File(BufReader::new(File::open(path)?))),
        }
    }
}

impl Output {
    pub fn from_arg(arg: impl Into<PathBuf>) -> Self {
        let path = arg.into();
        if path == Path::new("-") {
            Self::Stdout
        } else {
            Self::File(path)
        }
    }

    // Create or open a file for writing.
    // Set `new` if you would like to error if the file already exists.
    pub fn create(&self, new: bool) -> io::Result<OutputWriter> {
        match self {
            Self::Stdout => Ok(OutputWriter::Stdout(io::stdout().lock())),
            Self::File(path) => Ok(OutputWriter::File(BufWriter::new(
                File::options()
                    .write(true)
                    .create(!new)
                    .truncate(!new)
                    .create_new(new)
                    .open(path)?,
            ))),
        }
    }
}

impl<P: Into<PathBuf>> From<P> for Input {
    fn from(value: P) -> Self {
        Self::from_arg(value)
    }
}

impl<P: Into<PathBuf>> From<P> for Output {
    fn from(value: P) -> Self {
        Self::from_arg(value)
    }
}

impl FromStr for Input {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_arg(s))
    }
}

impl FromStr for Output {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_arg(s))
    }
}

impl From<Input> for Output {
    fn from(value: Input) -> Self {
        match value {
            Input::Stdin => Output::Stdout,
            Input::File(path) => Output::File(path),
        }
    }
}

impl<'a> Read for InputReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(lock) => lock.read(buf),
            Self::File(file) => file.read(buf),
        }
    }
}

impl<'a> Write for OutputWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(lock) => lock.write(buf),
            Self::File(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout(lock) => lock.flush(),
            Self::File(file) => file.flush(),
        }
    }
}
