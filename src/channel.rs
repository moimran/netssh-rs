use crate::error::NetmikoError;
use ssh2::Channel as SSH2Channel;
use std::io::{Read, Write};

pub struct Channel {
    inner: SSH2Channel,
}

impl Channel {
    pub fn new(channel: SSH2Channel) -> Self {
        Self { inner: channel }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetmikoError> {
        Ok(self.inner.read(buf)?)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, NetmikoError> {
        Ok(self.inner.write(data)?)
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<(), NetmikoError> {
        Ok(self.inner.write_all(data)?)
    }

    pub fn flush(&mut self) -> Result<(), NetmikoError> {
        Ok(self.inner.flush()?)
    }
}

impl Read for Channel {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl Write for Channel {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.write(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
