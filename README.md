**Warning:** This is an extremely early version.
We're just testing ideas.
Implementation might change completely.

[**PacOps**](https://github.com/ejiek/pacops/) is a package maintainers Swiss Army knife.
It's not an AUR helper, but it's made to help you maintain PKGBUILD files.

# General Goals:

* Upstream updates check
* Applying update to a PKGBUILD file
* Availability as GitHub Action
* AUR package maintenance (generate `.SRCINFO`, commit, subtree or regular push)
* Repo maintenance (build, sign, add to a repo db, upload)

# Development progress

* Usable locally
* Supports only debian repositories as upsteam, [example](https://packages.microsoft.com/repos/edge/pool/main/m/microsoft-edge-dev/) (with only one package and different version)
* Builds locally or in a user-provided chroot
* Commits updates with a commit message generate from a template
* Updates a local system or a chroot
* Somewhat runnable in Docker

**Take part!**
We have [discussions](https://github.com/ejiek/pacops/discussions), [issues](https://github.com/ejiek/pacops/issues) and [PRs](https://github.com/ejiek/pacops/pulls).

# Install

Should be available as

* [AUR package](https://aur.archlinux.org/packages/pacops/)
* binary package in [ejiek's repository](https://ejiek.com/repository/)
* [Cargo package](https://crates.io/crates/pacops)
* [Docker image](https://hub.docker.com/r/ejiek/pacops)

## Docker usage

PacOps docker image is based on archlinux:base-devel and uses itself as a build environment.

```
docker run -v ${PWD}:/usr/share/pacops pacops ${path_to_a_PKGBUILD}
```

Git variables:
`GIT_AUTHOR_NAME`
`GIT_AUTHOR_EMAIL`

Makepkg variables:
`PACKAGER="John Doe <john@doe.com>"`

# Roadmap

* Documentation
* GitHub as an upstream:
  * releases
  * tags
  * branches
* Rootless containers (currently we don't have a workflow for rootless podman)
* Migrate of dummy parsing to [NomCup](https://github.com/ejiek/nomcup)
* Clean chroot life cycle (create, update, delete)
* Navigate user through repo creation
