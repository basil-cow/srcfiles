use std::path::PathBuf;
use syn::{visit::Visit, ItemMod, LitStr, Macro};

use crate::common::ToTokenString;
use crate::error::Unresolved;
use crate::mod_path::{ModPath, ModSegment, ModStack};
use crate::source_desc::{SourceFileDesc, SourceFileType};

pub struct SourceFinder {
    pub source_candidates: Vec<SourceFileDesc>,
    pub unresolved_items: Vec<Unresolved>,
    pub mod_stack: ModStack,
}

impl SourceFinder {
    pub fn from_mod_path(mod_path: ModPath) -> Self {
        Self::new(vec![ModSegment::ModPath(mod_path)].into())
    }

    pub fn new(mod_stack: ModStack) -> Self {
        SourceFinder {
            source_candidates: vec![],
            unresolved_items: vec![],
            mod_stack,
        }
    }

    pub fn push(&mut self, result: Result<SourceFileDesc, Vec<Unresolved>>) {
        match result {
            Ok(source_file_desc) => self.source_candidates.push(source_file_desc),
            Err(unresolved) => self.unresolved_items.extend(unresolved),
        }
    }
}

impl<'ast> Visit<'ast> for SourceFinder {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        let (possible_segments, unresolved_segments) = crate::mod_path::get_possible_segments(node);
        self.unresolved_items.extend(unresolved_segments);

        for segment in possible_segments {
            self.mod_stack.push(segment);

            match &node.content {
                None => self.push(self.mod_stack.resolve_mod_path().map(Into::into)),
                Some((_, items)) => {
                    for item in items {
                        self.visit_item(item);
                    }
                }
            }

            self.mod_stack.pop();
        }
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        let macro_ident = node.path.segments.last().unwrap().ident.to_string();

        let source_type = match macro_ident.as_str() {
            "include_str" => SourceFileType::String,
            "include_bytes" => SourceFileType::Bytes,
            "include" => SourceFileType::RustSnippet(self.mod_stack.clone()),
            _ => return,
        };

        let path: PathBuf = match node.parse_body::<LitStr>() {
            Ok(path) => self
                .mod_stack
                .parent_file_path()
                .parent()
                .unwrap()
                .join(path.value()),
            Err(_) => {
                self.unresolved_items
                    .push(Unresolved::IncludeArgument(node.to_token_string()));
                return;
            }
        };

        let source_file_desc = SourceFileDesc::new(path, source_type, None);

        if source_file_desc.path.is_file() {
            self.source_candidates.push(source_file_desc);
        } else {
            self.unresolved_items
                .push(Unresolved::MissingFile(source_file_desc));
        }
    }
}
