FROM rust:1.68.0 as builder

WORKDIR /usr/src/pacops
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/rust/.cargo/git \
    --mount=type=cache,sharing=private,target=/usr/src/pacops/target \
    cargo build --release && \
    strip target/release/pacops && \
    cp /usr/src/pacops/target/release/pacops /usr/src/pacops

FROM archlinux:base-devel
LABEL version 0.0.1
LABEL description "PacOps Archlinux based build image"
LABEL maintainer="ejiek@mail.ru"

RUN useradd --uid 1001 --create-home --home-dir /usr/share/pacops pacops && /bin/echo -e 'Cmnd_Alias PACMAN=/usr/sbin/pacman *\npacops ALL= NOPASSWD: PACMAN' > /etc/sudoers.d/88_pacops
RUN pacman -Sy --noconfirm git

USER pacops
WORKDIR /usr/share/pacops
COPY --from=builder /usr/src/pacops/pacops /usr/bin/
ENTRYPOINT ["/usr/bin/pacops"]
