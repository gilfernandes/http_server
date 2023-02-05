use clap:: {
    Args,
    Parser,
    Subcommand
};

/// Simple Http Server
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct HttpServerArgs {

    #[clap(subcommand)]
    pub mode: Mode
}

#[derive(Debug, Subcommand)]
pub enum Mode {
    /// Run the server
    Run(RunCommand),

    /// Print info about the server
    Info(InfoCommand),
}

#[derive(Debug, Args)]
pub struct RunCommand {

    /// The server port
    #[clap(short, long)]
    pub port: u16,

    /// The server host
    #[clap(long)]
    pub host: String,

    #[clap(long, default_value_t = 4)]
    pub pool_size: usize,
}

#[derive(Debug, Args)]
pub struct InfoCommand {
}