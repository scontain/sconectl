use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::process::Command;
use which::which;
use serde_json;
use shells::*;


/// prints a help message. If `msg` is not empty, prints also the message in red.

fn help(msg: &str) -> ! {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    eprintln!(
        r#"sconectl [COMMAND] [OPTIONS]
sconectl helps to transform cloud-native applications into cloud-confidential applications. It
supports to transform native services into confidential services and services meshes into 
confidential service meshes. 

sconectl itself is a CLI that runs on your development machine and executes `scone` commands in a local
container: [`scone`](https://sconedocs.github.io/) is a platform to convert native applications 
into confidential applications. For now, sconectl uses docker to run the commands. 

By default, sconectl uses the docker engine. If you want to use podman instead, please set 
environment variable DOCKER_HOST to your podman API (printed by podman during startup).

COMMAND:
  apply   apply manifest. Execute `sconectl apply --help` for more info.


OPTIONS:
    
  --help
          Print help information. Other OPTIONS depend on the type of MANIFEST. 
          You need to specify -m <MANIFEST> for help to print more options.     


VERSION: sconectl {VERSION}"#
    );
    if msg != "" {
        eprintln!("ERROR: {}", msg.red());
    }

    process::exit(0x0101);
}

/// do some sanity checking
/// - check that all commands exists
/// - check that all required directories exist
/// - check that docker socket exists

fn sanity() -> String {
    // do some sanity checking first
    if let Err(_e) = which("sh") {
        help("Shell `sh` not installed. Please install!")
    }
    if let Err(_e) = which("docker") {
        help("Docker CLI (i.e. command `docker`) is not installed. Please install - see https://docs.docker.com/get-docker/")
    }
    
    let socket_path = if cfg!(target_os = "windows") {
        "//var/run/docker.sock".to_string()
    } else if cfg!(target_os = "unix") {
        "/var/run/docker.sock".to_string()
    } else {
        help("Docker engine does not seem to be installed since '/var/run/docker.sock'/'//var/run/docker.sock' does not exit). Please install the docker engine.")
    };
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(_e) => help("environment variable HOME not defined."),
    };
    let path = format!("{home}/.docker");
    if !Path::new(&path).exists() {
        eprintln!("Warning: $HOME/.docker (={path}) does not exist! Maybe try `docker` command on command line first or create directory manually in case you are using podman.");
    } else {
        let path = format!("{home}/.docker/config.json");
        match fs::read_to_string(path) {
            Ok(config_content) => {
                match serde_json::from_str::<serde_json::value::Value>(&config_content) {
                    Err(_e) => { eprintln!("Warning: In case you are using docker, please ensure that field 'credsStore' in 'config.json' is empty."); serde_json::from_str("{}").expect("No Error!") },
                    Ok(val)  => {
                        match val["credsStore"].as_str() {
                            None => {}, // ok
                            Some(value) => { if value != "" { eprintln!("{}", r#"ERROR: command execution will most likely fail. Please set field 'credsStore'" in file '~/.docker/config.json' to "")"#.red()) } },
                        }
                    },
                }
            },
            Err(_err) => eprintln!("Warning: In case you are using docker, please ensure that field 'credsStore' in 'config.json' is empty."),
        }
    }
    let path = format!("{home}/.cas");
    if !Path::new(&path).exists() {
        // create this path
        if let Err(e) = fs::create_dir(&path) {
            help(&format!("Error creating local directory {path}: {:?}!", e));
        }
    }
    let path = format!("{home}/.scone");
    if !Path::new(&path).exists() {
        // create this path
        if let Err(e) = fs::create_dir(&path) {
            help(&format!("Error creating local directory {path}: {:?}!", e));
        }
    }
    let vol = match env::var("DOCKER_HOST") {
        Ok(val) => { let vol = val.strip_prefix("unix://").unwrap_or(&val).to_string(); format!(r#"-e DOCKER_HOST="{val}" -v "{vol}":"{vol}""#) },
        Err(_e) => format!("-v {socket_path}:/var/run/docker.sock"),
    };
    vol
}


fn get_kube_config_volume() -> String {
    let kubeconfig_path = match env::var("KUBECONFIG") {
        Ok(kubeconfig_path) =>  kubeconfig_path,
        // if KUBECONFIG is not set, let us try the default path
        Err(_err) => {
            let home = match env::var("HOME") {
                Ok(val) => val,
                Err(_e) => help("environment variable HOME not defined."),
            };
            format!("$HOME/.kube/config")
        },
    };
    return format!(r#"-v "{kubeconfig_path}:/root/.kube/config""#) // kubeconfig_path
}


/// sconectl helps to transform cloud-native applications into cloud-confidential applications.
/// It supports to transform native services into confidential services and services meshes
/// into confidential service meshes.

/// --userns=keep-id works only in rootless - fails when running as root

fn main() {
    let image = "registry.scontain.com:5050/cicd/sconecli:latest";
    let vol = sanity();
    let kubeconfig_vol = get_kube_config_volume();
    let args: Vec<String> = env::args().collect();

    // always pull CLI
    let (code, _stdout, _stderr) = sh!("docker pull {image}");
    if code != 0 {
        eprintln!(r#"{} "docker pull {image}"! Do you have access rights? Please check and send email to info@scontain.com if you need access."#, "Failed to".red());
    }

    let mut s = format!(r#"docker run -t --rm {vol} {kubeconfig_vol} -v "$HOME/.docker":"/root/.docker" -v "$HOME/.cas":"/root/.cas" -v "$HOME/.scone":"/root/.scone" -v "$PWD":"/root" -w "/root" --pull always {image}"#);
    println!("cmd={s}");
    for i in 1..args.len() {
        if args[i] == "--help" && i == 1 {
            help("");
        }
        s.push_str(&format!(r#" "{}""#, args[i]));
    }
    if args.len() <= 1 {
        help("You need to specify a COMMAND.")
    }
    let mut cmd = Command::new("bash");
    let status =    cmd.args(["-c", &s])
        .status()
        .expect("failed to execute {s}.");
    if !status.success() {
        eprintln!("{} See messages above. Command {} returned result={:?}", "Execution failed!".red(), args[1].blue(), status);
        process::exit(0x0101);
    }
}

