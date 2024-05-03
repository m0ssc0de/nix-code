# nix-code

`nix-code` is a tool designed to streamline and guarantee the development environment for various projects with the help of Nix and Visual Studio Code. It assists in selecting and searching for Nix files to establish a Nix shell for a VS Code instance.

## Prerequisites

Before using, you need to install Nix and `nix-code` itself.

### Installing Nix

You can install Nix from the official website. Follow the instructions provided there to install it on your system.
- https://nixos.org/download/
- `sh <(curl -L https://nixos.org/nix/install)`

### Installing nix-code

You can install `nix-code` using the `cargo` command-line tool with the following command:

`cargo install --git https://github.com/m0ssc0de/nix-code.git`

Or install nix-code by nix(coming soon)


## Usage

### Opening a Folder with a Specific Tag

To open a folder with a specific tag, use the `ncd` command followed by the path to the folder and the `-t` option with the tag. For instance, to open a folder with the "rust" tag, use the following command:

```shell
ncd ./projects/tryrust -t rust
```

This command will create a universal Nix shell tailored for a Rust development environment.

### Opening a Git Repository

To open a Git repository cloned from GitHub, use the `ncd` command followed by the path to the repository. For example, to open the `kubernetes` project, use the following command:

```shell
git clone --depth=1 https://github.com/kubernetes/kubernetes.git ~/project/kubernetes
ncd ~/project/kubernetes
```

This command will search the [Nix code index](https://github.com/m0ssc0de/nix-code-index) and create a specific Nix shell optimized for the Kubernetes development environment.

### Working with nix file directly

You can also specify a Nix file directly using the `-f` option:

`ncd -f ./shell.nix ./my/rustproject`

Alternatively, you can create a `shell.nix` or `default.nix` file in the project root:

`touch ./my/rustproject/shell.nix && ncd ./my/rustproject`

These methods allow you to create a customized development shell environment.