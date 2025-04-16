use std::{
    fmt::Display,
    io::Write,
    sync::{LazyLock, RwLock},
};

use indicatif::HumanBytes;
use nu_ansi_term::Color::{self, Blue, Cyan, Green, LightRed, Magenta, Red};
use serde::Serialize;
use soar_core::{
    config::get_config, database::models::PackageExt, error::ErrorContext,
    package::install::InstallTarget, SoarResult,
};
use tracing::{error, info};

pub static COLOR: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(true));

pub fn interactive_ask(ques: &str) -> SoarResult<String> {
    print!("{}", ques);

    std::io::stdout()
        .flush()
        .with_context(|| "flushing stdout stream".to_string())?;

    let mut response = String::new();
    std::io::stdin()
        .read_line(&mut response)
        .with_context(|| "reading input from stdin".to_string())?;

    Ok(response.trim().to_owned())
}

pub struct Colored<T: Display>(pub Color, pub T);

impl<T: Display> Display for Colored<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = COLOR.read().unwrap();
        if *color {
            write!(f, "{}", self.0.prefix())?;
            self.1.fmt(f)?;
            write!(f, "{}", self.0.suffix())
        } else {
            self.1.fmt(f)
        }
    }
}

pub fn vec_string<T: Display + Serialize>(value: Option<Vec<T>>) -> Option<String> {
    value.and_then(|json| serde_json::to_string(&json).ok())
}

pub fn get_valid_selection(max: usize) -> SoarResult<usize> {
    loop {
        let response = interactive_ask("Select a package: ")?;
        match response.parse::<usize>() {
            Ok(n) if n > 0 && n <= max => return Ok(n - 1),
            _ => error!("Invalid selection, please try again."),
        }
    }
}

pub fn select_package_interactively<T: PackageExt>(
    pkgs: Vec<T>,
    package_name: &str,
) -> SoarResult<Option<T>> {
    info!("Multiple packages found for {package_name}");
    for (idx, pkg) in pkgs.iter().enumerate() {
        info!(
            "[{}] {}#{}:{} | {}",
            idx + 1,
            Colored(Blue, &pkg.pkg_name()),
            Colored(Cyan, &pkg.pkg_id()),
            Colored(Green, pkg.repo_name()),
            Colored(LightRed, pkg.version())
        );
    }

    let selection = get_valid_selection(pkgs.len())?;
    Ok(pkgs.into_iter().nth(selection))
}

pub fn has_no_desktop_integration(repo_name: &str, notes: Option<&[String]>) -> bool {
    !get_config().has_desktop_integration(repo_name)
        || notes.is_some_and(|all| {
            all.iter()
                .any(|note| note.contains("NO_DESKTOP_INTEGRATION"))
        })
}

pub fn pretty_package_size(ghcr_size: Option<u64>, size: Option<u64>) -> String {
    ghcr_size
        .map(|size| format!("{}", Colored(Magenta, HumanBytes(size))))
        .or_else(|| size.map(|size| format!("{}", Colored(Magenta, HumanBytes(size)))))
        .unwrap_or_default()
}

pub fn ask_target_action(targets: &[InstallTarget], action: &str) -> SoarResult<()> {
    info!(
        "\n{}\n",
        Colored(
            Green,
            format!(
                "These are the packages that would be {}:",
                if action == "install" {
                    "installed"
                } else {
                    "updated"
                }
            )
        )
    );
    for target in targets {
        info!(
            "{}#{}:{} ({})",
            Colored(Blue, &target.package.pkg_name),
            Colored(Cyan, &target.package.pkg_id),
            Colored(Green, &target.package.repo_name),
            Colored(LightRed, &target.package.version)
        )
    }

    info!(
        "Total: {} packages. Estimated download size: {}\n",
        targets.len(),
        HumanBytes(targets.iter().fold(0, |acc, target| {
            acc + target
                .package
                .ghcr_size
                .or(target.package.size)
                .unwrap_or_default()
        }))
    );
    let response = interactive_ask(&format!(
        "Would you like to {} these packages? [{}/{}] ",
        action,
        Colored(Green, "Yes"),
        Colored(Red, "No")
    ))?
    .to_lowercase();
    let response = response.trim();

    if !response.is_empty() && response != "y" {
        info!("Quitting");
        std::process::exit(0);
    }

    Ok(())
}
