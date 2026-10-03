# The usability session

**In one line:** two or three real members, each alone with the apps for about an hour, try to complete a node and a release without help while one person watches and writes down every place they stop; what they stumble on becomes fixes before 1.0.0 (`docs/RELEASE_PLAN.md` P7).

## Say it simply

You hand someone the apps and a task, and you say nothing. Every time they hesitate, look for a button, read a message twice or ask "what now?", you write it down. That pause is the finding, not their mistake.

**Where the story lies:** a session with three people does not prove the apps are easy for everyone; it finds the worst places. The automated walkthrough (`tests/browser/help.test.mjs`) proves every step can be found by what the screen says. Only people show whether they *do* find it.

## Who and what you need

| | |
|---|---|
| Testers | 2–3 members who have not used the apps: at least one future author and one future group lead. Not developers |
| Observer | One person, who only watches and writes. Not the one who built the apps |
| Computer | Chrome or Edge, Drive for desktop, the Drive pack copied into a test folder (not the real design folder) |
| Time | About an hour per tester, one tester at a time |
| Findings | The "Usability findings" sheet on Drive (one row per finding) |

Before each tester: copy a fresh Drive pack into the test folder, and clear the browser's site data for the apps (so the tours show as for a newcomer).

## The tasks

Read each task aloud, give the tester the card, and then say nothing. If they are stuck for three minutes, write the finding, then give the smallest hint and note it.

**As an author (Node app)**

1. Open the node app and the design folder. Type your name.
2. Find the node `gm_0` ("Dipole available per axis") and open it.
3. Take in what the spec already says about it.
4. Write where its explanation breaks, and fill in its belief record.
5. Make Review show nothing to fix, mark the node ready, and save.
6. Print the node.

**As a second engineer**

7. Open the same node under your own name and sign it as checked.

**As a group lead (Group app)**

8. Open the group app and the group `act`. Add yourself as its lead, and a colleague as owner of the stage `mtq`.
9. Issue a node of your choice to an author.
10. Find out how many of act's nodes are ready, and why `gm_0` is or is not confirmed.
11. As the stage owner, sign stage `mtq`.
12. As the lead, seal act's first release. Say how many nodes went in confirmed.
13. Re-open `gm_0` for its author, then compare the release with the group as it is now.

Done when a tester completes tasks 1–7 (as an author) or 8–13 (as a lead) with no hint.

## What the observer writes

One row per moment, in the findings sheet:

| Column | What goes in it |
|---|---|
| Tester | A, B or C (no names) |
| Task | its number |
| Where | the app, the screen, the field or button |
| What happened | what they did and said, in their words |
| Kind | stopped, wrong turn, misread, asked, gave up |
| Hint given | none, or the hint |
| Severity | blocks the task, slows it, cosmetic |

## After the session

1. Group the findings by where they happened.
2. For each group, decide one fix: the words on the screen, the help beside a field, a tour step, the manual page, or the behaviour.
3. Each fix goes into the apps or the manual (`design/manual/`), with a line in the browser tests where it can be checked.
4. Run the session again with one new tester on the fixed version. Done when nobody needs a hint.
