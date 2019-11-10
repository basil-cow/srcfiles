mod common;
pub mod error;
mod mod_path;
mod source_desc;
mod visitor;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use syn::visit::Visit;

pub use error::{Error, SourcesAndErrors};
pub use mod_path::ModPath;
pub use source_desc::{ModType, SourceFileDesc, SourceFileType};
use visitor::SourceFinder;

fn propagate_parent_file(path: &Path, source_descs_slice: &mut [SourceFileDesc]) {
    for source_desc in source_descs_slice {
        source_desc.parent_file = Some(path.to_owned());
    }
}

fn visit_source(
    path: &Path,
    mut source_finder: SourceFinder,
) -> Result<(Vec<SourceFileDesc>, Vec<Error>), Error> {
    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;

    source_finder.visit_file(&ast);

    propagate_parent_file(path, &mut source_finder.source_candidates);

    Ok((
        source_finder.source_candidates,
        source_finder.unresolved_items,
    ))
}

pub fn process_source(source: &SourceFileDesc) -> Result<(Vec<SourceFileDesc>, Vec<Error>), Error> {
    let source_finder = match &source.file_type {
        SourceFileType::Bytes | SourceFileType::String => return Ok((vec![], vec![])),
        SourceFileType::RustSnippet(mod_stack) => SourceFinder::new(mod_stack.clone()),
        SourceFileType::RustSource(mod_type) => {
            SourceFinder::from_mod_path(ModPath::new(source.path.clone(), *mod_type))
        }
    };

    Ok(visit_source(&source.path, source_finder)?)
}

pub fn crate_srcfiles(path: PathBuf) -> Result<Vec<SourceFileDesc>, SourcesAndErrors> {
    mod_srcfiles(ModPath::new(path, ModType::ModRs))
}

pub fn mod_srcfiles(mod_path: ModPath) -> Result<Vec<SourceFileDesc>, SourcesAndErrors> {
    let mut source_queue = Vec::with_capacity(100);
    let mut result = SourcesAndErrors::new(vec![]);

    source_queue.push(SourceFileDesc::new(
        mod_path.path,
        SourceFileType::RustSource(mod_path.mod_type),
        None,
    ));

    while let Some(source) = source_queue.pop() {
        match process_source(&source) {
            Ok((sources, src_errors)) => {
                source_queue.extend(sources);
                result.sources.push((source, src_errors));
            }
            Err(error) => result.sources.push((source, vec![error])),
        }
    }

    if result.sources.iter().all(|x| x.1.is_empty()) {
        Ok(result.into_sources())
    } else {
        Err(result)
    }
}
