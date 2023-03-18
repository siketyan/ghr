# 🚀 ghr
[![crates.io](https://img.shields.io/crates/v/ghr.svg)](https://crates.io/crates/ghr)
[![Rust](https://github.com/siketyan/ghr/actions/workflows/rust.yml/badge.svg)](https://github.com/siketyan/ghr/actions/workflows/rust.yml)

Yet another repository management with auto-attaching profiles.

## 🔥 Motivation
[ghq](https://github.com/x-motemen/ghq) is the most famous solution to resolve stress of our repository management currently.
However, I wanted to customise the git configuration by some rules, such as using my company email in their repositories.

To achieve that, ghq was not enough for me.
So I have rewritten them as simple, in Rust, the robust and modern language today.

## 📦 Installation
### Using Homebrew (easy)
```shell
brew install siketyan/tap/ghr
```

To upgrade:

```shell
brew upgrade siketyan/tap/ghr
```

### Using cargo (classic)
If you have not installed Rust environment, follow the instruction of [rustup](https://rustup.rs/).

```shell
cargo install ghr
```

For upgrading, we recommend to use [cargo-update](https://github.com/nabijaczleweli/cargo-update).

```shell
cargo install-update ghr
```

### 🔧 Installing the shell extension
To extend ghr features to maximum, it is recommended to install the shell extension.
Add the line below to your shell configuration script to enable it.

#### Bash
```shell
ghr shell bash | source
```

To enable completions, add the line into `~/.bash_completion`.

```shell
ghr shell bash --completion | source
```

#### Fish
```shell
ghr shell fish | source
```

To enable completions, add the line into `~/.config/fish/completions/ghr.fish`.

```shell
ghr shell fish --completion | source
```

## 💚 Usages
```
Usage: ghr <COMMAND>

Commands:
  cd       Changes directory into a repository (Shell extension required)
  clone    Clones a Git repository to local
  delete   Deletes a repository from local
  init     Initialises a Git repository in local
  list     Lists all managed repositories
  open     Opens a repository in an application
  path     Prints the path to root, owner, or a repository
  profile  Manages profiles to use in repositories
  shell    Writes a shell script to extend ghr features
  version  Prints the version of this application
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

### Cloning a repository
ghr supports many patterns or URLs of the repository to clone:

```shell
ghr clone <owner>/<repo>
ghr clone github.com:<owner>/<repo>
ghr clone https://github.com/<owner>/<repo>.git
ghr clone ssh://git@github.com/<owner>/<repo>.git
ghr clone git@github.com:<owner>/<repo>.git
```

If you have installed the shell extension, you can change directory into the cloned repository:

```shell
ghr clone <url_or_pattern> --cd
```

If you often use repositories of a specific owner, you can set the default owner to be resolved.

```toml
[defaults]
owner = "siketyan"
```

```shell
ghr clone <repo>
```

### Changing directory
You can change directory into a repository on the shell.
It requires installing the shell extension.

```shell
ghr cd <url_or_pattern>
```

### Attaching profiles
Create `~/.ghr/ghr.toml` and edit as you like:

```toml
[profiles.default]
user.name = "Your Name"
user.email = "your_name@personal.example.com"

[profiles.company]
user.name = "Your Name (ACME Inc.)"
user.email = "your_name@company.example.com"

[[rules]]
profile.name = "company"
owner = "acme" # Applies company profiles to all repositories in `acme` org

[[rules]]
profile.name = "default"
```

### Configuring applications to open repos in
Edit `~/.ghr/ghr.toml` and add entries as you like:

```toml
[applications.vscode]
cmd = "code"
args = ["%p"]
```

> **Note**
> `%p` will be replaced by the repository path.

### Finding path of the repository
```shell
ghr path # Root directory
ghr path <owner>/<repo> # Repository directory
ghr path <url> # Repository directory resolved by URL
ghr path github.com/<owner>/<repo> # Repository directory of the specified host
ghr path --owner=<owner> # Owner root
ghr path --host=github.com # Host root
ghr path --host=github.com --owner=<owner> # Owner root of the specified host
```

## 🛠 Customising
You can change the root of repositories managed by ghr by setting environment variable `GHR_ROOT` in your shell profile.

```shell
ghr path # ~/.ghr
GHR_ROOT=/path/to/root ghr path # /path/to/root
```
