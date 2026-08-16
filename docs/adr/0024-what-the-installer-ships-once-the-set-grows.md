# ADR 0024: What the installer ships once the package set grows

**Status:** Accepted  
**Date:** 2026-08-16  
**Decision owner:** Repository maintainer  
**Governing principles:** [`FOUNDATIONS.md`](../principles/FOUNDATIONS.md), [`CONSUMER-CONTRACT.md`](../principles/CONSUMER-CONTRACT.md), [`CAPABILITY-AND-WORKFLOW-BOUNDARIES.md`](../principles/CAPABILITY-AND-WORKFLOW-BOUNDARIES.md)

## Context

ADR 0021 item 7 fixed the shipped set at `reuse-evidence-capture` alone and pre-registered exactly this re-decision: `CONSUMER-CONTRACT.md` §4's requirement that "a future removal or rename of an installed package must define how stale assets are detected" is "therefore decided now for a set of one, and re-decided when the set grows." Its review trigger says the same: "Reopen item 7 when the second project-owned package is authored, which is when the stale-asset rule first faces a real set change." #41 authors `reuse-evidence-review` and ships it through this installer, so the trigger has fired.

### Every part of the installer is an enumeration where a rule belongs

The package name is written into each of five separate places. `SHIPPED_FILES` carries three entries whose relative paths each spell it (`src/skill_install.rs`:33–:48). `DISCOVERY_LINK` and `DISCOVERY_TARGET` are singular constants naming that one package (`:50`, `:51`). `SKILL_DIRECTORIES` spells the package name in three of its five entries — the package root and its two subdirectories — alongside the shared `.claude` and `.claude/skills` roots (`:52`–`:58`). `SkillInstallOutcome` holds one `discovery_link` field (`:64`–`:68`) and prints one link line under a plural heading (`:95`–`:106`). `Cargo.toml`:13 names the package again in the published file set.

Growing the set by editing five enumerations works exactly once. The third package reopens the same question, which is the outcome ADR 0021 item 7's own falsification clause anticipated and this ADR exists to prevent.

### The stale-asset obligation cannot be discharged by remembering

`CONSUMER-CONTRACT.md` §4 requires that a removal or rename "define how stale assets are detected rather than strand silent, conflicting instructions." The cost is concrete and asymmetric: a `SKILL.md` for a package this project no longer ships — or a reference document left behind after being dropped from a package it still ships — is a set of instructions an agent will follow, naming commands the installed binary no longer has. It is not stale documentation a human skims and discounts; it is an operational surface under §4's own framing that agent skills "are part of the operational product, not incidental documentation."

Nothing in an adopting repository records that a package was ever installed, and nothing may be added that does. The installer "creates no cache or index" (`README.md`:390), ADR 0020 refuses "a durable index, cache, or derived case list of any kind," and `FOUNDATIONS.md` §12 refuses control records as a second domain. So detection has to be computed from state that already exists: the names present in the target tree, compared against a set the running binary carries.

That leaves two candidate carriers for "not shipped," and only one of them cannot be forgotten. A hand-maintained list of retired names records each removal explicitly, and is silently wrong the first time a maintainer removes a package without updating it — which is precisely the silent strand §4 forbids, produced by the mechanism meant to prevent it. A reserved name prefix is derived from state that the removal itself changes: dropping the package from `.claude/skills/` is what makes its name stale, with nothing else to remember. `FOUNDATIONS.md` §12's instruction to "generate routing, status, hashes, and projections mechanically where possible" points the same way.

### The prefix is already unoccupied and already de facto this project's

`design/v0.1-scope-and-acceptance.md` §2 names four project-owned packages and every one is prefixed `reuse-evidence-`. The mounted upstream tree does not reach into that prefix: `skill-evidence` 0.12.0 ships `method-gap-research-status`, `skill-evidence-capture`, `skill-evolution`, and `skill-evolution-status` (`assets/skills/`), and the installer already "touches neither upstream operator-package names nor their discovery links" (`README.md`:390). Claiming the prefix therefore costs nothing that exists, and it is what makes a mechanical membership rule available at all.

### The published file set is already written for a namespace

ADR 0021 item 4 says the explicit `include` covers "the shipped `.claude/skills/reuse-evidence-*` subtree" — the namespace, not one package. `Cargo.toml`:13 spells one package instead, which followed ADR 0021 item 7's set of one rather than its item 4. Cargo's `include` accepts a glob inside a path component: on cargo 1.93.0 a probe package with `include = ["/.claude/skills/reuse-evidence-*/**/*"]` listed both matching packages including a nested `refs/` file and dropped a non-matching sibling package (verified 2026-08-16). So the manifest can be made to say what item 4 already says, and nothing about the narrow named file set needs re-deciding.

## Decision

**The shipped set is defined by a reserved package-name prefix rather than enumerated, and every part of the install spans that set.**

### 1. Membership is the `reuse-evidence-` prefix under `.claude/skills/`

The **shipped set** is every package directory in this repository's `.claude/skills/` whose name begins with `reuse-evidence-`. Authoring a package there is the act of shipping it. The **shipped paths** are what those packages contribute to a target: item 2's compiled file table, the directories containing those files, and the one discovery link per package that item 7 of this decision carries forward. The set names the packages; the paths are what an install compares against. There is no second list, no per-package shipping decision, and no rule to restate when the set reaches three. A package not yet meant to ship does not yet live in the namespace.

This item and item 4 together replace ADR 0021 item 7.

### 2. The compiled byte table is checked against the rule, not trusted as it

`include_bytes!` needs literal paths, so an explicit table remains. It is not the authority on membership: the implementing slice must check mechanically that the compiled table covers exactly the files under `.claude/skills/reuse-evidence-*` in this repository — naming none that is gone and missing none that is present — so a package or file added to the namespace cannot silently fail to ship, and one removed from it cannot silently keep shipping. ADR 0021 item 3 kept the evolution gate's target and the shipped bytes from diverging by making them one file; this keeps the *set* from diverging the same way, by making the enumeration a derived claim that something checks rather than a second place to maintain.

### 3. The prefix is reserved in an adopting repository

Entries named `reuse-evidence-*` within the target's `.claude/skills/` and `.agents/skills/`, and the contents of those entries, are this project's to write, detect, and report. The installer already creates those two shared directories when absent, as `create_dir_all` on each shipped file's parent and on the discovery link's parent (`src/skill_install.rs`:282, `:614`), and separately refuses when any path on its fixed directory list is occupied by a symbolic link or a non-directory (`:52`–`:58`, `:59`–`:60`, checked path by path at `:329`–`:336`). Enumerating their entries is new behaviour this decision requires: nothing in the installer reads a directory listing today, and stale detection cannot be done without one. No entry whose name falls outside the prefix is inspected, reported, written, or removed — apart from the installer's own transient paths, which item 4 excepts — so upstream operator packages, their discovery links, and any package the adopter authored under any other name remain untouched. This is what makes item 4 below sound: detection is confined to a namespace whose entire content this project is responsible for.

Reserving a name prefix inside someone else's repository is a new obligation on the adopter, so it is recorded where an adopter reads obligations rather than only here. `CONSUMER-CONTRACT.md` §6 gains one: "not authoring its own skill packages under a package-name prefix the tool reserves for installed assets." That amendment is part of this decision and accepted with it, because an ADR may not place a consumer obligation beneath the authority that states them.

### 4. A stale asset is any path under the reserved prefix the running binary does not ship

At install, the installer walks the reserved prefix in the target — the `reuse-evidence-*` entries of `.claude/skills/` and `.agents/skills/`, and the contents of those package directories — and treats as stale every path there that is not one of item 1's shipped paths. A shipped file, the directory holding it, and a shipped package's discovery link are never stale, which is what keeps a self-install write-free under item 7 of this decision.

**The installer's own transient paths are excepted, and the exception is load-bearing.** Where a write is staged at all, it is staged through a temporary sibling: `temporary_path_for` puts a `.<name>.<uuid>.tmp` next to its destination (`src/lib.rs`:588–:594). A *new* shipped file is staged that way and published by hard-link-then-unlink (`:527`–`:541`); a *replaced* one is staged the same way and published by rename (`:569`–`:580`); a *replaced* discovery link is staged in `.agents/skills/` and published by rename on Unix or remove-then-recreate on Windows (`src/skill_install.rs`:633, `:649`–`:651`, `:726`–`:753`); a *first-time* link is created straight at its destination with no temporary at all (`:609`–`:628`). The publication mechanism varies by path and platform; what this exception turns on is only where a temporary sits while one exists.

Those two staged locations land on opposite sides of the prefix, and the exception has to cover both. A shipped file's temporary sits inside a shipped package directory, so it is under the reserved prefix and is not a shipped path; the discovery link's temporary takes the same leading dot in `.agents/skills/`, so its name begins outside the prefix and item 4's walk never reaches it. Without the exception the first would be classified stale — item 5 would then refuse every later install because of it while equally forbidding the installer to remove it, wedging the adopter behind a refusal only a manual deletion could clear — and the second would be a write item 3 forbids.

The installer's transient paths are therefore its own to write and to remove, are never reported as stale, and are outside both item 3's prefix restriction and item 5's no-removal rule. That removal happens within the run that staged them, including on their own error path. A process killed outright leaves one behind, and no later run reuses or sweeps it, because each temporary carries a fresh identifier and is created exclusively (`src/lib.rs`:583, `:593`). The leftover is the accepted price of the exception, and it is recorded as a cost below rather than resolved here.

One rule covers both granularities, and it has to. A directory named for a package no longer shipped is stale; so is a file left inside a package still shipped, after that file was dropped from it. Both strand the same thing — instructions naming what the binary no longer supports — and a rule catching only the coarser one would argue past the reason given for having it at all. `CONSUMER-CONTRACT.md` §4's own words name a package, but its stated harm is the stranded instruction, and an orphaned reference document is one.

Nothing is written to record any of this and nothing is read except the paths themselves: the reserved prefix plus the compiled shipped paths is the whole mechanism, and it requires no manifest, receipt, index, or version file in the adopting repository.

A rename is a removal and an addition, and nothing is migrated: no move, no content transplant, no attempt to carry local modifications across. The old path goes stale and the new one installs.

### 5. A stale asset refuses the install, and the installer never removes it

Stale paths join the conflict set: a run that finds one refuses, writes nothing, and names every stale path.

`--force` does not resolve them. `--force` authorizes replacement of content the installer ships, and it ships nothing for a path it has retired — with no shipped bytes to compare against, the installer cannot distinguish a pristine retired asset from one the adopter modified, and deleting it under a flag that means "replace" is the silent overwrite of a locally modified asset that §4 forbids one step removed. Comparison before write is this installer's entire safety property; where comparison is undefined the honest act is to name the path and stop.

The resolution is the human's: delete the stale path, or move it outside the reserved prefix to keep using it. Both are one command, and the second is an honest description of what it now is — an asset this project no longer ships, kept somewhere this project does not own. This is the refusal shape the installer already uses for a path it cannot replace atomically, which names the paths and asks the human to remove them (`src/skill_install.rs`:232–:247).

### 6. Atomic refusal spans the whole shipped set

One install is one operation over the whole set. A content conflict or a stale asset anywhere refuses the entire install, writes nothing, and names the complete conflict set across every package in one refusal, rather than installing the packages that happen to be clean. This is the reading that preserves §4's "refuse atomically on conflict," and it keeps the guarantee an adopter already has from weakening as the set grows. Reporting likewise covers every targeted path across the set; its exact spelling stays a design concern under `CONSUMER-CONTRACT.md` §1 and belongs to the implementing issue, as ADR 0021 item 1 recorded for the command name.

### 7. ADR 0021 carries forward, restated for a set

- **A distinct command** (item 1) — the installer is still not a subcommand of `skills`.
- **The mechanic is this crate's** (item 2) — no `skill_evidence::assets`, no upstream change requested for this consumer, no shared installer crate.
- **One copy of each shipped package** (item 3) — `.claude/skills/reuse-evidence-*/` is both the live package under the skill-evolution gate and the embedded shipped source. There is no `assets/` mirror for any member of the set.
- **Self-install is an ordinary install** (item 5) — now for every shipped package at once. ADR 0021 item 3 makes each embedded copy the file it was embedded from, so the whole set compares equal and the run is a write-free no-op.
- **`.agents/skills/` is shared by package name** (item 6) — one link per shipped package, each relative to its own directory as `../../.claude/skills/<name>`. Item 3 *of this decision* extends this project's *detection* reach to the reserved prefix; its *write* reach is still exactly the names it ships, plus the transient paths item 4 excepts.
- **ADR 0021's own "does not authorize" list** — carried forward intact rather than restated below, so the two cannot drift. The installer still writes skill files and discovery links only, and writes no user-local configuration of any kind.

### 8. The crate's named file set follows the same rule

`Cargo.toml`:13's one-package entry becomes `/.claude/skills/reuse-evidence-*/**/*`, which is what ADR 0021 item 4 already describes and what the set of one had narrowed in the implementation alone. Authoring a package inside the namespace then ships it in the crate as well as through the installer, with no manifest edit, which keeps items 1 and 2 of this decision from splitting into two different membership rules.

### 9. ADR 0021's rejected alternatives stay rejected

The set growing changes none of their reasons, and a reader who expects it to is the reason this is stated:

- **Installing packages this project did not author**, including upstream operator packages, remains `skills evidence install`'s. Growth is growth of this project's own authored set; it does not make the installer a general host.
- **A host-mounting API** letting other crates rename and embed this command tree remains out of scope under `design/v0.1-scope-and-acceptance.md` §3. Item 3 reserves a name prefix inside a target repository; it does not expose a mounting surface to anyone.
- **A shared installer crate extracted with `skill-evidence`** remains rejected under `FOUNDATIONS.md` §3 and ADR 0008 item 6. A second package installed by the *same* installer is not a second independent consumer of the install responsibility — `FOUNDATIONS.md` §5 counts independently accepted consumer needs, and this is one need serving more assets. If anything the pressure moved further away, since the mechanic now generalises within this project without borrowing.

This does **not** authorize:

- removing, pruning, or rewriting any path the installer does not ship, a stale one included. Three things are unchanged: replacement of shipped content under `--force`, the existing removal of an owned discovery path on a platform without symbolic-link support (`src/skill_install.rs`:636–:642), and the installer's handling of its own transient paths under item 4;
- reading, writing, or trusting any installed manifest, receipt, lockfile, or version marker in an adopting repository;
- inspecting, reporting, writing, or removing any entry outside the reserved prefix, other than the installer's own transient paths under item 4;
- migrating content from a removed or renamed package into its successor;
- installing any package this project did not author, a host-mounting API, or a shared installer crate, per item 9;
- authoring `reuse-evidence-review` or changing the installer, both of which are separate slices under #41;
- any principle amendment beyond `CONSUMER-CONTRACT.md` §6's single added obligation named in item 3;
- anything ADR 0021's own "does not authorize" list refuses, which item 7 of this decision carries forward intact rather than copying here.

## Consequences

### Positive

- The third package costs no decision and no manifest edit. Membership, the discovery links, the created directories, the published file set, and stale detection all read from one rule.
- The stale-asset obligation is discharged by a mechanism that cannot be forgotten, because the removal that creates the staleness is the same act that makes the name detectable.
- `CONSUMER-CONTRACT.md` §4's atomic refusal is strengthened rather than diluted by the set growing: an adopter who could previously lose nothing to a partial install still cannot.
- No new artifact appears in an adopting repository. Detection needs no receipt, and `FOUNDATIONS.md` §12 and ADR 0020's refusal of durable indexes stay intact — the tension the issue named as a falsifier does not arise.
- The single-copy rule survives the set growing, and gains a mechanical check that the shipped table has not fallen behind the tree.

### Negative and risks

- The prefix is reserved in someone else's repository. An adopter who authors their own `reuse-evidence-anything` package, or who adds a note of their own inside a shipped package's directory, finds the install refusing until they move it. `CONSUMER-CONTRACT.md` §6 now states the obligation, but an adopter who has not read the contract still meets it first as a refusal.
- An older binary run against a repository installed by a newer one reports the newer package as stale, because it cannot distinguish "retired" from "not yet known." The refusal is loud and reversible, but it is a false positive produced by the rule's own mechanism, and it is the price of not maintaining a retired-name list.
- Item 5 makes every set shrink cost a manual deletion in every adopting repository before the next install succeeds. With one adopter that is trivial; the rule was chosen against a set of adopters that does not exist yet.
- A process killed mid-install leaves its temporary under the prefix indefinitely. Item 4 excepts it from staleness, so no later run reports it, none sweeps it, and none reuses it. The alternative — letting it wedge every later install behind a refusal the installer may not resolve — is worse, so the price is an invisible file the adopter can only find by looking.
- Detection fires only when the installer runs. An adopter who installs once and never upgrades keeps stale instructions indefinitely, and nothing in this decision reaches them. That is inherent to an installer with no daemon and no background check, neither of which `FOUNDATIONS.md` permits before a bounded fault-tested need exists.
- Item 1 means authoring a package inside the namespace ships it. A half-finished package committed there reaches adopters on the next release, and the protection is a naming convention rather than a gate.
- The decision is taken while the set is still one. Its first real exercise is #41's growth to two, and no removal or rename has ever happened, so item 5 is still being decided against a case it does not yet face — a narrower version of the same criticism ADR 0021 recorded of its own item 7.

### Operational burden

Nothing changes for an ordinary install: one command, a free preview through the non-force run, no configuration. Adding a package is authoring it in the right directory. Removing one costs the maintainer a namespace-aware release note and each adopter one deletion.

### Compatibility and migration

Nothing recorded changes. No event, schema, marker, or case evidence is touched. The installed asset surface changes shape — more packages, more links, a new refusal class — which is installed-asset behaviour under `CONSUMER-CONTRACT.md` §4 and revisable during `0.x` under §8. The published file set broadens from one named package to the namespace ADR 0021 item 4 already described; nothing has been published, so no consumer can be relying on the current contents. The existing refusal for content conflicts keeps its behaviour and gains stale paths as a second class within the same atomic refusal. `CONSUMER-CONTRACT.md` §6 gains one obligation; no other principle text changes, and nothing already promised to an adopter is withdrawn.

## Alternatives considered

| Alternative | Disposition | Reason |
|---|---|---|
| Re-decide for a set of two, enumerating both members | Rejected | It is what ADR 0021 item 7 already did once, and the third package reopens it a third time. The issue records this as the fallback only if a membership rule could not be stated without enumeration; it can. |
| A hand-maintained list of retired package names | Rejected | A retirement the maintainer forgets to list produces exactly the silent stranded instructions `CONSUMER-CONTRACT.md` §4 forbids, and `FOUNDATIONS.md` §12 disfavours a hand-authored record where a mechanical one exists. It is genuinely better in one respect — no downgrade false positive — which is why that risk is recorded above rather than dismissed. |
| Detect stale packages only, leaving a file dropped from a shipped package undetected | Rejected | `CONSUMER-CONTRACT.md` §4's words name a package, but the harm it names is the stranded instruction, and an orphaned reference document inside a current package is one. A package-only rule would argue past the reason given for having the rule. Covering both costs no new mechanism, since the installer already knows each shipped package's exact file set. |
| An installed manifest, receipt, or version file in the adopting repository | Rejected | ADR 0020 refuses a durable index or cache of any kind and `FOUNDATIONS.md` §12 refuses control records as a second domain. It would also become a second authority beside the files themselves, which could disagree with them. |
| `--force` removes stale packages | Rejected | The installer has no shipped bytes for a retired name, so it cannot tell a pristine retired package from a locally modified one. Deleting it under a flag that means "replace what differs" is `CONSUMER-CONTRACT.md` §4's silent overwrite of a locally modified asset, reached by a different route. |
| Report stale assets and install anyway | Rejected | The harm is an agent following retired instructions. Installing current instructions beside them does not stop that and produces a repository carrying two generations of the operational product at once. |
| Per-package atomicity: a conflict refuses only its own package | Rejected | It weakens an existing guarantee as a side effect of growth. `CONSUMER-CONTRACT.md` §4's "refuse atomically on conflict" reads over the operation, and a partially applied install is a state no adopter can currently reach. |
| A build script generating the shipped table from the tree | Rejected | New compile-time code generation, an `OUT_DIR` indirection between the gate's target and the embedded bytes, and a second thing to debug — to buy what item 2's check buys with a test. Reconsider if the table becomes large enough that hand-editing it is the actual failure. |
| Record all of this in #41's PRD or in the installer slice | Rejected | `docs/README.md` ranks PRDs and issues below ADRs and an issue closes. ADR 0021 item 7 pre-registered an ADR-level re-decision specifically so the next reader finds it above the issue layer; ADR 0012, ADR 0020, and ADR 0023 rejected the same placement for the same reason. |

## Verification and review trigger

The decision is fit when installing a set of two into an empty target writes both packages and creates both discovery links; when a conflicting local modification in either package refuses the whole install, names it, and leaves the other package untouched; when a directory named for a package the binary does not ship, and equally a file left inside a shipped package after being dropped from it, is named as stale and refuses the install without being removed; when a self-install of the whole set is a write-free no-op; when an install interrupted after staging a temporary leaves it unreported by the next run rather than classified stale; when `cargo package --list` contains this project's sources, every package in the namespace, and no package it did not author; and when authoring a third package requires no edit to `Cargo.toml`, no new constant, and no new rule.

**Falsify item 1** if a package this project authors genuinely must not ship — for instance an internal or experimental package that has to live in `.claude/skills/` under the prefix. The prefix rule would then be forcing a shipping decision the maintainer wants to take separately, and the honest successor is an explicit per-package opt-in, not a fuzzier prefix.

**Falsify item 5** if a real adopter meets a stale refusal and resolves it by deleting more than the stale package — the whole `.claude/skills/` tree, say. That would show the refusal communicates the wrong scope, and removal under explicit authorization becomes the safer answer after all.

**Falsify item 2** if the mechanical check proves impossible to state without duplicating the table it checks.

**Reopen item 4** the first time an older binary reports a newer binary's package as stale in real use, which is the predicted false positive becoming a measured one.

**Falsify item 3** if an adopter has a legitimate reason to author their own package under the reserved prefix, or if the obligation `CONSUMER-CONTRACT.md` §6 now carries proves to be one adopters cannot reasonably discover before meeting it as a refusal. Item 3 is the only part of this decision that binds a third party, and it rests on the least evidence: there is exactly one adopting repository today, and it is this one.

**Reopen item 5** the first time a package is actually removed or renamed, which is when the rule first faces the case ADR 0021 item 7 deferred and this ADR still decides in advance of.

**Park this decision** with the review package if #41's review slice is parked. The set would then never grow, and this would have been bought against a change that did not happen.

## Supersession

None. This decision amends without replacing. ADR 0021 item 7 is replaced by items 1 and 4 of this decision — the shipped set is no longer fixed at one member and is no longer re-decided per set size — and ADR 0021 item 6 is extended, its "each installer owns only the names it ships" becoming, for this project, detection across the reserved prefix with writes still confined to the names it ships.

ADR 0021 item 4 is **not** amended. It already covers "the shipped `.claude/skills/reuse-evidence-*` subtree" — the namespace, not one package — so item 8 of this decision changes only `Cargo.toml`:13, whose one-package spelling followed ADR 0021 item 7's set of one rather than its item 4. ADR 0021 item 4 is carried forward as written.

It amends `CONSUMER-CONTRACT.md` §6 by one obligation, per item 3, because an ADR may not place a new consumer obligation beneath the authority that states them. No other principle text changes.

ADR 0021 is not superseded. Its items 1, 2, 3, 4, and 5 stand unchanged, it remains Accepted, and it remains the record of why this project has its own installer at all.
