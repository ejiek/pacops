**Warning:** This is an extremely early version.
We're just testing ideas.
Implementation might change completely.

**PacOps** is a package maintainers Swiss Army knife.
It's not an AUR helper, but it's made to help you maintain PKGBUILD files.

# General Goals:

* Upstream updates check
* Applying update to a PKGBUILD file
* Availability as GitHub Action
* AUR package maintenance (generate `.SRCINFO`, commit, subtree or regular push)
* Repo maintenance (build, sign, add to a repo db, upload)

# Install

Should be available as

* AUR package
* binary package in ejiek's repo
* Cargo package
* Docker image

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

* Migrate of dummy parsing to [NomCup](https://github.com/ejiek/nomcup)
* GitHub as an upstream:
  * releases
  * tags
  * branches
* Rootless containers (currently we don't have a workflow for rootless podman)
* Clean chroot life cycle (create, update, delete)
* Navigate user through repo creation
