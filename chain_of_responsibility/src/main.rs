use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
enum Resolver {
    Aliases(HashMap<String, PathBuf>),
    Directory(PathBuf)
}

impl Resolver {
    fn try_resolve(&self, name: &str) -> Option<PathBuf> {
        match self {
            Resolver::Aliases(map) => {
                map.get(name).cloned()
            },
            Resolver::Directory(directory) => {
                let candidate = directory.join(name);
                candidate.is_file().then_some(candidate)
            },
        }
    }
}

#[derive(Debug, Default)]
struct ResolverChain {
    resolvers: Vec<Resolver>
}

impl ResolverChain {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, resolver: Resolver) {
        self.resolvers.push(resolver)
    }

    fn resolve(&self, name: &str) -> Option<PathBuf> {
        self.resolvers.iter().find_map(|r| r.try_resolve(name))
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_resolve_alias() {
        let mut chain = ResolverChain::new();
        let mut map = HashMap::new();
        map.insert("editor".to_string(), "/custom/editor".into());
        chain.push(Resolver::Aliases(map));
        assert_eq!(chain.resolve("editor"), Some(PathBuf::from("/custom/editor")));
    }

    #[test]
    fn test_resolve_directory() {
        let mut test_file = std::env::temp_dir();
        test_file.push("test.txt");
        std::fs::write(&test_file, "test content");
        let mut chain = ResolverChain::new();
        chain.push(Resolver::Aliases(HashMap::new()));
        chain.push(Resolver::Directory(std::env::temp_dir()));
        assert_eq!(chain.resolve("test.txt"), Some(test_file.clone()));
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_resolve_directory_first() {
        let mut test_file = std::env::temp_dir().join("test2.txt");
        std::fs::write(&test_file, "test content");
        let mut test_file_nested = std::env::temp_dir().join("foo/");
        fs::create_dir_all(&test_file_nested);
        test_file_nested.push("test2.txt");
        std::fs::write(&test_file_nested, "test content");
        let mut chain = ResolverChain::new();
        chain.push(Resolver::Aliases(HashMap::new()));
        chain.push(Resolver::Directory(std::env::temp_dir()));
        fs::remove_file(test_file).unwrap();
    }
}
