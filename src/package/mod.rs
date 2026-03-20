//! Package management for Bend-PVM
//!
//! This module provides semantic versioning, dependency resolution,
//! and package registry functionality for the Bend programming language.

#![allow(clippy::module_inception)]
pub mod package;

pub use package::{
    Dependency, DependencyResolver, Package, PackageError, PackageLock, PackageLockEntry,
    PackageManifest, PackageMetadata, PackageRegistry, Version,
};
