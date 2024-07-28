use std::{fs, io::Write};

pub trait Pohuy<T, E> {
    fn pohuy(&self) {}
}

impl<T, E> Pohuy<T, E> for Result<T, E> {}

pub struct FileOutStream {
    bytes: Vec<u8>,
    bytes_wrote: Vec<u8>,
    file_path: String,
}

impl FileOutStream {
    pub fn new(file_path: String, bytes: Vec<u8>) -> FileOutStream {
        FileOutStream {
            bytes,
            file_path,
            bytes_wrote: Vec::new(),
        }
    }
}

impl Write for FileOutStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.bytes_wrote.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.bytes.write(&self.bytes_wrote)?;
        fs::write(&self.file_path, &self.bytes)
    }
}
