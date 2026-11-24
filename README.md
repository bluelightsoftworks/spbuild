![Logo](assets/logo.png)

# Single Player Build System (sp)

> The c/c++ build system for singleplayer devs

## About

The goal of this project is to help little teams and solo devs configuring a dev environment easily. To run it, you only
need to use:

```bash
sp <command>
```

Here are some available options:

Here are the currently available commands:

- `build`: Builds the current directory if it is an spbuild project
- `init`: Initializes the current directory as an spbuild project
- `version`: Show version information and exit
  You can specify other parameters and arguments. Learn about them using --help

## Compilation requirements

### Linux

The default compiler for Linux is GCC so you will need to install cross-compilers if you want to compile for other
platforms.

If you also want to target different architectures, you will need to install the appropriate cross-compilers.

- For ARM targets, install `aarch64-linux-gnu-gcc` (on Arch, `sudo yay -S aarch64-linux-gnu-gcc`)
- For 32-bit targets, install `gcc-multilib` and `gcc-libc` (on Arch, `sudo pacman -S lib32-gcc-libc lib32-glibc`)
- For RISC-V targets, install `riscv64-elf-gcc` (on Arch, `sudo yay -S riscv64-elf-gcc`)

### Other systems

While multiplatform compilation is planned in the future it is not implemented yet. You can only compile from Linux
systems as of right now

## Naming

- Project : A single target for the compiler (executable, library, etc.)
    - Project's output directory : The directory where the compiled files of a single project are stored
    - Project source : The directory where the project's source code is located usually right next to spbuild.json
- Solution : A collection of projects (like Visual Studio solutions)
    - Solution root : The directory where spbuild.json is located
- Dependency : A project that another project depends on to compile
    - Local dependency : A dependency that is part of the same solution
    - External dependency : A dependency that is not part of the same solution (can be from the package manager)
- Additional `x`: Anything that is not part of the project dependencies but is still needed to compile the project
- (ex: additional include directories, additional static libraries, etc.)

### Project configuration file options

- `name` : Name of the project. Can be any string
- `version`: Version of the project. Can be any string
- `project_type`: Type of the project. Can be one of the following:
    - `Executable`: A standalone application
    - `StaticLib`: A static library that can be linked to other projects
    - `DynamicLib`: A dynamic library (like DLLs on Windows)
- `target_archs`: List of target architectures. Can be any of the following:
    - `x86_64`: 64-bit architecture
    - `x86`: 32-bit architecture
    - `aarch64`: ARM 64-bit architecture
    - `arm`: ARM 32-bit architecture
    - `riscv64`: RISC-V 64-bit architecture
- `target_platforms`: List of target platforms. Can be any of the following:
    - `win`: Microsoft Windows
    - `linux`: Linux-based operating systems
    - `macos-25.2`: Apple's macOS with Kernel version 25.2 (Needs additional configuration with osxcross)
- `path`: The path to the project folder (relative to the solution root)
- `dependencies`: List of other projects that this project depends on (by name). If a dependency is not found in
  the solution, spbuild will look for it in the package manager (not implemented yet)
    - Each dependency is an object with the following properties:
        - `name`: Name of the dependency project
        - `version`: Version of the dependency project
- `additional_includes`: List of additional include directories (relative to the project path) that are NOT in any local
  dependency
- `additional_static_libs`: List of additional static library files (relative to the project path) that are NOT in any
  local dependency
  and that are needed to link the project (ex: .a files on Linux, .lib files on Windows)

## Scripts

### cpdll.py

**Usage**:
Automatically detects and copies required DLLs from your mingw bin folder

```shell
python <path_to_cpdll.py> <path_to_exe>
```

*Note: automatically called during \*nix → Windows compilation*

## TODO list

- [x] Compile a basic solution
    - [x] Compile with GCC
        - [x] Single project solution
        - [x] Link
        - [x] Multi project solution
        - [x] Link
        - [x] Cross compile (Linux → Windows)
- [ ] Incremental build support
- [ ] Dependency and package manager (definitely)

## Road to 1.0

- 0.2: Simple GCC. Set the groundwork
    - 0.2.1: Fix strange documentation, add a bit more error handling
- 0.3: Cross compilation support, target architectures, target platforms
    - 0.3.1: Bugfixes
    - 0.3.2: Better dependencies (.dll, .so, .dylib, .lib handling) ← Latest
- 0.4: CLI Rework (`sp build`, `sp init`...) and external dependencies
- 0.5: Package manager
- 0.6: Incremental and parallel build support
- 0.7: Testing, bug fixing...
- 1.0: Final polish and Release!

> Note: while Windows and macOS support were planned, due to personal reasons, I am not able to implement them right now

## Legal notices

This project uses the following third-party libraries:

- [clap](https://github.com/clap-rs/clap)
- [serde](https://github.com/serde-rs/serde)
- [colored](https://github.com/marwes/colored)
- [strum](https://github.com/Peternator7/strum)
- [pkg-config](https://github.com/rust-lang/pkg-config-rs)