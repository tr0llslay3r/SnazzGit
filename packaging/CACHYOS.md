# Publishing SnazzGit to CachyOS Repositories

CachyOS maintains community package repositories. Here's how to get SnazzGit included.

## Option 1: Submit via GitHub

CachyOS packages are managed at:
https://github.com/CachyOS/CachyOS-PKGBUILDS

1. Fork the repository
2. Add a `snazzgit/PKGBUILD` based on the source PKGBUILD in `packaging/aur-git/`
3. Open a pull request with a description of the package

## Option 2: Community Request

1. Join the CachyOS Discord: https://discord.gg/cachyos
2. Request package inclusion in the appropriate channel
3. Provide a link to the AUR package once it's published

## Prerequisites

Before submitting to CachyOS:

1. Publish the package to AUR first (either `snazzgit-bin` or `snazzgit-git`)
2. Have at least one GitHub Release with working binaries
3. Ensure the PKGBUILD builds cleanly on a fresh CachyOS install

## AUR Publishing Steps

1. Create an AUR account at https://aur.archlinux.org
2. Generate and upload your SSH key
3. Clone the AUR package repo:
   ```bash
   git clone ssh://aur@aur.archlinux.org/snazzgit-bin.git
   ```
4. Copy the PKGBUILD from `packaging/aur/` and generate `.SRCINFO`:
   ```bash
   cp packaging/aur/PKGBUILD snazzgit-bin/
   cd snazzgit-bin
   makepkg --printsrcinfo > .SRCINFO
   git add PKGBUILD .SRCINFO
   git commit -m "Initial upload: snazzgit-bin 0.1.0"
   git push
   ```
5. Repeat for `snazzgit-git` if desired
