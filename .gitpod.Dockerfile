FROM gitpod/workspace-full
USER gitpod
RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --default-toolchain=nightly;rustup use nightly