FROM rust:1.51 as builder

WORKDIR /usr/src/pacops
COPY . .
RUN cargo build --release
RUN strip target/release/pacops

FROM archlinux:base-devel
LABEL version 0.0.1
LABEL description "PacOps Archlinux based build image"
LABEL maintainer="ejiek@mail.ru"

RUN useradd --create-home --home-dir /usr/share/pacops pacops && echo -e 'Cmnd_Alias PACMAN=/usr/sbin/pacman *\npacops ALL= NOPASSWD: PACMAN' > /etc/sudoers.d/88_pacops

USER pacops
WORKDIR /usr/share/pacops
COPY --from=builder /usr/src/pacops/target/release/pacops /usr/bin/
ENTRYPOINT ["/usr/bin/pacops"]
