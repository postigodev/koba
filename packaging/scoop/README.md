# Koba Scoop Packaging

This directory contains a reviewable Scoop manifest for testing Koba installation from GitHub Release artifacts before publishing a dedicated bucket.

## Local Manifest Test

From the Koba repository root:

```powershell
scoop install ./packaging/scoop/bucket/koba.json
koba --version
koba --help
koba scan
scoop uninstall koba
```

The manifest downloads the Windows archive from the `v0.1.1` GitHub Release and verifies the published SHA-256 checksum before exposing `koba.exe`.

## Publishing Boundary

This directory is staging only. It does not publish to Scoop's official buckets, create a bucket repository, push changes, or modify release artifacts.
