use clap::Parser;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use std::process::Command;

use crate::cli::{Cli, Commands};
use crate::error::{CartridgeError, Result};
use crate::manager::AppManager;
use crate::sandbox::SandboxRunner;

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let manager = AppManager::new();

    match cli.command {
        Commands::Search(args) => {
            println!("🔍 Searching for '{}'...", args.query);
            let results = manager
                .search(&args.query, args.limit, args.refresh)
                .await?;

            if results.is_empty() {
                println!("No applications found matching '{}'.", args.query);
                return Ok(());
            }

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    Cell::new("ID").fg(Color::Cyan),
                    Cell::new("Name").fg(Color::Cyan),
                    Cell::new("Description").fg(Color::Cyan),
                    Cell::new("Categories").fg(Color::Cyan),
                    Cell::new("Source").fg(Color::Cyan),
                ]);

            for r in results {
                let desc = if r.description.chars().count() > 60 {
                    format!("{}...", r.description.chars().take(57).collect::<String>())
                } else {
                    r.description.clone()
                };

                table.add_row(vec![
                    Cell::new(&r.id).fg(Color::Green),
                    Cell::new(&r.name),
                    Cell::new(desc),
                    Cell::new(r.categories.join(", ")),
                    Cell::new(&r.source).fg(Color::Yellow),
                ]);
            }

            println!("{table}");
            println!("💡 To install: cart install <ID>");
        }

        Commands::Install(args) => {
            manager.install(&args.target, args.name.as_deref()).await?;
        }

        Commands::Integrate(args) => {
            manager.integrate_local_file(&args.path, args.name.as_deref())?;
        }

        Commands::List => {
            manager.list()?;
        }

        Commands::Info(args) => {
            manager.info(&args.target).await?;
        }

        Commands::Update(args) => {
            let target = if args.all {
                None
            } else {
                args.app_id.as_deref()
            };
            manager.update(target).await?;
        }

        Commands::Rollback(args) => {
            manager.rollback(&args.app_id)?;
        }

        Commands::Remove(args) => {
            manager.remove(&args.app_id, args.purge)?;
        }

        Commands::Run(args) => {
            let app = manager.state().get_app(&args.app_id)?.ok_or_else(|| {
                CartridgeError::NotFound(format!("Application '{}' is not installed", args.app_id))
            })?;

            if !app.binary_path.exists() {
                return Err(CartridgeError::NotFound(format!(
                    "AppImage binary not found at {}",
                    app.binary_path.display()
                )));
            }

            if args.sandbox {
                println!("🛡️  Running {} in Bubblewrap sandbox...", app.name);
                let status =
                    SandboxRunner::run_sandboxed(&app.binary_path, &args.args, !args.offline)?;
                std::process::exit(status.code().unwrap_or(0));
            } else {
                let mut cmd = Command::new(&app.binary_path);
                cmd.args(&args.args);

                if !crate::util::fuse::is_fuse_available() {
                    let distro = crate::util::fuse::DistroInfo::detect();
                    let pkg = distro.recommended_fuse_package();
                    println!("ℹ️  libfuse2 not detected on host. Running in extraction mode (APPIMAGE_EXTRACT_AND_RUN=1).");
                    println!("💡 Tip: Install '{}' for faster native AppImage startup.", pkg);
                    cmd.env("APPIMAGE_EXTRACT_AND_RUN", "1");
                }

                let status = cmd
                    .status()
                    .map_err(|e| {
                        CartridgeError::Other(format!("Failed to execute application: {e}"))
                    })?;
                std::process::exit(status.code().unwrap_or(0));
            }
        }

        Commands::Clean => {
            manager.clean()?;
        }

        Commands::Refresh => {
            println!("🔄 Fetching fresh AppImageHub catalog feed...");
            let feed = manager
                .search_service()
                .appimagehub()
                .fetch_and_cache()
                .await?;
            println!(
                "✨ Successfully cached {} applications from AppImageHub!",
                feed.items.len()
            );
        }

        Commands::Completions(args) => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            let bin_name = std::env::args()
                .next()
                .and_then(|p| {
                    std::path::Path::new(&p)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| "cart".to_string());
            let bin_name = if bin_name.contains("cartridge") {
                "cartridge"
            } else {
                "cart"
            };
            clap_complete::generate(args.shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }

    Ok(())
}
