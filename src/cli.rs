use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cart",
    author = "yoel3imari",
    version,
    about = "Cartridge: The application cartridge manager for Linux",
    long_about = "Plug-and-play AppImage cartridge manager for Linux. Search, install, integrate, update, sandbox, and manage self-contained application cartridges seamlessly."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for AppImages across AppImageHub and GitHub
    Search(SearchArgs),

    /// Install an AppImage by name, GitHub repo (owner/repo), or direct URL
    Install(InstallArgs),

    /// Integrate an existing local AppImage file into the system
    Integrate(IntegrateArgs),

    /// List all installed AppImages
    List,

    /// View detailed information for an installed app or catalog entry
    Info(InfoArgs),

    /// Update one or all installed AppImages
    Update(UpdateArgs),

    /// Rollback an application to its previous version
    Rollback(RollbackArgs),

    /// Remove an installed AppImage and its desktop integration
    #[command(alias = "uninstall")]
    Remove(RemoveArgs),

    /// Run an installed AppImage (optionally with Bubblewrap sandboxing)
    Run(RunArgs),

    /// Clean broken symlinks, orphaned icons, and stale temporary files
    Clean,

    /// Force refresh the AppImageHub catalog cache
    Refresh,

    /// Generate shell completion scripts (bash, zsh, fish, elvish, powershell)
    Completions(CompletionsArgs),
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// Target shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search keyword (app name, category, or description)
    pub query: String,

    /// Maximum number of search results to return
    #[arg(short, long, default_value = "15")]
    pub limit: usize,

    /// Force refresh online catalog cache before searching
    #[arg(short, long)]
    pub refresh: bool,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// App name from catalog, GitHub repo ('owner/repo'), or direct URL
    pub target: String,

    /// Custom identifier / command name for the app
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct IntegrateArgs {
    /// Path to the local .AppImage file
    pub path: PathBuf,

    /// Custom identifier / command name for the app
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// App ID or catalog name
    pub target: String,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// App ID to update (leave empty or use --all to update all)
    pub app_id: Option<String>,

    /// Update all installed applications
    #[arg(short, long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct RollbackArgs {
    /// App ID to revert to its previous version
    pub app_id: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// App ID to remove
    pub app_id: String,

    /// Also purge configuration and data files in ~/.config
    #[arg(short, long)]
    pub purge: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// App ID to run
    pub app_id: String,

    /// Run inside an isolated Bubblewrap sandbox
    #[arg(short, long)]
    pub sandbox: bool,

    /// Disallow network access when sandboxed
    #[arg(long)]
    pub offline: bool,

    /// Arguments to forward to the AppImage
    #[arg(last = true)]
    pub args: Vec<String>,
}
