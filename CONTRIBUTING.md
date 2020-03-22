# Contributing

Thanks for showing an interest in `hmm`! This document aims to be a guide for
making contributions to `hmm`, including the philosophy behind the tool and
important concepts in the code.

## Glossary

- **`.hmm` file**: any file that contains `hmm` entries, referred to as a `.hmm`
  file because the default file name is `.hmm`.
- **entry**: an entry is a line in a `.hmm` file. Entries are represented as CSV
  with 2 columns: an RFC3339 datetime and a JSON encoded string message. The
  messages are JSON encoded in order to make them single lines.

## Philosophy

`hmm` is for jotting down in-the-moment thoughts while you're at your terminal.
If you're running a build or some tests and have a few minutes, talk a little
about the problem you're working on. Leave tips to future you. Vent. We all have
those moments where we're at, cursor in terminal, waiting for something to happen,
and in those moments I like to write down a thought. That's what `hmm` is about.

### Multiple binaries

I decided early on that I don't want one binary that handles everything through a 
dizzying array of flags. I want single-purpose binaries that take only the flags
that are relevant to them, no more no less.

If you're adding functionality you want to expose to a user and it doesn't neatly
fit in to an existing binary, you shouldn't feel afraid adding a new binary.

Here are the existing binaries and what they are for:

- **`hmm`**: anything to do with composing entries should go in here, but beware
  that it should always be possible for a user to compose a plain text entry
  through a naked invocation to `hmm`, e.g. `hmm I wonder if I could fix...`.

- **`hmmq`**: the querying binary. Its main function is being able to quickly
  find time ranges by taking advantage of the fact a `.hmm` files is always
  sorted lexicographically and binary searching. All the other stuff in it is
  convenience.
  
- **`hmmp`**: the outputting binary. This binary reads from stdin and formats
  entries based on a Handlebars template passed in the `--format` flag. There's
  some overlap here with `hmmq` as `hmmq` also takes a `--format` flag, but I
  didn't want everyone to always have to pipe to `hmmp`. It's a power-user binary
  for people who want to do complex slicing and dicing on their `.hmm` files
  outside of `hmmq`. Most users will just use `hmmq`.

- **`hmmdg`**: data generation for benchmarking. It is expected no users will ever
  need to run this binary unless they want to reproduce the benchmarks on their
  own machines. Useful for developers for seeing how their features measure up
  against other features, though.

### Cross platform

`hmm` works on Linux, Mac and Windows and that's how it's always going to be.

## Formatting

All code should be formatted with `cargo fmt`. There is a check for this in
CI so you can't submit a PR if your change hasn't been `cargo fmt`ed.

    rustup component add rustfmt
    cargo fmt

## Lint

All code should have no warnings from `clippy`, Rust's beloved linter. Again,
there is a CI check for this.

    rustup component add clippy
    cargo clippy -- -D warnings

## Testing

All code should be tested. I know, it doesn't have 100% coverage at the
moment, but that's no excuse for new code.

We have both unit tests and integration tests. Unit tests live inside of
the module being tested, integration tests live inside the binaries being
tested. Have a look at existing tests to get an idea of what's required.

## Git/Github Workflow

This is our preferred process for opening a PR on GitHub:

- Fork this repository
- Create a branch off of `master` for your work: `git checkout -b my-feature-branch`
- Make some changes, committing them along the way
- When your changes are ready for review, push your branch: `git push origin my-feature-branch`
- Create a pull request from your branch to `hmm/master`
- No need to assign the pull request to anyone, we'll review it when we can
- When the changes have been reviewed and approved, someone will squash and merge for you
