use self_update::backends::github::Update;
use self_update::cargo_crate_version;

pub async fn handle_command() -> color_eyre::Result<()> {
    println!("Checking for updates...");

    let status = Update::configure()
        .repo_owner("paullj")
        .repo_name("dough")
        .bin_name("dough")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    if status.updated() {
        println!("Successfully updated to version {}", status.version());
    } else {
        println!("Already up to date (version {})", cargo_crate_version!());
    }

    Ok(())
}

pub async fn check_update() -> color_eyre::Result<Option<String>> {
    // Simply check if update is available using the Update builder
    let updater = Update::configure()
        .repo_owner("paullj")
        .repo_name("dough")
        .bin_name("dough")
        .current_version(cargo_crate_version!())
        .build()?;

    // Check if update is needed
    match updater.get_latest_release() {
        Ok(release) => {
            let current = cargo_crate_version!();
            if release.version != current {
                return Ok(Some(release.version));
            }
            Ok(None)
        }
        Err(_) => Ok(None), // Silently fail on network errors
    }
}
