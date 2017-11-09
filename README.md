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

- Get initial list of repositories from a config file rather than hard coding
- Add ability to get issues from Zoho bugtracker and parse into something useful
- Add ability to connect Zoho issues to PRs by the ticket reference (From the look of the Zoho API this will be a sod. TODO: decide whether I need to keep information about known bugs in SQLite or something)
- Add ability to collect customer issues from Zoho and group/print with customer as heading
- Collate all data and autogenerate full release notes
- Create Release in GitHub with the name of the current Milestone in Zoho
- Trigger deploy process
- Email Admins to request smoke test on success, or to complain on failure
- Email support & clients to update on status of requested features/enhancements and bugfixes
