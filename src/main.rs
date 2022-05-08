use std::env;
use std::process;
use std::process::Command;
use colored::*;

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
"#);
    if msg != "" {
        eprintln!("ERROR: {}", msg.red());
    }

    process::exit(0x0100);
}


/// sconectl helps to transform cloud-native applications into cloud-confidential applications. 
/// It supports to transform native services into confidential services and services meshes 
/// into confidential service meshes. 

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmd =     Command::new("sh");
    let mut s = r#"docker run -it --rm -v "/var/run/docker.sock:/var/run/docker.sock" -v "$HOME/.docker:/root/.docker" -v "$HOME/.cas:/root/.cas" -v "$HOME/.scone:/root/.scone" -v "$PWD:/root" -w "/root" registry.scontain.com:5050/cicd/sconecli:latest"#.to_string();
    for i in 1..args.len() {
        if args[i] == "--help" && i == 1 {
            help("");
        }
        s.push_str(&format!(r#" "{}""#, args[i]));
    }
    if args.len() <= 1 { help("You need to specify a COMMAND.")}
    println!("Execute {s}");
    cmd.args(["-c", &s]).status().expect("failed to execute process");
}
