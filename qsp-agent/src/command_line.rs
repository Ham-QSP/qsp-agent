use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "Florian MAZEN", version = crate::APPLICATION_VERSION, about = "QSP Agent server. Open your transceiver to the cloud.")]
pub struct Cli {
    #[arg(short, long, value_name = "CONFIG_PATH")]
    pub(crate) config: Option<String>,

}