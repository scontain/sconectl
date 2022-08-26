use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::process::Command;
use which::which;

use shells::sh;
use spinners::{Spinner, Spinners};
use std::panic;

/// prints a help message. If `msg` is not empty, prints also the message in red.

fn help(msg: &str) -> ! {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    eprintln!(
        "{} {VERSION}",
        termimad::inline(
            r#"# `sconectl` [COMMAND] [OPTIONS]

`sconectl` helps to transform cloud-native applications into cloud-confidential applications. It supports converting native services into confidential services and services meshes into confidential service meshes. 

`sconectl` is a CLI that runs on your development machine and executes `scone` commands in a local container: [`scone`](https://sconedocs.github.io/) is a platform to convert native applications into confidential applications. `sconectl` uses docker or podman to run the commands. 

Ensure all files you want to pass along are in the current working directory or subdirectories. This is needed since we pass the current working directory to the docker image that executes the command.

If you want to use podman instead, please set the environment variable DOCKER_HOST to your podman API (printed by podman during startup). Currently, podman still has some open issues that need to be solved.

`sconectl` runs on macOS and Linux, and if there is some demand, on Windows. Try out

   https://github.com/scontain/scone_mesh_tutorial 

to test your `sconectl` setup. In particular, it will test that all prerequisites are satisfied
and gives some examples on how to use `sconectl`.

COMMAND:
  `apply`   apply manifest. Execute `sconectl apply --help` for more info.


OPTIONS:
  --cas-config
          CAS config JSON directory. Only absolute paths are supported. If the
          directory does not exist, a CAS config JSON will be created if
          `scone cas attest` command is used.
  --help
          Print help information. Other OPTIONS depend on the type of MANIFEST. 
          You need to specify -m <MANIFEST> to print more specific help messages.     

ENVIRONMENT:

  SCONECTL_REPO
           Set this to the OCI image repo that you are using. The default repo
           is 'registry.scontain.com:5050/sconectl'


  SCONECTL_NOPULL
           By default, `sconectl` pulls the CLI image 'sconecli:latest' first. If this environment 
           variable is defined, `sconectl` does not pull the image. 

  SCONECTL_CAS_CONFIG
           CAS config JSON directory. Only absolute paths are supported. If the
           directory does not exist, a CAS config JSON will be created if
           `scone cas attest` command is used. If `--cas-config` option is set, the value
           from the command line argument will be used instead of `SCONECTL_CAS_CONFIG`.

VERSION: `sconectl`"#
        )
    );
    if !msg.is_empty() {
        eprintln!("ERROR: {}", msg.red());
        process::exit(0x0101);
    }
    process::exit(0);
}

/// do some sanity checking
/// - check that all commands exists
/// - check that all required directories exist
/// - check that docker socket exists
/// Note: `https://github.com/scontain/scone_mesh_tutorial/blob/main/check_prerequisites.sh` does some more sanity checking
///       Run the `check_prerequisites.sh` to check more dependencies

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
    if Path::new(&path).exists() {
        let path = format!("{home}/.docker/config.json");
        match fs::read_to_string(path) {
            Ok(config_content) => {
                match serde_json::from_str::<serde_json::value::Value>(&config_content) {
                    Err(_e) => { eprintln!("Warning: In case you are using docker, please ensure that field 'credsStore' in 'config.json' is empty. (Warning 8870-21168-30218)"); serde_json::from_str("{}").expect("Docker config file seems to be garbled (Error 15572-27738-16119)") },
                    Ok(val)  => {
                        match val["credsStore"].as_str() {
                            None => {}, // ok
                            Some(value) => { if !value.is_empty() { eprintln!("{}", r#"ERROR: command execution will most likely fail. Please set field 'credsStore'" in file '~/.docker/config.json' to "" (Error 8352-13006-22294)"#.red()) } },
                        }
                    },
                }
            },
            Err(_err) => eprintln!("Warning: In case you are using docker, please ensure that field 'credsStore' in 'config.json' is empty. (Warning 22852-10923-23603)"),
        }
    } else {
        eprintln!("Warning: $HOME/.docker (={path}) does not exist! Maybe try `docker` command on command line first or create directory manually in case you are using podman. (Warning 22414-7450-14297)");
    }

    let path = format!("{home}/.scone");
    if !Path::new(&path).exists() {
        // create this path
        if let Err(e) = fs::create_dir(&path) {
            help(&format!(
                "Error creating local directory {path}: {:?}! (Error 29613-7923-17838)",
                e
            ));
        }
    }
    let volume = match env::var("DOCKER_HOST") {
        Ok(val) => {
            let volume = val.strip_prefix("unix://").unwrap_or(&val).to_string();
            format!(r#"-e DOCKER_HOST="{val}" -v "{volume}":"{volume}""#)
        }
        Err(_e) => "-v /var/run/docker.sock:/var/run/docker.sock".to_string(),
    };
    volume
}

fn get_kube_config_volume() -> String {
    let kubeconfig_path = match env::var("KUBECONFIG") {
        Ok(kubeconfig_path) => kubeconfig_path,
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
        }
    };
    return format!("-v {kubeconfig_path}:/root/.kube/config"); // kubeconfig_path
}

fn get_cas_config_dir_env() -> String {
    match env::var("SCONECTL_CAS_CONFIG") {
        Ok(value) => value,
        Err(_err) => "".to_owned(),
    }
}

fn extract_cas_config_dir_and_volume(args: Vec<String>) -> (String, String, Vec<String>){
    let mut new_args = args.to_vec();
    let cas_config_dir_args = match args.iter().position(|item| item == "--cas-config") {
        Some(index) => (
            match args.get(index + 1) {
                Some(value) => {
                    // do not pass --cas-config along to commands
                    new_args.remove(index);
                    new_args.remove(index);
                    value
                },
                None => {
                    eprintln!("No value provided for \"--cas-config\"");
                    process::exit(0x0101);
                },
            }
        ),
        None => "",
    };

    let mut cas_config_dir = get_cas_config_dir_env();
    // --cas--config has precedence over env var
    if !cas_config_dir_args.is_empty() {
        cas_config_dir = String::from(cas_config_dir_args);
    }

    if cas_config_dir.is_empty() {
        (String::from(""), String::from("-v \"$HOME/.cas\":\"/root/.cas\""), new_args.to_vec())
    } else {
        // We only support absolute paths
        if !cas_config_dir.starts_with("/") {
            eprintln!("Only absolute paths are supported for CAS config (Error 20237-24960-17289)");
            process::exit(0x0101);
        }

        if !Path::new(cas_config_dir.as_str()).exists() {
            // create this path
            if let Err(e) = fs::create_dir(cas_config_dir.to_owned()) {
                help(&format!(
                    "Error creating local directory for --cas-config {cas_config_dir}: {:?}! (Error 29466-27502-11632)",
                    e
                ));
            }
        }
        (cas_config_dir.to_owned(), format!("-v \"{cas_config_dir}\":\"/root/.cas\""), new_args.to_vec())
    }
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

    let repo = match env::var("SCONECTL_REPO") {
        Ok(repo) => repo,
        Err(_err) => "registry.scontain.com:5050/sconectl".to_string(),
    };
    let image = format!("{repo}/sconecli:latest");

    let mut s = format!(
        r#"docker run --entrypoint="" -t --rm {vol} {cas_config_dir_vol} {kubeconfig_vol} -e "SCONECTL_CAS_CONFIG={cas_config_dir_env}" -e "SCONECTL_REPO={repo}" -v "$HOME/.docker":"/root/.docker" -v "$HOME/.scone":"/root/.scone" -v "$PWD":"/wd" -w "/wd" {image}"#
    );
    for (i, arg) in args.iter().enumerate().skip(1) {
        if arg == "--help" && i == 1 {
            help("");
        }
        s.push_str(&format!(r#" "{}""#, arg));
    }
    if args.len() <= 1 {
        help("You need to specify a COMMAND. (Error 696-7363-5766)")
    }
    // pull image unless SCONECTL_NOPULL is set
    match env::var("SCONECTL_NOPULL") {
        Ok(_ignore) => println!("Warning: SCONECTL_NOPULL is set hence, not pulling CLI image"),
        Err(_err) => {
            let mut sp = Spinner::with_timer(Spinners::Dots12, format!("Pulling image '{image}'"));
            let (code, _stdout, _stderr) = sh!("docker pull {image}");
            sp.stop_with_newline();
            if code != 0 {
                eprintln!("\n{} 'docker pull {image}'! Do you have access rights? Please check and send email to info@scontain.com if you need access. (Error 24501-25270-6605)", "Failed to".red());
            }
        }
    }
    let mut sp = Spinner::with_timer(Spinners::Dots12, format!("Executing command '{}'", args[1]));
    let mut cmd = Command::new("sh");
    let status = cmd
        .args(["-c", &s])
        .status()
        .expect("failed to execute '{s}'. (Error 8914-6233-13917)");

    sp.stop_with_newline();
    if !status.success() {
        eprintln!("{} See messages above. Command {} returned error.\n  Error={:?} (Error 22597-24820-10449)", "Execution failed!".red(), args[1].blue(), status);
        process::exit(0x0101);
    }
}
