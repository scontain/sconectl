# sconectl

[`sconectl`](https://sconedocs.github.io/scone_mesh_tutorial/) helps to transform cloud-native applications into cloud-confidential applications. It
supports transforming native services into confidential services and services meshes into confidential service meshes.

`sconectl` is a program that runs on your development machine and executes `scone` commands in containers: [`scone`](https://sconedocs.github.io/) is a platform to convert native applications into confidential applications.

We implemented this as as a Rust crate. Alternatively, you can define an `alias` for your shell (see below).

## Relation to `sconify-image`

`sconectl` complements [`sconify_image`](https://sconedocs.github.io/ee_sconify_image/). Actually, `sconectl` includes a wrapper for `sconify_image`: we can declare the arguments of `sconify_image` with the help of one or more **yaml** files.

### Building confidential applications:

- `sconify_image` helps to build confidential services that are deployed with the help of a single container image.
- `sconectl` focuses on generating images for cloud-native services that are connected in a service mesh. The generation is declared with the help of *service files* (see below).

- `sconectl` can  connect the services within an application with the help of a *mesh file* (see below)

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

## `DOCKER_HOST`

`sconectl` will use `DOCKER_HOST` as the socket. If not set, it will use the default docker socket for now, i.e., `/var/run/docker.sock`.
You can connect to the docker 

## Publish a new version

To publish a new `sconectl` version, ensure that all your changes are committed and pushed. Then executed:

```bash
cargo publish
```

## CLI Reference

```text
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

SUPPORT: If you need help, send an email to info@scontain.com with a description of the
         issue. Ideally, with a log that shows the problem.

VERSION: sconectl 0.2.17
```
