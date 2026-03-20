use bend_pvm::package::{
    Dependency, Package, PackageError, PackageManifest, PackageMetadata, Version,
};

fn create_test_package() -> Package {
    Package::new("test_pkg".to_string(), Version::new(1, 0, 0))
}

mod package_tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_to_string() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_parse() {
        let version = Version::parse("2.1.0").unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_parse_invalid() {
        let result = Version::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_eq() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 0, 0);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_version_cmp() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        assert!(v1 < v2);
    }

    #[test]
    fn test_package_creation() {
        let pkg = create_test_package();
        assert_eq!(pkg.name(), "test_pkg");
        assert_eq!(pkg.version().major, 1);
        assert_eq!(pkg.version().minor, 0);
        assert_eq!(pkg.version().patch, 0);
    }

    #[test]
    fn test_package_metadata() {
        let mut pkg = create_test_package();
        pkg.set_description("A test package".to_string());
        pkg.add_author("Test Author".to_string());

        assert_eq!(pkg.description(), Some("A test package"));
        assert!(pkg.authors().contains(&"Test Author".to_string()));
    }

    #[test]
    fn test_package_with_dependencies() {
        let mut pkg = create_test_package();
        let dep = Dependency::new("other_pkg".to_string(), Version::new(1, 0, 0));
        pkg.add_dependency(dep.clone());

        assert!(pkg.dependencies().contains(&dep));
    }

    #[test]
    fn test_package_manifest_creation() {
        let manifest = PackageManifest::new("my_pkg".to_string(), Version::new(1, 0, 0));
        assert_eq!(manifest.package().name(), "my_pkg");
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_package_manifest_invalid_name() {
        let manifest = PackageManifest::new("".to_string(), Version::new(1, 0, 0));
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_package_manifest_with_dependencies() {
        let mut manifest = PackageManifest::new("my_pkg".to_string(), Version::new(1, 0, 0));
        manifest.add_dependency("dep_pkg".to_string(), Version::new(2, 0, 0));

        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.dependencies().len(), 1);
    }

    #[test]
    fn test_dependency_satisfied_by() {
        let dep = Dependency::new("pkg".to_string(), Version::new(1, 0, 0));

        assert!(dep.is_satisfied_by(&Version::new(1, 0, 0)));
        assert!(dep.is_satisfied_by(&Version::new(1, 5, 0)));
        assert!(!dep.is_satisfied_by(&Version::new(2, 0, 0)));
        assert!(!dep.is_satisfied_by(&Version::new(0, 9, 0)));
    }

    #[test]
    fn test_package_error_display() {
        let err = PackageError::InvalidName("bad".to_string());
        assert!(err.to_string().contains("Invalid package name"));

        let err = PackageError::InvalidVersion("bad".to_string());
        assert!(err.to_string().contains("Invalid version"));
    }

    #[test]
    fn test_package_lock_entry() {
        use bend_pvm::package::PackageLockEntry;

        let entry = PackageLockEntry::new(
            "locked_pkg".to_string(),
            Version::new(1, 0, 0),
            "sha256:abc123".to_string(),
        );

        assert_eq!(entry.name(), "locked_pkg");
        assert_eq!(entry.version().major, 1);
        assert_eq!(entry.integrity(), "sha256:abc123");
    }

    #[test]
    fn test_package_registry() {
        use bend_pvm::package::PackageRegistry;

        let mut registry = PackageRegistry::new();
        let pkg = create_test_package();

        registry.register(pkg.clone()).unwrap();
        assert!(registry.exists("test_pkg"));
        assert_eq!(registry.get("test_pkg").unwrap().name(), "test_pkg");
    }

    #[test]
    fn test_package_registry_duplicate() {
        use bend_pvm::package::PackageRegistry;

        let mut registry = PackageRegistry::new();
        let pkg = create_test_package();

        registry.register(pkg.clone()).unwrap();
        let result = registry.register(pkg);
        assert!(result.is_err());
    }

    #[test]
    fn test_package_resolver() {
        use bend_pvm::package::DependencyResolver;

        let resolver = DependencyResolver::new();
        let pkg = create_test_package();

        assert!(resolver.resolve(&pkg).is_ok());
    }

    #[test]
    fn test_package_resolver_with_conflicts() {
        use bend_pvm::package::DependencyResolver;

        let mut resolver = DependencyResolver::new();
        let mut pkg1 = Package::new("pkg1".to_string(), Version::new(1, 0, 0));
        pkg1.add_dependency(Dependency::new("shared".to_string(), Version::new(1, 0, 0)));

        let mut pkg2 = Package::new("pkg2".to_string(), Version::new(1, 0, 0));
        pkg2.add_dependency(Dependency::new("shared".to_string(), Version::new(2, 0, 0)));

        resolver.register(pkg1).unwrap();
        let result = resolver.register(pkg2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_from_str() {
        let version = Version::from_str("1.2.3");
        assert!(version.is_ok());
        let v = version.unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_package_name_validation() {
        assert!(PackageManifest::is_valid_name("valid_pkg"));
        assert!(PackageManifest::is_valid_name("my-package-123"));
        assert!(!PackageManifest::is_valid_name(""));
        assert!(!PackageManifest::is_valid_name("-invalid"));
    }

    #[test]
    fn test_package_lock_creation() {
        use bend_pvm::package::{PackageLock, PackageLockEntry};

        let mut lock = PackageLock::new();
        lock.add_entry(PackageLockEntry::new(
            "pkg".to_string(),
            Version::new(1, 0, 0),
            "hash123".to_string(),
        ));

        assert_eq!(lock.entries().len(), 1);
    }
}
