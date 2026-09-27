# Start here

> **Answer first.** Unzip the folder, run `start.bat` (Windows) or `start.sh` (macOS, Linux), and
> the tool opens in your browser. Set your inputs, run the design, keep the results. When the
> design itself is wrong or missing something, fill that node's form and send the file to the
> developers — they check it, build it in, and send you the next version. Your inputs and results
> are kept on your machine and carry over to every new version.
>
> **Kind:** tutorial + how-to · **For:** the team using the tool

This folder is the whole tool. It needs no internet, no installation and no account, and
nothing in it is edited by you — so replacing it with the next version never loses anything.

---

## 1 · Start it

| on | do this |
|---|---|
| Windows | double-click `start.bat` |
| macOS | open a terminal in this folder, run `xattr -dr com.apple.quarantine .` once, then `./start.sh` |
| Linux | `./start.sh` |

The tool opens in your browser, normally at `http://127.0.0.1:7777`; if that port is taken it
uses the next free one and opens that instead, and the window it runs in prints the address. It
is only on your machine — nobody else can reach it. To stop it, close the window (Windows) or
press Ctrl-C (macOS, Linux).

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
4. Press **save a filled copy**. You can check the filled file yourself on the **Forms** page —
   it shows what the developers will see, and changes nothing.
5. Send the filled file (e-mail, chat, a shared drive) to the developers.

They check it, apply it, and it arrives in the next version. If it cannot be applied, you get the
file back with the reason, line by line.

## 4 · When a new version arrives

Replace this folder with the new one and start it again. Your inputs, saved case and results are
kept under `~/.vleo/` (your home folder), not here, so they carry over. If an input no longer
exists, the tool says so by name; if a result you kept rested on a belief that has since changed,
the Results page says which one.

## 5 · What you cannot do here, and why

You cannot edit a row, add one or remove one in the tool. A change typed into one copy of the tool
would be a change nobody checked and everyone else would be running without. The form is the way
in, and it keeps your name on what you asked for.
