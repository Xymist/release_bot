# TODO

## Within ReleaseBot

- Switch to async instead of threads
- Use Octocrab for the GitHub API (did not exist when this was first written)
- Better abstraction for writing Markdown tables
- Better CSS for the PDF report
- Once stabilised, switch back to Stable toolchain and remove feature invocation for once_cell_try

## Within Zohohorrorshow

- Switch to async (need async_trait, and possibly removing the use of `Iterator` for the cached retrievals)
- Save access refresh token to avoid re-requesting every run
- Use a tower::Service middleware to handle rate limiting for the client as a whole
