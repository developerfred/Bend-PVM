use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Result<Self, PackageError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(PackageError::InvalidVersion(s.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| PackageError::InvalidVersion(s.to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| PackageError::InvalidVersion(s.to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| PackageError::InvalidVersion(s.to_string()))?;

        Ok(Version::new(major, minor, patch))
    }
}

impl std::str::FromStr for Version {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    name: String,
    version: Version,
}

impl Dependency {
    pub fn new(name: String, version: Version) -> Self {
        Dependency { name, version }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn is_satisfied_by(&self, version: &Version) -> bool {
        version.major == self.version.major && version >= &self.version
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Dependency {}

#[derive(Debug, Clone)]
pub struct PackageMetadata {
    description: Option<String>,
    authors: Vec<String>,
    license: Option<String>,
    repository: Option<String>,
}

impl PackageMetadata {
    pub fn new() -> Self {
        PackageMetadata {
            description: None,
            authors: Vec::new(),
            license: None,
            repository: None,
        }
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn authors(&self) -> &[String] {
        &self.authors
    }

    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    pub fn repository(&self) -> Option<&str> {
        self.repository.as_deref()
    }

    pub fn set_description(&mut self, desc: String) {
        self.description = Some(desc);
    }

    pub fn add_author(&mut self, author: String) {
        self.authors.push(author);
    }

    pub fn set_license(&mut self, license: String) {
        self.license = Some(license);
    }

    pub fn set_repository(&mut self, repo: String) {
        self.repository = Some(repo);
    }
}

impl Default for PackageMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    version: Version,
    metadata: PackageMetadata,
    dependencies: Vec<Dependency>,
}

impl Package {
    pub fn new(name: String, version: Version) -> Self {
        Package {
            name,
            version,
            metadata: PackageMetadata::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn description(&self) -> Option<&str> {
        self.metadata.description()
    }

    pub fn authors(&self) -> &[String] {
        self.metadata.authors()
    }

    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    pub fn metadata(&self) -> &PackageMetadata {
        &self.metadata
    }

    pub fn set_description(&mut self, desc: String) {
        self.metadata.set_description(desc);
    }

    pub fn add_author(&mut self, author: String) {
        self.metadata.add_author(author);
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }
}

#[derive(Debug, Clone)]
pub struct PackageManifest {
    package: Package,
}

impl PackageManifest {
    pub fn new(name: String, version: Version) -> Self {
        PackageManifest {
            package: Package::new(name, version),
        }
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn package_mut(&mut self) -> &mut Package {
        &mut self.package
    }

    pub fn dependencies(&self) -> &[Dependency] {
        self.package.dependencies()
    }

    pub fn add_dependency(&mut self, name: String, version: Version) {
        self.package.add_dependency(Dependency::new(name, version));
    }

    pub fn validate(&self) -> Result<(), PackageError> {
        if self.package.name.is_empty() {
            return Err(PackageError::InvalidName(self.package.name.clone()));
        }
        if !Self::is_valid_name(&self.package.name) {
            return Err(PackageError::InvalidName(self.package.name.clone()));
        }
        Ok(())
    }

    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('-')
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
}

#[derive(Debug, Clone)]
pub struct PackageLockEntry {
    name: String,
    version: Version,
    integrity: String,
}

impl PackageLockEntry {
    pub fn new(name: String, version: Version, integrity: String) -> Self {
        PackageLockEntry {
            name,
            version,
            integrity,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn integrity(&self) -> &str {
        &self.integrity
    }
}

#[derive(Debug, Clone)]
pub struct PackageLock {
    entries: HashMap<String, PackageLockEntry>,
}

impl PackageLock {
    pub fn new() -> Self {
        PackageLock {
            entries: HashMap::new(),
        }
    }

    pub fn entries(&self) -> &HashMap<String, PackageLockEntry> {
        &self.entries
    }

    pub fn add_entry(&mut self, entry: PackageLockEntry) {
        self.entries.insert(entry.name().to_string(), entry);
    }

    pub fn get(&self, name: &str) -> Option<&PackageLockEntry> {
        self.entries.get(name)
    }
}

impl Default for PackageLock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PackageRegistry {
    packages: HashMap<String, Package>,
}

impl PackageRegistry {
    pub fn new() -> Self {
        PackageRegistry {
            packages: HashMap::new(),
        }
    }

    pub fn register(&mut self, package: Package) -> Result<(), PackageError> {
        let name = package.name().to_string();
        if self.packages.contains_key(&name) {
            return Err(PackageError::AlreadyExists(name));
        }
        self.packages.insert(name, package);
        Ok(())
    }

    pub fn exists(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Package> {
        self.packages.get(name)
    }

    pub fn list(&self) -> Vec<&Package> {
        self.packages.values().collect()
    }
}

impl Default for PackageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DependencyResolver {
    packages: HashMap<String, Version>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        DependencyResolver {
            packages: HashMap::new(),
        }
    }

    pub fn register(&mut self, package: Package) -> Result<(), PackageError> {
        let name = package.name().to_string();
        let version = package.version().clone();

        if let Some(existing) = self.packages.get(&name) {
            if existing != &version {
                return Err(PackageError::VersionConflict {
                    package: name,
                    version1: existing.clone(),
                    version2: version,
                });
            }
        }

        self.packages.insert(name, version);
        Ok(())
    }

    pub fn resolve(&self, package: &Package) -> Result<Vec<Package>, PackageError> {
        let resolved = vec![package.clone()];

        for dep in package.dependencies() {
            if let Some(version) = self.packages.get(dep.name()) {
                if !dep.is_satisfied_by(version) {
                    return Err(PackageError::UnsatisfiedDependency {
                        package: dep.name().to_string(),
                        required: dep.version().clone(),
                        available: version.clone(),
                    });
                }
            }
        }

        Ok(resolved)
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageError {
    InvalidName(String),
    InvalidVersion(String),
    AlreadyExists(String),
    VersionConflict {
        package: String,
        version1: Version,
        version2: Version,
    },
    UnsatisfiedDependency {
        package: String,
        required: Version,
        available: Version,
    },
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageError::InvalidName(name) => {
                write!(f, "Invalid package name: {}", name)
            }
            PackageError::InvalidVersion(version) => {
                write!(f, "Invalid version: {}", version)
            }
            PackageError::AlreadyExists(name) => {
                write!(f, "Package already exists: {}", name)
            }
            PackageError::VersionConflict {
                package,
                version1,
                version2,
            } => {
                write!(
                    f,
                    "Version conflict for {}: {} vs {}",
                    package, version1, version2
                )
            }
            PackageError::UnsatisfiedDependency {
                package,
                required,
                available,
            } => {
                write!(
                    f,
                    "Unsatisfied dependency {}: required {} but available {}",
                    package, required, available
                )
            }
        }
    }
}

impl std::error::Error for PackageError {}
