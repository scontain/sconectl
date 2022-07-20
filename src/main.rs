use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::process::Command;
use which::which;
use serde_json;
use shells::*;
use std::panic;

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

If you want to use podman instead, please set  environment variable DOCKER_HOST to your podman API 
(printed by podman during startup). Currently, podman has still some open issues that need to be solved.

sconectl runs on MacOs and Linux and if there is some demand, on Windows. Try out

   https://github.com/scontain/scone_mesh_tutorial 

to test your sconectl setup. In particular, it will test that all prerequisites are satisfied
and gives some examples on how to use sconectl.

COMMAND:
  apply   apply manifest. Execute `sconectl apply --help` for more info.


OPTIONS:
    
  --help
          Print help information. Other OPTIONS depend on the type of MANIFEST. 
          You need to specify -m <MANIFEST> for help to print more options.     

ENVIRONMENT:

  SCONECTL_REPO
           Set this to the OCI image repo that you are using. The default repo
           is 'registry.scontain.com:5050'


  SCONECTL_NOPULL
           By default, sconectl pulls the CLI image 'cicd/sconecli:latest' first. If this environment 
           variable is defined, sconectl does not pull the image. 

VERSION: sconectl {VERSION}"#
    );
    if msg != "" {
        eprintln!("ERROR: {}", msg.red());
        process::exit(0x0101);
    }
    process::exit(0);
}

/// do some sanity checking
/// - check that all commands exists
/// - check that all required directories exist
/// - check that docker socket exists
/// Note: https://github.com/scontain/scone_mesh_tutorial/blob/main/check_prerequisites.sh does some more sanity checking
///       Run the check_prerequisites.sh to check more dependencies

fn sanity() -> String {
    // do some sanity checking first
    if let Err(_e) = which("sh") {
        help("Shell `sh` not installed. Please install! (Error 4497-4397-12312)")
    }
    if let Err(_e) = which("docker") {
        help("Docker CLI (i.e. command `docker`) is not installed. Please install - see https://docs.docker.com/get-docker/ (Error 21214-27681-19217)")
    }
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(_e) => help("environment variable HOME not defined. (Error 25873-23261-18708)"),
    };
    let path = format!("{home}/.docker");
    if !Path::new(&path).exists() {
        eprintln!("Warning: $HOME/.docker (={path}) does not exist! Maybe try `docker` command on command line first or create directory manually in case you are using podman. (Warning 22414-7450-14297)");
    } else {
        let path = format!("{home}/.docker/config.json");
        match fs::read_to_string(path) {
            Ok(config_content) => {
                match serde_json::from_str::<serde_json::value::Value>(&config_content) {
                    Err(_e) => { eprintln!("Warning: In case you are using docker, please ensure that field 'credsStore' in 'config.json' is empty. (Warning 8870-21168-30218)"); serde_json::from_str("{}").expect("Docker config file seems to be garbled (Error 15572-27738-16119)") },
                    Ok(val)  => {
                        match val["credsStore"].as_str() {
                            None => {}, // ok
                            Some(value) => { if value != "" { eprintln!("{}", r#"ERROR: command execution will most likely fail. Please set field 'credsStore'" in file '~/.docker/config.json' to "" (Error 8352-13006-22294)"#.red()) } },
                        }
                    },
                }
            },
            Err(_err) => eprintln!("Warning: In case you are using docker, please ensure that field 'credsStore' in 'config.json' is empty. (Warning 22852-10923-23603)"),
        }
    }
    let path = format!("{home}/.cas");
    if !Path::new(&path).exists() {
        // create this path
        if let Err(e) = fs::create_dir(&path) {
            help(&format!("Error creating local directory {path}: {:?}! (Error 32113-24496-13076)", e));
        }
    }
    let path = format!("{home}/.scone");
    if !Path::new(&path).exists() {
        // create this path
        if let Err(e) = fs::create_dir(&path) {
            help(&format!("Error creating local directory {path}: {:?}! (Error 29613-7923-17838)", e));
        }
    }
    let vol = match env::var("DOCKER_HOST") {
        Ok(val) => { let vol = val.strip_prefix("unix://").unwrap_or(&val).to_string(); format!(r#"-e DOCKER_HOST="{val}" -v "{vol}":"{vol}""#) },
        Err(_e) => format!("-v /var/run/docker.sock:/var/run/docker.sock"),
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
                Err(_e) => help("environment variable HOME not defined. (Error 12874-23995-6201)"),
            };
            let path = format!("{home}/.kube/config");
            if Path::new(&path).exists() {
                path
            } else {
                "".to_owned()
            }
        },
    };
    return format!("-v {kubeconfig_path}:/root/.kube/config") // kubeconfig_path
}


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
            eprintln!("A fatal error occurred: {}. (Error 30339-23400-21867)", err_msg.trim_start_matches("panicked at ").red());
        }
        process::exit(1);
    }));
    
    let vol = sanity();
    let kubeconfig_vol = get_kube_config_volume();
    let args: Vec<String> = env::args().collect();

    let repo = match env::var("SCONECTL_REPO") {
        Ok(repo) =>  repo,
        Err(_err) =>  format!("registry.scontain.com:5050")
    };
    let image = format!("{repo}/cicd/sconecli:latest");

    // pull image unless SCONECTL_NOPULL is set
    match env::var("SCONECTL_NOPULL") {
        Ok(_ignore) =>  println!("Warning: SCONECTL_NOPULL is set hence, not pulling CLI image"),
        Err(_err) => {    
            let (code, _stdout, _stderr) = sh!("docker pull {image}");
            if code != 0 {
                eprintln!(r#"{} "docker pull {image}"! Do you have access rights? Please check and send email to info@scontain.com if you need access. (Error 24501-25270-6605)"#, "Failed to".red());
            }
        },
    }
    let mut s = format!(r#"docker run -t --rm {vol} {kubeconfig_vol} -e "SCONECTL_REPO={repo}" -v "$HOME/.docker":"/root/.docker" -v "$HOME/.cas":"/root/.cas" -v "$HOME/.scone":"/root/.scone" -v "$PWD":"/root" -w "/root" {image}"#);
    for i in 1..args.len() {
        if args[i] == "--help" && i == 1 {
            help("");
        }
        s.push_str(&format!(r#" "{}""#, args[i]));
    }
    if args.len() <= 1 {
        help("You need to specify a COMMAND. (Error 696-7363-5766)")
    }
    let mut cmd = Command::new("sh");
    let status =    cmd.args(["-c", &s])
        .status()
        .expect("failed to execute '{s}'. (Error 8914-6233-13917)");
    if !status.success() {
        eprintln!("{} See messages above. Command {} returned error.\n  Error={:?} (Error 22597-24820-10449)", "Execution failed!".red(), args[1].blue(), status);
        process::exit(0x0101);
    }
}

