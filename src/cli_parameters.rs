pub use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliParams {
    #[clap(short, long, default_value = "0.0.0.0:3030")]
    pub socket_address: String,
}