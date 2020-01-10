# Dockerfiles
A collection of scripts to showcase how to build rustysd into a docker container

1. install_musl_target.sh: Use rustup to install the musl target
1. build_docker_image.sh: Build rustysd with musl, strip the binaries and make the docker image
1. run_docker_image.sh: Run the docker image 