use std::fmt;
use std::io::{self, BufWriter, IoSlice, IsTerminal, Stderr, Stdout, Write, stderr, stdout};

pub(crate) enum StreamImpl {
    Stdout(Stdout),
    Stderr(Stderr),
}

pub struct AutoStream(pub(crate) StreamImpl);

impl Default for AutoStream {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoStream {
    pub fn new() -> Self {
        let stdout = stdout();
        let stderr = stderr();
        if !stdout.is_terminal() && stderr.is_terminal() {
            Self(StreamImpl::Stderr(stderr))
        } else {
            Self(StreamImpl::Stdout(stdout))
        }
    }
}

impl Write for AutoStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.0 {
            StreamImpl::Stdout(s) => s.write(buf),
            StreamImpl::Stderr(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.0 {
            StreamImpl::Stdout(s) => s.flush(),
            StreamImpl::Stderr(s) => s.flush(),
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        match &mut self.0 {
            StreamImpl::Stdout(s) => s.write_vectored(bufs),
            StreamImpl::Stderr(s) => s.write_vectored(bufs),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match &mut self.0 {
            StreamImpl::Stdout(s) => s.write_all(buf),
            StreamImpl::Stderr(s) => s.write_all(buf),
        }
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        match &mut self.0 {
            StreamImpl::Stdout(s) => s.write_fmt(args),
            StreamImpl::Stderr(s) => s.write_fmt(args),
        }
    }
}

#[cfg(unix)]
impl std::os::fd::AsFd for AutoStream {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        match &self.0 {
            StreamImpl::Stdout(s) => s.as_fd(),
            StreamImpl::Stderr(s) => s.as_fd(),
        }
    }
}

#[cfg(unix)]
impl std::os::fd::AsRawFd for AutoStream {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        match &self.0 {
            StreamImpl::Stdout(s) => s.as_raw_fd(),
            StreamImpl::Stderr(s) => s.as_raw_fd(),
        }
    }
}

pub enum MaybeBuffered<W>
where
    W: io::Write,
{
    Buffered(BufWriter<W>),
    Unbuffered(W),
}

impl<W> io::Write for MaybeBuffered<W>
where
    W: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Buffered(s) => s.write(buf),
            Self::Unbuffered(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Buffered(s) => s.flush(),
            Self::Unbuffered(s) => s.flush(),
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        match self {
            Self::Buffered(s) => s.write_vectored(bufs),
            Self::Unbuffered(s) => s.write_vectored(bufs),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self {
            Self::Buffered(s) => s.write_all(buf),
            Self::Unbuffered(s) => s.write_all(buf),
        }
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        match self {
            Self::Buffered(s) => s.write_fmt(args),
            Self::Unbuffered(s) => s.write_fmt(args),
        }
    }
}
