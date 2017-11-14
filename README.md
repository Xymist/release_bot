# Release Bot

This was inspired by (and owes a chunk of the initial code to) the handy tool
at [Pulls Since](https://github.com/budziq/pulls_since). I needed a Rust
learning project, and writing release notes is tedious, so I thought I'd turn
the command line tool into a self-contained bot for writing them for me.

The eventual plan is for this to be run when a merge to Master is made.
It should generate the release notes from the bugtracker and PRs, cut a release,
trigger and babysit the deploy to live, and notify anyone who cares about a
given issue that it has been dealt with.

Plan:

- Create Release in GitHub with the name of the current Milestone in Zoho
- Trigger deploy process
- Email Admins to request smoke test on success, or to complain on failure
- Email support & clients to update on status of requested features/enhancements and bugfixes

Setup:

You will need a config.toml at the root of this repo:

```
github_token="GITHUB OAUTH TOKEN"
zoho_organisation="ZOHO ORGANISATION NAME"
zoho_authtoken="ZOHO API KEY"

[[repos]]
name="GITHUB REPO, E.G. ABC/XYZ"
base="BASE DEVELOPMENT BRANCH"

[[zoho_projects]]
name="ZOHO PROJECT NAME"
id="ZOHO PROJECT ID"
milestone="CURRENT ZOHO RELEASE MILESTONE"
```