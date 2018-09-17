pub mod sd;

use std::io;
use std::path::Path;

pub use fat32::traits;
use fat32::vfat::{self, Shared, VFat};

use self::sd::Sd;
use pi::mutex::Mutex;

pub struct FileSystem(Mutex<Option<Shared<VFat>>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub fn initialize(&self) {
        *self.0.lock() = Some(
            VFat::from(Sd::new().expect("Sd failed to initialize"))
                .expect("VFat failed to initalize from Sd"),
        );
    }
}

impl traits::FileSystem for FileSystem {
    type File = vfat::File;
    type Dir = vfat::Dir;
    type Entry = vfat::Entry;

    fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Self::Entry> {
        if self.0.lock().is_none() {
            (&self).initialize();
        }
        self.0.lock().as_ref().unwrap().open(path)
    }
    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only raspberry")
    }
    fn create_dir<P: AsRef<Path>>(self, _path: P, _parents: bool) -> io::Result<Self::Dir> {
        unimplemented!("read only raspberry")
    }
    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(self, _from: P, _to: Q) -> io::Result<()> {
        unimplemented!("read only raspberry")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only raspberry")
    }
}
