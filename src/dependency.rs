use toml_edit;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum DependencySource {
    Version {
        version: Option<String>,
        path: Option<String>,
    },
    Git(String),
}

/// A dependency handled by Cargo
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Dependency {
    /// The name of the dependency (as it is set in its `Cargo.toml` and known to crates.io)
    pub name: String,
    optional: bool,
    source: DependencySource,
    registry: Option<String>,
}

impl Default for Dependency {
    fn default() -> Dependency {
        Dependency {
            name: "".into(),
            optional: false,
            source: DependencySource::Version {
                version: None,
                path: None,
            },
            registry: None,
        }
    }
}

impl Dependency {
    /// Create a new dependency with a name
    pub fn new(name: &str) -> Dependency {
        Dependency {
            name: name.into(),
            ..Dependency::default()
        }
    }

    /// Set dependency to a given version
    pub fn set_version(mut self, version: &str) -> Dependency {
        let old_source = self.source;
        let old_path = match old_source {
            DependencySource::Version { path, .. } => path,
            _ => None,
        };
        self.source = DependencySource::Version {
            version: Some(version.into()),
            path: old_path,
        };
        self
    }

    /// Set dependency to a given repository
    pub fn set_git(mut self, repo: &str) -> Dependency {
        self.source = DependencySource::Git(repo.into());
        self
    }

    /// Set dependency to a given path
    pub fn set_path(mut self, path: &str) -> Dependency {
        let old_source = self.source;
        let old_version = match old_source {
            DependencySource::Version { version, .. } => version,
            _ => None,
        };
        self.source = DependencySource::Version {
            version: old_version,
            path: Some(path.into()),
        };
        self
    }

    /// Set whether the dependency is optional
    pub fn set_optional(mut self, opt: bool) -> Dependency {
        self.optional = opt;
        self
    }

    /// Set dependency to a given repository
    pub fn set_registry(mut self, registry: Option<String>) -> Dependency {
        self.registry = registry;
        self
    }

    /// Get version of dependency
    pub fn version(&self) -> Option<&str> {
        if let DependencySource::Version {
            version: Some(ref version),
            ..
        } = self.source
        {
            Some(version)
        } else {
            None
        }
    }

    /// Convert dependency to TOML
    ///
    /// Returns a tuple with the dependency's name and either the version as a `String`
    /// or the path/git repository as an `InlineTable`.
    /// (If the dependency is set as `optional`, an `InlineTable` is returned in any case.)
    pub fn to_toml(&self) -> (String, toml_edit::Item) {
        let data: toml_edit::Item = match (self.optional, self.source.clone(), self.registry.clone()) {
            // Extra short when version flag only
            (
                false,
                DependencySource::Version {
                    version: Some(v),
                    path: None,
                },
                None,
            ) => toml_edit::value(v),
            // Other cases are represented as an inline table
            (optional, source, registry) => {
                let mut data = toml_edit::InlineTable::default();

                match source {
                    DependencySource::Version { version, path } => {
                        if let Some(v) = version {
                            data.get_or_insert("version", v);
                        }
                        if let Some(p) = path {
                            data.get_or_insert("path", p);
                        }
                        if let Some(r) = registry {
                            data.get_or_insert("registry", r);
                        }
                    }
                    DependencySource::Git(v) => {
                        data.get_or_insert("git", v);
                    }
                }
                if self.optional {
                    data.get_or_insert("optional", optional);
                }

                data.fmt();
                toml_edit::value(toml_edit::Value::InlineTable(data))
            }
        };

        (self.name.clone(), data)
    }
}
