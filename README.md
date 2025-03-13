# `sconectl`

## What is `sconectl`

`sconectl` is a tool to transform cloud-native applications into cloud-confidential applications. 

`sconectl` can 

- transform native services into confidential services,  and 
- services meshes into confidential service meshes.

## Where to execute `sconectl`

`sconectl` is a program that runs on your development machine and executes `scone` CLI commands in containers. For more details, see [`scone`](https://sconedocs.github.io/). 

Existing (native) executable can be transformed into confidential executable with the help of CLI `scone-signer` command we can convert native executable into a confidential applications.

## What platforms / CPUs are supported by `sconectl`?

- `sconectl` should run on any platform that supports Rust. It executes the `scone` CLI in containers.  We implemented `sconectl` in Rust. Alternatively, you can define an `alias` for your shell (see below).

- While `scone` have been built for Intel SGX, the newest version also supports Intel TDX and AMD SEV-SNP. The 


**NOTE:** The `scone` CLI uses modern x86-64 CPU instructions. Running on ARM CPUs, not all x86-64  instructions are emulated. Hence, `scone` CLI command will fail if essential instructions like  (e.g., `rdrand`) are not available. 


## Examples

To build the service OCI container image, you might execute on your development machine:

```bash
sconectl apply -f service.yml
```

where `service.yml` describes the confidential service.

To build and upload the security policy for the application using:

```bash
sconectl apply -f mesh.yml
```

## Setting up `sconectl`

First, ensure that you have `Rust` installed on your system. If execution of

```bash
rustc --version
```

fails, you need to install `Rust`. You can use [`rustup`](https://www.rust-lang.org/tools/install) to do so.

To install `sconectl` just type.

```bash
cargo install sconectl
```

`sconectl` requires access to container images. For now, you would need to register an account at our [gitlab](https://gitlab.scontain.com/).


## CLI Reference

```
sconectl [COMMAND] [OPTIONS]

sconectl helps to transform cloud-native applications into cloud-confidential applications. It supports converting native services into confidential services and services meshes into confidential service meshes. 

sconectl is a CLI that runs on your development machine and executes scone commands in a local container: [scone](https://sconedocs.github.io/) is a platform to convert native applications into confidential applications. sconectl uses docker or podman to run the commands. 

Ensure all files you want to pass along are in the current working directory or subdirectories. This is needed since we pass the current working directory to the docker image that executes the command.

If you want to use podman instead, please set the environment variable DOCKER_HOST to your podman API (printed by podman during startup). Currently, podman still has some open issues that need to be solved.

sconectl runs on macOS and Linux, and if there is some demand, on Windows. Try out

   https://github.com/scontain/scone_mesh_tutorial 

to test your sconectl setup. In particular, it will test that all prerequisites are satisfied
and gives some examples on how to use sconectl.

COMMAND:
  apply   apply manifest. Execute sconectl apply --help for more info.


OPTIONS:
  --cas-config
          CAS config JSON directory. Only absolute paths are supported. If the
          directory does not exist, a CAS config JSON will be created if
          scone cas attest command is used.
  --help
          Print help information. Other OPTIONS depend on the type of MANIFEST. 
          You need to specify -m <MANIFEST> to print more specific help messages.     

  --quiet
          By default, sconectl shows a spinner. You can disable the spinner by setting
          option --quiet. 

ENVIRONMENT:

  SCONECTL_REPO
           Set this to the OCI image repo that you are using. The default repo
           is registry.scontain.com/sconectl


  SCONECTL_NOPULL
           By default, sconectl pulls the CLI image sconecli:$VERSION first. If this environment 
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

  VERSION
           Set the version of the sconecli image. Default is "latest"

  SCONECTL_VERSION
           In case you already use environment variable VERSION, you can use 
           SCONECTL_VERSION instead. Default is "latest" and it has priority over VERSION.


sconectl version 5.9.0
   If you need help, send an email to info@scontain.com with a description of the issue. 
   Ideally, add a log that has sufficient information to reproduce the issue.

```
