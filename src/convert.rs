use crate::Paragraph;

pub trait FromDeb822Paragraph {
    /// Convert a paragraph to this object.
    fn from_paragraph(paragraph: &Paragraph) -> Result<Self, String> where Self: Sized;
}

pub trait ToDeb822Paragraph {
    /// Convert this object to a paragraph.
    fn to_paragraph(&self) -> Paragraph;

    /// Update the given paragraph with the values from this object.
    fn update_paragraph(&self, paragraph: &mut Paragraph);
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "derive")]
    use super::*;

    #[cfg(feature = "derive")]
    mod derive {
        use crate as deb822_lossless;
        use crate::Deb822;
        use super::*;

        #[test]
        fn test_derive() {
            #[derive(Deb822)]
            struct Foo {
                bar: String,
                baz: i32,
                blah: Option<String>,
            }

            let foo = Foo {
                bar: "hello".to_string(),
                baz: 42,
                blah: None,
            };

            let paragraph = foo.to_paragraph();
            assert_eq!(paragraph.get("bar"), Some("hello".to_string()));
            assert_eq!(paragraph.get("baz"), Some("42".to_string()));
            assert_eq!(paragraph.get("blah"), None);
        }

        #[test]
        fn test_optional_missing() {
            #[derive(Deb822)]
            struct Foo {
                bar: String,
                baz: Option<String>,
            }

            let foo = Foo {
                bar: "hello".to_string(),
                baz: None,
            };

            let paragraph = foo.to_paragraph();
            assert_eq!(paragraph.get("bar"), Some("hello".to_string()));
            assert_eq!(paragraph.get("baz"), None);

            assert_eq!("bar: hello\n", paragraph.to_string());
        }

        #[test]
        fn test_update_preserve_comments() {
            let mut para: Paragraph = "bar: bar\n# comment\nbaz: blah\n".parse().unwrap();

            #[derive(Deb822)]
            struct Foo {
                bar: String,
                baz: String,
            }

            let mut foo: Foo = Foo::from_paragraph(&para).unwrap();
            assert_eq!(foo.bar, "bar");
            assert_eq!(foo.baz, "blah");

            foo.bar = "new".to_string();

            foo.update_paragraph(&mut para);

            assert_eq!(para.get("bar"), Some("new".to_string()));
            assert_eq!(para.get("baz"), Some("blah".to_string()));
            assert_eq!(para.to_string(), "bar: new\n# comment\nbaz: blah\n");
        }
    }
}
