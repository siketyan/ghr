# 🚀 ghr
[![crates.io](https://img.shields.io/crates/v/ghr.svg)](https://crates.io/crates/ghr)
[![Rust](https://github.com/siketyan/ghr/actions/workflows/rust.yml/badge.svg)](https://github.com/siketyan/ghr/actions/workflows/rust.yml)

Yet another repository management with auto-attaching profiles.

## 📦 Installation
```shell
cargo install ghr
```

## 💚 Usages
### Cloning a repository
ghr supports many patterns or URLs of the repository to clone:

```
ghr clone <owner>/<repo>
ghr clone github.com:<owner>/<repo>
ghr clone https://github.com/<owner>/<repo>.git
ghr clone ssh://git@github.com/<owner>/<repo>.git
ghr clone git@github.com:<owner>/<repo>.git
```

Easy!

### Attaching profiles
Create `~/.ghr/config.toml` and edit as you like:

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
