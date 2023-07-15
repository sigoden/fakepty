## fakepty

[![CI](https://github.com/sigoden/fakepty/actions/workflows/ci.yaml/badge.svg)](https://github.com/sigoden/fakepty/actions/workflows/ci.yaml)
[![Crates](https://img.shields.io/crates/v/fakepty.svg)](https://crates.io/crates/fakepty)


Run a command in a fake pty.

fakepty provides a pty whose size can be customized by environment variables COLUMNS and LINES.

```console
$ fakepty docker run --help

$ COLUMNS=200 fakepty docker run --help
```