# Start here

> **Answer first.** Install the one file with Python — `python -m pip install vleo-<version>-py3-none-any.whl`
> — then start it with `python -m vleo`, and the tool opens in your browser. (Or unzip the kit for
> your system and start the program in it.) Set your inputs, run the design, keep the results. When
> the design itself is wrong or missing something, fill that node's form and send the file to the
> maintainer — they check it, build it in, and send you a preview of your change to try. When it
> gives what you expect, press Approve and send back the file it saves; your change then goes into
> the next release for everyone. Your inputs and results are kept on your machine and carry over
> to every new version.
>
> **Kind:** tutorial + how-to · **For:** the team using the tool

The tool comes two ways, and both are the same tool with the same engine. It needs no internet
and no account, and nothing in it is edited by you — so the next version never loses anything.

---

## 1 · Start it

### A · The Python package — one file for every laptop (use this one)

`vleo-<version>-py3-none-any.whl` is the same file for Windows, macOS and Linux. It has no program
of its own in it — it runs inside Python — so Windows does not stop it as an unknown program, and
email, drives and browsers pass it like any other file.

1. **Once: Python 3.9 or newer.** On Windows, install *Python 3.12* from the Microsoft Store (no
   administrator needed). macOS and Linux usually have it already as `python3`.
2. **Install the file.** Open a terminal where you saved it — on Windows, in File Explorer click the
   address bar, type `cmd` and press Enter — and run:

       python -m pip install vleo-<version>-py3-none-any.whl

   (`python3` instead of `python` on macOS and Linux; `py` if Windows says `python` is not found.)
   It needs no internet.
3. **Start it,** from any terminal, whenever you want the tool:

       python -m vleo

   `python -m vleo --check` proves the install works on your computer without opening anything.
   The install also adds two commands: `vleo-design-tool` (the same as `python -m vleo`) and
   `vleo-app`, which starts it like the desktop app below — no terminal window to keep open, and
   it closes by itself.

### B · The desktop app — double-click and it opens

| on | download | do this |
|---|---|---|
| Windows | `vleo-<version>-x86_64-pc-windows-msvc.zip` | right-click the zip → **Properties** → tick **Unblock** → **OK**, then **Extract All** to `C:\vleo`; open the folder and double-click **`VLEO Design Tool.exe`** |
| macOS | `VLEO-Design-Tool-<version>-macos-arm64.zip` | double-click the zip, drag **VLEO Design Tool** into **Applications**, then open it |
| Linux | `vleo-<version>-x86_64-unknown-linux-gnu.zip` | `./vleo-app` in the folder (or `./start.sh` to see its log) |

The app opens in your browser. It has no window of its own to close: **Quit** at the top of the
page ends it, and it also ends by itself a few minutes after you close the last tab. Opening it
again while it runs just opens the page again.

**The first time, your computer asks.** These programs are not signed yet, so Windows and macOS
say they do not know them: on Windows click **More info → Run anyway**; on a Mac, open it, then
**System Settings → Privacy & Security → Open Anyway**. Each asks once. [FIRST_RUN.md](FIRST_RUN.md)
has the steps with every message you may see, and how to check the download is genuine. **If your
company's antivirus removes the program**, use **A** — or ask IT to allow the folder. The tool only
listens on your own machine and never uses the network.

### Either way

The tool opens in your browser, normally at `http://127.0.0.1:7777`; if that port is taken it uses
the next free one and opens that instead. It is only on your machine — nobody else can reach it.
Started from a terminal, stop it by closing that window or pressing Ctrl-C; the app, with **Quit**.

## 2 · Use it

- **The four layers** across the top are the design, from the programme down to the run. Click
  any row to open its page. Every page starts with the answer, then says it simply, then shows the
  real relation and where it comes from, then where the simple version stops being true.
- **Learn · Read · Expert** at the top right sets how much each page explains.
- **Inputs** is your case: every number you can set, with its default. Change them on the page, or
  download the CSV, edit it and upload it back.
- **Run** a row from its page. **save this result** keeps what it returned, with the inputs it ran
  on; **Results** lists what you kept, and each can be downloaded as a CSV or a report page to send
  to someone.
- **? Manual** answers "how do I…" for everything above.

## 3 · Ask for the design to change

The tool never changes the design itself. A change is a **node form**: one file you fill in your
browser, offline, and send to the developers.

1. Open the row, choose the tab **the node form — fill it anywhere**, and download the form. For a
   row that does not exist yet, go to **Forms** and download **the form for a new node**.
2. Open the downloaded file in your browser — it works offline. Fill what you know: say the row
   simply, correct the numbers, add an input, give a known value and where it comes from.
3. If your change moves what the row computes, fill **Why it is changing**: what we believed,
   what you tested, what you now know, and what would break the new belief. The form tells you as
   you type which answers it still needs. Wording alone needs none of them.
   **If it is your relation**, also fill **The method** (the relation as a few lines — see
   `docs/PSEUDOCODE.md`, and *Show the example* beside each question), **Your code** (the code you
   wrote and tested, and the script that ran it) and **Your test cases**: at least three answers
   your code gave and one input it refuses. The form runs your method on your cases as you type;
   send it when its check says **Sound**.
4. Press **save a filled copy**. You can check the filled file yourself on the **Forms** page —
   it shows what the developers will see, and changes nothing.
5. Send the filled file to the maintainer, the way your team shares files.
6. **You get a preview back** — the tool with your change in it, marked with an orange **PREVIEW**
   banner. Install it like a new version (`python -m pip install <file>.whl`), open each node the
   banner names, run it and check it against your own answers.
7. **When it is right, press _Approve this preview…_**, tick that you ran it, and send back the file
   it saves. If something is wrong, do not approve — say what, and you get a new preview.

Only then is your change merged, and it reaches everyone in the next release. If your form cannot
be taken in, you get a note back with the reason, line by line, and nothing was changed.

**Your guide:** `docs/roles/user.html` in this folder is an interactive page with every step for
you — using the tool, filling a form, approving a preview — at three depths (Learn, Read, Expert).

## 4 · When a new version arrives

**A:** install the new `.whl` the same way — `python -m pip install vleo-<new version>-py3-none-any.whl`
replaces the old one. **B:** replace the folder with the new one. Then start it again.

Your inputs, saved case and results are kept under `~/.vleo/` (your home folder — on Windows
`C:\Users\<you>\.vleo`), not in the tool, so they carry over — and A and B read the same place, so
you can switch between them. (Kits before 0.1.2 kept them inside the kit folder on Windows: copy
its hidden `.vleo` folder into your home folder before you delete the old one.) If an input no
longer exists, the tool says so by name; if a result you kept rested on a belief that has since
changed, the Results page says which one.

## 5 · What you cannot do here, and why

You cannot edit a row, add one or remove one in the tool. A change typed into one copy of the tool
would be a change nobody checked and everyone else would be running without. The form is the way
in, and it keeps your name on what you asked for.
