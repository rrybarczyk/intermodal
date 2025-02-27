use crate::common::*;

use structopt::clap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("Must provide at least one announce URL"))]
  AnnounceEmpty,
  #[snafu(display("Failed to parse announce URL: {}", source))]
  AnnounceUrlParse { source: url::ParseError },
  #[snafu(display("Failed to deserialize torrent metainfo from `{}`: {}", path.display(), source))]
  MetainfoLoad {
    source: bendy::serde::Error,
    path: PathBuf,
  },
  #[snafu(display("Failed to serialize torrent metainfo: {}", source))]
  MetainfoSerialize { source: bendy::serde::Error },
  #[snafu(display("Failed to parse byte count `{}`: {}", text, source))]
  ByteParse {
    text: String,
    source: ParseFloatError,
  },
  #[snafu(display("Failed to parse byte count `{}`, invalid suffix: `{}`", text, suffix))]
  ByteSuffix { text: String, suffix: String },
  #[snafu(display("{}", source))]
  Clap { source: clap::Error },
  #[snafu(display("Failed to invoke command `{}`: {}", command, source,))]
  CommandInvoke { command: String, source: io::Error },
  #[snafu(display("Command `{}` returned bad exit status: {}", command, status))]
  CommandStatus { command: String, status: ExitStatus },
  #[snafu(display("Filename was not valid unicode: {}", filename.display()))]
  FilenameDecode { filename: PathBuf },
  #[snafu(display("Path had no file name: {}", path.display()))]
  FilenameExtract { path: PathBuf },
  #[snafu(display("I/O error at `{}`: {}", path.display(), source))]
  Filesystem { source: io::Error, path: PathBuf },
  #[snafu(display("Invalid glob: {}", source))]
  GlobParse { source: globset::Error },
  #[snafu(display("Unknown lint: {}", text))]
  LintUnknown { text: String },
  #[snafu(display("DHT node port missing: {}", text))]
  NodeParsePortMissing { text: String },
  #[snafu(display("Failed to parse DHT node host `{}`: {}", text, source))]
  NodeParseHost {
    text: String,
    source: url::ParseError,
  },
  #[snafu(display("Failed to parse DHT node port `{}`: {}", text, source))]
  NodeParsePort { text: String, source: ParseIntError },
  #[snafu(display("Failed to find opener utility, please install one of {}", tried.join(",")))]
  OpenerMissing { tried: &'static [&'static str] },
  #[snafu(display("Output path already exists: `{}`", path.display()))]
  OutputExists { path: PathBuf },
  #[snafu(display(
    "Interal error, this may indicate a bug in intermodal: {}\n\
     Consider filing an issue: https://github.com/casey/imdl/issues/new",
    message,
  ))]
  Internal { message: String },
  #[snafu(display(
    "Path `{}` contains non-normal component: {}",
    path.display(),
    component.display(),
  ))]
  PathComponent { component: PathBuf, path: PathBuf },
  #[snafu(display(
    "Path `{}` contains non-unicode component: {}",
    path.display(),
    component.display(),
  ))]
  PathDecode { path: PathBuf, component: PathBuf },
  #[snafu(display(
    "Path `{}` empty after stripping prefix `{}`",
    path.display(),
    prefix.display(),
  ))]
  PathStripEmpty { path: PathBuf, prefix: PathBuf },
  #[snafu(display(
    "Failed to strip prefix `{}` from path `{}`: {}",
    prefix.display(),
    path.display(),
    source
  ))]
  PathStripPrefix {
    path: PathBuf,
    prefix: PathBuf,
    source: path::StripPrefixError,
  },
  #[snafu(display(
    "Piece length `{}` too large. The maximum supported piece length is {}.",
    bytes,
    Bytes(u32::max_value().into())
  ))]
  PieceLengthTooLarge {
    bytes: Bytes,
    source: TryFromIntError,
  },
  #[snafu(display("Piece length `{}` is not an even power of two", bytes))]
  PieceLengthUneven { bytes: Bytes },
  #[snafu(display("Piece length must be at least 16 KiB"))]
  PieceLengthSmall,
  #[snafu(display("Piece length cannot be zero"))]
  PieceLengthZero,
  #[snafu(display("Failed to write to standard error: {}", source))]
  Stderr { source: io::Error },
  #[snafu(display("Failed to write to standard output: {}", source))]
  Stdout { source: io::Error },
  #[snafu(display(
      "Attempted to create torrent from symlink `{}`. To override, pass the \
      `--follow-symlinks` flag.",
      root.display()
  ))]
  SymlinkRoot { root: PathBuf },
  #[snafu(display("Failed to retrieve system time: {}", source))]
  SystemTime { source: SystemTimeError },
  #[snafu(display(
    "Feature `{}` cannot be used without passing the `--unstable` flag",
    feature
  ))]
  Unstable { feature: &'static str },
  #[snafu(display("Torrent verification failed: {}", status))]
  Verify { status: Status },
}

impl Error {
  pub(crate) fn lint(&self) -> Option<Lint> {
    match self {
      Self::PieceLengthUneven { .. } => Some(Lint::UnevenPieceLength),
      Self::PieceLengthSmall { .. } => Some(Lint::SmallPieceLength),
      _ => None,
    }
  }

  pub(crate) fn internal(message: impl Into<String>) -> Error {
    Error::Internal {
      message: message.into(),
    }
  }
}

impl From<clap::Error> for Error {
  fn from(source: clap::Error) -> Self {
    Self::Clap { source }
  }
}

impl From<globset::Error> for Error {
  fn from(source: globset::Error) -> Self {
    Self::GlobParse { source }
  }
}

impl From<SystemTimeError> for Error {
  fn from(source: SystemTimeError) -> Self {
    Self::SystemTime { source }
  }
}

impl From<walkdir::Error> for Error {
  fn from(walkdir_error: walkdir::Error) -> Self {
    let path = walkdir_error.path().unwrap().to_owned();

    if let Some(source) = walkdir_error.into_io_error() {
      Self::Filesystem { source, path }
    } else {
      unreachable!("Encountered unexpected walkdir error")
    }
  }
}
