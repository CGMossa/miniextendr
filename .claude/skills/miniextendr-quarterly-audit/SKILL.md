---
name: miniextendr-quarterly-audit
description: "Use for the quarterly maintenance pass on this repo: running scripts/skill-freshness-audit.sh to catch stale paths, symbols and line cites in .claude/skills/*/SKILL.md, and rebasing the two upstream release pins (r-universe macos-libs cranlibs date, r-windows rtools release). Also use before any release tag."
---

# Quarterly skill-freshness and release-pin audit

The Claude Code skills under `.claude/skills/<slug>/SKILL.md` cite file paths,
symbols, and line numbers that drift as the code evolves. Run
`bash scripts/skill-freshness-audit.sh` **once a quarter** (and repair drift in
the same pass — source wins, fix the SKILL.md). It flags, per skill: missing
cited paths (BLOCKING, exits non-zero so it can gate CI), symbols that grep
finds nowhere in the repo (WARN), and out-of-range `file.rs:NNN` line cites
(WARN). The script's header documents its known false-positive modes
(illustrative placeholders, R functions that look like paths, generated/
gitignored artifacts, scaffolded-package layout paths). CLAUDE.md ↔ skill
contradictions are not auto-detected — eyeball any skill that restates a
CLAUDE.md fact when triaging.

The same quarterly pass also rebases two upstream release pins (folded from
#596): the `r-universe-org/macos-libs` `cranlibs-everything.tar.xz` date
pinned in `minirextendr/inst/templates/r-release.yml` (Gotcha 6) and
`docs/RELEASE_WORKFLOW.md`, and the r-windows rtools release number in
`docs/CRAN_COMPATIBILITY.md`'s Layer 3 table. Also force-rebase both before
any release tag, and smoke-test a bumped tarball URL via a real CI run
(download + extract paths drift upstream).
