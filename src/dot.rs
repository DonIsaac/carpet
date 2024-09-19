use std::{collections::HashMap, fmt::Display, io};
pub trait ToDot {
    fn to_dot<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
}

#[derive(Debug)]
pub enum DotAttribute {
    Ident(String),
    String(String),
}
impl DotAttribute {
    pub fn label<S: Into<String>>(label: S) -> (&'static str, DotAttribute) {
        ("label", Self::from(label))
    }
    pub fn color<S: Into<String>>(color: S) -> (&'static str, DotAttribute) {
        ("color", Self::Ident(color.into()))
    }
}
impl Display for DotAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(id) => id.fmt(f),
            Self::String(s) => write!(f, r#""{s}""#),
        }
    }
}
impl<S: Into<String>> From<S> for DotAttribute {
    fn from(value: S) -> Self {
        Self::String(value.into())
    }
}

#[derive(Debug)]
pub struct DotBuilder<K> {
    name: Option<String>,
    nodes: HashMap<K, String>,
    edges: Vec<String>,
}
impl<K> Default for DotBuilder<K> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            nodes: Default::default(),
            edges: Default::default(),
        }
    }
}

impl<K> DotBuilder<K> {
    pub fn new(name: String) -> Self {
        Self {
            name: Some(name),
            nodes: Default::default(),
            edges: Default::default(),
        }
    }
}
// type Attributes<S: AsRef<str>> = Iterator<Item = (S, DotAttribute)>;
impl<K: Clone + PartialEq + Eq + std::hash::Hash + Display> DotBuilder<K> {
    const INDENT: &'static str = "  ";
    pub fn add_node<I, S>(&mut self, id: &K, attributes: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, DotAttribute)>,
    {
        if self.nodes.contains_key(id) {
            return;
        }

        let attrs = Self::attrs_as_string(attributes.into_iter());
        let line = format!(r#"{}"{}" [{}];"#, Self::INDENT, id, attrs);
        self.nodes.insert(id.clone(), line);
    }

    pub fn add_edge<I, S>(&mut self, from: &K, to: K, attributes: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = (S, DotAttribute)>,
    {
        let attrs = Self::attrs_as_string(attributes.into_iter());
        let line = format!(r#"{}"{}" -> "{}" [{}];"#, Self::INDENT, from, to, attrs);
        self.edges.push(line);
    }
    pub fn add_edge_simple(&mut self, from: &K, to: &K) {
        let line = format!(r#"{}"{}" -> "{}";"#, Self::INDENT, from, to);
        self.edges.push(line);
    }

    fn attrs_as_string<I, S>(attrs: I) -> String
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, DotAttribute)>,
    {
        attrs
            .map(|(key, value)| {
                let k = key.as_ref();
                format!("{k}={value}")
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let name = self.name.as_deref().unwrap_or("G");
        writeln!(writer, "digraph {name} {{")?;
        writeln!(writer, "{}rankdir=LR;", Self::INDENT)?;
        writer.write_all(b"\n")?;

        for (_, node) in self.nodes.iter() {
            writeln!(writer, "{node}")?;
        }

        writer.write_all(b"\n")?;

        for edge in self.edges.iter() {
            writeln!(writer, "{edge}")?;
        }

        writer.write_all(b"}")?;
        writer.flush()
    }
}

impl<K: Clone + PartialEq + Eq + std::hash::Hash + Display> ToDot for DotBuilder<K> {
    fn to_dot<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write(writer)
    }
}
