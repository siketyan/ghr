use std::convert::Infallible;
use std::fmt::{Display, Result as FmtResult};
use std::fs::File as StdFile;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::str::FromStr;

use anyhow::Result;

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Comment(String),
    Include(String),
    Exclude(String),
}

impl FromStr for Node {
    type Err = Infallible;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let mut chars = s.chars();

        Ok(match chars.next() {
            Some('#') => Node::Comment(chars.collect()),
            Some('!') => Node::Include(chars.collect()),
            Some(_) => Node::Exclude(s.to_string()),
            None => Node::Empty,
        })
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        let str = match self {
            Node::Empty => String::new(),
            Node::Comment(c) => format!("#{}", c),
            Node::Include(i) => format!("!{}", i),
            Node::Exclude(e) => e.to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug, Default)]
pub struct File {
    nodes: Vec<Node>,
}

impl File {
    pub fn read<R>(reader: R) -> Result<Self>
    where
        R: Read,
    {
        let reader = BufReader::new(reader);

        Ok(Self {
            nodes: reader
                .lines()
                .map(|l| l.map(|l| Node::from_str(&l).unwrap()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Self::read(StdFile::open(Self::file_path(path))?)
    }

    pub fn write<W>(&self, writer: W) -> Result<()>
    where
        W: Write,
    {
        let mut writer = BufWriter::new(writer);
        for node in &self.nodes {
            writeln!(writer, "{}", node)?;
        }

        Ok(())
    }

    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.write(StdFile::create(Self::file_path(path))?)
    }

    pub fn add_or_noop(&mut self, node: Node) {
        if !self.nodes.contains(&node) {
            self.nodes.push(node);
        }
    }

    fn file_path<P>(path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        path.as_ref().join(".git").join("info").join("exclude")
    }
}

impl Deref for File {
    type Target = Vec<Node>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_node_from_str() {
        assert_eq!(
            Node::Comment("This is a comment".to_string()),
            Node::from_str("#This is a comment").unwrap(),
        );
        assert_eq!(
            Node::Include("/path/to/**/file.ext".to_string()),
            Node::from_str("!/path/to/**/file.ext").unwrap(),
        );
        assert_eq!(
            Node::Exclude("/path/to/**/file.ext".to_string()),
            Node::from_str("/path/to/**/file.ext").unwrap(),
        );
    }

    #[test]
    fn test_node_to_string() {
        assert_eq!(
            "#This is a comment".to_string(),
            Node::Comment("This is a comment".to_string()).to_string(),
        );
        assert_eq!(
            "!/path/to/**/file.ext".to_string(),
            Node::Include("/path/to/**/file.ext".to_string()).to_string(),
        );
        assert_eq!(
            "/path/to/**/file.ext".to_string(),
            Node::Exclude("/path/to/**/file.ext".to_string()).to_string(),
        );
    }

    #[test]
    fn test_file_read() {
        let content = br#"
# File patterns to ignore; see `git help ignore` for more information.
# Lines that start with '#' are comments.
.idea
"#;

        assert_eq!(
            vec![
                Node::Empty,
                Node::Comment(
                    " File patterns to ignore; see `git help ignore` for more information."
                        .to_string()
                ),
                Node::Comment(" Lines that start with '#' are comments.".to_string()),
                Node::Exclude(".idea".to_string()),
            ],
            File::read(&content[..]).unwrap().nodes,
        );
    }

    #[test]
    fn test_file_write() {
        let mut content = Vec::<u8>::new();
        let mut file = File::default();

        file.push(Node::Comment("This is a comment".to_string()));
        file.push(Node::Empty);
        file.push(Node::Include("/path/to/include".to_string()));
        file.push(Node::Exclude("/path/to/exclude".to_string()));
        file.write(&mut content).unwrap();

        assert_eq!(
            r#"#This is a comment

!/path/to/include
/path/to/exclude
"#,
            String::from_utf8(content).unwrap(),
        )
    }
}
