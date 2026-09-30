# Releasing TRI-NETRA ADCS

> **Answer first.** Set `VERSION`, merge, push the tag `v<VERSION>`. The release workflow
> checks everything, builds every download, runs the Windows kit on Windows and the wheel
> in a fresh Python, and publishes them with their checksums. Signing switches on by
> itself when its secrets exist.
>
> **Kind:** how-to · **For:** the maintainer

## A release

    echo 1.1.0 > VERSION          # and say what changed in docs/RELEASE_NOTES.md
    python3 tools/check_all.py    # every check passes
    # merge to main, then
    git tag v1.1.0 && git push origin v1.1.0

`.github/workflows/release.yml` then:

| job | what it proves or makes |
|---|---|
| prove | the tag is `VERSION`; every check passes |
| linux | `adcs` and the app; the Linux kit, flown from outside its folder |
| windows | both programs cross-built with MinGW GCC (the compiler the flight software's bit-for-bit parity rests on), with the app's icon and version information |
| windows-run | the Windows kit on a real Windows machine: a flight, C against Rust, the app started and quit |
| macos | arm64 and x86_64; the macOS kit and `TRI-NETRA ADCS.app` |
| wheel | `trinetra_adcs-<v>-py3-none-any.whl` with every system's programs, installed and run |
| publish | the GitHub release: kits, app, wheel, the MATLAB SILS zip, the flight-software zip, the V&V report, `SHA256SUMS.txt` |

## Signing

Unsigned programs work; the first start asks once ([FIRST_RUN.md](FIRST_RUN.md)). Adding
these repository secrets (Settings → Secrets and variables → Actions) removes the prompts on
the next release. Nothing else changes.

**Windows** (an Authenticode code-signing certificate, `.pfx`):

| secret | value |
|---|---|
| `WINDOWS_CERT_PFX` | the `.pfx` file, base64 (`base64 -w0 cert.pfx`) |
| `WINDOWS_CERT_PASSWORD` | its password |

Both programs are signed with SHA-256 and timestamped. An EV certificate removes the
SmartScreen prompt at once; a standard one after the program builds a reputation.

**macOS** (an Apple Developer ID Application certificate):

| secret | value |
|---|---|
| `APPLE_CERT_P12` | the certificate and key, `.p12`, base64 |
| `APPLE_CERT_PASSWORD` | the `.p12` password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Agastya (TEAMID)` |
| `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | for notarisation (an app-specific password) |

With the first three the app is signed with the hardened runtime; with all six it is also
notarised and stapled, so macOS opens it without asking. Without them it is signed ad hoc,
which carries no identity but keeps Apple-silicon Macs from calling it damaged.

## What each download is

- `trinetra-adcs-<v>-<system>.zip`: a kit, with `adcs`, the app, the data, the cases, the
  ephemeris, `VERSION`, `START_HERE.md`, `FIRST_RUN.md`, `COMMANDS.md` and `ENVIRONMENT.md`.
- `TRI-NETRA-ADCS-<v>-macos-arm64.zip`: the macOS app.
- `trinetra_adcs-<v>-py3-none-any.whl`: the Python package, for Windows, macOS and Linux.
- `TRINETRA_ADCS_SILS_matlab_*.zip`, `TRINETRA_ADCS_flight_engine_*.zip`,
  `TRINETRA_ADCS_VV_report.pdf`: the MATLAB SILS, the flight software with the engine's
  source, and the V&V report.
