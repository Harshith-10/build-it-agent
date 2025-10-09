# GitHub Actions Workflow - Release Pipeline

## Overview

The project uses a single, unified GitHub Actions workflow (`.github/workflows/release.yml`) that handles building, signing, and releasing binaries for all platforms.

## Workflow Triggers

The workflow runs on:
- **Version tags**: Push a tag like `v1.0.0` to trigger a release
- **Manual dispatch**: Can be triggered manually from GitHub Actions UI

```bash
# Create and push a release tag
git tag v1.0.0
git push origin v1.0.0
```

## Build Matrix

The workflow builds binaries for 6 different targets:

| Platform | Architecture | Target | Binary Extension | Archive Format |
|----------|-------------|--------|------------------|----------------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | - | `.tar.gz` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | - | `.tar.gz` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `.exe` | `.zip` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `.exe` | `.zip` |
| macOS | Intel | `x86_64-apple-darwin` | - | `.zip` |
| macOS | Apple Silicon | `aarch64-apple-darwin` | - | `.zip` |

## Windows Code Signing

### Automatic Signing (Recommended for Production)

If code signing secrets are configured, Windows binaries are automatically signed during the build:

1. **Set up GitHub Secrets** (Repository Settings → Secrets and variables → Actions):
   - `CERTIFICATE_BASE64`: Your code signing certificate (.pfx) encoded as base64
   - `CERTIFICATE_PASSWORD`: Password for the certificate

2. **Generate base64 certificate** (PowerShell):
   ```powershell
   $certBytes = [IO.File]::ReadAllBytes("path\to\your\certificate.pfx")
   $base64 = [Convert]::ToBase64String($certBytes)
   $base64 | Set-Clipboard  # Copy to clipboard
   ```

3. **Add secrets to GitHub**:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Add `CERTIFICATE_BASE64` with the base64 string
   - Add `CERTIFICATE_PASSWORD` with your certificate password

### Signing Process

When secrets are configured:
1. Workflow decodes the base64 certificate
2. Finds SignTool from Windows SDK
3. Signs the binary with SHA256 and timestamp
4. Verifies signing succeeded
5. Cleans up temporary certificate file

### Without Signing

If no certificate is configured:
- Binaries are built but **unsigned**
- Workflow displays a warning message
- Binary may trigger Windows Defender false positives
- Users will need to add exclusions or submit to Microsoft

## Release Artifacts

### What Gets Released

For each tag push, the workflow creates:

1. **Platform-specific archives**:
   - `build-it-agent-x86_64-unknown-linux-gnu.tar.gz`
   - `build-it-agent-aarch64-unknown-linux-gnu.tar.gz`
   - `build-it-agent-x86_64-pc-windows-msvc.zip` (signed if certificate available)
   - `build-it-agent-aarch64-pc-windows-msvc.zip` (signed if certificate available)
   - `build-it-agent-x86_64-apple-darwin.zip`
   - `build-it-agent-aarch64-apple-darwin.zip`

2. **Checksums file**: `checksums.txt` with SHA256 hashes for all binaries

### GitHub Release

The workflow automatically:
- Creates a GitHub release for the tag
- Uploads all platform archives
- Includes checksums file
- Uses tag name as release title

## Workflow Jobs

### Job 1: Build (Matrix)

Runs in parallel for all 6 platforms:
1. Checkout code
2. Install Rust toolchain with target
3. Set up cargo caching (registry, index, build)
4. Install cross-compilation tools (if needed)
5. Build release binary
6. **Sign Windows binaries** (if certificate available)
7. Create platform-specific archive
8. Upload artifact for release job

### Job 2: Release

Runs after all build jobs complete:
1. Download all platform artifacts
2. Generate SHA256 checksums
3. Create GitHub release with all files

## Caching Strategy

The workflow uses aggressive caching to speed up builds:
- **Cargo registry**: Cached by OS + target + Cargo.lock
- **Cargo index**: Cached by OS + target + Cargo.lock  
- **Build artifacts**: Cached by OS + target + Cargo.lock

Cache is automatically invalidated when dependencies change.

## Example: Creating a Release

### Step 1: Prepare Release
```bash
# Make sure all changes are committed
git add .
git commit -m "Prepare v1.0.0 release"
git push origin master
```

### Step 2: Create and Push Tag
```bash
# Create annotated tag
git tag -a v1.0.0 -m "Release version 1.0.0"

# Push tag to trigger workflow
git push origin v1.0.0
```

### Step 3: Monitor Workflow
- Go to GitHub → Actions tab
- Watch the "Build and Release Binaries" workflow
- Each platform builds in parallel (~10-15 minutes total)

### Step 4: Check Release
- Go to GitHub → Releases
- Find the new `v1.0.0` release
- Download artifacts for your platform
- Windows binaries will be signed if certificate was configured

## Troubleshooting

### Build Fails for Specific Platform

Check the job logs for that platform:
1. Go to Actions → Failed workflow run
2. Click on the failing job (e.g., "Build x86_64-pc-windows-msvc")
3. Review error messages

Common issues:
- Cross-compilation tools not installed
- Cargo.lock out of sync
- Platform-specific dependencies missing

### Signing Fails

If Windows signing fails:
1. Verify `CERTIFICATE_BASE64` is valid base64
2. Check `CERTIFICATE_PASSWORD` is correct
3. Ensure certificate is a valid .pfx file
4. Check certificate hasn't expired

The workflow will still create the release with unsigned binaries.

### Release Not Created

If the release job fails:
1. Check GitHub token permissions
2. Verify workflow has `contents: write` permission
3. Ensure tag follows `v*` pattern (e.g., `v1.0.0`, not `1.0.0`)

## Manual Workflow Dispatch

To trigger the workflow manually without a tag:

1. Go to GitHub → Actions → "Build and Release Binaries"
2. Click "Run workflow"
3. Select branch
4. Click "Run workflow"

Note: Manual runs create artifacts but not GitHub releases (releases require tags).

## Maintenance

### Updating Rust Version

Edit `.github/workflows/release.yml`:
```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable
  with:
    targets: ${{ matrix.target }}
    # Change to specific version if needed:
    # toolchain: 1.75.0
```

### Adding New Platform

Add to the build matrix:
```yaml
# Example: FreeBSD x86_64
- os: ubuntu-latest
  target: x86_64-unknown-freebsd
  archive: tar.gz
  bin_ext: ""
```

Then install necessary cross-compilation tools in a new step.

### Updating Actions Versions

Keep GitHub Actions up to date:
- `actions/checkout@v4` → Check for newer versions
- `dtolnay/rust-toolchain@stable` → Already using latest
- `actions/cache@v3` → Update to v4 when stable
- `actions/upload-artifact@v4` → Already latest
- `softprops/action-gh-release@v2` → Already latest

## Performance

Typical build times (with cold cache):
- Linux builds: ~5-8 minutes
- Windows builds: ~8-12 minutes (includes signing)
- macOS builds: ~10-15 minutes

With warm cache (dependencies cached):
- All builds: ~2-5 minutes

Total workflow time (parallel): ~15-20 minutes

## Security Best Practices

1. **Protect certificate secrets**: Never commit .pfx files
2. **Use protected branches**: Require PR reviews before merging to master
3. **Restrict tag creation**: Limit who can push tags
4. **Review workflow logs**: Don't expose sensitive data
5. **Rotate certificates**: Update secrets when certificate is renewed

## Cost Considerations

GitHub Actions is free for public repositories with limits:
- Linux/Windows runners: 2,000 minutes/month (free tier)
- macOS runners: Uses 10x minutes (macOS builds count as 100-150 minutes each)

For private repos or heavy usage, consider:
- GitHub Pro/Team for more minutes
- Self-hosted runners for unlimited builds
- Limiting builds to specific platforms

## Migration from Old Workflows

Previously had two separate workflows:
- ❌ `build-windows.yml` - Windows-only build with signing
- ❌ `release.yml` - Multi-platform builds without signing

Now merged into single workflow:
- ✅ `.github/workflows/release.yml` - Multi-platform builds **with** Windows signing

### What Changed
- Windows binaries now signed automatically in release workflow
- Single workflow for all platforms
- Better caching strategy
- Unified release process

### Migration Steps
1. ✅ Merged workflows into `release.yml`
2. ✅ Deleted redundant `build-windows.yml`
3. ✅ Updated documentation references
4. ✅ Tested workflow with signing

## Summary

✅ **Single unified workflow** for all platforms  
✅ **Automatic Windows signing** if certificate configured  
✅ **Parallel builds** for fast releases  
✅ **Aggressive caching** for speed  
✅ **Multi-architecture** support (x86_64, ARM64)  
✅ **Automated releases** on version tags  
✅ **SHA256 checksums** for verification  

**Next Steps**:
1. Configure code signing secrets (optional but recommended)
2. Create a release tag: `git tag v1.0.0 && git push origin v1.0.0`
3. Download signed binaries from GitHub Releases
