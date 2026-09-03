use std::sync::Arc;
use std::io::{self, ErrorKind};
use std::path::{PathBuf, Path};
use std::fs::read_to_string;
use std::collections::HashMap;

trait DocumentSource {
    fn load(&mut self, path: &Path) -> io::Result<Arc<str>>;
}

struct FileSource {}

impl DocumentSource for FileSource {
    fn load(&mut self, path: &Path) -> io::Result<Arc<str>> {
        let content = read_to_string(path)?;
        Ok(Arc::from(content))
    }
}

struct CachingSource<S> {
    source: S,
    cache: HashMap<PathBuf, Arc<str>>,
}

impl<S: DocumentSource> CachingSource<S> {
    fn new(source: S) -> Self {
        Self {source, cache: HashMap::new() }
    }

    fn into_inner(self) -> S {
        self.source
    }
}


impl<S: DocumentSource> DocumentSource for CachingSource<S> {
    fn load(&mut self, path: &Path) -> io::Result<Arc<str>> {
        if let Some(entry) = self.cache.get(path) {
            return Ok(Arc::clone(entry));
        }
        let content = self.source.load(path)?;
        self.cache.insert(path.to_path_buf(), Arc::clone(&content));
        Ok(content)

    }
}

fn preview(source: &mut impl DocumentSource, path: &Path) -> io::Result<String> {
    let content = source.load(path)?;
    Ok(content.lines().next().unwrap_or_default().to_owned())
}

#[derive(Default)]
struct CountingSource {
    count: usize
}

impl DocumentSource for CountingSource {
    fn load(&mut self, path: &Path) -> io::Result<Arc<str>> {
        self.count += 1;
        if path == Path::new("/error/") {
            return Err(ErrorKind::NotFound.into());
        }
        Ok(Arc::from(format!("CountingSource count={}", self.count)))
    }
}



fn main() {
    let mut source = CachingSource::new(FileSource{});
    let result = preview(&mut source, Path::new("/tmp/test.txt")).unwrap();
    println!("Hello, {}!", result);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_incrememnts() {
        let mut source = CachingSource::new(CountingSource::default());
        let result = preview(&mut source, Path::new("/tmp/test.txt")).unwrap();
        assert_eq!(source.into_inner().count, 1);
    }

    #[test]
    fn test_load_twice_doesnt_incrememnt() {
        let mut source = CachingSource::new(CountingSource::default());
        let result = source.load(Path::new("/tmp/test.txt")).unwrap();
        let result2 = source.load(Path::new("/tmp/test.txt")).unwrap();
        assert!(Arc::ptr_eq(&result, &result2));
        assert_eq!(source.into_inner().count, 1);
    }

    #[test]
    fn test_load_different_path_increment() {
        let mut source = CachingSource::new(CountingSource::default());
        let _result = source.load(Path::new("/tmp/test.txt")).unwrap();
        let _result2 = source.load(Path::new("/tmp/test2.txt")).unwrap();
        assert_eq!(source.into_inner().count, 2);
    }

    #[test]
    fn test_error_load_twice_increment() {
        let mut source = CachingSource::new(CountingSource::default());
        assert!(source.load(Path::new("/error/")).is_err());
        assert!(source.load(Path::new("/error/")).is_err());
        assert_eq!(source.into_inner().count, 2);
    }

}
