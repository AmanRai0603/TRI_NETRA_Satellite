# Opening TRI-NETRA ADCS the first time

> **Answer first.** Until the programs are signed, Windows and macOS ask once before they
> open them. This page shows each message, what to press, and how to check that the file
> you downloaded is the one the release built.
>
> **Kind:** how-to · **For:** whoever uses the tool

Nothing here is a warning about the tool itself. A program from outside an app store that
carries no publisher's certificate is unknown to the system, so the system asks. It asks
**once per version**; after that the app opens on a double-click. The Python package never
asks, because it runs inside Python: if a computer will not open the app at all, use that.

The tool only listens on your own computer (`127.0.0.1`) and never uses the network. Your
runs stay in `.trinetra/store` in your home folder.

---

## Windows

1. **Before unzipping:** right-click the zip → **Properties** → tick **Unblock** → **OK**.
2. **Extract All** to a folder of your own. Do not run it from inside the zip.
3. Double-click **TRI-NETRA ADCS**.

If a blue box says **"Windows protected your PC"**: click **More info**, check that it names
**TRI-NETRA ADCS**, then **Run anyway**.

No console window opens; the app opens your browser. **Quit** on the page ends it. If the
company antivirus removes the program, that is its rule for unknown programs: ask IT to
allow the folder, or use the Python package.

## macOS

1. Double-click the zip; **TRI-NETRA ADCS** appears beside it. Drag it into **Applications**.
2. Open it. macOS says it cannot verify the developer: press **Done**.
3. **System Settings → Privacy & Security**, scroll to *Security*: *"TRI-NETRA ADCS" was
   blocked*. Press **Open Anyway**, confirm, then **Open**.

On macOS 14 and older, **right-click** the app → **Open** → **Open** does the same.

**"The app is damaged and can't be opened"** means the download was changed on the way,
usually by unpacking it with a tool other than the Finder. Delete it and unzip the original
again with a double-click.

## Linux

Unzip, then `./trinetra-app` (the app) or `./adcs help` (the command line). If it says
*permission denied*: `chmod +x adcs trinetra-app`.

---

## Checking a download

Every release lists `SHA256SUMS.txt` beside its files, the fingerprint of each file as it
was built:

| on | in the folder you downloaded to |
|---|---|
| Windows (PowerShell) | `Get-FileHash .\<file>` and compare with its line |
| macOS | `shasum -a 256 -c SHA256SUMS.txt --ignore-missing` |
| Linux | `sha256sum -c SHA256SUMS.txt --ignore-missing` |

A mismatch means do not open it: download it again from the release page.

## When it does not start

The app shows a page saying why, and keeps it in `.trinetra/log`:

- **The data is not beside the program.** The whole folder was not unzipped, or the program
  was moved out of it. Unzip the whole zip and open the program where it is.
- **No port was free.** Set the environment variable `TRINETRA_PORT` to another number (such
  as `8800`) and open it again.

A crash writes a report to `.trinetra/log`; send it to whoever looks after the tool.

When the programs are signed ([RELEASE_SETUP.md](RELEASE_SETUP.md)), none of the prompts
above appear.
