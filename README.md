# MADX-LS 
[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=awegsche1_madxls&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=awegsche1_madxls)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=awegsche1_madxls&metric=coverage)](https://sonarcloud.io/summary/new_code?id=awegsche1_madxls)

A comprehensive language toolchain for the [MADX](http://mad.web.cern.ch/mad/) scripting language, providing both IDE integration and static analysis capabilities.

## Overview

MADX-LS is a Rust-based toolkit designed to enhance the development experience for MADX (Methodical Accelerator Design) scripts. The project consists of two powerful binaries that serve different aspects of the development workflow:

### 🚀 **Scanner** - Static Analysis & Code Quality
A command-line tool that performs comprehensive analysis of MADX code, delivering:
- **Syntax Error Detection**: Identify and report syntax issues in your MADX scripts
- **Code Quality Analysis**: Detect bad practices and potential improvements
- **Code Metrics**: Generate detailed metrics about your codebase
- **SonarQube Integration**: Seamlessly integrates with SonarQube servers for continuous code quality monitoring

### 🔧 **Server** - Language Server Protocol (LSP)
An LSP implementation that brings intelligent code assistance to your favorite IDE. Currently under heavy refactoring to improve stability and feature completeness.

> **⚠️ Note**: The LSP server is currently undergoing significant refactoring and may not be fully functional. We recommend using the scanner for immediate code analysis needs.

## Supported Editors

The LSP server is designed to work with any editor that supports the Language Server Protocol, including:

- **Visual Studio Code** - Full IDE experience with syntax highlighting, error detection, and IntelliSense
- **Vim/Neovim** - Lightweight integration for terminal-based development
- **Emacs** - Rich editing experience with LSP support
- **And many more** - Any editor with LSP client support

## Installation

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/learn/get-started) (latest stable version)

### Install from Source

- Scanner
```bash
cargo install --git https://github.com/awegsche/madxls.git --bin madx_scanner
```
- LSP
```bash
cargo install --git https://github.com/awegsche/madxls.git --bin madxls
```

> **Tip**: After installation, ensure the Cargo binary directory is in your PATH. You can find it with `cargo --list` and add it to your shell profile.

## Usage

### Scanner (Recommended)
The scanner is the most stable and feature-complete component:

```bash
# Analyze a single file
madx_scanner --input-file path/to/your/script.madx

# Analyze a single file, print highlights and code metrics
madx_scanner --input-file path/to/your/script.madx --highlight --metrics
```

### LSP Server (Under Development)
Once the refactoring is complete, the LSP server will provide real-time assistance in your editor.

#### Neovim Configuration
```lua
function StartMadx()
    vim.lsp.start({
        name = "madx",
        cmd = {"madxls"},
    })
end

vim.api.nvim_create_autocmd({"BufEnter", "BufWinEnter"}, {
    pattern = {"*.madx"},
    callback = StartMadx,
})
```

#### VS Code
A dedicated VS Code extension is planned for seamless integration.

#### Emacs
Configure using your preferred LSP client (e.g., `lsp-mode` or `eglot`).

## SonarQube Integration

The scanner integrates via a dedicated [SonarQube plugin](https://github.com/awegsche/sonar-madx) that enables continuous code quality monitoring:

- **On-Premise SonarQube**: Full support for self-hosted SonarQube instances
- **Cloud Support**: Currently limited due to plugin deployment constraints (coming soon)

## Contributing

We welcome contributions! The project is actively developed, and we're particularly interested in:
- LSP server stability improvements
- Additional code quality rules
- Enhanced editor integrations

## Roadmap

- [ ] Complete LSP server refactoring
- [ ] Enhanced semantic highlighting
    - [x] Sonarqube
    - [ ] LSP
- [ ] Hover documentation for built-in commands
- [ ] Jump-to-definition functionality
- [ ] SonarQube cloud plugin deployment
- [ ] VS Code extension development
