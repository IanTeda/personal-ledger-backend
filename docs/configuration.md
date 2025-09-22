# Configuration

This document explains how the Personal Ledger backend loads configuration, what defaults it uses, how to provide a config file, and how to override values with environment variables.

Primary implementation: `src/config.rs` (types `LedgerConfig` and `ServerConfig`, loader `LedgerConfig::parse`).

## Configuration sources and precedence

The loader merges three sources in increasing priority (later sources override earlier ones):

1. Defaults (lowest priority)
2. Optional configuration file at `config/ledger-backend.conf`
3. Environment variables prefixed with `LEDGER_BACKEND_` (highest priority)

This means environment variables override values in the config file, and the config file overrides built-in defaults.

## Built-in defaults

The application provides sensible defaults when no configuration is provided:

- `server.address` — `127.0.0.1`
- `server.port` — `50059`
- `server.tls_enabled` — `false`
- `server.tls_cert_path` — `null` (none)
- `server.tls_key_path` — `null` (none)

The constants that control these defaults are defined in `src/config.rs` as `DEFAULT_SERVER_ADDRESS`, `DEFAULT_SERVER_PORT`, and `DEFAULT_TLS_ENABLED`.

## Config file format (INI/CONF)

The loader expects INI by default and looks for `config/ledger-backend.conf`. A minimal file looks like this:

```INI
[server]
# The IP address the gRPC server will bind to.
# Defaults to 127.0.0.1 when not provided.
address = "127.0.0.1"

# The port the gRPC server will listen on.
# Defaults to 50059 when not provided.
port = 50059

# Enable TLS for the gRPC server. Set to true to enable TLS and provide the
# certificate and key file paths below.
tls_enabled = false

# Optional paths to the TLS certificate and key files (PEM format).
# When `tls_enabled = true` both values should be provided.
# tls_cert_path = "/path/to/tls/cert.pem"
# tls_key_path = "/path/to/tls/key.pem"
```

The repository also contains a commented example `config/ledger-backend.conf` (INI-like) with the same fields — this is provided as a reference and convenience for developers, but the loader expects .conf unless changed in code.

## Environment variable overrides

Any configuration key may be overridden using environment variables using the `LEDGER_BACKEND` prefix. The loader uses underscores as separators; example mappings:

- `server.address` → `LEDGER_BACKEND_SERVER_ADDRESS`
- `server.port` → `LEDGER_BACKEND_SERVER_PORT`
- `server.tls_enabled` → `LEDGER_BACKEND_SERVER_TLS_ENABLED`
- `server.tls_cert_path` → `LEDGER_BACKEND_SERVER_TLS_CERT_PATH`
- `server.tls_key_path` → `LEDGER_BACKEND_SERVER_TLS_KEY_PATH`

Examples:

```bash
export LEDGER_BACKEND_SERVER_ADDRESS=0.0.0.0
export LEDGER_BACKEND_SERVER_PORT=50059
export LEDGER_BACKEND_SERVER_TLS_ENABLED=true
export LEDGER_BACKEND_SERVER_TLS_CERT_PATH=/etc/personal-ledger/cert.pem
export LEDGER_BACKEND_SERVER_TLS_KEY_PATH=/etc/personal-ledger/key.pem

cargo run
```

You can also set env vars inline when running:

```bash
LEDGER_BACKEND_SERVER_ADDRESS=0.0.0.0 LEDGER_BACKEND_SERVER_PORT=50059 cargo run
```

## Troubleshooting

- Error: `Config(Parsing(missing field `server`))` — This means the deserialized data did not contain the required `server` section. Add the `server:` section to your YAML file or set the required fields via environment variables.
- If the server fails to bind, ensure the `server.address:server.port` combination is valid and not already in use.
- If enabling TLS, make sure `tls_enabled` is `true` and both `tls_cert_path` and `tls_key_path` point to valid PEM files.

## Quick run examples

Run with defaults (no file, no env):

```bash
cargo run
```

Run with file and env overrides:

```bash
# File: config/ledger-backend.yaml exists with values
LEDGER_BACKEND_SERVER_PORT=60000 cargo run
```

## Where to look in the code

- `src/config.rs` — loader, defaults, and `LedgerConfig`/`ServerConfig` types
- `src/error.rs` — mapping configuration errors into `LedgerError` / `tonic::Status` where relevant
