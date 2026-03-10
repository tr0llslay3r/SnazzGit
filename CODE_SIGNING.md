# Code Signing Policy

## Windows

SnazzGit Windows installers (`.msi` and `.exe`) are signed by **[SignPath Foundation](https://signpath.org)** under their free code signing program for open source projects.

The certificate is issued to: **SnazzGit**

Signed releases can be verified in Windows by right-clicking the installer → Properties → Digital Signatures.

## Signing Process

Builds are compiled via GitHub Actions. The SignPath Foundation signs the resulting artifacts directly from the verified build pipeline, ensuring that what you download matches exactly what was built from the public source code in this repository.

The signing configuration and build scripts are fully visible in this repository:
- Build workflow: `.github/workflows/`
- Tauri bundle config: `src-tauri/tauri.conf.json`

## Unsigned Builds

If you build SnazzGit from source yourself, the resulting binaries will not be signed. Windows may show a SmartScreen warning — this is expected for self-built binaries.

## Privacy

SnazzGit is a fully local application. **It does not transmit any data to external servers.** All git operations run locally on your machine using the bundled `git2` library. No telemetry, analytics, or network calls are made except for explicit user-initiated git remote operations (fetch, pull, push) to repositories you have configured.

## Contact

If you have concerns about a signed SnazzGit release, please open an issue at:
https://github.com/tr0llslay3r/SnazzGit/issues
