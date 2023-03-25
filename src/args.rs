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

#[derive(Debug, Args, Clone)]
pub struct RunCommand {

    /// The server port
    #[clap(short, long)]
    pub port: u16,

    /// The server host
    #[clap(long)]
    pub host: String,

    // The size of the thread pool
    #[clap(long, default_value_t = 4)]
    pub pool_size: usize,

    /// The root folder
    #[clap(long, default_value_t = String::from("root"))]
    pub root_folder: String,

    #[clap(subcommand)]
    pub auth_mode: AuthMode
}

#[derive(Debug, Args)]
pub struct InfoCommand {
}

#[derive(Debug, Subcommand, Clone)]
pub enum AuthMode {
    /// No authentication
    None(NoneAuthCommand),
    /// Just basic authentication
    Basic(BasicAuthCommand)
}

#[derive(Debug, Args, Clone)]
pub struct NoneAuthCommand {
}

#[derive(Debug, Args, Clone)]
pub struct BasicAuthCommand {

    /// The protected folders
    #[clap(long, default_value_t = String::from("root"))]
    pub protected_folders: String,

    /// The user name
    #[clap(long, default_value_t = String::from("admin"))]
    pub username: String,

    /// The password
    #[clap(long, default_value_t = String::from("password"))]
    pub password: String,
}


