# MADX-LS

An LSP implementation for the [MADX](http://mad.web.cern.ch/mad/) scripting language.

## Features

- [x] Semantic highlighting (in progress, most commands are done)
- [x] Hover 
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
  git clone ...
  ```
- Use it with your code editor

## Code Editor Specific Usage

### nvim

Add the following to your ocnfiguration (todo: simplify this):

```lua
function StartMadx()
    vim.lsp.start({
        name = "madx",
        cmd = {"/media/awegsche/HDD1/rust/madxls/target/release/madxls"},
    })
end

vim.api.nvim_create_autocmd({"BufEnter", "BufWinEnter"}, {
    pattern = {"*.madx"},
    callback = StartMadx,
})

```

### vscode

Wait for a madxls plugin
