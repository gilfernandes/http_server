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

Here are the instructions generated by Clap:

```
Usage: http_server.exe <COMMAND>

Commands:
  run   Run the server
  info  Print info about the server
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

The command `run` is the most important one and has the following parameters:

```
Usage: http_server.exe run [OPTIONS] --port <PORT> --host <HOST>

Options:
  -p, --port <PORT>            The server port
      --host <HOST>            The server host
      --pool-size <POOL_SIZE>  [default: 4]
  -h, --help                   Print help

```

A typical execution command would be:

```http_server.exe run --host localhost --port 7878```
