use crate::io;
/// I/O compatibility extensions for `no_std` support.
///
/// In `no_std` mode, `no_std_io` doesn't provide `Write` for `Cursor<Vec<u8>>`,
/// so we provide a `WritableCursor` wrapper that implements `Write`.
use alloc::vec::Vec;

/// A cursor wrapping a `Vec<u8>` that implements `io::Write`.
///
/// In `std` mode this is just a `io::Cursor<Vec<u8>>`. In `no_std` mode
/// this is a newtype providing the `Write` impl that `no_std_io` lacks.
#[cfg(feature = "std")]
pub(crate) type WritableCursor = io::Cursor<Vec<u8>>;

#[cfg(not(feature = "std"))]
pub(crate) struct WritableCursor {
    inner: Vec<u8>,
    pos: u64,
}

#[cfg(not(feature = "std"))]
impl WritableCursor {
    pub(crate) fn new(inner: Vec<u8>) -> Self {
        Self { inner, pos: 0 }
    }

    pub(crate) fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

#[cfg(not(feature = "std"))]
impl io::Write for WritableCursor {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pos = self.pos as usize;

        // If position is past the end, fill with zeros
        if pos > self.inner.len() {
            self.inner.resize(pos, 0);
        }

        if pos + buf.len() > self.inner.len() {
            self.inner.resize(pos + buf.len(), 0);
        }
        self.inner[pos..pos + buf.len()].copy_from_slice(buf);
        self.pos = (pos + buf.len()) as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(not(feature = "std"))]
impl io::Seek for WritableCursor {
    fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
        let (base, offset) = match style {
            io::SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            io::SeekFrom::End(n) => (self.inner.len() as u64, n),
            io::SeekFrom::Current(n) => (self.pos, n),
        };
        match base.checked_add_signed(offset) {
            Some(n) => {
                self.pos = n;
                Ok(n)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }
}
