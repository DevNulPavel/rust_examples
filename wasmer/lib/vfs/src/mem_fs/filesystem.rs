//! This module contains the [`FileSystem`] type itself.

use super::*;
use crate::{DirEntry, FileType, FsError, Metadata, OpenOptions, ReadDir, Result};
use slab::Slab;
use std::convert::identity;
use std::ffi::OsString;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

/// The in-memory file system!
///
/// It's a thin wrapper around [`FileSystemInner`]. This `FileSystem`
/// type can be cloned, it's a light copy of the `FileSystemInner`
/// (which is behind a `Arc` + `RwLock`.
#[derive(Clone, Default)]
pub struct FileSystem {
    pub(super) inner: Arc<RwLock<FileSystemInner>>,
}

impl crate::FileSystem for FileSystem {
    fn read_dir(&self, path: &Path) -> Result<ReadDir> {
        // Read lock.
        let fs = self.inner.try_read().map_err(|_| FsError::Lock)?;

        // Canonicalize the path.
        let (path, inode_of_directory) = fs.canonicalize(path)?;

        // Check it's a directory and fetch the immediate children as `DirEntry`.
        let children = match fs.storage.get(inode_of_directory) {
            Some(Node::Directory { children, .. }) => children
                .iter()
                .filter_map(|inode| fs.storage.get(*inode))
                .map(|node| DirEntry {
                    path: {
                        let mut entry_path = path.to_path_buf();
                        entry_path.push(node.name());

                        entry_path
                    },
                    metadata: Ok(node.metadata().clone()),
                })
                .collect(),

            _ => return Err(FsError::InvalidInput),
        };

        Ok(ReadDir::new(children))
    }

    fn create_dir(&self, path: &Path) -> Result<()> {
        let (inode_of_parent, name_of_directory) = {
            // Read lock.
            let fs = self.inner.try_read().map_err(|_| FsError::Lock)?;

            // Canonicalize the path without checking the path exists,
            // because it's about to be created.
            let path = fs.canonicalize_without_inode(path)?;

            // Check the path has a parent.
            let parent_of_path = path.parent().ok_or(FsError::BaseNotDirectory)?;

            // Check the directory name.
            let name_of_directory = path
                .file_name()
                .ok_or(FsError::InvalidInput)?
                .to_os_string();

            // Find the parent inode.
            let inode_of_parent = fs.inode_of_parent(parent_of_path)?;

            (inode_of_parent, name_of_directory)
        };

        {
            // Write lock.
            let mut fs = self.inner.try_write().map_err(|_| FsError::Lock)?;

            // Creating the directory in the storage.
            let inode_of_directory = fs.storage.vacant_entry().key();
            let real_inode_of_directory = fs.storage.insert(Node::Directory {
                inode: inode_of_directory,
                name: name_of_directory,
                children: Vec::new(),
                metadata: {
                    let time = time();

                    Metadata {
                        ft: FileType {
                            dir: true,
                            ..Default::default()
                        },
                        accessed: time,
                        created: time,
                        modified: time,
                        len: 0,
                    }
                },
            });

            assert_eq!(
                inode_of_directory, real_inode_of_directory,
                "new directory inode should have been correctly calculated",
            );

            // Adding the new directory to its parent.
            fs.add_child_to_node(inode_of_parent, inode_of_directory)?;
        }

        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        let (inode_of_parent, position, inode_of_directory) = {
            // Read lock.
            let fs = self.inner.try_read().map_err(|_| FsError::Lock)?;

            // Canonicalize the path.
            let (path, _) = fs.canonicalize(path)?;

            // Check the path has a parent.
            let parent_of_path = path.parent().ok_or(FsError::BaseNotDirectory)?;

            // Check the directory name.
            let name_of_directory = path
                .file_name()
                .ok_or(FsError::InvalidInput)?
                .to_os_string();

            // Find the parent inode.
            let inode_of_parent = fs.inode_of_parent(parent_of_path)?;

            // Get the child index to remove in the parent node, in
            // addition to the inode of the directory to remove.
            let (position, inode_of_directory) = fs
                .from_parent_get_position_and_inode_of_directory(
                    inode_of_parent,
                    &name_of_directory,
                    DirectoryMustBeEmpty::Yes,
                )?;

            (inode_of_parent, position, inode_of_directory)
        };

        {
            // Write lock.
            let mut fs = self.inner.try_write().map_err(|_| FsError::Lock)?;

            // Remove the directory from the storage.
            fs.storage.remove(inode_of_directory);

            // Remove the child from the parent directory.
            fs.remove_child_from_node(inode_of_parent, position)?;
        }

        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let ((position_of_from, inode, inode_of_from_parent), (inode_of_to_parent, name_of_to)) = {
            // Read lock.
            let fs = self.inner.try_read().map_err(|_| FsError::Lock)?;

            let from = fs.canonicalize_without_inode(from)?;
            let to = fs.canonicalize_without_inode(to)?;

            // Check the paths have parents.
            let parent_of_from = from.parent().ok_or(FsError::BaseNotDirectory)?;
            let parent_of_to = to.parent().ok_or(FsError::BaseNotDirectory)?;

            // Check the names.
            let name_of_from = from
                .file_name()
                .ok_or(FsError::InvalidInput)?
                .to_os_string();
            let name_of_to = to.file_name().ok_or(FsError::InvalidInput)?.to_os_string();

            // Find the parent inodes.
            let inode_of_from_parent = fs.inode_of_parent(parent_of_from)?;
            let inode_of_to_parent = fs.inode_of_parent(parent_of_to)?;

            // Get the child indexes to update in the parent nodes, in
            // addition to the inode of the directory to update.
            let (position_of_from, inode) = fs
                .from_parent_get_position_and_inode(inode_of_from_parent, &name_of_from)?
                .ok_or(FsError::NotAFile)?;

            (
                (position_of_from, inode, inode_of_from_parent),
                (inode_of_to_parent, name_of_to),
            )
        };

        {
            // Write lock.
            let mut fs = self.inner.try_write().map_err(|_| FsError::Lock)?;

            // Update the file name, and update the modified time.
            fs.update_node_name(inode, name_of_to)?;

            // The parents are different. Let's update them.
            if inode_of_from_parent != inode_of_to_parent {
                // Remove the file from its parent, and update the
                // modified time.
                fs.remove_child_from_node(inode_of_from_parent, position_of_from)?;

                // Add the file to its new parent, and update the modified
                // time.
                fs.add_child_to_node(inode_of_to_parent, inode)?;
            }
            // Otherwise, we need to at least update the modified time of the parent.
            else {
                match fs.storage.get_mut(inode_of_from_parent) {
                    Some(Node::Directory {
                        metadata: Metadata { modified, .. },
                        ..
                    }) => *modified = time(),
                    _ => return Err(FsError::UnknownError),
                }
            }
        }

        Ok(())
    }

    fn metadata(&self, path: &Path) -> Result<Metadata> {
        // Read lock.
        let fs = self.inner.try_read().map_err(|_| FsError::Lock)?;

        Ok(fs
            .storage
            .get(fs.inode_of(path)?)
            .ok_or(FsError::UnknownError)?
            .metadata()
            .clone())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let (inode_of_parent, position, inode_of_file) = {
            // Read lock.
            let fs = self.inner.try_read().map_err(|_| FsError::Lock)?;

            // Canonicalize the path.
            let path = fs.canonicalize_without_inode(path)?;

            // Check the path has a parent.
            let parent_of_path = path.parent().ok_or(FsError::BaseNotDirectory)?;

            // Check the file name.
            let name_of_file = path
                .file_name()
                .ok_or(FsError::InvalidInput)?
                .to_os_string();

            // Find the parent inode.
            let inode_of_parent = fs.inode_of_parent(parent_of_path)?;

            // Find the inode of the file if it exists, along with its position.
            let maybe_position_and_inode_of_file =
                fs.from_parent_get_position_and_inode_of_file(inode_of_parent, &name_of_file)?;

            match maybe_position_and_inode_of_file {
                Some((position, inode_of_file)) => (inode_of_parent, position, inode_of_file),
                None => return Err(FsError::NotAFile),
            }
        };

        {
            // Write lock.
            let mut fs = self.inner.try_write().map_err(|_| FsError::Lock)?;

            // Remove the file from the storage.
            fs.storage.remove(inode_of_file);

            // Remove the child from the parent directory.
            fs.remove_child_from_node(inode_of_parent, position)?;
        }

        Ok(())
    }

    fn new_open_options(&self) -> OpenOptions {
        OpenOptions::new(Box::new(FileOpener {
            filesystem: self.clone(),
        }))
    }
}

impl fmt::Debug for FileSystem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fs: &FileSystemInner = &self.inner.read().unwrap();

        fs.fmt(formatter)
    }
}

/// The core of the file system. It contains a collection of `Node`s,
/// indexed by their respective `Inode` in a slab.
pub(super) struct FileSystemInner {
    pub(super) storage: Slab<Node>,
}

impl FileSystemInner {
    /// Get the inode associated to a path if it exists.
    pub(super) fn inode_of(&self, path: &Path) -> Result<Inode> {
        // SAFETY: The root node always exists, so it's safe to unwrap here.
        let mut node = self.storage.get(ROOT_INODE).unwrap();
        let mut components = path.components();

        match components.next() {
            Some(Component::RootDir) => {}
            _ => return Err(FsError::BaseNotDirectory),
        }

        for component in components {
            node = match node {
                Node::Directory { children, .. } => children
                    .iter()
                    .filter_map(|inode| self.storage.get(*inode))
                    .find_map(|node| {
                        if node.name() == component.as_os_str() {
                            Some(node)
                        } else {
                            None
                        }
                    })
                    .ok_or(FsError::NotAFile)?,
                _ => return Err(FsError::BaseNotDirectory),
            };
        }

        Ok(node.inode())
    }

    /// Get the inode associated to a “parent path”. The returned
    /// inode necessarily represents a directory.
    pub(super) fn inode_of_parent(&self, parent_path: &Path) -> Result<Inode> {
        let inode_of_parent = self.inode_of(parent_path)?;

        // Ensure it is a directory.
        match self.storage.get(inode_of_parent) {
            Some(Node::Directory { .. }) => Ok(inode_of_parent),
            _ => Err(FsError::BaseNotDirectory),
        }
    }

    /// From the inode of a parent node (so, a directory), returns the
    /// child index of `name_of_directory` along with its inode.
    pub(super) fn from_parent_get_position_and_inode_of_directory(
        &self,
        inode_of_parent: Inode,
        name_of_directory: &OsString,
        directory_must_be_empty: DirectoryMustBeEmpty,
    ) -> Result<(usize, Inode)> {
        match self.storage.get(inode_of_parent) {
            Some(Node::Directory { children, .. }) => children
                .iter()
                .enumerate()
                .filter_map(|(nth, inode)| self.storage.get(*inode).map(|node| (nth, node)))
                .find_map(|(nth, node)| match node {
                    Node::Directory {
                        inode,
                        name,
                        children,
                        ..
                    } if name.as_os_str() == name_of_directory => {
                        if directory_must_be_empty.no() || children.is_empty() {
                            Some(Ok((nth, *inode)))
                        } else {
                            Some(Err(FsError::DirectoryNotEmpty))
                        }
                    }

                    _ => None,
                })
                .ok_or(FsError::InvalidInput)
                .and_then(identity), // flatten
            _ => Err(FsError::BaseNotDirectory),
        }
    }

    /// From the inode of a parent node (so, a directory), returns the
    /// child index of `name_of_file` along with its inode.
    pub(super) fn from_parent_get_position_and_inode_of_file(
        &self,
        inode_of_parent: Inode,
        name_of_file: &OsString,
    ) -> Result<Option<(usize, Inode)>> {
        match self.storage.get(inode_of_parent) {
            Some(Node::Directory { children, .. }) => children
                .iter()
                .enumerate()
                .filter_map(|(nth, inode)| self.storage.get(*inode).map(|node| (nth, node)))
                .find_map(|(nth, node)| match node {
                    Node::File { inode, name, .. } if name.as_os_str() == name_of_file => {
                        Some(Some((nth, *inode)))
                    }

                    _ => None,
                })
                .or(Some(None))
                .ok_or(FsError::InvalidInput),

            _ => Err(FsError::BaseNotDirectory),
        }
    }

    /// From the inode of a parent node (so, a directory), returns the
    /// child index of `name_of` along with its inode, whatever the
    /// type of inode is (directory or file).
    fn from_parent_get_position_and_inode(
        &self,
        inode_of_parent: Inode,
        name_of: &OsString,
    ) -> Result<Option<(usize, Inode)>> {
        match self.storage.get(inode_of_parent) {
            Some(Node::Directory { children, .. }) => children
                .iter()
                .enumerate()
                .filter_map(|(nth, inode)| self.storage.get(*inode).map(|node| (nth, node)))
                .find_map(|(nth, node)| match node {
                    Node::File { inode, name, .. } | Node::Directory { inode, name, .. }
                        if name.as_os_str() == name_of =>
                    {
                        Some(Some((nth, *inode)))
                    }

                    _ => None,
                })
                .or(Some(None))
                .ok_or(FsError::InvalidInput),

            _ => Err(FsError::BaseNotDirectory),
        }
    }

    /// Set a new name for the node represented by `inode`.
    pub(super) fn update_node_name(&mut self, inode: Inode, new_name: OsString) -> Result<()> {
        let node = self.storage.get_mut(inode).ok_or(FsError::UnknownError)?;

        node.set_name(new_name);
        node.metadata_mut().modified = time();

        Ok(())
    }

    /// Add a child to a directory node represented by `inode`.
    ///
    /// This function also updates the modified time of the directory.
    ///
    /// # Safety
    ///
    /// `inode` must represents an existing directory.
    pub(super) fn add_child_to_node(&mut self, inode: Inode, new_child: Inode) -> Result<()> {
        match self.storage.get_mut(inode) {
            Some(Node::Directory {
                children,
                metadata: Metadata { modified, .. },
                ..
            }) => {
                children.push(new_child);
                *modified = time();

                Ok(())
            }
            _ => Err(FsError::UnknownError),
        }
    }

    /// Remove the child at position `position` of a directory node
    /// represented by `inode`.
    ///
    /// This function also updates the modified time of the directory.
    ///
    /// # Safety
    ///
    /// `inode` must represents an existing directory.
    pub(super) fn remove_child_from_node(&mut self, inode: Inode, position: usize) -> Result<()> {
        match self.storage.get_mut(inode) {
            Some(Node::Directory {
                children,
                metadata: Metadata { modified, .. },
                ..
            }) => {
                children.remove(position);
                *modified = time();

                Ok(())
            }
            _ => Err(FsError::UnknownError),
        }
    }

    /// Canonicalize a path, i.e. try to resolve to a canonical,
    /// absolute form of the path with all intermediate components
    /// normalized:
    ///
    /// * A path must starts with a root (`/`),
    /// * A path can contain `..` or `.` components,
    /// * A path must not contain a Windows prefix (`C:` or `\\server`),
    /// * A normalized path exists in the file system.
    pub(super) fn canonicalize(&self, path: &Path) -> Result<(PathBuf, Inode)> {
        let new_path = self.canonicalize_without_inode(path)?;
        let inode = self.inode_of(&new_path)?;

        Ok((new_path, inode))
    }

    /// Like `Self::canonicalize` but without returning the inode of
    /// the path, which means that there is no guarantee that the path
    /// exists in the file system.
    pub(super) fn canonicalize_without_inode(&self, path: &Path) -> Result<PathBuf> {
        let mut components = path.components();

        match components.next() {
            Some(Component::RootDir) => {}
            _ => return Err(FsError::InvalidInput),
        }

        let mut new_path = PathBuf::with_capacity(path.as_os_str().len());
        new_path.push("/");

        for component in components {
            match component {
                // That's an error to get a `RootDir` a second time.
                Component::RootDir => return Err(FsError::UnknownError),

                // Nothing to do on `new_path`.
                Component::CurDir => (),

                // Pop the lastly inserted component on `new_path` if
                // any, otherwise it's an error.
                Component::ParentDir => {
                    if !new_path.pop() {
                        return Err(FsError::InvalidInput);
                    }
                }

                // A normal
                Component::Normal(name) => {
                    new_path.push(name);
                }

                // We don't support Windows path prefix.
                Component::Prefix(_) => return Err(FsError::InvalidInput),
            }
        }

        Ok(new_path)
    }
}

impl fmt::Debug for FileSystemInner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "\n{inode:<8}    {ty:<4}    name",
            inode = "inode",
            ty = "type",
        )?;

        fn debug(
            nodes: Vec<&Node>,
            slf: &FileSystemInner,
            formatter: &mut fmt::Formatter<'_>,
            indentation: usize,
        ) -> fmt::Result {
            for node in nodes {
                writeln!(
                    formatter,
                    "{inode:<8}    {ty:<4}   {indentation_symbol:indentation_width$}{name}",
                    inode = node.inode(),
                    ty = match node {
                        Node::File { .. } => "file",
                        Node::Directory { .. } => "dir",
                    },
                    name = node.name().to_string_lossy(),
                    indentation_symbol = " ",
                    indentation_width = indentation * 2 + 1,
                )?;

                if let Node::Directory { children, .. } = node {
                    debug(
                        children
                            .iter()
                            .filter_map(|inode| slf.storage.get(*inode))
                            .collect(),
                        slf,
                        formatter,
                        indentation + 1,
                    )?;
                }
            }

            Ok(())
        }

        debug(
            vec![self.storage.get(ROOT_INODE).unwrap()],
            &self,
            formatter,
            0,
        )
    }
}

impl Default for FileSystemInner {
    fn default() -> Self {
        let time = time();

        let mut slab = Slab::new();
        slab.insert(Node::Directory {
            inode: ROOT_INODE,
            name: OsString::from("/"),
            children: Vec::new(),
            metadata: Metadata {
                ft: FileType {
                    dir: true,
                    ..Default::default()
                },
                accessed: time,
                created: time,
                modified: time,
                len: 0,
            },
        });

        Self { storage: slab }
    }
}

#[cfg(test)]
mod test_filesystem {
    use crate::{mem_fs::*, DirEntry, FileSystem as FS, FileType, FsError};

    macro_rules! path {
        ($path:expr) => {
            std::path::Path::new($path)
        };

        (buf $path:expr) => {
            std::path::PathBuf::from($path)
        };
    }

    #[test]
    fn test_new_filesystem() {
        let fs = FileSystem::default();
        let fs_inner = fs.inner.read().unwrap();

        assert_eq!(fs_inner.storage.len(), 1, "storage has a root");
        assert!(
            matches!(
                fs_inner.storage.get(ROOT_INODE),
                Some(Node::Directory {
                    inode: ROOT_INODE,
                    name,
                    children,
                    ..
                }) if name == "/" && children.is_empty(),
            ),
            "storage has a well-defined root",
        );
    }

    #[test]
    fn test_create_dir() {
        let fs = FileSystem::default();

        assert_eq!(
            fs.create_dir(path!("/")),
            Err(FsError::BaseNotDirectory),
            "creating a directory that has no parent",
        );

        assert_eq!(fs.create_dir(path!("/foo")), Ok(()), "creating a directory",);

        {
            let fs_inner = fs.inner.read().unwrap();
            assert_eq!(
                fs_inner.storage.len(),
                2,
                "storage contains the new directory"
            );
            assert!(
                matches!(
                    fs_inner.storage.get(ROOT_INODE),
                    Some(Node::Directory {
                        inode: ROOT_INODE,
                        name,
                        children,
                        ..
                    }) if name == "/" && children == &[1]
                ),
                "the root is updated and well-defined",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(1),
                    Some(Node::Directory {
                        inode: 1,
                        name,
                        children,
                        ..
                    }) if name == "foo" && children.is_empty(),
                ),
                "the new directory is well-defined",
            );
        }

        assert_eq!(
            fs.create_dir(path!("/foo/bar")),
            Ok(()),
            "creating a sub-directory",
        );

        {
            let fs_inner = fs.inner.read().unwrap();
            assert_eq!(
                fs_inner.storage.len(),
                3,
                "storage contains the new sub-directory",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(ROOT_INODE),
                    Some(Node::Directory {
                        inode: ROOT_INODE,
                        name,
                        children,
                        ..
                    }) if name == "/" && children == &[1]
                ),
                "the root is updated again and well-defined",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(1),
                    Some(Node::Directory {
                        inode: 1,
                        name,
                        children,
                        ..
                    }) if name == "foo" && children == &[2]
                ),
                "the new directory is updated and well-defined",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(2),
                    Some(Node::Directory {
                        inode: 2,
                        name,
                        children,
                        ..
                    }) if name == "bar" && children.is_empty()
                ),
                "the new directory is well-defined",
            );
        }
    }

    #[test]
    fn test_remove_dir() {
        let fs = FileSystem::default();

        assert_eq!(
            fs.remove_dir(path!("/")),
            Err(FsError::BaseNotDirectory),
            "removing a directory that has no parent",
        );

        assert_eq!(
            fs.remove_dir(path!("/foo")),
            Err(FsError::NotAFile),
            "cannot remove a directory that doesn't exist",
        );

        assert_eq!(fs.create_dir(path!("/foo")), Ok(()), "creating a directory",);

        assert_eq!(
            fs.create_dir(path!("/foo/bar")),
            Ok(()),
            "creating a sub-directory",
        );

        {
            let fs_inner = fs.inner.read().unwrap();
            assert_eq!(
                fs_inner.storage.len(),
                3,
                "storage contains all the directories",
            );
        }

        assert_eq!(
            fs.remove_dir(path!("/foo")),
            Err(FsError::DirectoryNotEmpty),
            "removing a directory that has children",
        );

        assert_eq!(
            fs.remove_dir(path!("/foo/bar")),
            Ok(()),
            "removing a sub-directory",
        );

        assert_eq!(fs.remove_dir(path!("/foo")), Ok(()), "removing a directory",);

        {
            let fs_inner = fs.inner.read().unwrap();
            assert_eq!(
                fs_inner.storage.len(),
                1,
                "storage contains all the directories",
            );
        }
    }

    #[test]
    fn test_rename() {
        let fs = FileSystem::default();

        assert_eq!(
            fs.rename(path!("/"), path!("/bar")),
            Err(FsError::BaseNotDirectory),
            "renaming a directory that has no parent",
        );
        assert_eq!(
            fs.rename(path!("/foo"), path!("/")),
            Err(FsError::BaseNotDirectory),
            "renaming to a directory that has no parent",
        );

        assert_eq!(fs.create_dir(path!("/foo")), Ok(()));
        assert_eq!(fs.create_dir(path!("/foo/qux")), Ok(()));

        assert_eq!(
            fs.rename(path!("/foo"), path!("/bar/baz")),
            Err(FsError::NotAFile),
            "renaming to a directory that has parent that doesn't exist",
        );

        assert_eq!(fs.create_dir(path!("/bar")), Ok(()));

        assert!(
            matches!(
                fs.new_open_options()
                    .write(true)
                    .create_new(true)
                    .open(path!("/bar/hello1.txt")),
                Ok(_),
            ),
            "creating a new file (`hello1.txt`)",
        );
        assert!(
            matches!(
                fs.new_open_options()
                    .write(true)
                    .create_new(true)
                    .open(path!("/bar/hello2.txt")),
                Ok(_),
            ),
            "creating a new file (`hello2.txt`)",
        );

        {
            let fs_inner = fs.inner.read().unwrap();

            assert_eq!(fs_inner.storage.len(), 6, "storage has all files");
            assert!(
                matches!(
                    fs_inner.storage.get(ROOT_INODE),
                    Some(Node::Directory {
                        inode: ROOT_INODE,
                        name,
                        children,
                        ..
                    }) if name == "/" && children == &[1, 3]
                ),
                "`/` contains `foo` and `bar`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(1),
                    Some(Node::Directory {
                        inode: 1,
                        name,
                        children,
                        ..
                    }) if name == "foo" && children == &[2]
                ),
                "`foo` contains `qux`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(2),
                    Some(Node::Directory {
                        inode: 2,
                        name,
                        children,
                        ..
                    }) if name == "qux" && children.is_empty()
                ),
                "`qux` is empty",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(3),
                    Some(Node::Directory {
                        inode: 3,
                        name,
                        children,
                        ..
                    }) if name == "bar" && children == &[4, 5]
                ),
                "`bar` is contains `hello.txt`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(4),
                    Some(Node::File {
                        inode: 4,
                        name,
                        ..
                    }) if name == "hello1.txt"
                ),
                "`hello1.txt` exists",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(5),
                    Some(Node::File {
                        inode: 5,
                        name,
                        ..
                    }) if name == "hello2.txt"
                ),
                "`hello2.txt` exists",
            );
        }

        assert_eq!(
            fs.rename(path!("/bar/hello2.txt"), path!("/foo/world2.txt")),
            Ok(()),
            "renaming (and moving) a file",
        );

        assert_eq!(
            fs.rename(path!("/foo"), path!("/bar/baz")),
            Ok(()),
            "renaming a directory",
        );

        assert_eq!(
            fs.rename(path!("/bar/hello1.txt"), path!("/bar/world1.txt")),
            Ok(()),
            "renaming a file (in the same directory)",
        );

        {
            let fs_inner = fs.inner.read().unwrap();

            dbg!(&fs_inner);

            assert_eq!(
                fs_inner.storage.len(),
                6,
                "storage has still all directories"
            );
            assert!(
                matches!(
                    fs_inner.storage.get(ROOT_INODE),
                    Some(Node::Directory {
                        inode: ROOT_INODE,
                        name,
                        children,
                        ..
                    }) if name == "/" && children == &[3]
                ),
                "`/` contains `bar`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(1),
                    Some(Node::Directory {
                        inode: 1,
                        name,
                        children,
                        ..
                    }) if name == "baz" && children == &[2, 5]
                ),
                "`foo` has been renamed to `baz` and contains `qux` and `world2.txt`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(2),
                    Some(Node::Directory {
                        inode: 2,
                        name,
                        children,
                        ..
                    }) if name == "qux" && children.is_empty()
                ),
                "`qux` is empty",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(3),
                    Some(Node::Directory {
                        inode: 3,
                        name,
                        children,
                        ..
                    }) if name == "bar" && children == &[4, 1]
                ),
                "`bar` contains `bar` (ex `foo`)  and `world1.txt` (ex `hello1`)",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(4),
                    Some(Node::File {
                        inode: 4,
                        name,
                        ..
                    }) if name == "world1.txt"
                ),
                "`hello1.txt` has been renamed to `world1.txt`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(5),
                    Some(Node::File {
                        inode: 5,
                        name,
                        ..
                    }) if name == "world2.txt"
                ),
                "`hello2.txt` has been renamed to `world2.txt`",
            );
        }
    }

    #[test]
    fn test_metadata() {
        use std::thread::sleep;
        use std::time::Duration;

        let fs = FileSystem::default();
        let root_metadata = fs.metadata(path!("/"));

        assert!(matches!(
            root_metadata,
            Ok(Metadata {
                ft: FileType { dir: true, .. },
                accessed,
                created,
                modified,
                len: 0
            }) if accessed == created && created == modified && modified > 0
        ));

        assert_eq!(fs.create_dir(path!("/foo")), Ok(()));

        let foo_metadata = fs.metadata(path!("/foo"));
        assert!(foo_metadata.is_ok());
        let foo_metadata = foo_metadata.unwrap();

        assert!(matches!(
            foo_metadata,
            Metadata {
                ft: FileType { dir: true, .. },
                accessed,
                created,
                modified,
                len: 0
            } if accessed == created && created == modified && modified > 0
        ));

        sleep(Duration::from_secs(3));

        assert_eq!(fs.rename(path!("/foo"), path!("/bar")), Ok(()));

        assert!(
            matches!(
                fs.metadata(path!("/bar")),
                Ok(Metadata {
                    ft: FileType { dir: true, .. },
                    accessed,
                    created,
                    modified,
                    len: 0
                }) if
                    accessed == foo_metadata.accessed &&
                    created == foo_metadata.created &&
                    modified > foo_metadata.modified
            ),
            "the modified time is updated when file is renamed",
        );
        assert!(
            matches!(
                fs.metadata(path!("/")),
                Ok(Metadata {
                    ft: FileType { dir: true, .. },
                    accessed,
                    created,
                    modified,
                    len: 0
                }) if
                    accessed == foo_metadata.accessed &&
                    created == foo_metadata.created &&
                    modified > foo_metadata.modified
            ),
            "the modified time of the parent is updated when file is renamed",
        );
    }

    #[test]
    fn test_remove_file() {
        let fs = FileSystem::default();

        assert!(
            matches!(
                fs.new_open_options()
                    .write(true)
                    .create_new(true)
                    .open(path!("/foo.txt")),
                Ok(_)
            ),
            "creating a new file",
        );

        {
            let fs_inner = fs.inner.read().unwrap();

            assert_eq!(fs_inner.storage.len(), 2, "storage has all files");
            assert!(
                matches!(
                    fs_inner.storage.get(ROOT_INODE),
                    Some(Node::Directory {
                        inode: ROOT_INODE,
                        name,
                        children,
                        ..
                    }) if name == "/" && children == &[1]
                ),
                "`/` contains `foo.txt`",
            );
            assert!(
                matches!(
                    fs_inner.storage.get(1),
                    Some(Node::File {
                        inode: 1,
                        name,
                        ..
                    }) if name == "foo.txt"
                ),
                "`foo.txt` exists and is a file",
            );
        }

        assert_eq!(
            fs.remove_file(path!("/foo.txt")),
            Ok(()),
            "removing a file that exists",
        );

        {
            let fs_inner = fs.inner.read().unwrap();

            assert_eq!(fs_inner.storage.len(), 1, "storage no longer has the file");
            assert!(
                matches!(
                    fs_inner.storage.get(ROOT_INODE),
                    Some(Node::Directory {
                        inode: ROOT_INODE,
                        name,
                        children,
                        ..
                    }) if name == "/" && children == &[]
                ),
                "`/` is empty",
            );
        }

        assert_eq!(
            fs.remove_file(path!("/foo.txt")),
            Err(FsError::NotAFile),
            "removing a file that exists",
        );
    }

    #[test]
    fn test_readdir() {
        let fs = FileSystem::default();

        assert_eq!(fs.create_dir(path!("/foo")), Ok(()), "creating `foo`");
        assert_eq!(fs.create_dir(path!("/foo/sub")), Ok(()), "creating `sub`");
        assert_eq!(fs.create_dir(path!("/bar")), Ok(()), "creating `bar`");
        assert_eq!(fs.create_dir(path!("/baz")), Ok(()), "creating `bar`");
        assert!(
            matches!(
                fs.new_open_options()
                    .write(true)
                    .create_new(true)
                    .open(path!("/a.txt")),
                Ok(_)
            ),
            "creating `a.txt`",
        );
        assert!(
            matches!(
                fs.new_open_options()
                    .write(true)
                    .create_new(true)
                    .open(path!("/b.txt")),
                Ok(_)
            ),
            "creating `b.txt`",
        );

        let readdir = fs.read_dir(path!("/"));

        assert!(readdir.is_ok(), "reading the directory `/`");

        let mut readdir = readdir.unwrap();

        assert!(
            matches!(
                readdir.next(),
                Some(Ok(DirEntry {
                    path,
                    metadata: Ok(Metadata { ft, .. }),
                }))
                    if path == path!(buf "/foo") && ft.is_dir()
            ),
            "checking entry #1",
        );
        assert!(
            matches!(
                readdir.next(),
                Some(Ok(DirEntry {
                    path,
                    metadata: Ok(Metadata { ft, .. }),
                }))
                    if path == path!(buf "/bar") && ft.is_dir()
            ),
            "checking entry #2",
        );
        assert!(
            matches!(
                readdir.next(),
                Some(Ok(DirEntry {
                    path,
                    metadata: Ok(Metadata { ft, .. }),
                }))
                    if path == path!(buf "/baz") && ft.is_dir()
            ),
            "checking entry #3",
        );
        assert!(
            matches!(
                readdir.next(),
                Some(Ok(DirEntry {
                    path,
                    metadata: Ok(Metadata { ft, .. }),
                }))
                    if path == path!(buf "/a.txt") && ft.is_file()
            ),
            "checking entry #4",
        );
        assert!(
            matches!(
                readdir.next(),
                Some(Ok(DirEntry {
                    path,
                    metadata: Ok(Metadata { ft, .. }),
                }))
                    if path == path!(buf "/b.txt") && ft.is_file()
            ),
            "checking entry #5",
        );
        assert!(matches!(readdir.next(), None), "no more entries");
    }

    #[test]
    fn test_canonicalize() {
        let fs = FileSystem::default();

        assert_eq!(fs.create_dir(path!("/foo")), Ok(()), "creating `foo`");
        assert_eq!(fs.create_dir(path!("/foo/bar")), Ok(()), "creating `bar`");
        assert_eq!(
            fs.create_dir(path!("/foo/bar/baz")),
            Ok(()),
            "creating `baz`",
        );
        assert_eq!(
            fs.create_dir(path!("/foo/bar/baz/qux")),
            Ok(()),
            "creating `qux`",
        );
        assert!(
            matches!(
                fs.new_open_options()
                    .write(true)
                    .create_new(true)
                    .open(path!("/foo/bar/baz/qux/hello.txt")),
                Ok(_)
            ),
            "creating `hello.txt`",
        );

        let fs_inner = fs.inner.read().unwrap();

        assert_eq!(
            fs_inner.canonicalize(path!("/")),
            Ok((path!(buf "/"), ROOT_INODE)),
            "canonicalizing `/`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("foo")),
            Err(FsError::InvalidInput),
            "canonicalizing `foo`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("/././././foo/")),
            Ok((path!(buf "/foo"), 1)),
            "canonicalizing `/././././foo/`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("/foo/bar//")),
            Ok((path!(buf "/foo/bar"), 2)),
            "canonicalizing `/foo/bar//`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("/foo/bar/../bar")),
            Ok((path!(buf "/foo/bar"), 2)),
            "canonicalizing `/foo/bar/../bar`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("/foo/bar/../..")),
            Ok((path!(buf "/"), ROOT_INODE)),
            "canonicalizing `/foo/bar/../..`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("/foo/bar/../../..")),
            Err(FsError::InvalidInput),
            "canonicalizing `/foo/bar/../../..`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!("C:/foo/")),
            Err(FsError::InvalidInput),
            "canonicalizing `C:/foo/`",
        );
        assert_eq!(
            fs_inner.canonicalize(path!(
                "/foo/./../foo/bar/../../foo/bar/./baz/./../baz/qux/../../baz/./qux/hello.txt"
            )),
            Ok((path!(buf "/foo/bar/baz/qux/hello.txt"), 5)),
            "canonicalizing a crazily stupid path name",
        );
    }
}

#[allow(dead_code)] // The `No` variant.
pub(super) enum DirectoryMustBeEmpty {
    Yes,
    No,
}

impl DirectoryMustBeEmpty {
    pub(super) fn yes(&self) -> bool {
        matches!(self, Self::Yes)
    }

    pub(super) fn no(&self) -> bool {
        !self.yes()
    }
}
