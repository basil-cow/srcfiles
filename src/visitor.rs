use std::path::PathBuf;
use syn::{visit::Visit, ItemMod, LitStr, Macro};

use crate::common::ToTokenString;
use crate::error::Error;
use crate::mod_path::{ModPath, ModSegment, ModStack};
use crate::source_desc::{SourceFileDesc, SourceFileType};

mod cfg_if;

use cfg_if::{CfgExpr, CfgIf};

pub struct SourceFinder {
    pub source_candidates: Vec<SourceFileDesc>,
    pub unresolved_items: Vec<Error>,
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

    pub fn push(&mut self, result: Result<SourceFileDesc, Vec<Error>>) {
        match result {
            Ok(source_file_desc) => self.source_candidates.push(source_file_desc),
            Err(unresolved) => self.unresolved_items.extend(unresolved),
        }
    }

    fn visit_cfg_if(&mut self, node: &CfgIf) {
        self.visit_block(&node.then_branch);

        if let Some((_, cfg_expr_box)) = &node.else_branch {
            match cfg_expr_box.as_ref() {
                CfgExpr::Block(block) => self.visit_block(block),
                CfgExpr::If(cfg_if) => self.visit_cfg_if(cfg_if),
            }
        }
    }

    pub fn process_cfg_if(&mut self, node: &Macro) {
        let cfg_if = node.parse_body::<CfgIf>().unwrap();
        self.visit_cfg_if(&cfg_if);
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

        if macro_ident.as_str() == "cfg_if" {
            self.process_cfg_if(node);
            return;
        }

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
                    .push(Error::UnresolvedIncludeArg(node.to_token_string()));
                return;
            }
        };

        let source_file_desc = SourceFileDesc::new(path, source_type, None);

        if source_file_desc.path.is_file() {
            self.source_candidates.push(source_file_desc);
        } else {
            self.unresolved_items
                .push(Error::MissingFile(source_file_desc));
        }
    }
}
