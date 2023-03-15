use colored::Colorize;
use shells::sh;
use spinners::{Spinner, Spinners};
use std::env;
use std::panic;
use std::process;
mod helpers;
use helpers::{cmd, sanity, is_quite};
mod config;
use config::{extract_cas_config_dir_and_volume, get_kube_config_volume};

use std::io::{self, Write};
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

    let mut command = cmd();
    // let mut apply_help = show_apply_help(&matches);
    let mut apply_help_help = false;
    let show_spinner = is_quite(command);
    // TODO APPLY ARGS
    // let apply_filname: String = get_apply_filename(&matches).unwrap();
    



    let vol = sanity();
    let kubeconfig_vol = get_kube_config_volume();

    let original_args: Vec<String> = env::args().collect();
    let result = extract_cas_config_dir_and_volume(original_args);
    let cas_config_dir_env = result.0;
    let cas_config_dir_vol = result.1;

    let repo = match env::var("SCONECTL_REPO") {
        Ok(repo) => repo,
        Err(_err) => "registry.scontain.com/sconectl".to_string(),
    };
    let image = format!("{repo}/sconecli:latest");

    // pull image unless SCONECTL_NOPULL is set
    match env::var("SCONECTL_NOPULL") {
        Ok(_ignore) => println!("Warning: SCONECTL_NOPULL is set hence, not pulling CLI image"),
        Err(_err) => {
            let stop = if show_spinner {
                Some(Spinner::with_timer(
                    Spinners::Dots12,
                    format!("Pulling image '{image}'"),
                ))
            } else {
                None
            };
            let (code, _stdout, _stderr) = sh!("docker pull {image}");
            if let Some(mut sp) = stop {
                sp.stop_with_newline();
            }
            if code != 0 {
                eprintln!("\n{} 'docker pull {image}'! Do you have access rights? Please check and send email to info@scontain.com if you need access. (Error 24501-25270-6605)", "Failed to".red());
            }
        }
    }

    // let stop = if show_spinner {
    //     Some(Spinner::with_timer(
    //         Spinners::Dots12,
    //         format!("Executing command '{}'", args[command_index]),
    //     ))
    // } else {
    //     None
    // };
    //  docker run -d --platform linux/amd64 -e SCONE_NO_TIME_THREAD=1 --entrypoint=""
    // --rm -v /var/run/docker.sock:/var/run/docker.sock -v "$PWD":"/wd" -w "/wd"
    // -e "SCONECTL_CAS_CONFIG=" -e "SCONECTL_REPO=registry.scontain.com/sconectl"
    // -v "$HOME/.cas":"/root/.cas"
    // -v  $HOME/.kube/config:/root/.kube/config
    // -v "$HOME/.docker":"/root/.docker"
    // -v "$HOME/.scone":"/root/.scone"
    // registry.scontain.com/sconectl/sconecli:latest
    let mut docker_sconecli_d_cmd = format!(
        r#"docker run -d --platform linux/amd64 -e SCONE_NO_TIME_THREAD=1 --entrypoint="" --rm -e "SCONECTL_CAS_CONFIG={cas_config_dir_env}" -e "SCONECTL_REPO={repo}" {image} sleep 100000"#
    );
    let mut container_id = execute_sh(docker_sconecli_d_cmd);

    // lets remove /n at the end of the string
    container_id.pop();

    // docker exec $ID mkdir -p /root/.docker
    // docker cp $HOME/.docker/config.json $ID:/root/.docker/config.json
    // TODO add parameters to parse args
    // dir=$(dirname $(realpath service.yaml))
    // bdir=$(basename $dir)
    // docker cp $dir $ID:/
    // docker exec $ID cp -a /$bdir/. /wd
    // docker exec $ID apply -f service.yaml -vvvvv
    // docker cp $ID:/wd/target .

    execute_sh(format!(
        r#"docker cp ~/.docker {container_id}:/root/.docker"#
    ));
    execute_sh(format!(
        r#"docker exec {container_id} ls -la /root/.docker"#
    ));
    // let dir = execute_sh(format!(r#"dirname $(realpath {apply_filname})"#));
    // println!("{}", dir);
    // let bdir = execute_sh(format!(r#"basename {dir}"#));
    // println!("{}", bdir);

    // if let Some(mut sp) = stop {
    //     sp.stop_with_newline();
    // }
}

pub fn execute_sh(command: String) -> String {
    println!("{}", command);
    let (code, stdout, stderr) = sh!("{}", command);

    if code == 0 {
        // if verbose
        // println!("return code: {}", code);
        // eprintln!("stdout: {}", stdout);
        // eprintln!("stderr: {}", stderr);
        stdout
    } else {
        println!("return code: {}", code);
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", stderr);
        eprintln!("{} See messages above. Command {} returned error.\n  Error={:?} (Error 22597-24820-10449)", "Execution failed!".red(), command, code);
        process::exit(0x0101);
    }
}
