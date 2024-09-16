# Release Bot

This is a bot to automate the release process for Market Dojo.

The eventual plan is for this to be run when a merge to Master is made.
It should generate the release notes from the bugtracker and PRs, cut a release,
trigger and babysit the deploy to live, and notify anyone who cares about a
given issue that it has been dealt with.

Features (i.e. things this does at this time):

- Fetch all issues and PRs from the given Milestone on GitHub
- Build a Markdown file with the release notes
- Convert the Markdown file to a PDF

Plan (i.e. things this does _not_ do at this time):

- Create Release in GitHub with the name of the current Milestone
- Trigger deploy process
- Email Admins to request smoke test on success, or to complain on failure
- Email support & clients to update on status of requested features/enhancements and bugfixes

Setup:

- Fetch the latest build from the `Releases` section of this repository on GitHub
- Create a GitHub token with the `repo` scope and set it as an environment variable `GITHUB_TOKEN`
- Ensure you know what the milestone number is for the current release
- Ensure you have `wkhtmltopdf` installed, for PDF generation
- Run with `release_bot --milestone <milestone_number>`
- The release notes will be generated in the `releases` directory
