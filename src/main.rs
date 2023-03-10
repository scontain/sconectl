use colored::Colorize;
use shells::sh;
use spinners::{Spinner, Spinners};
use std::env;
use std::panic;
use std::process;
use std::process::Command;
use std::process::Output;

mod helpers;
use helpers::{help, sanity};
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
    
    // for (i, arg) in args.iter().enumerate().skip(1) {
    //     if arg == "--help" && i == 1 {
    //         help("");
    //     }
    //     if arg == "--quiet" && i == 1 {
    //         show_spinner = false;
    //         command_index += 1;
    //     } else {
    //         docker_sconecli_d_cmd.push_str(&format!(r#" "{arg}""#));
    //     }
    // }
    // if args.len() <= command_index {
    //     help("You need to specify a COMMAND. (Error 696-7363-5766)")
    // }

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
    // --rm -v /var/run/docker.sock:/var/run/docker.sock -v "$PWD":"/wd" -w "/wd" -v "$HOME/.cas":"/root/.cas" 
    // -v /home/vasyl/.kube/config:/root/.kube/config -e "SCONECTL_CAS_CONFIG=" -e "SCONECTL_REPO=registry.scontain.com/sconectl" 
    // -v "$HOME/.docker":"/root/.docker" -v "$HOME/.scone":"/root/.scone" registry.scontain.com/sconectl/sconecli:latest
    let mut docker_sconecli_d_cmd = format!(
        r#"docker run -d --platform linux/amd64 -e SCONE_NO_TIME_THREAD=1 --entrypoint="" --rm -e "SCONECTL_CAS_CONFIG={cas_config_dir_env}" -e "SCONECTL_REPO={repo}" {image} sleep 100000"#
    );
    let mut output = execute_sh(&mut docker_sconecli_d_cmd);
 
    // lets remove /n at the end
    output.stdout.pop();

    let s = match String::from_utf8(output.stdout) {
        Ok(path) => Ok(path),
        Err(e) => Err(format!("Invalid UTF-8 sequence: {}", e)),
    };
    
    let container_id = s.unwrap();

    // docker exec $ID mkdir -p /root/.docker
    // docker cp $HOME/.docker/config.json $ID:/root/.docker/config.json
    // TODO add parameters to parse args
    // dir=$(dirname $(realpath service.yaml))
    // bdir=$(basename $dir)
    // docker cp $dir $ID:/
    // docker exec $ID cp -a /$bdir/. /wd
    // docker exec $ID apply -f service.yaml -vvvvv
    // docker cp $ID:/wd/target .

    execute_sh(&mut format!(r#"docker cp ~/.docker {container_id}:/root/.docker"#));
    execute_sh(&mut format!(r#"docker exec {container_id} ls -la /root/.docker"#));
    output = execute_sh(&mut format!(r#"$(dirname $(realpath service.yaml))"#));
    let s = match String::from_utf8(output.stdout) {
        Ok(path) => Ok(path),
        Err(e) => Err(format!("Invalid UTF-8 sequence: {}", e)),
    };
    println!("{}",s.unwrap());

    // if let Some(mut sp) = stop {
    //     sp.stop_with_newline();
    // }

}


pub fn execute_sh(command: &mut String) -> Output {
    println!("{}",command);
    let output = Command::new("sh")
        .args(["-c", &command])
        .output()
        .expect("failed to execute '{command}'. (Error 8914-6233-13917)");
    
    if output.status.success() {
        println!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        output
    } else {
        // if verbose
            // println!("status: {}", output.status);
            // io::stdout().write_all(&output.stdout).unwrap();
            // io::stderr().write_all(&output.stderr).unwrap(); 
        eprintln!("{} See messages above. Command {} returned error.\n  Error={:?} (Error 22597-24820-10449)", "Execution failed!".red(), command, output.status);
        process::exit(0x0101);
    }
  }