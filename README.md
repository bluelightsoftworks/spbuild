# Single Player Build System (spbuild)
> The build system for singleplayer devs

**WARNING: PROJECT IS NOT READY FOR USE YET. Please note that some of these options don't work yet**

## About
The goal of this project is to help little teams and solo devs configuring a multiplatform dev environment in a [WORA](https://en.wikipedia.org/wiki/Write_once,_run_anywhere) fashion.
To use, you only need to run
```bash
spbuild build [OPTIONS]
```
Here are some available options:
- `-s`, `--solution-path {path}`: Path to the project config file (If folder passed, defaults to spbuild.json)
- `-p`, `--platform {win|linux|macos-25.2}`: Target platform to build for. Will fail if the compiler or the solution doesn't support the specified platform
- `-a`, `--arch {x86|x64|arm32|arm64|riscv64}`: Target architecture to build for. Will fail if the compiler or the solution doesn't support the specified architecture
- `-v`, `--verbose`: Enable verbose output

## Compilation requirements
### Linux
The default compiler for linux is GCC so you will need to install cross-compilers if you want to compile for other platforms.
- For Windows targets, install `mingw-w64` and peldd (on Arch, `sudo yay -S mingw-w64 peldd`) 
- For MacOS targets, cross compilation requires a bit more work and `osxcross`. Follow the instructions on their [GitHub page](https://github.com/tpoechtrager/osxcross) for gcc
  Please remember that MacOS compilation isn't planned to be supported in the near future.

If you also want to target different architectures, you will need to install the appropriate cross-compilers.
- For ARM targets, install `aarch64-linux-gnu-gcc` (on Arch, `sudo yay -S aarch64-linux-gnu-gcc`)
- For 32-bit targets, install `gcc-multilib` and `gcc-libc` (on Arch, `sudo pacman -S lib32-gcc-libc lib32-glibc`)
- For RISC-V targets, install `riscv64-elf-gcc` (on Arch, `sudo yay -S riscv64-elf-gcc`)

## Naming
- Project : A single target for the compiler (executable, library, etc.)
  - Project's output directory : The directory where the compiled files of a single project are stored
  - Project source : The directory where the project's source code is located.. usually right next to spbuild.json
- Solution : A collection of projects (like Visual Studio solutions)
  - Solution root : The directory where spbuild.json is located
- Dependency : A project that another project depends on to compile
  - Local dependency : A dependency that is part of the same solution
  - External dependency : A dependency that is not part of the same solution (can be from the package manager)

### Project configuration file options
- `name` : Name of the project. Can be any string
- `version`: Version of the project. Can be any string
- `project_type`: Type of the project. Can be one of the following:
  - `Executable`: A standalone application
  - `StaticLib`: A static library that can be linked to other projects
  - `DynamicLib`: A dynamic library (like DLLs on Windows)
- `target_archs`: List of target architectures. Can be any of the following:
  - `X64`: 64-bit architecture
  - `x86`: 32-bit architecture
  - `ARM64`: ARM 64-bit architecture
  - `ARM`: ARM 32-bit architecture
- `target_platforms`: List of target platforms. Can be any of the following:
  - `windows`: Microsoft Windows
  - `linux`: Linux-based operating systems
  - `macos-25.2`: Apple's MacOS with Kernel version 25.2 (Needs additional configuration with osxcross)
- `path`: The path to the project folder (relative to the solution root)
- `dependencies`: List of other projects that this project depends on (by name). If a dependency is not found in
    the solution, spbuild will look for it in the package manager (not implemented yet)
  - Each dependency is an object with the following properties:
    - `name`: Name of the dependency project
    - `version`: Version of the dependency project
    - `optional`: If true, the build will continue even if the dependency is not found
- `additional_includes`: List of additional include directories (relative to the project path) that are NOT in any local dependency

## Scripts
### cpdll.py
**Usage**: 
Automatically detects and copies required DLLs from your mingw bin folder
```shell
python <path_to_cpdll.py> <path_to_exe>
```
*Note: automatically called during \*nix -> Windows compilation* 

## TODO list
- [ ] Compile a basic solution
  - [ ] Compile with MSVC
    - [ ] Single project solution
    - [ ] Multi project solution
    - [ ] Link
    - [ ] Cross compile (Windows -> Linux)
  - [x] Compile with GCC
    - [x] Single project solution
    - [x] Link
    - [x] Multi project solution
    - [x] Link
    - [x] Cross compile (Linux -> Windows)
- [ ] Incremental build support
- [ ] Dependency and package manager (definitely)

## Road to 1.0
- 0.2: Simple GCC.. Set the groundwork 
  - 0.2.1: Fix strange documentation, add a bit more error handling <- Latest
- 0.3: Cross compilation support, target architectures, target platforms 
  - 0.3.1: Bugfixes <- Dev branch
  - 0.3.2: Better dependencies (.dll, .so, .dylib, .lib handling)
- 0.4: More compiler support (Clang, MSVC) <- First Prerelease
- 0.5: Incremental build support
- 0.6: Cleanup, refactor, documentation
- 0.7: Package manager and external dependencies
- 0.8: Testing, bug fixing...
- 0.9: Final Polish, prepare for release

- 1.0: Release!


