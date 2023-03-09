use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::{fs, io};

use console::style;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crates::is_new_crates_format;
use crate::crates_index::rewrite_config_json;

use crate::rustup::download_platform_list;
use crate::serve::TlsConfig;
use crate::verify;

#[derive(Error, Debug)]
pub enum MirrorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("TOML deserialization error: {0:?}")]
    Parse(#[from] toml_edit::de::Error),

    #[error("Config file error: {0}")]
    Config(String),

    #[error("Command line error: {0}")]
    CmdLine(String),

    #[error("Download error: {0}")]
    DownloadError(#[from] crate::download::DownloadError),

    #[error("Toml error: {0}")]
    Serialize(#[from] toml_edit::TomlError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMirror {
    pub retries: usize,
    pub contact: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigRustup {
    pub sync: bool,
    pub download_threads: usize,
    pub source: String,
    pub download_dev: Option<bool>,
    pub download_gz: Option<bool>,
    pub download_xz: Option<bool>,
    pub platforms_unix: Option<Vec<String>>,
    pub platforms_windows: Option<Vec<String>>,
    pub keep_latest_stables: Option<usize>,
    pub keep_latest_betas: Option<usize>,
    pub keep_latest_nightlies: Option<usize>,
    pub pinned_rust_versions: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigCrates {
    pub sync: bool,
    pub download_threads: usize,
    pub source: String,
    pub source_index: String,
    pub use_new_crates_format: Option<bool>,
    pub base_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub mirror: ConfigMirror,
    pub rustup: Option<ConfigRustup>,
    pub crates: Option<ConfigCrates>,
}

pub fn create_mirror_directories(path: &Path, ignore_rustup: bool) -> Result<(), io::Error> {
    if !ignore_rustup {
        // Rustup directories
        fs::create_dir_all(path.join("rustup/dist"))?;
        fs::create_dir_all(path.join("dist"))?;
    }

    // Crates directories
    fs::create_dir_all(path.join("crates.io-index"))?;
    fs::create_dir_all(path.join("crates"))?;
    Ok(())
}

pub fn create_mirror_toml(path: &Path, ignore_rustup: bool) -> Result<bool, MirrorError> {
    if path.join("mirror.toml").exists() {
        return Ok(false);
    }

    // Read the defautlt toml, edit if required, using toml_edit to keep format
    let config = include_str!("mirror.default.toml");
    let mut config = config.parse::<toml_edit::Document>()?;

    if ignore_rustup {
        config["rustup"]["sync"] = toml_edit::value(false);
    }

    let path = path.join("mirror.toml");
    let bytes = config.to_string();
    fs::write(path, bytes)?;

    Ok(true)
}

pub fn load_mirror_toml(path: &Path) -> Result<Config, MirrorError> {
    Ok(toml_edit::easy::from_str(&fs::read_to_string(
        path.join("mirror.toml"),
    )?)?)
}

pub fn init(path: &Path, ignore_rustup: bool) -> Result<(), MirrorError> {
    create_mirror_directories(path, ignore_rustup)?;
    if create_mirror_toml(path, ignore_rustup)? {
        eprintln!("Successfully created mirror base at `{}`.", path.display());
    } else {
        eprintln!("Mirror base already exists at `{}`.", path.display());
    }
    eprintln!(
        "Make any desired changes to {}/mirror.toml, then run panamax sync {}.",
        path.display(),
        path.display()
    );

    Ok(())
}

pub fn default_user_agent() -> String {
    format!("Panamax/{}", env!("CARGO_PKG_VERSION"))
}

pub async fn sync(path: &Path, vendor_path: Option<PathBuf>) -> Result<(), MirrorError> {
    if !path.join("mirror.toml").exists() {
        eprintln!(
            "Mirror base not found! Run panamax init {} first.",
            path.display()
        );
        return Ok(());
    }
    let mirror = load_mirror_toml(path)?;

    // Fail if use_new_crates_format is not true, and old format is detected.
    // If use_new_crates_format is true and new format is detected, warn the user.
    // If use_new_crates_format is true, ignore the format and assume it's new.
    if let Some(crates) = &mirror.crates {
        if crates.sync && !is_new_crates_format(&path.join("crates"))? {
            eprintln!("Your crates directory is using the old 0.2 format, however");
            eprintln!("Panamax 0.3+ has deprecated this format for a new one.");
            eprintln!("Please delete crates/ from your mirror directory to continue.");
            return Ok(());
        }
    }

    // Handle the contact information
    let user_agent_str = if let Some(ref contact) = mirror.mirror.contact {
        if contact != "your@email.com" {
            format!("Panamax/{} ({})", env!("CARGO_PKG_VERSION"), contact)
        } else {
            default_user_agent()
        }
    } else {
        default_user_agent()
    };

    // Set the user agent with contact information.
    let user_agent = match HeaderValue::from_str(&user_agent_str) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Your contact information contains invalid characters!");
            eprintln!("It's recommended to use a URL or email address as contact information.");
            eprintln!("{e:?}");
            return Ok(());
        }
    };

    if let Some(rustup) = mirror.rustup {
        if rustup.sync {
            crate::rustup::sync(path, &mirror.mirror, &rustup, &user_agent).await?;
        } else {
            eprintln!("Rustup sync is disabled, skipping...");
        }
    } else {
        eprintln!("Rustup section missing, skipping...");
    }

    if let Some(crates) = mirror.crates {
        if crates.sync {
            sync_crates(path, vendor_path, &mirror.mirror, &crates, &user_agent).await;
        } else {
            eprintln!("Crates sync is disabled, skipping...");
        }
    } else {
        eprintln!("Crates section missing, skipping...");
    }

    eprintln!("Sync complete.");

    Ok(())
}

/// Rewrite the config.toml only.
///
/// Note that this will also fast-forward the repository
/// from origin/master, to keep a clean slate.
pub fn rewrite(path: &Path, base_url: Option<String>) -> Result<(), MirrorError> {
    if !path.join("mirror.toml").exists() {
        eprintln!(
            "Mirror base not found! Run panamax init {} first.",
            path.display()
        );
        return Ok(());
    }
    let mirror = load_mirror_toml(path)?;

    if let Some(crates) = mirror.crates {
        if let Some(base_url) = base_url.as_deref().or(crates.base_url.as_deref()) {
            if let Err(e) = rewrite_config_json(&path.join("crates.io-index"), base_url) {
                eprintln!("Updating crates.io-index config failed: {e:?}");
            }
        } else {
            eprintln!("No base_url was provided.");
            eprintln!(
                "This needs to be provided by command line or in the mirror.toml to continue."
            )
        }
    } else {
        eprintln!("Crates section missing in mirror.toml.");
    }

    Ok(())
}

/// Synchronize and handle the crates.io-index repository.
pub async fn sync_crates(
    path: &Path,
    vendor_path: Option<PathBuf>,
    mirror: &ConfigMirror,
    crates: &ConfigCrates,
    user_agent: &HeaderValue,
) {
    eprintln!("{}", style("Syncing Crates repositories...").bold());

    if let Err(e) = crate::crates_index::sync_crates_repo(path, crates) {
        eprintln!("Downloading crates.io-index repository failed: {e:?}");
        eprintln!("You will need to sync again to finish this download.");
        return;
    }

    if let Err(e) =
        crate::crates::sync_crates_files(path, vendor_path, mirror, crates, user_agent).await
    {
        eprintln!("Downloading crates failed: {e:?}");
        eprintln!("You will need to sync again to finish this download.");
        return;
    }

    if let Err(e) = crate::crates_index::update_crates_config(path, crates) {
        eprintln!("Updating crates.io-index config failed: {e:?}");
        eprintln!("You will need to sync again to finish this download.");
    }

    eprintln!("{}", style("Syncing Crates repositories complete!").bold());
}

pub async fn serve(
    path: PathBuf,
    listen: Option<IpAddr>,
    port: Option<u16>,
    cert_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
) -> Result<(), MirrorError> {
    let listen = listen.unwrap_or_else(|| {
        "::".parse()
            .expect(":: IPv6 address should never fail to parse")
    });
    let port = port.unwrap_or_else(|| if cert_path.is_some() { 8443 } else { 8080 });
    let socket_addr = SocketAddr::new(listen, port);

    match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => {
            crate::serve::serve(
                path,
                socket_addr,
                Some(TlsConfig {
                    cert_path,
                    key_path,
                }),
            )
            .await
        }
        (None, None) => crate::serve::serve(path, socket_addr, None).await,
        (Some(_), None) => {
            return Err(MirrorError::CmdLine(
                "cert_path set but key_path not set.".to_string(),
            ))
        }
        (None, Some(_)) => {
            return Err(MirrorError::CmdLine(
                "key_path set but cert_path not set.".to_string(),
            ))
        }
    };

    Ok(())
}

/// Print out a list of all platforms.
pub(crate) async fn list_platforms(source: String, channel: String) -> Result<(), MirrorError> {
    let targets = download_platform_list(source.as_str(), channel.as_str()).await?;

    println!("All currently available platforms for the {channel} channel:");
    for t in targets {
        println!("  {t}");
    }

    Ok(())
}

/// Verify coherence between local mirror and local crates.io-index.
/// This function is bale to fix mirror by downloading missing crates.
/// Users can alter the actual downloaded file at run time.
pub(crate) async fn verify(
    path: PathBuf,
    dry_run: bool,
    assume_yes: bool,
) -> Result<(), MirrorError> {
    if !path.join("mirror.toml").exists() {
        eprintln!(
            "Mirror base not found! Run panamax init {} first.",
            path.display()
        );
        return Ok(());
    }
    let config = load_mirror_toml(&path)?;

    // Fail if use_new_crates_format is not true, and old format is detected.
    // If use_new_crates_format is true and new format is detected, warn the user.
    // If use_new_crates_format is true, ignore the format and assume it's new.
    if let Some(config) = &config.crates {
        if config.sync && !is_new_crates_format(&path.join("crates"))? {
            eprintln!("Your crates directory is using the old 0.2 format, however");
            eprintln!("Panamax 0.3+ has deprecated this format for a new one.");
            eprintln!("Please delete crates/ from your mirror directory to continue.");
            return Ok(());
        }
    }

    eprintln!("{}", style("Verifying mirror state...").bold());

    // Getting crates.sync config state
    let crates_config = config.crates.as_ref();
    let sync = crates_config.map_or(false, |crate_config| crate_config.sync);

    // Determining number of steps
    let steps = if dry_run || !sync { 1 } else { 2 };
    let mut current_step = 1;

    if let Some(mut missing_crates) =
        verify::verify_mirror(path.clone(), &mut current_step, steps).await?
    {
        if dry_run || !sync {
            if !sync {
                eprintln!("Crates sync is disabled, only printing missing crates...");
            }
            missing_crates.iter().for_each(|c| {
                println!("Missing crate: {} - version {}", c.get_name(), c.get_vers());
            });
            return Ok(());
        }

        // Safe to unwrap here
        let crates_config = crates_config.unwrap();

        debug_assert_ne!(steps, current_step);

        // Ask users to choose whether to filter missing crates to download or not
        if !assume_yes {
            missing_crates = verify::handle_user_input(missing_crates).await?;
        }

        let mirror_config = &config.mirror;

        // Downloading missing crates
        verify::fix_mirror(
            mirror_config,
            crates_config,
            path,
            missing_crates,
            &mut current_step,
            steps,
        )
        .await?;
    }

    Ok(())
}
