Gil HTTP Server
---------------

Tiny HTTP server implementation in Rust which should consist of a single file with multi threading support.
Only supports GET and HEAD requests for now.

### Building

Release build:

```
cargo build -r
```

### Executing

Out of the box expects a *root* folder in the current directory.

If you want another root folder, please set the `ROOT_FOLDER` environment variable with the root folder.