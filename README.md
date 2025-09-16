<!-- Improved compatibility of back to top link -->
<a name="readme-top"></a>

<!-- Repo Badges | https://shields.io/badges -->
<div align="center">
  
[![License: GPL][license-shield]][license-url]
[![Issues][issues-shield]][issues-url]

</div>

---

<!-- PROJECT HEADER -->
<br />
<div align="center">
    <a href="https://github.com/IanTeda/personal-ledger-backend">
        <img src="docs/images/logo.png" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Personal Ledger | Backend</h3>
    <p align="center">
        Backend server for the Personal Ledger application. This repository contains a Rust gRPC daemon that interfaces with the database and serves data to clients.
				<br />
        <a href="https://ianteda.github.io/personal-ledger-backend/"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    ·
    <a href="https://github.com/IanTeda/personal-ledger">Personal Ledger (Parent Repo)</a>
    ·
    <a href="https://ianteda.github.io/personal-ledger-backend/issues">Report Bug</a>
    ·
    <a href="https://ianteda.github.io/personal-ledger-backend/issues">Request Feature</a>
    ·		
  </p>
</div>



## Quickstart

- Build: `cargo build`
- Run: `cargo run`
- Tests: `cargo test`

If you use the provided devcontainer, rebuild it first (Command Palette → "Dev Containers: Rebuild Container") and then open a terminal in the container to run the commands above.


## Development

- Language: Rust
- Primary crates & tooling: `tonic` (gRPC), `sqlx` (Postgres), `tracing`, `serde`, `thiserror`.
- Devcontainer: `.devcontainer/devcontainer.json` includes tools such as `mdbook`, `cargo-make`, `cargo-watch`, `cargo-audit`, `clippy`, `rustfmt`, and `protoc`.
- Copilot instructions can be found in `.github/copilot-instructions.md`

## Documentation

Project docs live in the `docs/` folder. They can be browsed at [https://ianteda.github.io/personal-ledger-backend/](https://ianteda.github.io/personal-ledger-backend/) or local running the command:

- Serve mdBook documentation (uses port `8001` in the devcontainer):

	```bash
	mdbook serve --port 8001
	```

## Protobuf / gRPC

Protobuf files are stored under `protos/` (submodule). The build script `build.rs` compiles protos into Rust types — ensure `protoc` is available (the devcontainer installs `protobuf-compiler`).

Concirm gRPC reflections service with `grpcurl -plaintext localhost:50051 list`

## Code Quality

- Formatting: `cargo fmt` / `rustfmt`
- Linting: `cargo clippy`
- Coverage: `cargo tarpaulin` (available in the devcontainer)

## Running the MCP / awesome-copilot server (optional)

If you want to run the `awesome-copilot` MCP image from the `awesome-copilot` project locally (for experimenting with Copilot customizations), you can run it with Docker inside the devcontainer:

```bash
docker run -i --rm ghcr.io/microsoft/mcp-dotnet-samples/awesome-copilot:latest
```

Do not commit any credential or secret files. If the MCP server requires configuration, prefer using a workspace-local `.vscode/` configuration file and local secrets (user settings or environment variables).

## Contributing

Please open issues or pull requests for bugs and feature requests. Follow the repository coding standards: add tests, run `cargo fmt`, and run `cargo clippy` before submitting.

## License

See the repository `LICENSE` file for licensing information.


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
<!-- https://shields.io/badges -->
[contributors-shield]: https://img.shields.io/github/contributors/IanTeda/personal-ledger-backend.svg?style=for-the-badge
[contributors-url]: https://github.com/IanTeda/personal-ledger-backend/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/IanTeda/personal-ledger-backend.svg?style=for-the-badge
[forks-url]: https://github.com/IanTeda/personal-ledger-backend/network/members
[stars-shield]: https://img.shields.io/github/stars/IanTeda/personal-ledger-backend.svg?style=for-the-badge
[stars-url]: https://github.com/IanTeda/personal-ledger-backend/stargazers
[issues-shield]: https://img.shields.io/github/issues/IanTeda/personal-ledger-backend.svg?style=for-the-badge
[issues-url]: https://github.com/IanTeda/personal-ledger-backend/issues
[license-shield]: https://img.shields.io/github/license/IanTeda/personal-ledger-backend.svg?style=for-the-badge
[license-url]: https://github.com/IanTeda/personal-ledger-backend/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/ianteda
[product-screenshot]: docs/images/screenshot.png