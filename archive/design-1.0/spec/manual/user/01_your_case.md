# Your case
<!-- kind: how-to; depth: read -->

**In one line:** write your case in the case editor (or the CSV template), let its first sentence tell you whether it is ready and what your blanks block, then download the CSV and open it in the software.

A **case** is your whole input: the satellite, its orbit and mission, and what its ADCS must achieve. It is one CSV in the fixed format `adcs-case/1`, which every face of the software reads the same way. Why blanks, ranges and levels behave as they do is explained on the editor's own page, under "What happens to a blank, a range and a level".

## Write a case in the case editor

1. **Open the editor.** `case_new_case.editor.html` in the node library or the MATLAB zip's `forms/`, or **Case** in the web app. It works offline.
2. **Start from something.** "Start from…" loads a reference case (`ais_3u` needs 10° pointing; `ais_img_3u` is the same satellite needing 0.01°). "Open a CSV…" loads a file you have. Or start blank.
3. **About the case.** Give an id (lower-case letters, digits and `_`, such as `sat42_eo`) and a title. The class and the families are optional; blank lets the software decide.
4. **What the ADCS must achieve.** Write each requirement you have, in the unit shown. Under each one the editor says whether the achieved value must be **at most** or **at least** your number. Give a **level** in per cent if you know it (99.73 for 3σ). A requirement you leave blank is not judged.
5. **The satellite.** Fill what you know. Each row says what it means, and what a blank does: *blocks what needs it, by name*, or *takes the reference value, listed as assumed*.
6. **Read the first sentence again.** It says whether the format is right, how many requirements are written, how many inputs are unstated and which scenarios they block. "Is it ready?" at the bottom has the detail, scenario by scenario.
7. **Download CSV.** It writes `<case id>.csv`. "Save editor copy" keeps the page with your values in it, to carry on later.

If you close the page before saving, the editor offers your values back next time, in the same browser. The downloaded CSV is the record.

## Write a case in a spreadsheet

1. Take the template: `case_template.csv` in the MATLAB zip, or **Case → Download template** in the web app.
2. Keep every row, in order. Do not change the `section`, `key`, `label` or `unit` columns: the software refuses a unit that differs from the template rather than converting it.
3. Fill `value` (and `lo`, `hi`, `level` where the row allows them). Replace the `note` with where your number came from; start it with `UNCONFIRMED` if nobody has confirmed it.
4. Open the file in the case editor ("Open a CSV…") and read its first sentence before you run.

| Column | What you write |
|---|---|
| `value` | the number, in the unit shown; blank means not stated |
| `lo`, `hi` | only where the row allows a range: the spread; both or neither |
| `level` | requirements only: the per cent of cases it must hold for; blank takes 99.73, listed as assumed |
| `note` | where the number came from; `UNCONFIRMED` first if it is a stand-in |

## Check a case from a terminal

<!-- since P1 -->
```
adcs case check mycase.csv
```

1. Read **the format** part first: anything the software would refuse, by line and key.
2. Then **what the case says**: requirements written, inputs stated and unstated, blanks that take the reference value, levels assumed, values marked UNCONFIRMED.
3. Then **what can run**: each scenario, "can run" or "blocked — needs …", and which of your requirements it judges.

## Change a case

1. Change the CSV, or the editor copy, never anything inside the software.
2. Open it again. The software files every version separately, by the hash of its content, so every earlier result stays beside the exact version it ran on.

## If it goes wrong

| You see | It means | Do |
|---|---|---|
| "Not ready: 2 rows break the case format" | a value is not a number, a range is incomplete, a level is out of 0–100 | follow the links in "Is it ready?" to each row |
| "That file's header is not the case format" | the file is not an `adcs-case/1` CSV, or a column was renamed | start from the template |
| "unit … the format says …" | a row's unit was changed | put the template's unit back and convert your number |
| a scenario is "blocked — needs mass.imax" | a *stated* row is blank | fill it, or choose a scenario that does not need it |
| the format lacks a key you need | the format belongs to the software | ask for it with any node form, choosing "Something else" ([Asking for changes](04_asking_for_changes.md)) |
