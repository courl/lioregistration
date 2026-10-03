use std::path::{Path, PathBuf};
use std::sync::Arc;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

use crate::error::{AppError, AppResult};

pub struct EmailAttachmentContext {
    root: PathBuf,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl EmailAttachmentContext {
    pub fn create(assets_root: PathBuf) -> AppResult<Self> {
        let mut book = FontBook::new();
        let mut fonts = Vec::new();

        // 1. Embedded fallback fonts (always available)
        for data in typst_assets::fonts() {
            for font in Font::iter(Bytes::new(data)) {
                book.push(font.info().clone());
                fonts.push(font);
            }
        }

        Ok(Self {
            root: assets_root,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
        })
    }
}

struct EmailAttachmentWorld {
    ctx: Arc<EmailAttachmentContext>,
    main_id: FileId,
    main_source: Source,
}

impl EmailAttachmentWorld {
    fn new(ctx: Arc<EmailAttachmentContext>, source_text: &str) -> AppResult<Self> {
        let main_id = RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("entry.typ").map_err(|e| AppError::Internal(e.into()))?,
        )
        .intern();
        let main_source = Source::new(main_id, source_text.to_string());
        Ok(Self {
            ctx,
            main_id,
            main_source,
        })
    }

    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        Ok(id
            .vpath()
            .realize(&self.ctx.root)
            .map_err(|e| FileError::Other(Some(format!("Cannot realize path: {:?}", e).into())))?
            .to_path_buf())
    }
}

impl World for EmailAttachmentWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.ctx.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.ctx.book
    }
    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_id {
            return Ok(self.main_source.clone());
        }
        let path = self.resolve(id)?;
        let text = std::fs::read_to_string(&path).map_err(|_| FileError::NotFound(path))?;
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve(id)?;
        let data = std::fs::read(&path).map_err(|_| FileError::NotFound(path))?;
        Ok(Bytes::new(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.ctx.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 8, 23)
    }
}

pub fn compile_typst_to_pdf(source: &str, ctx: &Arc<EmailAttachmentContext>) -> AppResult<Vec<u8>> {
    let world = EmailAttachmentWorld::new(Arc::clone(ctx), source)?;
    let result = typst::compile(&world);

    match result.output {
        Ok(document) => typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Compilation failed: {:#?}", e))),
        Err(diags) => Err(AppError::Internal(anyhow::anyhow!(
            "Compilation failed: {:#?}",
            diags
        ))),
    }
}
