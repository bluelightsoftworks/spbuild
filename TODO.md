## High priority

- [x] Wire up the build run (main never calls build_solution / starts compilation). (src/main.rs)
- [ ] Fix wrong import/use of Console in dependency_resolver (uses crate::Console instead of crate::helpers::console::Console) — will fail to compile. (src/dependency_manager/dependency_resolver.rs)
- [ ] Make Linker::new handle detect_linker_kind returning None instead of unwrap() to avoid panics and emit a user-friendly error. (src/linker.rs)
- [ ] Fix Builder.set_flag (signature accepts value but implementation discards it) so flags/values are applied to the compiler. (src/builder.rs)
- [ ] Ensure builder output mapping is correct: cc::Build output names/locations vs how output_files_paths are constructed (joining source name to output_path is brittle/wrong). Need to map generated object names to linker inputs. (src/builder.rs)
## Medium priority
- [ ] Config format/README mismatch: README mentions spbuild.json and examples, main expects spbuild.yaml — decide one format (or support both) and update docs and code. (README.md, src/main.rs)
- [ ] Pass CLI options (verbose, platform, arch) through to dependency resolution and builder so verbose logs and target-specific behavior actually occur. (src/main.rs, dependency resolver + builder)
- [ ] Integrate dependency scanning (.d files or compiler -M options) / header dependency tracking so headers trigger recompiles (code finds headers but it's not wired into build/incremental logic). (src/dependency_manager, src/builder.rs)
- [ ] Make a centralized way to abort builds with consistent exit codes and log messages. (multiple files: main.rs, builder.rs, dependency_resolver.rs)
- [ ] Replace panics/unwraps with recoverable errors and consistent exit codes/log messages across the codebase (multiple files: main.rs, builder.rs, dependency_resolver.rs).
## Lower priority / release housekeeping
- [ ] Bump package version from "0.4.0-dev" to "0.4.0" (Cargo.toml) when releasing.
- [ ] Make README’s v0.4.0 checklist reflect current state (it says “Switch to cc crate” — ensure all cc-related plumbing is complete).
- [ ] Run and fix unit tests (version_tools and target tests exist); add CI if desired.