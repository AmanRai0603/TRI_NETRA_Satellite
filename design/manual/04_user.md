# Guide for users
<!-- role: user; id: user -->

**In one line:** you use the software by writing one case (your satellite, its orbit and mission, what its ADCS must achieve) and reading the result it gives back; every node behind a result shows whether it was confirmed by its group or is still UNCONFIRMED.

## Say it simply

You hand in a question (your case) and get back a report (the result). The report names the edition of every chapter it used, and marks anything that was not confirmed.

**Where the story lies:** a report from a book is only as good as the book. Here each number also says how far to trust it: a confirmed node was checked by a second engineer against answers from outside the code; an UNCONFIRMED one was not, and the result says so.

## Now the real thing

- **Your case** is the only thing you edit. The software never guesses a number you left blank: it names the key and stops.
- **The result** is one file holding every number and plot of the run. Opening it runs nothing.
- **Asking for something different** is a node form sent to the developer team, or a word with the group lead who owns the node.
- **Which release.** The software shows which release of each group it contains; the release notes list every node still UNCONFIRMED.

The user manual in full is in `spec/manual/user/`, starting with *Start here*.
