## Development Container (`devcontainer`)

The Personal Ledger Backend project provides a pre-configured development container (devcontainer) to ensure a consistent and reproducible development environment.

### What is a Devcontainer?

A devcontainer is a Docker-based environment defined by configuration files (i.e. `.devcontainer/devcontainer.json`) that sets up the tools, dependencies, and settings needed for development. When you open the project in [Visual Studio Code](https://code.visualstudio.com/) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers), VS Code will automatically build and connect to this container, giving you a ready-to-code workspace.

### What's Included?

The devcontainer for this project is based on the official Rust image and includes:

- **Rust toolchain** (latest stable, with `clippy`, `rustfmt`)
- **Protobuf compiler** (`protoc` and `libprotobuf-dev`)
- **Common Rust utilities**: `mdbook`, `cargo-make`, `cargo-watch`, `cargo-audit`
- **Docker CLI** (for running containers inside the devcontainer)
- **Git** and **GitHub CLI**
- **grpcurl** (for testing gRPC endpoints)
- **VS Code extensions** for Rust, Docker, Markdown, SQL, YAML, and Protocol Buffers
- **Port forwarding** for mdBook documentation server (port 8001)
- **Pre-configured environment** for code formatting, linting, and testing

### How to Use the Devcontainer

1. **Open the Project in VS Code**  
Open the root of the repository in Visual Studio Code.

2. **Rebuild the Devcontainer (First Time or After Changes)**  
Open the Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P` on Mac) and run: `Dev Container: Build and Run`

3. **Wait for Setup**  
VS Code will build the container image, install all dependencies, and set up the environment. This may take a few minutes the first time.

4. **Start Developing**  
- Use the integrated terminal for running commands (`cargo build`, `cargo run`, `cargo test`, etc.).
- All tools and extensions are pre-installed and ready to use.
- The container includes everything needed for Rust, gRPC, and documentation workflows.

5. **Serving Documentation**  
To serve the project documentation locally (mdBook), run: