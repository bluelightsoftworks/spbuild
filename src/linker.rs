use std::path::PathBuf;
use target_lexicon::Triple;
use which::which;

use crate::linker_backends::common::LinkerBackend;
use crate::linker_backends::{unix_backend, msvc_backend};

pub enum LinkerKind {
    // For Linux, BSD & unix-like platforms
    // Typical linker: ld.bfd, Gnu Gold linker, lld, mold...
    // Invoked via g++ or clang++
    Unix,

    // For Windows platforms
    // Typical linker: MSVC's link.exe, lld-link...
    // Invoked via cl.exe or clang-cl.exe
    Msvc,
}

pub struct Linker {
    kind: LinkerKind,
    backend: Box<dyn LinkerBackend>,
}

impl Linker {
    /// Parses the right linker for the given target triple.
    /// Returns the linker kind and the executable name to invoke it.
    pub fn detect_linker_kind(target: &str) -> Option<(LinkerKind, String)> {
        let triple = target.parse::<Triple>().ok()?;

        match triple.operating_system.to_string().as_str() {
            "windows" => {
                if which("link.exe").is_ok() {
                    Some((LinkerKind::Msvc, "link.exe".into()))
                } else {
                    None
                }
            }
            "macos" | "ios" | "tvos" | "watchos" => {
                if which("clang++").is_ok() {
                    Some((LinkerKind::Unix, "clang++".into()))
                } else if which("ld64").is_ok() {
                    Some((LinkerKind::Unix, "ld64".into()))
                } else {
                    None
                }
            }
            _ => {
                if which("mold").is_ok() {
                    Some((LinkerKind::Unix, "mold".into()))
                } else if which("gold").is_ok() {
                    Some((LinkerKind::Unix, "gold".into()))
                } else if which("ld.bfd").is_ok() {
                    Some((LinkerKind::Unix, "ld.bfd".into()))
                } else if which("ld").is_ok() {
                    Some((LinkerKind::Unix, "ld".into()))
                } else {
                    None
                }
            }
        }
    }

    pub fn link(&self, target: &str, files: Vec<PathBuf>)
    {
        self.backend.link(target, files, vec![]);
    }

    pub fn new(target: &str) -> Result<Self, String> {
        let (linker_kind, _) = Linker::detect_linker_kind(target)
            .ok_or_else(|| format!("No supported linker found for target: {}", target))?;

        let backend: Box<dyn LinkerBackend> = match linker_kind {
            LinkerKind::Unix => Box::new(unix_backend::GnuLinkerBackend::new()),
            LinkerKind::Msvc => Box::new(msvc_backend::MSVCLinkerBackend::new()),
        };

        Ok(Linker { kind: linker_kind, backend })
    }
}