use clap::ArgMatches;
use colored::Colorize;

use clap::{Arg, Command};
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use which::which;

static  long_sconeclt_about : &str = "sconectl helps to transform cloud-native applications into cloud-confidential applications. It supports converting native services into confidential services and services meshes into confidential service meshes.

    sconectl is a CLI that runs on your development machine and executes scone commands in a local container: [scone](https://sconedocs.github.io/) is a platform to convert native applications into confidential applications. sconectl uses docker or podman to run the commands.

    Ensure all files you want to pass along are in the current working directory or subdirectories. This is needed since we pass the current working directory to the docker image that executes the command.

    If you want to use podman instead, please set the environment variable DOCKER_HOST to your podman API (printed by podman during startup). Currently, podman still has some open issues that need to be solved.

    sconectl runs on macOS and Linux, and if there is some demand, on Windows. Try out

    https://github.com/scontain/scone_mesh_tutorial

    to test your sconectl setup. In particular, it will test that all prerequisites are satisfied
    and gives some examples on how to use sconectl.
ENVIRONMENT:

  SCONECTL_REPO
           Set this to the OCI image repo that you are using. The default repo
           is registry.scontain.com/sconectl

  SCONECTL_NOPULL
           By default, sconectl pulls the CLI image sconecli:latest first. If this environment
           variable is defined, sconectl does not pull the image.

  SCONECTL_CAS_CONFIG
           CAS config JSON directory. Only absolute paths are supported. If the
           directory does not exist, a CAS config JSON will be created if
           scone cas attest command is used. If --cas-config option is set, the value
           from the command line argument will be used instead of SCONECTL_CAS_CONFIG.

  KUBECONFIG
           By default we use path $HOME/.kube/config for the Kubernetes config.
           If the $KUBECONFIG environment variable is set, then this file is used instead.

           **NOTE**: We assume that the certificates are embedded in the config file.
           You might therefore need to start minikube as follows:
                minikube start --embed-certs

           **NOTE**: We only support a single file in KUBECONFIG, i.e., no lists of config
           files are supported yet.

  DOCKER_HOST
           By default we use socket /var/run/docker.sock to talk to the Docker engine.
           One can overwrite this default with the help of this environment variable. For
           example, you might want to overwrite this in case you are using podman.

SUPPORT: If you need help, send an email to info@scontain.com with a description of the
         issue. Ideally, with a log that shows the problem.
";

/// do some sanity checking
/// - check that all commands exists
/// - check that all required directories exist
/// - check that docker socket exists
/// Note: `https://github.com/scontain/scone_mesh_tutorial/blob/main/check_prerequisites.sh` does some more sanity checking
///       Run the `check_prerequisites.sh` to check more dependencies

pub fn cmd() -> ArgMatches {
    let m = Command::new("sconectl")
        .author("info@scontain.com")
        .version(env!("CARGO_PKG_VERSION"))
        .before_long_help(long_sconeclt_about)
        .arg(
            Arg::new("cas_config")
                .long("cas_config")
                .help("CAS config JSON directory")
                .long_help(
                    "CAS config JSON directory. Only absolute paths are supported. If the
            directory does not exist, a CAS config JSON will be created if
            scone cas attest command is used.",
                ),
        )
        .arg(
            Arg::new("quite")
                .long("quite")
                .help("Disable spinner")
                .action(clap::ArgAction::SetFalse)
                .long_help(
                    "By default, sconectl shows a spinner. You can disable the spinner by setting
                    option --quiet.",
                ),
        )
        .subcommand(
            Command::new("apply")
                .disable_help_flag(true)
                .allow_external_subcommands(true)
                .about("Controls configuration features")
                .arg(Arg::new("help").long("help").action(clap::ArgAction::Count))
                .arg(Arg::new("filename").short('f').help("Path to manifest")),
        )
        .after_help(
            "Longer explanation to appear after the options when \
                 displaying the help information from --help or -h",
        )
        .get_matches();
    m
}

pub fn sanity() -> String {
    // do some sanity checking first
    if let Err(_e) = which("sh") {
        eprintln!(
            "{}",
            "Shell `sh` not installed. Please install! (Error 4497-4397-12312)".red()
        )
    }
    if let Err(_e) = which("docker") {
        eprintln!("{}", "Docker CLI (i.e. command `docker`) is not installed. Please install - see https://docs.docker.com/get-docker/ (Error 21214-27681-19217)".red())
    }
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(_e) => {
            eprintln!(
                "{}",
                "environment variable HOME not defined. Using ~ instead. (Error 25873-23261-18708)"
                    .red()
            );
            process::exit(0x0101);
        }
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
            eprintln!(
                "{}",
                &format!("Error creating local directory {path}: {e:?}! (Error 29613-7923-17838)")
            );
        }
    }

    // 172.17.0.0 is default docker network
    let mut docker0_ip = String::from("172.17.0.1");
    let mut docker0_if_exist = false;
    for iface in get_if_addrs::get_if_addrs().unwrap() {
        if iface.name == "docker0" {
            docker0_if_exist = true;
            docker0_ip = iface.ip().to_string();
            break;
        }
    }

    let mut vol = match env::var("DOCKER_HOST") {
        Ok(val) => {
            if val.starts_with("unix://") {
                let vol = val.strip_prefix("unix://").unwrap_or(&val).to_string();
                format!(r#"-e DOCKER_HOST="{val}" -v "{vol}":"{vol}""#)
            } else if val.starts_with("tcp://") || val.starts_with(&docker0_ip) {
                if !docker0_if_exist {
                    eprintln!("Interface 'docker0' was not found but docker socket with TCP schema was detected.");
                }
                eprintln!("Docker socket with TCP schema was detected. Will use DOCKER_HOST={val} to access docker socket inside container." );
                format!(r#"-e DOCKER_HOST="{val}""#)
            } else {
                eprintln!("Docker socket: {val} with unknown schema was detected.");
                r#"-e DOCKER_HOST=/var/run/docker.sock -v /var/run/docker.sock:/var/run/docker.sock"#.to_string()
            }
        }
        Err(_e) => "-v /var/run/docker.sock:/var/run/docker.sock".to_string(),
    };

    match env::var("HOST_HOME") {
        Ok(val) => {
            vol.push_str(&format!(r#" "-e$HOME={val}""#));
        }
        Err(_e) => {
            vol.push_str(&format!(r#""#));
        }
    };

    match env::var("HOST_WD") {
        Ok(val) => {
            vol.push_str(&format!(r#" -v "{val}":"{val}" -w "{val}"#));
        }
        Err(_e) => {
            vol.push_str(&format!(r#" -v "$PWD":"/wd" -w "/wd""#));
        }
    };

    vol
}
