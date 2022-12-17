use colored::Colorize;
use spinners::{Spinner, Spinners};
use std::panic;
use shells::sh;
use std::process;
use std::process::Command;
use std::env;

mod helpers;
use helpers::{help, sanity};
mod config;
use config::{get_kube_config_volume, extract_cas_config_dir_and_volume};

/// sconectl helps to transform cloud-native applications into cloud-confidential applications.
/// It supports to transform native services into confidential services and services meshes
/// into confidential service meshes.

/// --userns=keep-id works only in rootless - fails when running as root

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            let err_msg = format!("{s:?}").red();
            eprintln!("A fatal error occurred: {err_msg} (Error 23882-16605-12717)");
        } else {
            let err_msg = format!("{panic_info}");
            eprintln!(
                "A fatal error occurred: {}. (Error 30339-23400-21867)",
                err_msg.trim_start_matches("panicked at ").red()
            );
        }
        process::exit(1);
    }));

    let vol = sanity();
    let kubeconfig_vol = get_kube_config_volume();

    let original_args: Vec<String> = env::args().collect();
    let result = extract_cas_config_dir_and_volume(original_args);
    let cas_config_dir_env = result.0;
    let cas_config_dir_vol = result.1;
    let args = result.2;
    let mut command_index = 1;
    let mut show_spinner = true;

    let repo = match env::var("SCONECTL_REPO") {
        Ok(repo) => repo,
        Err(_err) => "registry.scontain.com/sconectl".to_string(),
    };
    let image = format!("{repo}/sconecli:latest");

    let mut s = format!(
        r#"docker run --entrypoint="" -ti --network=host --rm {vol} {cas_config_dir_vol} {kubeconfig_vol} -e "SCONECTL_CAS_CONFIG={cas_config_dir_env}" -e "SCONECTL_REPO={repo}" -v "$HOME/.docker":"/root/.docker" -v "$HOME/.scone":"/root/.scone" -v "$PWD":"/wd" -w "/wd" {image}"#
    );
    for (i, arg) in args.iter().enumerate().skip(1) {
        if arg == "--help" && i == 1 {
            help("");
        }
        if arg == "--quiet" && i == 1 {
            show_spinner = false;
            command_index += 1;
        } else {
            s.push_str(&format!(r#" "{}""#, arg));
        }
    }
    if args.len() <= command_index {
        help("You need to specify a COMMAND. (Error 696-7363-5766)")
    }
    // pull image unless SCONECTL_NOPULL is set
    match env::var("SCONECTL_NOPULL") {
        Ok(_ignore) => println!("Warning: SCONECTL_NOPULL is set hence, not pulling CLI image"),
        Err(_err) => {
            let stop = if show_spinner {
                Some(Spinner::with_timer(Spinners::Dots12, format!("Pulling image '{image}'")))
            } else {
                None
            };
            let (code, _stdout, _stderr) = sh!("docker pull {image}");
            if let  Some(mut sp) = stop {
                sp.stop_with_newline();
            }
            if code != 0 {
                eprintln!("\n{} 'docker pull {image}'! Do you have access rights? Please check and send email to info@scontain.com if you need access. (Error 24501-25270-6605)", "Failed to".red());
            }
        }
    }
    let stop = if show_spinner {
        Some(Spinner::with_timer(Spinners::Dots12, format!("Executing command '{}'", args[command_index])))
    } else {
        None
    };
    let mut cmd = Command::new("sh");
    let status = cmd
        .args(["-c", &s])
        .status()
        .expect("failed to execute '{s}'. (Error 8914-6233-13917)");

    if let Some(mut sp) = stop {
        sp.stop_with_newline();
    }
    if !status.success() {
        eprintln!("{} See messages above. Command {} returned error.\n  Error={:?} (Error 22597-24820-10449)", "Execution failed!".red(), args[command_index].blue(), status);
        process::exit(0x0101);
    } else {
        println!();
    }
}
