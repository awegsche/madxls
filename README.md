# MADX-LS

An LSP implementation for the [MADX](http://mad.web.cern.ch/mad/) scripting language.

## Features

- [x] Semantic highlighting (in progress, most commands are done)
- [ ] Hover 
    - [x] defined macros
    - [ ] built-in commands
    - [ ] variables in scope
- [ ] errors
    - [ ] syntax errors
    - [x] command usage
- [ ] hints
- [ ] jump to definition

## Usage

- Install the [rust toolchain ](https://www.rust-lang.org/learn/get-started)
- Checkout this repository
  ``` sh
  git clone git@github.com:awegsche/madxls.git
  ```
- Install with cargo
  ``` sh
  cd madxls
  cargo install
  ```
- Add cargo bin dir to PATH (todo: some hints on how to do that)
- Use it with your code editor, see below.

## Code Editor Specific Usage

### neovim

Add the following to your configuration (todo: simplify this):

```lua
function StartMadx()
    vim.lsp.start({
        name = "madx",
        cmd = {"madx"},
    })
end

vim.api.nvim_create_autocmd({"BufEnter", "BufWinEnter"}, {
    pattern = {"*.madx"},
    callback = StartMadx,
})

```

### vscode

Wait for a madxls plugin

### emacs

cf. emacs LSP configuration
