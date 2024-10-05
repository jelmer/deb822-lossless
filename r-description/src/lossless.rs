use crate::RCode;
/// A library for parsing and manipulating R DESCRIPTION files.
///
/// See https://r-pkgs.org/description.html for more information.
use deb822_lossless::Paragraph;
pub use relations::{Relation, Relations};

pub struct RDescription(Paragraph);

impl std::fmt::Display for RDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for RDescription {
    fn default() -> Self {
        Self(Paragraph::new())
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(deb822_lossless::ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<deb822_lossless::ParseError> for Error {
    fn from(e: deb822_lossless::ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::str::FromStr for RDescription {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Paragraph::from_str(s)?))
    }
}

impl RDescription {
    pub fn new() -> Self {
        Self(Paragraph::new())
    }

    pub fn package(&self) -> Option<String> {
        self.0.get("Package")
    }
    pub fn set_package(&mut self, package: &str) {
        self.0.insert("Package", package);
    }

    /// One line description of the package, and is often shown in a package listing
    ///
    /// It should be plain text (no markup), capitalised like a title, and NOT end in a period.
    /// Keep it short: listings will often truncate the title to 65 characters.
    pub fn title(&self) -> Option<String> {
        self.0.get("Title")
    }

    pub fn maintainer(&self) -> Option<String> {
        self.0.get("Maintainer")
    }

    pub fn set_maintainer(&mut self, maintainer: &str) {
        self.0.insert("Maintainer", maintainer);
    }

    pub fn authors(&self) -> Option<RCode> {
        self.0.get("Authors@R").map(|s| s.parse().unwrap())
    }

    pub fn set_authors(&mut self, authors: &RCode) {
        self.0.insert("Authors@R", &authors.to_string());
    }

    pub fn set_title(&mut self, title: &str) {
        self.0.insert("Title", title);
    }

    pub fn description(&self) -> Option<String> {
        self.0.get("Description")
    }

    pub fn set_description(&mut self, description: &str) {
        self.0.insert("Description", description);
    }

    pub fn version(&self) -> Option<String> {
        self.0.get("Version")
    }

    pub fn set_version(&mut self, version: &str) {
        self.0.insert("Version", version);
    }

    pub fn encoding(&self) -> Option<String> {
        self.0.get("Encoding")
    }

    pub fn set_encoding(&mut self, encoding: &str) {
        self.0.insert("Encoding", encoding);
    }

    pub fn license(&self) -> Option<String> {
        self.0.get("License")
    }

    pub fn set_license(&mut self, license: &str) {
        self.0.insert("License", license);
    }

    pub fn roxygen_note(&self) -> Option<String> {
        self.0.get("RoxygenNote")
    }

    pub fn set_roxygen_note(&mut self, roxygen_note: &str) {
        self.0.insert("RoxygenNote", roxygen_note);
    }

    pub fn roxygen(&self) -> Option<String> {
        self.0.get("Roxygen")
    }

    pub fn set_roxygen(&mut self, roxygen: &str) {
        self.0.insert("Roxygen", roxygen);
    }

    /// The URL of the package's homepage.
    pub fn url(&self) -> Option<String> {
        // TODO: parse list of URLs, separated by commas
        self.0.get("URL")
    }

    pub fn set_url(&mut self, url: &str) {
        // TODO: parse list of URLs, separated by commas
        self.0.insert("URL", url);
    }

    pub fn bug_reports(&self) -> Option<url::Url> {
        self.0
            .get("BugReports")
            .map(|s| url::Url::parse(s.as_str()).unwrap())
    }

    pub fn set_bug_reports(&mut self, bug_reports: &url::Url) {
        self.0.insert("BugReports", bug_reports.as_str());
    }

    pub fn imports(&self) -> Option<Vec<String>> {
        self.0
            .get("Imports")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_imports(&mut self, imports: &[&str]) {
        self.0.insert("Imports", &imports.join(", "));
    }

    pub fn suggests(&self) -> Option<Relations> {
        self.0.get("Suggests").map(|s| s.parse().unwrap())
    }

    pub fn set_suggests(&mut self, suggests: Relations) {
        self.0.insert("Suggests", &suggests.to_string());
    }

    pub fn depends(&self) -> Option<Relations> {
        self.0.get("Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_depends(&mut self, depends: Relations) {
        self.0.insert("Depends", &depends.to_string());
    }

    pub fn linking_to(&self) -> Option<Vec<String>> {
        self.0
            .get("LinkingTo")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_linking_to(&mut self, linking_to: &[&str]) {
        self.0.insert("LinkingTo", &linking_to.join(", "));
    }

    pub fn lazy_data(&self) -> Option<bool> {
        self.0.get("LazyData").map(|s| s == "true")
    }

    pub fn set_lazy_data(&mut self, lazy_data: bool) {
        self.0
            .insert("LazyData", if lazy_data { "true" } else { "false" });
    }

    pub fn collate(&self) -> Option<String> {
        self.0.get("Collate")
    }

    pub fn set_collate(&mut self, collate: &str) {
        self.0.insert("Collate", collate);
    }

    pub fn vignette_builder(&self) -> Option<Vec<String>> {
        self.0
            .get("VignetteBuilder")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_vignette_builder(&mut self, vignette_builder: &[&str]) {
        self.0
            .insert("VignetteBuilder", &vignette_builder.join(", "));
    }

    pub fn system_requirements(&self) -> Option<Vec<String>> {
        self.0
            .get("SystemRequirements")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_system_requirements(&mut self, system_requirements: &[&str]) {
        self.0
            .insert("SystemRequirements", &system_requirements.join(", "));
    }

    pub fn date(&self) -> Option<String> {
        self.0.get("Date")
    }

    pub fn set_date(&mut self, date: &str) {
        self.0.insert("Date", date);
    }
}

pub mod relations {
    //! Parser for relationship fields like `Depends`, `Recommends`, etc.
    //!
    //! # Example
    //! ```
    //! use r_description::lossless::{Relations, Relation};
    //! use r_description::relations::VersionConstraint;
    //!
    //! let mut relations: Relations = r"cli (>= 0.19.0), R".parse().unwrap();
    //! assert_eq!(relations.to_string(), "cli (>= 0.19.0), R");
    //! assert!(relations.satisfied_by(|name: &str| -> Option<r_description::version::Version> {
    //!    match name {
    //!    "cli" => Some("0.19.0".parse().unwrap()),
    //!    "R" => Some("2.25.1".parse().unwrap()),
    //!    _ => None
    //!    }}));
    //! relations.remove_relation(1);
    //! assert_eq!(relations.to_string(), "cli (>= 0.19.0)");
    //! ```
    use crate::relations::SyntaxKind::{self, *};
    use crate::relations::VersionConstraint;
    use crate::version::Version;
    use rowan::{Direction, NodeOrToken};

    /// Error type for parsing relations fields
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError(Vec<String>);

    impl std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            for err in &self.0 {
                writeln!(f, "{}", err)?;
            }
            Ok(())
        }
    }

    impl std::error::Error for ParseError {}

    /// Second, implementing the `Language` trait teaches rowan to convert between
    /// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
    /// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Lang {}
    impl rowan::Language for Lang {
        type Kind = SyntaxKind;
        fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
            unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
        }
        fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
            kind.into()
        }
    }

    /// GreenNode is an immutable tree, which is cheap to change,
    /// but doesn't contain offsets and parent pointers.
    use rowan::{GreenNode, GreenToken};

    /// You can construct GreenNodes by hand, but a builder
    /// is helpful for top-down parsers: it maintains a stack
    /// of currently in-progress nodes
    use rowan::GreenNodeBuilder;

    /// The parse results are stored as a "green tree".
    /// We'll discuss working with the results later
    struct Parse {
        green_node: GreenNode,
        #[allow(unused)]
        errors: Vec<String>,
    }

    fn parse(text: &str) -> Parse {
        struct Parser {
            /// input tokens, including whitespace,
            /// in *reverse* order.
            tokens: Vec<(SyntaxKind, String)>,
            /// the in-progress tree.
            builder: GreenNodeBuilder<'static>,
            /// the list of syntax errors we've accumulated
            /// so far.
            errors: Vec<String>,
        }

        impl Parser {
            fn error(&mut self, error: String) {
                self.errors.push(error);
                self.builder.start_node(SyntaxKind::ERROR.into());
                if self.current().is_some() {
                    self.bump();
                }
                self.builder.finish_node();
            }

            fn parse_relation(&mut self) {
                self.builder.start_node(SyntaxKind::RELATION.into());
                if self.current() == Some(IDENT) {
                    self.bump();
                } else {
                    self.error("Expected package name".to_string());
                }
                match self.peek_past_ws() {
                    Some(COMMA) => {}
                    None | Some(L_PARENS) => {
                        self.skip_ws();
                    }
                    e => {
                        self.skip_ws();
                        self.error(format!(
                            "Expected ':' or '|' or '[' or '<' or ',' but got {:?}",
                            e
                        ));
                    }
                }

                if self.peek_past_ws() == Some(L_PARENS) {
                    self.skip_ws();
                    self.builder.start_node(VERSION.into());
                    self.bump();
                    self.skip_ws();

                    self.builder.start_node(CONSTRAINT.into());

                    while self.current() == Some(L_ANGLE)
                        || self.current() == Some(R_ANGLE)
                        || self.current() == Some(EQUAL)
                    {
                        self.bump();
                    }

                    self.builder.finish_node();

                    self.skip_ws();

                    if self.current() == Some(IDENT) {
                        self.bump();
                    } else {
                        self.error("Expected version".to_string());
                    }

                    if self.current() == Some(R_PARENS) {
                        self.bump();
                    } else {
                        self.error("Expected ')'".to_string());
                    }

                    self.builder.finish_node();
                }

                self.builder.finish_node();
            }

            fn parse(mut self) -> Parse {
                self.builder.start_node(SyntaxKind::ROOT.into());

                self.skip_ws();

                while self.current().is_some() {
                    match self.current() {
                        Some(IDENT) => self.parse_relation(),
                        Some(COMMA) => {
                            // Empty relation, but that's okay - probably?
                        }
                        Some(c) => {
                            self.error(format!("expected identifier or comma but got {:?}", c));
                        }
                        None => {
                            self.error("expected identifier but got end of file".to_string());
                        }
                    }

                    self.skip_ws();
                    match self.current() {
                        Some(COMMA) => {
                            self.bump();
                        }
                        None => {
                            break;
                        }
                        c => {
                            self.error(format!("expected comma or end of file but got {:?}", c));
                        }
                    }
                    self.skip_ws();
                }

                self.builder.finish_node();
                // Turn the builder into a GreenNode
                Parse {
                    green_node: self.builder.finish(),
                    errors: self.errors,
                }
            }
            /// Advance one token, adding it to the current branch of the tree builder.
            fn bump(&mut self) {
                let (kind, text) = self.tokens.pop().unwrap();
                self.builder.token(kind.into(), text.as_str());
            }
            /// Peek at the first unprocessed token
            fn current(&self) -> Option<SyntaxKind> {
                self.tokens.last().map(|(kind, _)| *kind)
            }
            fn skip_ws(&mut self) {
                while self.current() == Some(WHITESPACE) || self.current() == Some(NEWLINE) {
                    self.bump()
                }
            }

            fn peek_past_ws(&self) -> Option<SyntaxKind> {
                let mut i = self.tokens.len();
                while i > 0 {
                    i -= 1;
                    match self.tokens[i].0 {
                        WHITESPACE | NEWLINE => {}
                        _ => return Some(self.tokens[i].0),
                    }
                }
                None
            }
        }

        let mut tokens = crate::relations::lex(text);
        tokens.reverse();
        Parser {
            tokens,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
        .parse()
    }

    /// To work with the parse results we need a view into the
    /// green tree - the Syntax tree.
    /// It is also immutable, like a GreenNode,
    /// but it contains parent pointers, offsets, and
    /// has identity semantics.

    type SyntaxNode = rowan::SyntaxNode<Lang>;
    #[allow(unused)]
    type SyntaxToken = rowan::SyntaxToken<Lang>;
    #[allow(unused)]
    type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

    impl Parse {
        fn root_mut(&self) -> Relations {
            Relations::cast(SyntaxNode::new_root_mut(self.green_node.clone())).unwrap()
        }
    }

    macro_rules! ast_node {
        ($ast:ident, $kind:ident) => {
            /// A node in the syntax tree representing a $ast
            #[repr(transparent)]
            pub struct $ast(SyntaxNode);
            impl $ast {
                #[allow(unused)]
                fn cast(node: SyntaxNode) -> Option<Self> {
                    if node.kind() == $kind {
                        Some(Self(node))
                    } else {
                        None
                    }
                }
            }

            impl std::fmt::Display for $ast {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(&self.0.text().to_string())
                }
            }
        };
    }

    ast_node!(Relations, ROOT);
    ast_node!(Relation, RELATION);

    impl PartialEq for Relations {
        fn eq(&self, other: &Self) -> bool {
            self.relations().collect::<Vec<_>>() == other.relations().collect::<Vec<_>>()
        }
    }

    impl PartialEq for Relation {
        fn eq(&self, other: &Self) -> bool {
            self.name() == other.name() && self.version() == other.version()
        }
    }

    #[cfg(feature = "serde")]
    impl serde::Serialize for Relations {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let rep = self.to_string();
            serializer.serialize_str(&rep)
        }
    }

    #[cfg(feature = "serde")]
    impl<'de> serde::Deserialize<'de> for Relations {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            let relations = s.parse().map_err(serde::de::Error::custom)?;
            Ok(relations)
        }
    }

    impl std::fmt::Debug for Relations {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut s = f.debug_struct("Relations");

            for relation in self.relations() {
                s.field("relation", &relation);
            }

            s.finish()
        }
    }

    impl std::fmt::Debug for Relation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut s = f.debug_struct("Relation");

            s.field("name", &self.name());

            if let Some((vc, version)) = self.version() {
                s.field("version", &vc);
                s.field("version", &version);
            }

            s.finish()
        }
    }

    #[cfg(feature = "serde")]
    impl serde::Serialize for Relation {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let rep = self.to_string();
            serializer.serialize_str(&rep)
        }
    }

    #[cfg(feature = "serde")]
    impl<'de> serde::Deserialize<'de> for Relation {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            let relation = s.parse().map_err(serde::de::Error::custom)?;
            Ok(relation)
        }
    }

    impl Default for Relations {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Relations {
        /// Create a new relations field
        pub fn new() -> Self {
            Self::from(vec![])
        }

        /// Wrap and sort this relations field
        #[must_use]
        pub fn wrap_and_sort(self) -> Self {
            let mut entries = self
                .relations()
                .map(|e| e.wrap_and_sort())
                .collect::<Vec<_>>();
            entries.sort();
            // TODO: preserve comments
            Self::from(entries)
        }

        /// Iterate over the entries in this relations field
        pub fn relations(&self) -> impl Iterator<Item = Relation> + '_ {
            self.0.children().filter_map(Relation::cast)
        }

        /// Iterate over the entries in this relations field
        pub fn iter(&self) -> impl Iterator<Item = Relation> + '_ {
            self.relations()
        }

        /// Remove the entry at the given index
        pub fn get_relation(&self, idx: usize) -> Option<Relation> {
            self.relations().nth(idx)
        }

        /// Remove the relation at the given index
        pub fn remove_relation(&mut self, idx: usize) -> Relation {
            let mut relation = self.get_relation(idx).unwrap();
            relation.remove();
            relation
        }

        /// Insert a new relation at the given index
        pub fn insert(&mut self, idx: usize, relation: Relation) {
            let is_empty = !self.0.children_with_tokens().any(|n| n.kind() == COMMA);
            let (position, new_children) = if let Some(current_relation) = self.relations().nth(idx)
            {
                let to_insert: Vec<NodeOrToken<GreenNode, GreenToken>> = if idx == 0 && is_empty {
                    vec![relation.0.green().into()]
                } else {
                    vec![
                        relation.0.green().into(),
                        NodeOrToken::Token(GreenToken::new(COMMA.into(), ",")),
                        NodeOrToken::Token(GreenToken::new(WHITESPACE.into(), " ")),
                    ]
                };

                (current_relation.0.index(), to_insert)
            } else {
                let child_count = self.0.children_with_tokens().count();
                (
                    child_count,
                    if idx == 0 {
                        vec![relation.0.green().into()]
                    } else {
                        vec![
                            NodeOrToken::Token(GreenToken::new(COMMA.into(), ",")),
                            NodeOrToken::Token(GreenToken::new(WHITESPACE.into(), " ")),
                            relation.0.green().into(),
                        ]
                    },
                )
            };
            // We can safely replace the root here since Relations is a root node
            self.0 = SyntaxNode::new_root_mut(
                self.0.replace_with(
                    self.0
                        .green()
                        .splice_children(position..position, new_children),
                ),
            );
        }

        /// Replace the relation at the given index
        pub fn replace(&mut self, idx: usize, relation: Relation) {
            let current_relation = self.get_relation(idx).unwrap();
            self.0.splice_children(
                current_relation.0.index()..current_relation.0.index() + 1,
                vec![relation.0.into()],
            );
        }

        /// Push a new relation to the relations field
        pub fn push(&mut self, relation: Relation) {
            let pos = self.relations().count();
            self.insert(pos, relation);
        }

        /// Parse a relations field from a string, allowing syntax errors
        pub fn parse_relaxed(s: &str) -> (Relations, Vec<String>) {
            let parse = parse(s);
            (parse.root_mut(), parse.errors)
        }

        /// Check if this relations field is satisfied by the given package versions.
        pub fn satisfied_by(
            &self,
            package_version: impl crate::relations::VersionLookup + Copy,
        ) -> bool {
            self.relations().all(|e| e.satisfied_by(package_version))
        }

        /// Check if this relations field is empty
        pub fn is_empty(&self) -> bool {
            self.relations().count() == 0
        }

        /// Get the number of entries in this relations field
        pub fn len(&self) -> usize {
            self.relations().count()
        }
    }

    impl From<Vec<Relation>> for Relations {
        fn from(entries: Vec<Relation>) -> Self {
            let mut builder = GreenNodeBuilder::new();
            builder.start_node(ROOT.into());
            for (i, relation) in entries.into_iter().enumerate() {
                if i > 0 {
                    builder.token(COMMA.into(), ",");
                    builder.token(WHITESPACE.into(), " ");
                }
                inject(&mut builder, relation.0);
            }
            builder.finish_node();
            Relations(SyntaxNode::new_root_mut(builder.finish()))
        }
    }

    impl From<Relation> for Relations {
        fn from(relation: Relation) -> Self {
            Self::from(vec![relation])
        }
    }

    fn inject(builder: &mut GreenNodeBuilder, node: SyntaxNode) {
        builder.start_node(node.kind().into());
        for child in node.children_with_tokens() {
            match child {
                rowan::NodeOrToken::Node(child) => {
                    inject(builder, child);
                }
                rowan::NodeOrToken::Token(token) => {
                    builder.token(token.kind().into(), token.text());
                }
            }
        }
        builder.finish_node();
    }

    impl Relation {
        /// Create a new relation
        ///
        /// # Arguments
        /// * `name` - The name of the package
        /// * `version_constraint` - The version constraint and version to use
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::{Relation};
        /// use r_description::relations::VersionConstraint;
        /// let relation = Relation::new("vign", Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())));
        /// assert_eq!(relation.to_string(), "vign (>= 2.0)");
        /// ```
        pub fn new(name: &str, version_constraint: Option<(VersionConstraint, Version)>) -> Self {
            let mut builder = GreenNodeBuilder::new();
            builder.start_node(SyntaxKind::RELATION.into());
            builder.token(IDENT.into(), name);
            if let Some((vc, version)) = version_constraint {
                builder.token(WHITESPACE.into(), " ");
                builder.start_node(SyntaxKind::VERSION.into());
                builder.token(L_PARENS.into(), "(");
                builder.start_node(SyntaxKind::CONSTRAINT.into());
                for c in vc.to_string().chars() {
                    builder.token(
                        match c {
                            '>' => R_ANGLE.into(),
                            '<' => L_ANGLE.into(),
                            '=' => EQUAL.into(),
                            _ => unreachable!(),
                        },
                        c.to_string().as_str(),
                    );
                }
                builder.finish_node();

                builder.token(WHITESPACE.into(), " ");

                builder.token(IDENT.into(), version.to_string().as_str());

                builder.token(R_PARENS.into(), ")");

                builder.finish_node();
            }

            builder.finish_node();
            Relation(SyntaxNode::new_root_mut(builder.finish()))
        }

        /// Wrap and sort this relation
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::Relation;
        /// let relation = "  vign  (  >= 2.0) ".parse::<Relation>().unwrap();
        /// assert_eq!(relation.wrap_and_sort().to_string(), "vign (>= 2.0)");
        /// ```
        #[must_use]
        pub fn wrap_and_sort(&self) -> Self {
            let mut builder = GreenNodeBuilder::new();
            builder.start_node(SyntaxKind::RELATION.into());
            builder.token(IDENT.into(), self.name().as_str());
            if let Some((vc, version)) = self.version() {
                builder.token(WHITESPACE.into(), " ");
                builder.start_node(SyntaxKind::VERSION.into());
                builder.token(L_PARENS.into(), "(");
                builder.start_node(SyntaxKind::CONSTRAINT.into());
                builder.token(
                    match vc {
                        VersionConstraint::GreaterThanEqual => R_ANGLE.into(),
                        VersionConstraint::LessThanEqual => L_ANGLE.into(),
                        VersionConstraint::Equal => EQUAL.into(),
                        VersionConstraint::GreaterThan => R_ANGLE.into(),
                        VersionConstraint::LessThan => L_ANGLE.into(),
                    },
                    vc.to_string().as_str(),
                );
                builder.finish_node();
                builder.token(WHITESPACE.into(), " ");
                builder.token(IDENT.into(), version.to_string().as_str());
                builder.token(R_PARENS.into(), ")");
                builder.finish_node();
            }
            builder.finish_node();
            Relation(SyntaxNode::new_root_mut(builder.finish()))
        }

        /// Create a new simple relation, without any version constraints.
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::Relation;
        /// let relation = Relation::simple("vign");
        /// assert_eq!(relation.to_string(), "vign");
        /// ```
        pub fn simple(name: &str) -> Self {
            Self::new(name, None)
        }

        /// Remove the version constraint from the relation.
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::{Relation};
        /// use r_description::relations::VersionConstraint;
        /// let mut relation = Relation::new("vign", Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())));
        /// relation.drop_constraint();
        /// assert_eq!(relation.to_string(), "vign");
        /// ```
        pub fn drop_constraint(&mut self) -> bool {
            let version_token = self.0.children().find(|n| n.kind() == VERSION);
            if let Some(version_token) = version_token {
                // Remove any whitespace before the version token
                while let Some(prev) = version_token.prev_sibling_or_token() {
                    if prev.kind() == WHITESPACE || prev.kind() == NEWLINE {
                        prev.detach();
                    } else {
                        break;
                    }
                }
                version_token.detach();
                return true;
            }

            false
        }

        /// Return the name of the package in the relation.
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::Relation;
        /// let relation = Relation::simple("vign");
        /// assert_eq!(relation.name(), "vign");
        /// ```
        pub fn name(&self) -> String {
            self.0
                .children_with_tokens()
                .find_map(|it| match it {
                    SyntaxElement::Token(token) if token.kind() == IDENT => Some(token),
                    _ => None,
                })
                .unwrap()
                .text()
                .to_string()
        }

        /// Return the version constraint and the version it is constrained to.
        pub fn version(&self) -> Option<(VersionConstraint, Version)> {
            let vc = self.0.children().find(|n| n.kind() == VERSION);
            let vc = vc.as_ref()?;
            let constraint = vc.children().find(|n| n.kind() == CONSTRAINT);

            let version = vc.children_with_tokens().find_map(|it| match it {
                SyntaxElement::Token(token) if token.kind() == IDENT => Some(token),
                _ => None,
            });

            if let (Some(constraint), Some(version)) = (constraint, version) {
                let vc: VersionConstraint = constraint.to_string().parse().unwrap();
                return Some((vc, (version.text().to_string()).parse().unwrap()));
            } else {
                None
            }
        }

        /// Set the version constraint for this relation
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::{Relation};
        /// use r_description::relations::VersionConstraint;
        /// let mut relation = Relation::simple("vign");
        /// relation.set_version(Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())));
        /// assert_eq!(relation.to_string(), "vign (>= 2.0)");
        /// ```
        pub fn set_version(&mut self, version_constraint: Option<(VersionConstraint, Version)>) {
            let current_version = self.0.children().find(|n| n.kind() == VERSION);
            if let Some((vc, version)) = version_constraint {
                let mut builder = GreenNodeBuilder::new();
                builder.start_node(VERSION.into());
                builder.token(L_PARENS.into(), "(");
                builder.start_node(CONSTRAINT.into());
                match vc {
                    VersionConstraint::GreaterThanEqual => {
                        builder.token(R_ANGLE.into(), ">");
                        builder.token(EQUAL.into(), "=");
                    }
                    VersionConstraint::LessThanEqual => {
                        builder.token(L_ANGLE.into(), "<");
                        builder.token(EQUAL.into(), "=");
                    }
                    VersionConstraint::Equal => {
                        builder.token(EQUAL.into(), "=");
                    }
                    VersionConstraint::GreaterThan => {
                        builder.token(R_ANGLE.into(), ">");
                    }
                    VersionConstraint::LessThan => {
                        builder.token(L_ANGLE.into(), "<");
                    }
                }
                builder.finish_node(); // CONSTRAINT
                builder.token(WHITESPACE.into(), " ");
                builder.token(IDENT.into(), version.to_string().as_str());
                builder.token(R_PARENS.into(), ")");
                builder.finish_node(); // VERSION

                if let Some(current_version) = current_version {
                    self.0.splice_children(
                        current_version.index()..current_version.index() + 1,
                        vec![SyntaxNode::new_root_mut(builder.finish()).into()],
                    );
                } else {
                    let name_node = self.0.children_with_tokens().find(|n| n.kind() == IDENT);
                    let idx = if let Some(name_node) = name_node {
                        name_node.index() + 1
                    } else {
                        0
                    };
                    let new_children = vec![
                        GreenToken::new(WHITESPACE.into(), " ").into(),
                        builder.finish().into(),
                    ];
                    let new_root = SyntaxNode::new_root_mut(
                        self.0.green().splice_children(idx..idx, new_children),
                    );
                    if let Some(parent) = self.0.parent() {
                        parent.splice_children(
                            self.0.index()..self.0.index() + 1,
                            vec![new_root.into()],
                        );
                        self.0 = parent
                            .children_with_tokens()
                            .nth(self.0.index())
                            .unwrap()
                            .clone()
                            .into_node()
                            .unwrap();
                    } else {
                        self.0 = new_root;
                    }
                }
            } else if let Some(current_version) = current_version {
                // Remove any whitespace before the version token
                while let Some(prev) = current_version.prev_sibling_or_token() {
                    if prev.kind() == WHITESPACE || prev.kind() == NEWLINE {
                        prev.detach();
                    } else {
                        break;
                    }
                }
                current_version.detach();
            }
        }

        /// Remove this relation
        ///
        /// # Example
        /// ```
        /// use r_description::lossless::{Relation, Relations};
        /// let mut relations: Relations = r"cli (>= 0.19.0), blah (<< 1.26.0)".parse().unwrap();
        /// let mut relation = relations.get_relation(0).unwrap();
        /// assert_eq!(relation.to_string(), "cli (>= 0.19.0)");
        /// relation.remove();
        /// assert_eq!(relations.to_string(), "blah (<< 1.26.0)");
        /// ```
        pub fn remove(&mut self) {
            let is_first = !self
                .0
                .siblings(Direction::Prev)
                .skip(1)
                .any(|n| n.kind() == RELATION);
            if !is_first {
                // Not the first item in the list. Remove whitespace backwards to the previous
                // pipe, the pipe and any whitespace until the previous relation
                while let Some(n) = self.0.prev_sibling_or_token() {
                    if n.kind() == WHITESPACE || n.kind() == NEWLINE {
                        n.detach();
                    } else if n.kind() == COMMA {
                        n.detach();
                        break;
                    } else {
                        break;
                    }
                }
                while let Some(n) = self.0.prev_sibling_or_token() {
                    if n.kind() == WHITESPACE || n.kind() == NEWLINE {
                        n.detach();
                    } else {
                        break;
                    }
                }
            } else {
                // First item in the list. Remove whitespace up to the pipe, the pipe and anything
                // before the next relation
                while let Some(n) = self.0.next_sibling_or_token() {
                    if n.kind() == WHITESPACE || n.kind() == NEWLINE {
                        n.detach();
                    } else if n.kind() == COMMA {
                        n.detach();
                        break;
                    } else {
                        panic!("Unexpected node: {:?}", n);
                    }
                }

                while let Some(n) = self.0.next_sibling_or_token() {
                    if n.kind() == WHITESPACE || n.kind() == NEWLINE {
                        n.detach();
                    } else {
                        break;
                    }
                }
            }
            self.0.detach();
        }

        /// Check if this relation is satisfied by the given package version.
        pub fn satisfied_by(
            &self,
            package_version: impl crate::relations::VersionLookup + Copy,
        ) -> bool {
            let name = self.name();
            let version = self.version();
            if let Some(version) = version {
                if let Some(package_version) = package_version.lookup_version(&name) {
                    match version.0 {
                        VersionConstraint::GreaterThanEqual => {
                            package_version.into_owned() >= version.1
                        }
                        VersionConstraint::LessThanEqual => {
                            package_version.into_owned() <= version.1
                        }
                        VersionConstraint::Equal => package_version.into_owned() == version.1,
                        VersionConstraint::GreaterThan => package_version.into_owned() > version.1,
                        VersionConstraint::LessThan => package_version.into_owned() < version.1,
                    }
                } else {
                    false
                }
            } else {
                true
            }
        }
    }

    impl PartialOrd for Relation {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            // Compare by name first, then by version
            let name_cmp = self.name().cmp(&other.name());
            if name_cmp != std::cmp::Ordering::Equal {
                return Some(name_cmp);
            }

            let self_version = self.version();
            let other_version = other.version();

            match (self_version, other_version) {
                (Some((self_vc, self_version)), Some((other_vc, other_version))) => {
                    let vc_cmp = self_vc.cmp(&other_vc);
                    if vc_cmp != std::cmp::Ordering::Equal {
                        return Some(vc_cmp);
                    }

                    Some(self_version.cmp(&other_version))
                }
                (Some(_), None) => Some(std::cmp::Ordering::Greater),
                (None, Some(_)) => Some(std::cmp::Ordering::Less),
                (None, None) => Some(std::cmp::Ordering::Equal),
            }
        }
    }

    impl Eq for Relation {}

    impl Ord for Relation {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl std::str::FromStr for Relations {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parse = parse(s);
            if parse.errors.is_empty() {
                Ok(parse.root_mut())
            } else {
                Err(parse.errors.join("\n"))
            }
        }
    }

    impl std::str::FromStr for Relation {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let rels = s.parse::<Relations>()?;
            let mut relations = rels.relations();

            let relation = if let Some(relation) = relations.next() {
                relation
            } else {
                return Err("No relation found".to_string());
            };

            if relations.next().is_some() {
                return Err("Multiple relations found".to_string());
            }

            Ok(relation)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse() {
            let input = "cli";
            let parsed: Relations = input.parse().unwrap();
            assert_eq!(parsed.to_string(), input);
            assert_eq!(parsed.relations().count(), 1);
            let relation = parsed.relations().next().unwrap();
            assert_eq!(relation.to_string(), "cli");
            assert_eq!(relation.version(), None);

            let input = "cli (>= 0.20.21)";
            let parsed: Relations = input.parse().unwrap();
            assert_eq!(parsed.to_string(), input);
            assert_eq!(parsed.relations().count(), 1);
            let relation = parsed.relations().next().unwrap();
            assert_eq!(relation.to_string(), "cli (>= 0.20.21)");
            assert_eq!(
                relation.version(),
                Some((
                    VersionConstraint::GreaterThanEqual,
                    "0.20.21".parse().unwrap()
                ))
            );
        }

        #[test]
        fn test_multiple() {
            let input = "cli (>= 0.20.21), cli (<< 0.21)";
            let parsed: Relations = input.parse().unwrap();
            assert_eq!(parsed.to_string(), input);
            assert_eq!(parsed.relations().count(), 2);
            let relation = parsed.relations().next().unwrap();
            assert_eq!(relation.to_string(), "cli (>= 0.20.21)");
            assert_eq!(
                relation.version(),
                Some((
                    VersionConstraint::GreaterThanEqual,
                    "0.20.21".parse().unwrap()
                ))
            );
            let relation = parsed.relations().nth(1).unwrap();
            assert_eq!(relation.to_string(), "cli (<< 0.21)");
            assert_eq!(
                relation.version(),
                Some((VersionConstraint::LessThan, "0.21".parse().unwrap()))
            );
        }

        #[test]
        fn test_new() {
            let r = Relation::new(
                "cli",
                Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())),
            );

            assert_eq!(r.to_string(), "cli (>= 2.0)");
        }

        #[test]
        fn test_drop_constraint() {
            let mut r = Relation::new(
                "cli",
                Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())),
            );

            r.drop_constraint();

            assert_eq!(r.to_string(), "cli");
        }

        #[test]
        fn test_simple() {
            let r = Relation::simple("cli");

            assert_eq!(r.to_string(), "cli");
        }

        #[test]
        fn test_remove_first_relation() {
            let mut rels: Relations = r#"cli (>= 0.20.21), cli (<< 0.21)"#.parse().unwrap();
            let removed = rels.remove_relation(0);
            assert_eq!(removed.to_string(), "cli (>= 0.20.21)");
            assert_eq!(rels.to_string(), "cli (<< 0.21)");
        }

        #[test]
        fn test_remove_last_relation() {
            let mut rels: Relations = r#"cli (>= 0.20.21), cli (<< 0.21)"#.parse().unwrap();
            rels.remove_relation(1);
            assert_eq!(rels.to_string(), "cli (>= 0.20.21)");
        }

        #[test]
        fn test_remove_middle() {
            let mut rels: Relations =
                r#"cli (>= 0.20.21), cli (<< 0.21), cli (<< 0.22)"#.parse().unwrap();
            rels.remove_relation(1);
            assert_eq!(rels.to_string(), "cli (>= 0.20.21), cli (<< 0.22)");
        }

        #[test]
        fn test_remove_added() {
            let mut rels: Relations = r#"cli (>= 0.20.21)"#.parse().unwrap();
            let relation = Relation::simple("cli");
            rels.push(relation);
            rels.remove_relation(1);
            assert_eq!(rels.to_string(), "cli (>= 0.20.21)");
        }

        #[test]
        fn test_push() {
            let mut rels: Relations = r#"cli (>= 0.20.21)"#.parse().unwrap();
            let relation = Relation::simple("cli");
            rels.push(relation);
            assert_eq!(rels.to_string(), "cli (>= 0.20.21), cli");
        }

        #[test]
        fn test_push_from_empty() {
            let mut rels: Relations = "".parse().unwrap();
            let relation = Relation::simple("cli");
            rels.push(relation);
            assert_eq!(rels.to_string(), "cli");
        }

        #[test]
        fn test_insert() {
            let mut rels: Relations = r#"cli (>= 0.20.21), cli (<< 0.21)"#.parse().unwrap();
            let relation = Relation::simple("cli");
            rels.insert(1, relation);
            assert_eq!(rels.to_string(), "cli (>= 0.20.21), cli, cli (<< 0.21)");
        }

        #[test]
        fn test_insert_at_start() {
            let mut rels: Relations = r#"cli (>= 0.20.21), cli (<< 0.21)"#.parse().unwrap();
            let relation = Relation::simple("cli");
            rels.insert(0, relation);
            assert_eq!(rels.to_string(), "cli, cli (>= 0.20.21), cli (<< 0.21)");
        }

        #[test]
        fn test_insert_after_error() {
            let (mut rels, errors) = Relations::parse_relaxed("@foo@, debhelper (>= 1.0)");
            assert_eq!(
                errors,
                vec![
                    "expected identifier or comma but got ERROR",
                    "expected comma or end of file but got Some(IDENT)",
                    "expected identifier or comma but got ERROR"
                ]
            );
            let relation = Relation::simple("bar");
            rels.push(relation);
            assert_eq!(rels.to_string(), "@foo@, debhelper (>= 1.0), bar");
        }

        #[test]
        fn test_insert_before_error() {
            let (mut rels, errors) = Relations::parse_relaxed("debhelper (>= 1.0), @foo@, bla");
            assert_eq!(
                errors,
                vec![
                    "expected identifier or comma but got ERROR",
                    "expected comma or end of file but got Some(IDENT)",
                    "expected identifier or comma but got ERROR"
                ]
            );
            let relation = Relation::simple("bar");
            rels.insert(0, relation);
            assert_eq!(rels.to_string(), "bar, debhelper (>= 1.0), @foo@, bla");
        }

        #[test]
        fn test_replace() {
            let mut rels: Relations = r#"cli (>= 0.20.21), cli (<< 0.21)"#.parse().unwrap();
            let relation = Relation::simple("cli");
            rels.replace(1, relation);
            assert_eq!(rels.to_string(), "cli (>= 0.20.21), cli");
        }

        #[test]
        fn test_parse_relation() {
            let parsed: Relation = "cli (>= 0.20.21)".parse().unwrap();
            assert_eq!(parsed.to_string(), "cli (>= 0.20.21)");
            assert_eq!(
                parsed.version(),
                Some((
                    VersionConstraint::GreaterThanEqual,
                    "0.20.21".parse().unwrap()
                ))
            );
            assert_eq!(
                "foo, bar".parse::<Relation>().unwrap_err(),
                "Multiple relations found"
            );
            assert_eq!("".parse::<Relation>().unwrap_err(), "No relation found");
        }

        #[test]
        fn test_relations_satisfied_by() {
            let rels: Relations = "cli (>= 0.20.21), cli (<< 0.21)".parse().unwrap();
            let satisfied = |name: &str| -> Option<Version> {
                match name {
                    "cli" => Some("0.20.21".parse().unwrap()),
                    _ => None,
                }
            };
            assert!(rels.satisfied_by(satisfied));

            let satisfied = |name: &str| match name {
                "cli" => Some("0.21".parse().unwrap()),
                _ => None,
            };
            assert!(!rels.satisfied_by(satisfied));

            let satisfied = |name: &str| match name {
                "cli" => Some("0.20.20".parse().unwrap()),
                _ => None,
            };
            assert!(!rels.satisfied_by(satisfied));
        }

        #[test]
        fn test_wrap_and_sort_relation() {
            let relation: Relation = "   cli   (>=   11.0)".parse().unwrap();

            let wrapped = relation.wrap_and_sort();

            assert_eq!(wrapped.to_string(), "cli (>= 11.0)");
        }

        #[test]
        fn test_wrap_and_sort_relations() {
            let relations: Relations = "cli (>= 0.20.21)  , \n\n\n\ncli (<< 0.21)".parse().unwrap();

            let wrapped = relations.wrap_and_sort();

            assert_eq!(wrapped.to_string(), "cli (<< 0.21), cli (>= 0.20.21)");
        }

        #[cfg(feature = "serde")]
        #[test]
        fn test_serialize_relations() {
            let relations: Relations = "cli (>= 0.20.21), cli (<< 0.21)".parse().unwrap();
            let serialized = serde_json::to_string(&relations).unwrap();
            assert_eq!(serialized, r#""cli (>= 0.20.21), cli (<< 0.21)""#);
        }

        #[cfg(feature = "serde")]
        #[test]
        fn test_deserialize_relations() {
            let relations: Relations = "cli (>= 0.20.21), cli (<< 0.21)".parse().unwrap();
            let serialized = serde_json::to_string(&relations).unwrap();
            let deserialized: Relations = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.to_string(), relations.to_string());
        }

        #[cfg(feature = "serde")]
        #[test]
        fn test_serialize_relation() {
            let relation: Relation = "cli (>= 0.20.21)".parse().unwrap();
            let serialized = serde_json::to_string(&relation).unwrap();
            assert_eq!(serialized, r#""cli (>= 0.20.21)""#);
        }

        #[cfg(feature = "serde")]
        #[test]
        fn test_deserialize_relation() {
            let relation: Relation = "cli (>= 0.20.21)".parse().unwrap();
            let serialized = serde_json::to_string(&relation).unwrap();
            let deserialized: Relation = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.to_string(), relation.to_string());
        }

        #[test]
        fn test_relation_set_version() {
            let mut rel: Relation = "vign".parse().unwrap();
            rel.set_version(None);
            assert_eq!("vign", rel.to_string());
            rel.set_version(Some((
                VersionConstraint::GreaterThanEqual,
                "2.0".parse().unwrap(),
            )));
            assert_eq!("vign (>= 2.0)", rel.to_string());
            rel.set_version(None);
            assert_eq!("vign", rel.to_string());
            rel.set_version(Some((
                VersionConstraint::GreaterThanEqual,
                "2.0".parse().unwrap(),
            )));
            rel.set_version(Some((
                VersionConstraint::GreaterThanEqual,
                "1.1".parse().unwrap(),
            )));
            assert_eq!("vign (>= 1.1)", rel.to_string());
        }

        #[test]
        fn test_wrap_and_sort_removes_empty_entries() {
            let relations: Relations = "foo, , bar, ".parse().unwrap();
            let wrapped = relations.wrap_and_sort();
            assert_eq!(wrapped.to_string(), "bar, foo");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = r###"Package: mypackage
Title: What the Package Does (One Line, Title Case)
Version: 0.0.0.9000
Authors@R: 
    person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
           comment = c(ORCID = "YOUR-ORCID-ID"))
Description: What the package does (one paragraph).
License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a
    license
Encoding: UTF-8
Roxygen: list(markdown = TRUE)
RoxygenNote: 7.3.2
"###;
        let desc: RDescription = s.parse().unwrap();

        assert_eq!(desc.package(), Some("mypackage".to_string()));
        assert_eq!(
            desc.title(),
            Some("What the Package Does (One Line, Title Case)".to_string())
        );
        assert_eq!(desc.version(), Some("0.0.0.9000".to_string()));
        assert_eq!(
            desc.authors(),
            Some(RCode(
                r#"person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
comment = c(ORCID = "YOUR-ORCID-ID"))"#
                    .to_string()
            ))
        );
        assert_eq!(
            desc.description(),
            Some("What the package does (one paragraph).".to_string())
        );
        assert_eq!(
            desc.license(),
            Some(
                "`use_mit_license()`, `use_gpl3_license()` or friends to pick a\nlicense"
                    .to_string()
            )
        );
        assert_eq!(desc.encoding(), Some("UTF-8".to_string()));
        assert_eq!(desc.roxygen(), Some("list(markdown = TRUE)".to_string()));
        assert_eq!(desc.roxygen_note(), Some("7.3.2".to_string()));

        assert_eq!(desc.to_string(), s);
    }

    #[test]
    fn test_parse_dplyr() {
        let s = include_str!("../testdata/dplyr.desc");

        let desc: RDescription = s.parse().unwrap();
        assert_eq!("dplyr", desc.package().unwrap());
        assert_eq!(
            "https://dplyr.tidyverse.org, https://github.com/tidyverse/dplyr",
            desc.url().unwrap().as_str()
        );
    }
}
