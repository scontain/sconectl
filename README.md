# sconectl

`sconectl` helps to transform cloud-native applications into cloud-confidential applications. It
supports to transform native services into confidential services and services meshes into confidential service meshes.

`sconectl` is a program that runs on your development machine and executes `scone` commands in containers: [`scone`](https://sconedocs.github.io/) is a platform to convert native applications into confidential applications.

We implemented this as as a Rust crate. Alternatively, you can define an `alias` for your shell (see below).

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

## Manual

```man
sconectl [CMD] [OPTIONS]

CMD:
  apply   apply manifest. Execute sconectl apply --help for more info.


OPTIONS:
    
  -h, --help
          Print help information. Other OPTIONS depend on the type of MANIFEST. 
          You need to specify -m <MANIFEST> for help to print more options.     
```

## Podman support

Our focus is to support `podman` instead of `docker` (legacy). To ensure that we can run both with `docker` as well as `podman`, we use the Docker API for now. After starting `podman`, please set the environment variable `DOCKER_HOST` as instructed by `podman`.

`sconectl` will use `DOCKER_HOST` as the socket. If not set, it will use the default docker socket for now, i.e., `/var/run/docker.sock`.

## Publish new versoin

To publish a new `sconectl` version, ensure that all your changes are committed and pushed. Then executed:

```bash
cargo publish
```

## Alternative: shell `alias`

In case you want to run `sconectl` from your development machine but you do not want to install this crate, you can use this `alias` instead:

```bash
alias sconectl="docker run -it --rm \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -v \"$HOME/.docker:/root/.docker\" \
    -v \"$HOME/.cas:/root/.cas\" \
    -v \"$HOME/.scone:/root/.scone\" \
    -v \"\$PWD:/wd\" \
    -w /wd \
    registry.scontain.com/cicd/sconecli:latest"
```
