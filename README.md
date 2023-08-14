# MADX-LS

An LSP implementation for the [MADX](http://mad.web.cern.ch/mad/) scripting language.

## Features

- [x] Semantic highlighting (in progress, most commands are done)
- [ ] Hover 
    - [x] defined macros
    - [ ] built-in commands
    - [ ] variables in scope
- [ ] Errors
    - [ ] syntax errors
    - [x] command usage
- [ ] Hints
- [ ] Jump to definition

## Usage

- Install the [rust toolchain ](https://www.rust-lang.org/learn/get-started)
- Install with cargo
  ``` sh
  cargo install --git https://github.com/awegsche/madxls.git
  ```
- Maybe you then have to add the cargo bin dir to PATH (todo: some hints on how to do that).
- Use it with your code editor. If your code editor supports the LSP, this is just a matter of
adding a corresponding entry in your configuration file. Some examples are listed below:

## Code Editor Specific Usage

### neovim

Add the following to your configuration (todo: simplify this):

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

### vscode

Wait for a madxls plugin

### emacs

cf. emacs LSP configuration
