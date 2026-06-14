# DiskPeek

Disk space visualizer. Scans directories and renders an interactive treemap.

## Build & Run

```bash
cargo run
```

Opens http://localhost:3030 in your browser. Enter a directory path or leave empty to scan home.

## Architecture

- **Rust binary** (~180 lines): HTTP server + recursive directory scanner
- **HTML viewer** (~150 lines): D3.js treemap with drill-down

Zero build steps. No Node.js, no bundler, no framework.

## License

MIT
