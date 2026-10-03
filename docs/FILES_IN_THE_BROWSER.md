# Design files in the browser

How the offline pages (TRI-NETRA Files today; the node app and the group app are built on it) open,
change and save the design files of `design/schema.toml` on a Drive folder. This is the work of
`docs/RELEASE_PLAN.md` P3.

## For a member

1. Install Drive for desktop. Your group's shared drive shows up as a folder on your computer.
2. Open `pages/files.html` from the kit in **Chrome or Edge**. Double-clicking it is enough: it runs
   from disk, and nothing is sent anywhere.
3. Choose **Open folder…** and pick your group's folder. The first time, type your name. It is
   written beside every change you save. There are no accounts; Drive's record of who saved each
   file backs it up.
4. Click a file to open it, change it, and choose **Save**. Every save is read back and checked
   before the page says it is saved.

Other browsers (Firefox, Safari) cannot open folders. There you choose **Open a file…**, change the
file and save it by downloading. Nobody else editing the file can be seen from there, and the page
says so.

| What happens | What you see |
|---|---|
| Someone else has the file open | It opens read-only, with their name and when they were last seen. **Open for editing anyway** is there for when you know they have stopped |
| Someone left a file open (closed the browser, computer off) | After 15 minutes without a sign of them, you can edit it; the page says whose marker it took over |
| Your own tab crashed with the file open | You get it straight back for editing |
| The browser or computer crashed before you saved | When you open the file again, **Restore them** brings back every change shown before the crash |
| Drive made a conflict copy (`name (1).node.tndb`) | The folder flags the file, and the file names the copy. Open both, carry over what is missing, then delete the copy in Drive |
| The file changed on disk since you opened it (someone else's save arrived) | **Save** is refused, never overwriting theirs; **Save a copy** puts your work beside it to compare |
| A picture over 500 KB, a file over 50 MB | Refused, with the limit named |
| A file from a newer TRI-NETRA | Refused by name; an older one is upgraded by `tools/tndb.py check` (which keeps the original) |

**Undo** and **Redo** step through your changes since you opened the file. The **History** of a
file lists every save: who, when and what. Drive also keeps every saved version (right-click the
file in Drive, Manage versions).

## How it works

| Piece | Where | What it does |
|---|---|---|
| SQLite in WebAssembly | `design/vendor/sqljs` (sql.js 1.14.2, pinned by SHA-256 in `design/vendor/VENDOR.toml`) | The file is read whole into memory, edited there and written back whole. This needs no server settings and no storage mode a page opened from disk cannot have |
| The file layer | `design/js/tnfile.js` | Opens a file and checks it as `tools/tndb.py check` does (format, version, every table and column, caps). Runs every change as one undoable step. Keeps unsaved work in IndexedDB. Writes the open-elsewhere marker and saves with checks |
| Open elsewhere | `<file>.editing` beside the file, plus the browser's lock | The marker (who, since, last seen, which browser profile) travels with Drive to the other computers, refreshed every minute. Within one browser a Web Lock refuses a second tab at once |
| Conflict copies | `conflictBase()` in `tnfile.js` | Drive's `name (1).ext`, `name.ext (1)` and `… conflict …` copies are matched to their original |
| Crash-safe saving | `FileSession.save()` | Refuses when read-only, taken over, changed on disk or over a cap. Writes through the browser's swap file, so the old file stays whole until the new one is complete. Reads the result back and compares hashes, then clears the unsaved-work copy |
| Undo | temporary triggers (`installUndo`) | Each change records the SQL that undoes it ([sqlite.org/undoredo](https://sqlite.org/undoredo.html)); reals come back exactly |
| The component set | `design/js/tn_ui.js`, `design/css/tn.css` | Buttons, fields, banners, badges, tables, dialogs, toasts, the page shell; light and dark, phone width. Pages make no controls or styles of their own |
| The fonts | `design/vendor/fonts` (Inter, JetBrains Mono, SIL OFL 1.1) | Inlined into each page |
| The page builder | `tools/pages.py build` / `check` | Inlines all of the above into one HTML file under `build/pages/`. Refuses an unpinned vendored file, a page with its own controls or styles, and anything loaded from an outside host. Built pages are not committed; CI builds them and the kits carry them (`pages/`) |

## How it is proven

`tests/test_pages.py` (in `check_all` as `offline-pages`, and in CI with Playwright's Chromium) runs
`tests/browser/files.test.mjs` against a fresh build:

- a node file saved to a folder reopens with the change, and its history names who and what;
  the saved file passes `tools/tndb.py check`;
- undo and redo, with a real restored exactly;
- a renderer crash before saving loses nothing (restored from IndexedDB), and the person gets the file
  back for editing;
- a renderer crash in the middle of a write leaves the file byte for byte whole;
- a second tab gets the file read-only; another computer's fresh marker refuses; a stale one is
  taken over and said so; closing removes the marker;
- conflict copies are flagged in the folder and in the file;
- a file changed on disk is never overwritten, and the work goes to a copy;
- the picture and file caps hold, in the page and in the file layer;
- a newer format is refused by name;
- from disk (`file://`) with no request leaving the page: open a file, change it, save by downloading;
- phone width: nothing runs off the side.

What the tests cannot do is pick a real Drive folder: the browser's folder dialog needs a person.
The tests hand the page the browser's private folder instead, which has the same interface
(`FileSystemDirectoryHandle`), so everything after the picker is the code a person runs. That
folder cannot be used by a page opened from disk, so those tests serve the page from 127.0.0.1.

**Limits, stated:**
- The marker reaches other computers only as fast as Drive syncs it. Two people who open the same
  file within that delay can both edit it. The second save is then refused as "changed on disk",
  or Drive makes a conflict copy, and both are reported.
- A typed name is not a proof of identity (`docs/RELEASE_PLAN.md` §2, row 12).
