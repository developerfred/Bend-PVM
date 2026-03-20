#![allow(clippy::module_inception)]
pub mod package;

pub use package::{
    Dependency, DependencyResolver, Package, PackageError, PackageLock, PackageLockEntry,
    PackageManifest, PackageMetadata, PackageRegistry, Version,
};
