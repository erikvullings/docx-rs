use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use zip::{result::ZipError, write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{
    app::App,
    content_type::ContentTypes,
    core::Core,
    document::{BodyContent, Document, Para},
    error::Result,
    font_table::FontTable,
    rels::Relationships,
    schema::{
        SCHEMA_CORE, SCHEMA_FONT_TABLE, SCHEMA_OFFICE_DOCUMENT, SCHEMA_REL_EXTENDED, SCHEMA_STYLES,
    },
    style::{Style, Styles},
};

/// A WordprocessingML package
#[derive(Debug, Default)]
pub struct Docx<'a> {
    /// Specifies package-level properties part
    pub app: Option<App<'a>>,
    /// Specifies core properties part
    pub core: Option<Core<'a>>,
    /// Specifies the content type of relationship parts and the main document part.
    pub content_types: ContentTypes<'a>,
    /// Specifies the main document part.
    pub document: Document<'a>,
    /// Specifies the font table part
    pub font_table: Option<FontTable<'a>>,
    /// Specifies the style definitions part
    pub styles: Option<Styles<'a>>,
    /// Specifies the package-level relationship to the main document part
    pub rels: Relationships<'a>,
    /// Specifies the part-level relationship to the main document part
    pub document_rels: Option<Relationships<'a>>,
}

impl<'a> Docx<'a> {
    pub fn write<W: Write + Seek>(&mut self, writer: W) -> Result<W> {
        let mut zip = ZipWriter::new(writer);
        let opt = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        macro_rules! write {
            ($xml:expr, $name:tt) => {
                zip.start_file($name, opt)?;
                $xml.write(&mut zip)?;
            };
            ($xml:expr, $name:tt, $rel:expr, $schema:expr, $target:tt) => {
                write!($xml, $name);
                $rel.add_rel($schema, $target);
            };
        }

        macro_rules! option_write {
            ($xml:expr, $($rest:tt)*) => {
                if let Some(ref xml) = $xml {
                    write!(xml, $($rest)*);
                }
            };
        }

        // content types
        write!(self.content_types, "[Content_Types].xml");

        // document properties
        option_write!(
            self.app,
            "docProps/app.xml",
            self.rels,
            SCHEMA_REL_EXTENDED,
            "docProps/app.xml"
        );
        option_write!(
            self.core,
            "docProps/core.xml",
            self.rels,
            SCHEMA_CORE,
            "docProps/core.xml"
        );

        // documents specific parts
        write!(
            self.document,
            "word/document.xml",
            self.rels, SCHEMA_OFFICE_DOCUMENT, "word/document.xml"
        );
        option_write!(
            self.styles,
            "word/styles.xml",
            self.document_rels.get_or_insert(Relationships::default()),
            SCHEMA_STYLES,
            "styles.xml"
        );
        option_write!(
            self.font_table,
            "word/fontTable.xml",
            self.document_rels.get_or_insert(Relationships::default()),
            SCHEMA_FONT_TABLE,
            "fontTable.xml"
        );

        // relationships
        write!(self.rels, "_rels/.rels");
        option_write!(self.document_rels, "word/_rels/document.xml.rels");

        Ok(zip.finish()?)
    }

    pub fn write_file<P: AsRef<Path>>(&mut self, path: P) -> Result<File> {
        let file = File::create(path)?;
        self.write(file)
    }

    #[inline]
    pub fn insert_para(&mut self, para: Para<'a>) -> &mut Self {
        self.document.body.content.push(BodyContent::Para(para));
        self
    }

    #[inline]
    pub fn insert_style(&mut self, style: Style<'a>) -> &mut Self {
        self.styles
            .get_or_insert(Styles::default())
            .styles
            .push(style);
        self
    }

    /// Creates a style, and returns it.
    #[inline]
    pub fn create_style(&mut self) -> &mut Style<'a> {
        self.styles.get_or_insert(Styles::default()).create_style()
    }

    pub fn into_owned(self) -> Docx<'static> {
        Docx {
            app: self.app.map(|x| x.into_owned()),
            content_types: self.content_types.into_owned(),
            core: self.core.map(|x| x.into_owned()),
            document: self.document.into_owned(),
            document_rels: self.document_rels.map(|x| x.into_owned()),
            font_table: self.font_table.map(|x| x.into_owned()),
            rels: self.rels.into_owned(),
            styles: self.styles.map(|x| x.into_owned()),
        }
    }
}

/// A extracted docx file
pub struct DocxFile {
    app: Option<String>,
    content_types: String,
    core: Option<String>,
    document: String,
    document_rels: Option<String>,
    font_table: Option<String>,
    rels: String,
    styles: Option<String>,
}

impl DocxFile {
    /// Extracts from reader
    pub fn from_reader<T: Read + Seek>(reader: T) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;

        macro_rules! read {
            ($xml:tt, $name:expr) => {{
                let mut file = zip.by_name($name)?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                buffer
            }};
        }

        macro_rules! option_read {
            ($xml:tt, $name:expr) => {
                match zip.by_name($name) {
                    Err(ZipError::FileNotFound) => None,
                    Err(e) => return Err(e.into()),
                    Ok(mut file) => {
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)?;
                        Some(buffer)
                    }
                };
            };
        }

        let app = option_read!(App, "docProps/app.xml");
        let content_types = read!(ContentTypes, "[Content_Types].xml");
        let core = option_read!(Core, "docProps/core.xml");
        let document_rels = option_read!(Relationships, "word/_rels/document.xml.rels");
        let document = read!(Document, "word/document.xml");
        let font_table = option_read!(FontTable, "word/fontTable.xml");
        let rels = read!(Relationships, "_rels/.rels");
        let styles = option_read!(Styles, "word/styles.xml");

        Ok(DocxFile {
            app,
            content_types,
            core,
            document_rels,
            document,
            font_table,
            rels,
            styles,
        })
    }

    /// Extracts from file
    #[inline]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_reader(File::open(path)?)
    }

    /// Parses content into `Docx` struct
    pub fn parse<'a>(&'a self) -> Result<Docx<'a>> {
        let app = if let Some(content) = &self.app {
            Some(App::from_str(content)?)
        } else {
            None
        };

        let document = Document::from_str(&self.document)?;

        let content_types = ContentTypes::from_str(&self.content_types)?;

        let core = if let Some(content) = &self.core {
            Some(Core::from_str(content)?)
        } else {
            None
        };

        let document_rels = if let Some(content) = &self.document_rels {
            Some(Relationships::from_str(content)?)
        } else {
            None
        };

        let font_table = if let Some(content) = &self.font_table {
            Some(FontTable::from_str(content)?)
        } else {
            None
        };

        let rels = Relationships::from_str(&self.rels)?;

        let styles = if let Some(content) = &self.styles {
            Some(Styles::from_str(content)?)
        } else {
            None
        };

        Ok(Docx {
            app,
            content_types,
            core,
            document,
            document_rels,
            font_table,
            rels,
            styles,
        })
    }
}