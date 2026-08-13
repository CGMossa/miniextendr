# Accepted insta snapshot removed its `.snap.new` artifact

## What was attempted

Move the rejected `.snap.new` file to trash after applying its expected diff to
the tracked snapshot and rerunning the focused snapshot test successfully.

## What went wrong

The trash command reported that the file no longer existed, so the chained
format and test commands did not run.

## Root cause

Insta removed the stale `.snap.new` artifact automatically after the tracked
snapshot matched the newly generated output.

## Fix

Do not perform a cleanup operation when the artifact has already disappeared;
rerun formatting and tests as independent commands.
