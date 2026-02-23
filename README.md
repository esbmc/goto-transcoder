# GOTO Transcoder

[![Build](https://github.com/esbmc/goto-transcoder/actions/workflows/rust.yml/badge.svg)](https://github.com/esbmc/goto-transcoder/actions/workflows/rust.yml)
[![Integration](https://github.com/esbmc/goto-transcoder/actions/workflows/integration.yml/badge.svg)](https://github.com/esbmc/goto-transcoder/actions/workflows/integration.yml)
[![codecov](https://codecov.io/gh/esbmc/goto-transcoder/branch/main/graph/badge.svg)](https://codecov.io/gh/esbmc/goto-transcoder)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This project is still in early development stages. The goal here is to have a tool that facilitates visualizing and changing GOTO programs generated from ESBMC and CBMC by:
- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing into GBF to ESBMC/CBMC. Allowing to convert between both versions.

Check the [wiki](https://github.com/esbmc/goto-transcoder/wiki/Steps-to-verify-Rust-code) for use examples.

### Contributing

- [Architecture](docs/Architecture.md)
- [Development](docs/Development.md)

## Roadmap

### v0.1
- [ ] Full support for CBMC regression (in `ESBMC/regression/cbmc`). Such that CBMC -> ESBMC has the same verdict of invoking ESBMC directly.
- [ ] String interner.

### Future projects

I expect to be able to use this as a library, opening a few possibilities:
- [ ] LibTranscoder: Use this project as a way to parse/generate goto binary programs.
- [ ] Partial equivalence checking (see [alive2](https://github.com/AliveToolkit/alive2)).
- [ ] GOTO optimizer.
- [ ] GOTO interpreter. 
