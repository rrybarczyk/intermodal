use crate::common::*;

pub(crate) struct Env {
  args: Vec<OsString>,
  dir: PathBuf,
  pub(crate) err: Box<dyn Write>,
  pub(crate) out: Box<dyn Write>,
  err_style: Style,
  out_style: Style,
  out_is_term: bool,
  err_is_term: bool,
}

impl Env {
  pub(crate) fn main() -> Self {
    let dir = match env::current_dir() {
      Ok(dir) => dir,
      Err(error) => panic!("Failed to get current directory: {}", error),
    };

    let no_color = env::var_os("NO_COLOR").is_some()
      || env::var_os("TERM").as_deref() == Some(OsStr::new("dumb"));

    let err_style = if no_color || !atty::is(atty::Stream::Stderr) {
      Style::inactive()
    } else {
      Style::active()
    };

    let out_style = if no_color || !atty::is(atty::Stream::Stdout) {
      Style::inactive()
    } else {
      Style::active()
    };

    let out_is_term = atty::is(atty::Stream::Stdout);
    let err_is_term = atty::is(atty::Stream::Stderr);

    Self::new(
      dir,
      io::stdout(),
      out_style,
      out_is_term,
      io::stderr(),
      err_style,
      err_is_term,
      env::args(),
    )
  }

  pub(crate) fn run(&mut self) -> Result<(), Error> {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().ok();

    #[cfg(not(test))]
    pretty_env_logger::init();

    let args = Arguments::from_iter_safe(&self.args)?;

    match args.options().use_color {
      UseColor::Always => self.err_style = Style::active(),
      UseColor::Auto => {}
      UseColor::Never => self.err_style = Style::inactive(),
    }

    args.run(self)
  }

  pub(crate) fn new<O, E, S, I>(
    dir: PathBuf,
    out: O,
    out_style: Style,
    out_is_term: bool,
    err: E,
    err_style: Style,
    err_is_term: bool,
    args: I,
  ) -> Self
  where
    O: Write + 'static,
    E: Write + 'static,
    S: Into<OsString>,
    I: IntoIterator<Item = S>,
  {
    Self {
      args: args.into_iter().map(Into::into).collect(),
      err: Box::new(err),
      out: Box::new(out),
      dir,
      out_style,
      out_is_term,
      err_style,
      err_is_term,
    }
  }

  pub(crate) fn status(&mut self) -> Result<(), i32> {
    use structopt::clap::ErrorKind;

    if let Err(error) = self.run() {
      if let Error::Clap { source } = error {
        if source.use_stderr() {
          write!(&mut self.err, "{}", source).ok();
        } else {
          write!(&mut self.out, "{}", source).ok();
        }
        match source.kind {
          ErrorKind::VersionDisplayed | ErrorKind::HelpDisplayed => Ok(()),
          _ => Err(EXIT_FAILURE),
        }
      } else {
        writeln!(
          &mut self.err,
          "{}{}: {}{}",
          self.err_style.error().paint("error"),
          self.err_style.message().prefix(),
          error,
          self.err_style.message().suffix(),
        )
        .ok();

        if let Some(lint) = error.lint() {
          writeln!(
            &mut self.err,
            "{}: This check can be disabled with `--allow {}`.",
            self.err_style.message().paint("note"),
            lint.name()
          )
          .ok();
        }

        Err(EXIT_FAILURE)
      }
    } else {
      Ok(())
    }
  }

  pub(crate) fn dir(&self) -> &Path {
    &self.dir
  }

  pub(crate) fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
    self.dir().join(path).clean()
  }

  pub(crate) fn out_is_term(&self) -> bool {
    self.out_is_term
  }

  pub(crate) fn out_style(&self) -> Style {
    self.out_style
  }

  pub(crate) fn err_is_term(&self) -> bool {
    self.err_is_term
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn error_message_on_stdout() {
    let mut env = test_env! {
      args: [
        "torrent",
        "create",
        "--input",
        "foo",
        "--announce",
        "udp:bar.com",
        "--announce-tier",
        "foo",
      ],
      tree: {
        foo: "",
      }
    };
    env.status().ok();
    let err = env.err();
    if !err.starts_with("error: Failed to parse announce URL:") {
      panic!("Unexpected standard error output: {}", err);
    }

    assert_eq!(env.out(), "");
  }
}
