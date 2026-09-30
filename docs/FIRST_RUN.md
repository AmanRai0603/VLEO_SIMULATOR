# Opening the tool the first time

> **Answer first.** The programs are not signed yet, so Windows and macOS ask once before opening them; this page shows each message, what to press, and how to check the file you downloaded is the one that was built.
>
> **Kind:** how-to · **For:** the team using the tool

Nothing here is a warning about the tool. A program from outside an app store
that carries no publisher's certificate is unknown to the system, and the
system asks. It asks **once per version**; after that the app opens on a
double-click like any other. The Python package (`.whl`) never asks, because it
runs inside Python: if a laptop will not open the app at all, use that.

The tool only listens on your own machine, at `127.0.0.1`, and never uses the
network. Your inputs and results stay in `.vleo` in your home folder.

---

## Windows

1. **Before unzipping:** right-click the zip → **Properties** → tick **Unblock**
   at the bottom → **OK**. This tells Windows you meant to download it, so it
   does not ask again for every file inside.
2. **Extract All** to a folder of your own, such as `C:\vleo`. Do not run it
   from inside the zip.
3. Double-click **`VLEO Design Tool.exe`**.

If a blue box says **"Windows protected your PC"** (Microsoft Defender
SmartScreen): click **More info**, check that it names **VLEO Design Tool**,
then **Run anyway**.

No black console window opens: the app runs in the background and opens your
browser. **Quit** at the top of the page ends it.

**If the company antivirus deletes or blocks the file,** that is its rule for
unknown programs, not a finding about this one. Ask IT to allow the folder, or
use the Python package instead.

## macOS

1. Double-click the zip. **VLEO Design Tool** appears beside it.
2. Drag it into **Applications**.
3. Open it. macOS says it **cannot verify the developer**, and offers **Done**
   or **Move to Bin**: press **Done**.
4. Open **System Settings → Privacy & Security**, scroll to **Security**. It
   says *"VLEO Design Tool" was blocked*. Press **Open Anyway**, confirm with
   your password or Touch ID, then **Open**.

On macOS 14 or older, **right-click** the app → **Open** → **Open** also works.

The app has no Dock icon and no window of its own: it opens your browser.
**Quit** at the top of the page ends it.

**"The app is damaged and can't be opened"** means the download was changed on
the way, most often by unpacking it with a tool other than the Finder. Delete
it and unzip the original again with a double-click.

## Linux

Unzip, then run `./vleo-app` in the folder (or `./start.sh` to watch its log in
the terminal). If it says *permission denied*: `chmod +x vleo-app start.sh`.

---

## Checking a download is genuine

Every release lists `SHA256SUMS.txt` beside its files: the fingerprint of each
file as it was built. Compare yours:

| on | run, in the folder you downloaded to |
|---|---|
| Windows (PowerShell) | `Get-FileHash .\<file>.zip` and compare with the line for that file |
| macOS | `shasum -a 256 -c SHA256SUMS.txt --ignore-missing` |
| Linux | `sha256sum -c SHA256SUMS.txt --ignore-missing` |

Matching fingerprints mean the file is byte for byte the one the release
built. A mismatch means do not open it: download it again from the release page.

## When it does not start

The app shows a page saying why, and the same page is kept in `.vleo/log` in
your home folder. The usual reasons:

- **All the ports from 7777 are taken.** Another program is using them. Set
  the environment variable `VLEO_PORT` to another number, such as `8800`, and
  open the app again.
- **It was opened from inside the zip.** Unzip it first.

A crash writes a report to `.vleo/log` as well. Send it to whoever looks after
the tool; it says what happened and where, and nothing about your work.

---

When the programs are signed, which the release pipeline is ready for (see
[RELEASE_SETUP.md](RELEASE_SETUP.md)), none of the prompts above will appear.
