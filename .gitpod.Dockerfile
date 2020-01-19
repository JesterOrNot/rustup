FROM gitpod/workspace-full
USER gitpod
RUN set -o pipefail \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; \
       rustup use -y nightly
