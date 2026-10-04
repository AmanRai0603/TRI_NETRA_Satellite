# Test, deliver, accept, ship

**In one line:** each group's sealed release is tested and delivered by the developer side, in five waves in dependency order; its lead accepts the delivery in the group app; what ships says, group by group, who accepted which version, and which group goes visibly UNCONFIRMED and why (`tools/delivery.py`, `docs/RELEASE_PLAN.md` P12).

## Say it simply

A kitchen sends each dish to the table it was ordered by. The waiter brings it with a card: what is in it, that it was tasted, how to taste it yourself. The person who ordered tastes it and signs the card. Dishes go out in order: the soup before the main course that comes with it. Nothing leaves the kitchen unlabelled: a dish nobody has signed for goes out marked so.

**Where the story lies:** a lead's signature is not a tasting of every node. It says the group ships as sealed; a node sealed UNCONFIRMED stays UNCONFIRMED, with its reason, after the lead accepts.

## The loop, for one group

| Step | Who | Where | What it leaves |
|---|---|---|---|
| Seal | the lead | group app, **Release → Seal** | `releases/<group>-<version>.tnrel` |
| Deliver | the developer side | `python3 tools/delivery.py deliver DIR GROUP` | `deliveries/<group>-<version>.delivery.json`, `.md` and `.test-app.html` |
| Accept | the lead | group app, **Release → Deliveries → Accept** | a signature `accepted` in the group file, naming the release's fingerprint and the delivery's |
| Ship | the developer side | `python3 tools/delivery.py ship DIR --out shipping.json` | the shipping record, JSON and Markdown |

**Delivering checks, in order:** the release passes `tools/group.py verify`; every group of an earlier wave that has a release is delivered first; the merged design (`design.tndb`) holds it; the repository's generated code is this release's wiring (the functions its nodes state, and every computing row's function, inputs and test vectors, are those of `design/groups/`), else it says to wire, generate and test first (`tools/groupcode.py`, the coordinator skill); the group's tests are run (`cargo test -p adcs-groups`: the generated Rust reproduces the interpreter and every node's own test vectors) and recorded pass or fail; the group's test app is built.

**Accepting is refused** to anyone but the lead, for a version not sealed or not delivered, for a delivery whose tests failed, and twice for the same version. An acceptance that names another delivery than the one now in `deliveries/` (delivered again since) does not count.

## The five waves

| Wave | Groups | Why in this order |
|---|---|---|
| A · inputs | `case`, `dyn`, `env` | everything reads the case, the satellite's dynamics and the environment |
| B · hardware | `act`, `sens` | read A; feed navigation, control and FDIR |
| C · GNC | `ctl`, `fdir`, `gdn`, `nav` | read B |
| D · system | `catalogue`, `design`, `fsw`, `kpi`, `pnt` | read C; the design loop sizes every subsystem; the KPI closures read everything |
| E · verification and business | `business`, `hils`, `lab`, `oils`, `risk`, `vv` | read D |

Each group's wave is in `design/groups.toml` (`wave`), checked by `tools/groups.py`. `--out-of-order REASON` delivers ahead of an earlier wave, and the delivery records the reason.

## How it is proven

`tests/test_delivery.py` rehearses the whole of P12 on a design folder seeded and carried over as the team gets it, with stand-in leads (test people, in a scratch folder, never in the real design): wave by wave, every group is sealed in the group app's own code (`tests/js/waves.test.mjs`), delivered (wave A with its tests run), and accepted, but for one group left unaccepted. It holds that a wave cannot be delivered before the one it reads is sealed, that someone who is not the lead cannot accept, that the group left unaccepted ships UNCONFIRMED with why, that an acceptance of a delivery written again since no longer counts, and that the merged design holds all 20 groups and the engine's inputs.

## Where it stands

No group lead has been named yet, so no group has sealed a release: all 20 groups would ship UNCONFIRMED today, each named, as the owner's decision allows (`docs/RELEASE_PLAN.md` §9). As each lead seals, the developer side delivers and the lead accepts; `python3 tools/delivery.py status DIR` shows it, wave by wave.

| Waits on | For |
|---|---|
| a lead named for each group | sealing, then accepting |
| the owner | the decision to ship any group not accepted by release as UNCONFIRMED (suggested), or to wait |
