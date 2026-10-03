# Guide to TRI-NETRA Files
<!-- role: everyone; id: files; app: files -->

**In one line:** TRI-NETRA Files opens any one design file, lets you change it with undo, saves it safely, and shows its history; it is the tool for looking inside a file, not for writing nodes or leading a group.

## Say it simply

It is a careful text editor for design files. It refuses to overwrite a file someone else has open, keeps your unsaved work if the computer stops, and checks every save by reading it back.

**Where the story lies:** a text editor lets you type anything. This one only writes what the design files' format allows, and the node app and group app hold their files to more rules than it does. Use those apps for their work.

## Now the real thing

1. **Open a folder or a file.** **Open folder** lists every design file in it, with who has one open and any conflict copies. **Open a file** opens one.
2. **Change, undo, save.** Every change is one step you can undo. **Save** writes the file, reads it back and compares it; a file changed on disk since you opened it is never overwritten: your work goes to a named copy.
3. **History.** Every save is a revision: who, when, what. Drive keeps every saved version too.

| What you see | What it means | What to do |
|---|---|---|
| Read-only: open for editing by someone | Someone else has it open | Wait, or ask them to close it |
| Unsaved work from before | The computer stopped before a save | **Restore them**, then Save |
| A newer format | The file was written by a newer TRI-NETRA | Use the newer apps |
