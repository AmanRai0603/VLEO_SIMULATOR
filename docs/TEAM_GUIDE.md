# Start here

> **Answer first.** Install the one file with Python — `python -m pip install vleo-<version>-py3-none-any.whl`
> — then start it with `python -m vleo`, and the tool opens in your browser. (Or unzip the kit for
> your system and start the program in it.) Set your inputs, run the design, keep the results. When
> the design itself is wrong or missing something, its node engineer changes it in the node's own
> file, and it reaches the design with the group's sealed release. Once the subsystem engineer
> seals that release and puts it on the shared drive, the tool builds today's design from it, for
> everyone who opens it on the drive. Your inputs and results are kept on your machine and carry
> over to every new version.
>
> **Kind:** tutorial + how-to · **For:** everyone using the tool — the programme manager, the system engineer, subsystem engineers and node engineers

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

### B · The kit — a folder for your system

| on | do this |
|---|---|
| Windows | right-click the zip → **Properties** → tick **Unblock** → **OK**, then **Extract All** to `C:\vleo`; open the folder and double-click **`Start VLEO.exe`** |
| macOS | open a terminal in the folder, run `xattr -dr com.apple.quarantine .` once, then `./start.sh` |
| Linux | `./start.sh` in the folder |

**Windows says "Windows protected your PC"?** The program is not signed, so Windows does not know
it. Click **More info → Run anyway**; it asks once. **If your company's antivirus removes
`Start VLEO.exe`** (it can, as an unknown program), use **A** — or ask IT to exclude the folder
`C:\vleo`. The tool only listens on your own machine and never uses the network.

### Either way

The tool opens in your browser, normally at `http://127.0.0.1:7777`; if that port is taken it uses
the next free one and opens that instead, and the window it runs in prints the address. It is only
on your machine — nobody else can reach it. To stop it, close that window or press Ctrl-C.

## 2 · Use it

- **The four layers** across the top are the design, from the programme down to the run. Click
  any row to open its page. Every page starts with the answer, then says it simply, then shows the
  real relation and where it comes from, then where the simple version stops being true.
- **Learn · Read · Expert** at the top right sets how much each page explains.
- **Inputs** is your case: every number you can set, with its default. Change them on the page, or
  download the CSV, edit it and upload it back.
- **Run** a row from its page. **save this result** keeps what it returned, with the inputs it ran
  on — and **save this sweep** keeps a sweep. A question already kept is not run again: the tool
  shows the saved answer and says which it is (**run again anyway** runs it fresh). **Results**
  lists what you kept, draws saved sweeps again, and each can be downloaded as a report page to
  send to someone — uploaded, it comes back whole.
- **? Manual** answers "how do I…" for everything above.

## 3 · Change the design

The tool never changes the design itself. A node changes in its own file, and only its node
engineer writes it.

1. **Your subsystem engineer issues you your node file**, `<node>.vnode`, on your group's shared
   drive.
2. **Open it in the node application**, `node.html` in the group's `apps/` folder — it works
   offline. It walks you through, step by step: the node's contract, the explanation, the theory,
   the pseudocode (checked as you type — every line, every unit), the inputs, your own results,
   the evidence, pictures and your code. Then **Check & sign**.
3. **Put the file back** in `nodes/` on the drive, replacing the old one.
4. **Your subsystem engineer assembles the group's release**, has each node signed and seals it.
5. **The sealed release goes on the drive**, in the group's `releases/` folder. Nobody takes it in
   by hand.

When the tool opens on the drive, it builds today's design from every group's latest sealed
release that passes its checks — its seal, its content, and each node against the design. If your
group's latest release is refused, your group's part is built from its last good release, and the
tool says which release it is and why the new one was refused; fix what it names, seal again, and
put the new release beside the old one. The group's side, step by step, is `docs/GROUP_APPS.md`.

**Your guide:** `docs/roles/user.html` in this folder is an interactive page with every step for
you — using the tool, and how a change reaches the design — at three depths (Learn, Read, Expert).

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
would be a change nobody checked and everyone else would be running without. The node's own file is
the way in, and your signature keeps your name on it.
