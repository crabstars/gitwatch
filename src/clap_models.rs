use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[clap(version = "0.1")]
pub struct Command {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    New(New),
    Rm(Remove),
    Init(Init),
    Set(Set),
    Add(Add),
    List(List),
    Watch(NoArgs),
    Info(NoArgs),
}

#[derive(Parser)]
pub struct NoArgs {}

#[derive(Parser)]
pub struct New {
    // Overwrite existing config
    #[arg(short, long, action= ArgAction::Set, default_value_t = false)]
    pub overwrite: bool,
}

#[derive(Parser)]
pub struct Init {
    // unique name for repo
    #[clap(short, long)]
    pub name: String,

    // unique name for repo
    #[clap(short, long)]
    pub branch: String,

    // absolute path for repo location
    #[clap(short, long)]
    pub path: String,
}

#[derive(Parser)]
pub struct Remove {
    // file or repo
    #[clap(short, long, value_enum)]
    pub r#type: Type,

    // unique name for repo
    #[clap(short, long)]
    pub name: String,

    // relativ file path from repo
    #[clap(short, long)]
    pub file: Option<String>,
}

#[derive(Parser)]
pub struct Add {
    // unique name for repo
    #[clap(short, long)]
    pub name: String,

    // relativ file path from repo
    #[clap(short, long)]
    pub file: String,
}

#[derive(Parser)]
pub struct List {
    // file or repo
    #[clap(short, long, value_enum)]
    pub r#type: TypePlural,

    // unique name for repo
    #[clap(short, long)]
    pub name: Option<String>,
}

#[derive(Parser)]
pub struct Set {
    // unique name for repo
    #[clap(short, long)]
    pub name: String,

    // which prop to change
    #[clap(short, long, value_enum)]
    pub property: Property,

    // when prop branch this should be set
    #[clap(short, long, value_enum)]
    pub branch: Option<String>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum Property {
    Push,
    Active,
    Branch,
}

#[derive(clap::ValueEnum, Clone)]
pub enum Type {
    File,
    Repo,
}

#[derive(clap::ValueEnum, Clone)]
pub enum TypePlural {
    Files,
    Repos,
}
