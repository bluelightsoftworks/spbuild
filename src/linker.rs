use std::path::PathBuf;
use target_lexicon::Triple;
use which::which;

use crate::linker_backends::common::LinkerBackend;
use crate::linker_backends::{gnu_backend, llvmmsvc_backend, llvmelf_backend, msvc_backend, apple_backend};

pub enum LinkerKind {
    // For Linux, BSD & unix-like platforms
    // Typical linker: ld.bfd, Gnu Gold linker, lld, mold...
    // Invoked via g++ or clang++
    Gnu,
    LlvmElf,

    // For Windows platforms
    // Typical linker: MSVC's link.exe, lld-link...
    // Invoked via cl.exe or clang-cl.exe
    Msvc,
    LlvmMsvc,

    // For Apple platforms
    // Typical linker: ld64, ld64.lld...
    // Invoked via clang++
    Apple,
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
                } else if which("lld-link").is_ok() {
                    Some((LinkerKind::LlvmMsvc, "lld-link".into()))
                } else {
                    None
                }
            }
            "macos" | "ios" | "tvos" | "watchos" => {
                if which("clang++").is_ok() {
                    Some((LinkerKind::Apple, "clang++".into()))
                } else if which("ld64").is_ok() {
                    Some((LinkerKind::Apple, "ld64".into()))
                } else {
                    None
                }
            }
            _ => {
                if which("mold").is_ok() {
                    Some((LinkerKind::Gnu, "mold".into()))
                } else if which("ld.lld").is_ok() {
                    Some((LinkerKind::LlvmElf, "ld.lld".into()))
                } else if which("gold").is_ok() {
                    Some((LinkerKind::Gnu, "gold".into()))
                } else if which("ld.bfd").is_ok() {
                    Some((LinkerKind::Gnu, "ld.bfd".into()))
                } else {
                    None
                }
            }
        }
    }

    pub fn link(&self, target: &str, files: &Vec<PathBuf>)
    {
        self.backend.link(&target.to_string(), &files, &vec![]);
    }

    pub fn new(target: &str) -> Self {
        let linker_kind = Linker::detect_linker_kind(target).unwrap().0;

        let backend: Box<dyn LinkerBackend> = match linker_kind {
            LinkerKind::Gnu => Box::new(gnu_backend::GnuLinkerBackend::new()),
            LinkerKind::LlvmElf => Box::new(llvmelf_backend::LLVMElfLinkerBackend::new()),
            LinkerKind::Msvc => Box::new(msvc_backend::MSVCLinkerBackend::new()),
            LinkerKind::LlvmMsvc => todo!(),
            LinkerKind::Apple => todo!(),
        };

        Linker {
            kind: linker_kind,
            backend,
        }
    }
}