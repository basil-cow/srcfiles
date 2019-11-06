use srcfiles;

use srcfiles::{error::Error, SourceFileDesc, SrcError, Unresolved};
use std::path::PathBuf;

fn assert_has_source(srcfiles: &[SourceFileDesc], path: &str) {
    assert!(
        srcfiles
            .iter()
            .any(|desc| desc.path.canonicalize().unwrap()
                == PathBuf::from(path).canonicalize().unwrap()),
        format!("No source with path {}", path)
    );
}

fn assert_missing_files(srcfiles: &[SrcError], path: &str) {
    assert!(
        srcfiles
            .iter()
            .map(|src_err| &src_err.error)
            .flat_map(
                |err| if let Error::Unresolved(Unresolved::MissingFile(desc)) = err {
                    Some(desc)
                } else {
                    None
                }
            )
            .any(|desc| desc.path == PathBuf::from(path)),
        format!("No missing file with path {}", path)
    );
}

#[test]
fn simple_test() {
    let (srcfiles, errors) =
        srcfiles::crate_srcfiles(PathBuf::from("test_projects/simple/src/main.rs"));

    assert_eq!(srcfiles.len(), 7);
    assert_has_source(&srcfiles, "test_projects/simple/src/main.rs");
    assert_has_source(&srcfiles, "test_projects/simple/src/a.rs");
    assert_has_source(&srcfiles, "test_projects/simple/src/a/c.rs");
    assert_has_source(&srcfiles, "test_projects/simple/src/a/d/mod.rs");
    assert_has_source(&srcfiles, "test_projects/simple/src/b/mod.rs");
    assert_has_source(&srcfiles, "test_projects/simple/src/b/f/mod.rs");
    assert_has_source(&srcfiles, "test_projects/simple/src/b/g.rs");

    assert_eq!(errors.len(), 4);
    assert_missing_files(&errors, "test_projects/simple/src/c.rs");
    assert_missing_files(&errors, "test_projects/simple/src/c/mod.rs");
    assert_missing_files(&errors, "test_projects/simple/src/a/c/d.rs");
    assert_missing_files(&errors, "test_projects/simple/src/a/c/d/mod.rs");
}

#[test]
fn path_attr_test() {
    let (srcfiles, errors) =
        srcfiles::crate_srcfiles(PathBuf::from("test_projects/paths/src/main.rs"));

    assert_eq!(srcfiles.len(), 7);
    assert_has_source(&srcfiles, "test_projects/paths/src/main.rs");
    assert_has_source(&srcfiles, "test_projects/paths/src/a.rs");
    assert_has_source(&srcfiles, "test_projects/paths/src/b.rs");
    assert_has_source(&srcfiles, "test_projects/paths/src/d.rs");
    assert_has_source(&srcfiles, "test_projects/paths/src/c/mod.rs");
    assert_has_source(&srcfiles, "test_projects/paths/g/mod.rs");
    assert_has_source(&srcfiles, "test_projects/paths/g/actual_mod.rs");
    assert_eq!(errors.len(), 3);
    assert_missing_files(&errors, "test_projects/paths/src/../src/b/c.rs");
    assert_missing_files(&errors, "test_projects/paths/src/../src/b/c/mod.rs");
    assert_missing_files(&errors, "test_projects/paths/src/../g/../src/f.rs");
}

#[test]
fn inline_mods_test() {
    let (srcfiles, errors) =
        srcfiles::crate_srcfiles(PathBuf::from("test_projects/inline/src/lib.rs"));
    assert_eq!(srcfiles.len(), 5);
    assert_has_source(&srcfiles, "test_projects/inline/src/lib.rs");
    assert_has_source(&srcfiles, "test_projects/inline/g/mod.rs");
    assert_has_source(&srcfiles, "test_projects/inline/g/h.rs");
    assert_has_source(&srcfiles, "test_projects/inline/src/a/c/d/mod.rs");
    assert_has_source(&srcfiles, "test_projects/inline/src/a/c/e/e/e.rs");
    assert_eq!(errors.len(), 0);
}

#[test]
fn include_test() {
    let (srcfiles, errors) =
        srcfiles::crate_srcfiles(PathBuf::from("test_projects/inline/src/lib.rs"));
    assert_eq!(srcfiles.len(), 5);
    assert_has_source(&srcfiles, "test_projects/inline/src/lib.rs");
    assert_has_source(&srcfiles, "test_projects/inline/g/mod.rs");
    assert_has_source(&srcfiles, "test_projects/inline/g/h.rs");
    assert_has_source(&srcfiles, "test_projects/inline/src/a/c/d/mod.rs");
    assert_has_source(&srcfiles, "test_projects/inline/src/a/c/e/e/e.rs");
    assert_eq!(errors.len(), 0);
}
