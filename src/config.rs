use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

pub fn get_kube_config_volume() -> String {
    let kubeconfig_path = match env::var("KUBECONFIG") {
        Ok(kubeconfig_path) => kubeconfig_path,
        // if KUBECONFIG is not set, let us try the default path
        Err(_err) => {
            let home = match env::var("HOME") {
                Ok(val) => val,
                Err(_e) => {
                    eprintln!(
                        "{}",
                        "environment variable HOME not defined. (Error 12874-23995-6201)".red()
                    );
                    process::exit(0x0101);
                }
            };
            let path = format!("{home}/.kube/config");
            if Path::new(&path).exists() {
                path
            } else {
                "".to_owned()
            }
        }
    };
    format!("-v {kubeconfig_path}:/root/.kube/config") // kubeconfig_path
}

pub fn get_cas_config_dir_env() -> String {
    match env::var("SCONECTL_CAS_CONFIG") {
        Ok(value) => value,
        Err(_err) => "".to_owned(),
    }
}

pub fn extract_cas_config_dir_and_volume(args: Vec<String>) -> (String, String, Vec<String>) {
    let mut new_args = args.to_vec();
    let cas_config_dir_args = match args.iter().position(|item| item == "--cas-config") {
        Some(index) => {
            match args.get(index + 1) {
                Some(value) => {
                    // do not pass --cas-config along to commands
                    new_args.remove(index);
                    new_args.remove(index);
                    value
                }
                None => {
                    eprintln!("No value provided for \"--cas-config\"");
                    process::exit(0x0101);
                }
            }
        }
        None => "",
    };

    let mut cas_config_dir = get_cas_config_dir_env();
    // --cas--config has precedence over env var
    if !cas_config_dir_args.is_empty() {
        cas_config_dir = String::from(cas_config_dir_args);
    }

    if cas_config_dir.is_empty() {
        (
            String::from(""),
            String::from("-v \"$HOME/.cas\":\"/root/.cas\""),
            new_args.to_vec(),
        )
    } else {
        // We only support absolute paths
        if !cas_config_dir.starts_with('/') {
            eprintln!("Only absolute paths are supported for CAS config (Error 20237-24960-17289)");
            process::exit(0x0101);
        }

        if !Path::new(cas_config_dir.as_str()).exists() {
            // create this path
            if let Err(e) = fs::create_dir(&cas_config_dir) {
                eprintln!("{}",&format!("Error creating local directory for --cas-config {cas_config_dir}: {e:?}! (Error 29466-27502-11632)").red());
                process::exit(0x0101);
            }
        }
        (
            cas_config_dir.to_owned(),
            format!("-v \"{cas_config_dir}\":\"/root/.cas\""),
            new_args.to_vec(),
        )
    }
}
