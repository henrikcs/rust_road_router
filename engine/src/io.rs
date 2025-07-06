//! Utilities for reading and writing data structures from and to disk.
//!
//! This module contains a few traits and blanket implementations
//! for (de)serializing and writing/reading data to/from the disc.
//! To use it you should import the `Load` and `Store` traits and use the
//! `load_from` and `write_to` methods.
//!
//! # Example
//!
//! ```no_run
//! # use rust_road_router::io::*;
//!
//! let head = Vec::<u32>::load_from("head_file_name")?;
//! let lat = Vec::<f32>::load_from("node_latitude_file_name")?;
//! head.write_to(&"output_file")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::{
    ffi::OsStr,
    fs::{metadata, File},
    io::{prelude::*, Result},
    mem,
    path::Path,
    slice,
};

/// A trait which allows accessing the data of an object as a slice of bytes.
/// The bytes should represent a serialization of the object and allow
/// recreating it when reading these bytes again from the disk.
///
/// Do not use this Trait but rather the `Store` trait.
pub trait DataBytes {
    /// Should return the serialized object as a slice of bytes
    fn data_bytes(&self) -> &[u8];
}

/// A trait which mutably exposes the internal data of an object so that
/// a serialized object can be loaded from disk and written back into a precreated
/// object of the right size.
///
/// Do not use this Trait but rather the `Load` trait.
pub trait DataBytesMut {
    /// Should return a mutable slice of the internal data of the object
    fn data_bytes_mut(&mut self) -> &mut [u8];
}

impl<T: Copy> DataBytes for [T] {
    fn data_bytes(&self) -> &[u8] {
        let num_bytes = self.len() * mem::size_of::<T>();
        unsafe { slice::from_raw_parts(self.as_ptr() as *const u8, num_bytes) }
    }
}

impl<T: Copy> DataBytes for &[T] {
    fn data_bytes(&self) -> &[u8] {
        self[..].data_bytes()
    }
}

impl<T: Copy> DataBytes for Vec<T> {
    fn data_bytes(&self) -> &[u8] {
        self[..].data_bytes()
    }
}

impl<T: Copy> DataBytes for Box<[T]> {
    fn data_bytes(&self) -> &[u8] {
        self[..].data_bytes()
    }
}

impl<T: Copy> DataBytesMut for [T] {
    fn data_bytes_mut(&mut self) -> &mut [u8] {
        let num_bytes = self.len() * mem::size_of::<T>();
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, num_bytes) }
    }
}

impl<T: Copy> DataBytesMut for Vec<T> {
    fn data_bytes_mut(&mut self) -> &mut [u8] {
        (&mut self[..]).data_bytes_mut()
    }
}

impl<T: Copy> DataBytesMut for Box<[T]> {
    fn data_bytes_mut(&mut self) -> &mut [u8] {
        (&mut self[..]).data_bytes_mut()
    }
}

/// A trait which extends the `DataBytes` trait and exposes a method to write objects to disk.
pub trait Store: DataBytes {
    /// Writes the serialized object to the file with the given path
    fn write_to(&self, path: &dyn AsRef<Path>) -> Result<()> {
        File::create(path)?.write_all(self.data_bytes())
    }
}

impl<T: DataBytes> Store for T {}
impl<T> Store for [T] where [T]: DataBytes {}

/// A trait to load serialized data back into objects.
pub trait Load: DataBytesMut + Sized {
    /// This method must create an object of the correct size for serialized data with the given number of bytes.
    /// It should not be necessary to call this method directly.
    fn new_with_bytes(num_bytes: usize) -> Self;

    /// This method will load serialized data from the disk, create an object of the appropriate size,
    /// deserialize the bytes into the object and return the object.
    fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let metadata = metadata(path.as_ref())?;
        let mut file = File::open(path)?;

        let mut object = Self::new_with_bytes(metadata.len() as usize);
        assert_eq!(metadata.len() as usize, object.data_bytes_mut().len());
        file.read_exact(object.data_bytes_mut())?;

        Ok(object)
    }
}

impl<T: Default + Copy> Load for Vec<T> {
    fn new_with_bytes(num_bytes: usize) -> Self {
        assert_eq!(num_bytes % mem::size_of::<T>(), 0);
        let num_elements = num_bytes / mem::size_of::<T>();
        (0..num_elements).map(|_| T::default()).collect()
    }
}

impl<T: Default + Copy> Load for Box<[T]> {
    fn new_with_bytes(num_bytes: usize) -> Self {
        assert_eq!(num_bytes % mem::size_of::<T>(), 0);
        let num_elements = num_bytes / mem::size_of::<T>();
        (0..num_elements).map(|_| T::default()).collect()
    }
}

/// A trait to allow serializing more complex objects
/// which need more than a single file.
pub trait Deconstruct: Sized {
    /// Will be called indirectly and should call the `store_callback` for each file that should be written to disk.
    /// The first param of the callback is a name to identify the file, the second param the data to be stored.
    fn save_each(&self, store_callback: &dyn Fn(&str, &dyn Save) -> Result<()>) -> Result<()>;

    /// Call with a directory arg to store this object in this directory.
    fn deconstruct_to(&self, dir: &dyn AsRef<Path>) -> Result<()> {
        if !dir.as_ref().exists() {
            std::fs::create_dir(dir)?;
        }
        self.save_each(&|name, object: &dyn Save| object.save(&dir.as_ref().join(name)))
    }
}

pub trait Save {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<()>;
}

impl<T: Store> Save for T {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<()> {
        self.write_to(path)
    }
}

pub struct Sub<'a, T>(pub &'a T);
impl<T: Deconstruct> Save for Sub<'_, T> {
    fn save(&self, path: &dyn AsRef<Path>) -> Result<()> {
        self.0.deconstruct_to(path)
    }
}

/// Helper struct for loading multiple objects back from disk.
/// Basically used as a callback for each object to load.
pub struct Loader<'a> {
    path: &'a Path,
}

impl<'a> Loader<'a> {
    /// Call this method for each file that should be loaded back from disk.
    /// The path param should be the same name that was used with the `store_each` callback.
    /// Will return the deserialized data.
    pub fn load<T: Load, P: AsRef<Path>>(&self, path: P) -> Result<T> {
        T::load_from(self.path.join(path))
    }

    pub fn reconstruct<T: Reconstruct, P: AsRef<Path>>(&self, path: P) -> Result<T> {
        T::reconstruct_from(&self.path.join(path))
    }

    pub fn reconstruct_prepared<T, R: ReconstructPrepared<T>, P: AsRef<Path>>(&self, path: P, prep: R) -> Result<T> {
        prep.reconstruct_from(&self.path.join(path))
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}

/// A trait to allow deserializing more complex objects of a different type `T` (similar to `Reconstruct`).
/// This can be used to prepare some data in an object of the type implementing this trait and then loading
/// the rest of the data from the disk to create the `T` object.
pub trait ReconstructPrepared<T: Sized>: Sized {
    /// Will be called indirectly and should use the loader passed along to load all the necessary objects back.
    /// Will consume the current object.
    /// Should return the full deserialized object of type `T`.
    fn reconstruct_with(self, loader: Loader) -> Result<T>;

    /// Call with a directory arg to reconstruct an object from this directory.
    fn reconstruct_from<D: AsRef<OsStr>>(self, dir: &D) -> Result<T> {
        let path = Path::new(dir);
        self.reconstruct_with(Loader { path })
    }
}

/// A trait to allow deserializing more complex objects which need more than a single file.
pub trait Reconstruct: Sized {
    /// Will be called indirectly and should use the loader passed along to load all the necessary objects back.
    /// Should return the full deserialized object.
    fn reconstruct_with(loader: Loader) -> Result<Self>;

    /// Call with a directory arg to reconstruct an object from this directory.
    fn reconstruct_from<D: AsRef<OsStr>>(dir: &D) -> Result<Self> {
        let path = Path::new(dir);
        Self::reconstruct_with(Loader { path })
    }
}

/// Helper function to write a vector of strings to a file, each string on a new line.
/// We need this method to keep track of the edge ids given by sumo
/// We cannot let Vec<String> implement Store, because it rust says that in future versions String might implement Copy
pub fn write_strings_to_file<P: AsRef<Path>>(path: P, strings: &Vec<&String>) -> Result<()> {
    let mut file = File::create(path)?;
    for s in strings {
        writeln!(file, "{}", s)?;
    }
    Ok(())
}

/// Helper function to read a vector of strings from a file, each line as a separate string.
/// We need this to convert the edge indices back to the edge ids given by sumo.
/// We cannot let Vec<String> implement Load, because it rust says that in future versions String might implement Copy
pub fn read_strings_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    reader.lines().collect::<Result<Vec<_>>>()
}
