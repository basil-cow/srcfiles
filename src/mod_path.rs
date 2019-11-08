use std::path::{Path, PathBuf};
use syn::{Attribute, Ident, ItemMod, Lit, Meta};

use crate::error::Error;
use crate::source_desc::{ModType, SourceFileDesc, SourceFileType};

#[derive(Debug, Clone)]
pub struct ModPath {
    pub path: PathBuf,
    pub mod_type: ModType,
}

impl ModPath {
    pub fn new(path: PathBuf, mod_type: ModType) -> Self {
        ModPath { path, mod_type }
    }
}

impl Into<SourceFileDesc> for ModPath {
    fn into(self) -> SourceFileDesc {
        SourceFileDesc::new(self.path, SourceFileType::RustSource(self.mod_type), None)
    }
}

// Mod stack segment, representing one mod statement or top-level mod (file itself)
#[derive(Debug, Clone)]
pub enum ModSegment {
    InlinePath(PathBuf),
    Ident(Ident),
    ModPath(ModPath),
}

#[derive(Debug, Clone)]
pub struct ModStack(Vec<ModSegment>);

impl ModStack {
    pub fn push(&mut self, mod_segment: ModSegment) {
        self.0.push(mod_segment);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn parent_file_path(&self) -> &Path {
        match self.0.first().expect("Using empty modstack") {
            ModSegment::ModPath(ModPath { path, .. }) => &path,
            _ => panic!("Invalid ModStack structure"),
        }
    }

    fn get_mod_path_candidates(&self) -> Vec<ModPath> {
        if self.0.is_empty() {
            return vec![];
        }

        let mut path = PathBuf::new();

        for i in 0..self.0.len() - 1 {
            match &self.0[i] {
                ModSegment::Ident(ident) => path.push(ident.to_string()),
                ModSegment::InlinePath(inline_path) => path.push(inline_path),
                ModSegment::ModPath(mod_path) => {
                    path.push(mod_path.path.parent().unwrap());
                    if let (ModSegment::Ident(_), ModType::Adjacent) =
                        (&self.0[i + 1], mod_path.mod_type)
                    {
                        path.push(mod_path.path.file_stem().unwrap());
                    }
                }
            }
        }

        match self.0.last().unwrap() {
            ModSegment::ModPath(mod_path) => {
                path.push(&mod_path.path);
                vec![ModPath::new(path, mod_path.mod_type)]
            }
            ModSegment::Ident(ident) => {
                let ident = ident.to_string();

                let mut adjacent_candidate = path.join(&ident);
                adjacent_candidate.set_extension("rs");
                let adjacent_candidate = ModPath::new(adjacent_candidate, ModType::Adjacent);
                let mut mod_rs_candidate = path.join(&ident);
                mod_rs_candidate.push("mod.rs");
                let mod_rs_candidate = ModPath::new(mod_rs_candidate, ModType::ModRs);
                vec![adjacent_candidate, mod_rs_candidate]
            }
            _ => unreachable!(),
        }
    }

    pub fn resolve_mod_path(&self) -> Result<ModPath, Vec<Error>> {
        let candidates = self.get_mod_path_candidates();

        for i in &candidates {
            if i.path.is_file() {
                return Ok(i.clone());
            }
        }

        Err(candidates
            .into_iter()
            .map(|path| path.into())
            .map(Error::MissingFile)
            .collect())
    }
}

impl From<Vec<ModSegment>> for ModStack {
    fn from(mod_stack: Vec<ModSegment>) -> Self {
        Self(mod_stack)
    }
}

fn parse_possible_path(attr: &Attribute) -> Option<Result<PathBuf, Error>> {
    if let Ok(Meta::NameValue(name_value)) = attr.parse_meta() {
        let attr_path = &name_value.path.segments;
        if attr_path.len() == 1 && attr_path.first().unwrap().ident == "path" {
            if let Lit::Str(path_value) = name_value.lit {
                return Some(Ok(path_value.value().into()));
            }
        }
    }

    None
}

pub fn get_possible_segments(item_mod: &ItemMod) -> (Vec<ModSegment>, Vec<Error>) {
    let possible_path_attrs: Vec<_> = item_mod
        .attrs
        .iter()
        .flat_map(parse_possible_path)
        .collect();

    if possible_path_attrs.is_empty() {
        (vec![ModSegment::Ident(item_mod.ident.clone())], vec![])
    } else {
        let mut segments = Vec::new();
        let mut unresolved = Vec::new();

        for path_attr in possible_path_attrs {
            match path_attr {
                Ok(path) => {
                    if item_mod.content.is_some() {
                        segments.push(ModSegment::InlinePath(path))
                    } else {
                        segments.push(ModSegment::ModPath(ModPath::new(path, ModType::ModRs)))
                    }
                }
                Err(unresolved_item) => unresolved.push(unresolved_item),
            }
        }

        (segments, unresolved)
    }
}
