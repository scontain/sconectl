use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::process::Command;
use which::which;

/// prints a help message. If `msg` is not empty, prints also the message in red.

fn help(msg: &str) -> ! {
    eprintln!(
        r#"sconectl [COMMAND] [OPTIONS]
sconectl helps to transform cloud-native applications into cloud-confidential applications. It
supports to transform native services into confidential services and services meshes into 
confidential service meshes. 

sconectl itself is a CLI that runs on your development machine and executes `scone` commands in a local
container: [`scone`](https://sconedocs.github.io/) is a platform to convert native applications 
into confidential applications. For now, sconectl uses docker to run the commands. 



COMMAND:
  apply   apply manifest. Execute `sconectl apply --help` for more info.


OPTIONS:
    
  -h, --help
          Print help information. Other OPTIONS depend on the type of MANIFEST. 
          You need to specify -m <MANIFEST> for help to print more options.     
"#
    );
    if msg != "" {
        eprintln!("ERROR: {}", msg.red());
    }

    process::exit(0x0100);
}

/// do some sanity checking
/// - check that all commands exists
/// - check that all required directories exist
/// - check that docker socket exists

fn sanity() {
    // do some sanity checking first
    if let Err(_e) = which("sh") {
        help("Shell `sh` not installed. Please install!")
    }
    if let Err(_e) = which("docker") {
        help("Docker (i.e. `docker`) is not installed. Please install - see https://docs.docker.com/get-docker/")
    }
    if !Path::new("/var/run/docker.sock").exists() {
        help("Docker engine does not seem to be installed since '/var/run/docker.sock' does not exit). Please install the docker engine.")
    }
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(_e) => help("environment variable HOME not defined."),
    };
    let path = format!("{home}/.docker");
    if !Path::new(&path).exists() {
        help(&format!("$HOME/.docker (={path}) does not exist! Maybe try `docker` command on command line first."));
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
}

/// sconectl helps to transform cloud-native applications into cloud-confidential applications.
/// It supports to transform native services into confidential services and services meshes
/// into confidential service meshes.

fn main() {
    sanity();
    let args: Vec<String> = env::args().collect();
    let mut cmd = Command::new("sh");
    let mut s = r#"docker run -it --rm -v "/var/run/docker.sock:/var/run/docker.sock" -v "$HOME/.docker:/root/.docker" -v "$HOME/.cas:/root/.cas" -v "$HOME/.scone:/root/.scone" -v "$PWD:/root" -w "/root" registry.scontain.com:5050/cicd/sconecli:latest"#.to_string();
    for i in 1..args.len() {
        if args[i] == "--help" && i == 1 {
            help("");
        }
        s.push_str(&format!(r#" "{}""#, args[i]));
    }
    if args.len() <= 1 {
        help("You need to specify a COMMAND.")
    }
    cmd.args(["-c", &s])
        .status()
        .expect("failed to execute {s}.");
}
