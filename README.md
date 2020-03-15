# hmm


[![Build status](https://github.com/samwho/hmm/workflows/Build/badge.svg)](https://github.com/samwho/hmm/actions)
[![Crates.io](https://img.shields.io/crates/v/hmmcli.svg)](https://crates.io/crates/hmmcli)

`hmm` is a small command-line note taking app written in Rust. Notes are
written in plain text and indexed by the time they were written.

`hmm` is inspired by [jrnl][1], though with a slightly different use-case in
mind.

# Comparison to `jrnl`

Features `jrnl` has that `hmm` doesn't:

- Encryption of your notes.
- Ability to add note at arbitrary points in the past.
- In-built notion of "tags."
- In-built notion of "starring."
- Ability to edit entries.
- Ability to parse English dates/times, e.g. "yesterday" and "2 weeks ago."

Features `hmm` has that `jrnl` doesn't:

- Unambigous date-format.
- File-format optimised for searching by time.
- Ability to format entries however you want.
- No external dependencies, the `hmm` binary is standalone.

If any of the features `jrnl` has that `hmm` is missing are essential to your
workflow, `hmm` isn't for you. That said, I am open to feature requests but
very much plan to keep `hmm` focused on the use-case I designed it for: quick
note taking in the terminal with the ability to search later.

# Installation

I plan to upload `hmm` to various package repositories, but until then...

## Arch Linux (AUR)

`hmm` is in the AUR, and can be installed with an AUR helper such as `yay`:

    yay -S hmm-bin

## Using cargo

[Install Rust][2], then run:
    
    cargo install hmmcli

Now the `hmm` and `hmmq` binaries should be available in your terminal.

## From source

[Install Rust][2], [install git][3] then run:

    git clone https://github.com/samwho/hmm
    cd hmm
    cargo install

# Usage

`hmm` is split in to two binaries: `hmm` and `hmmq`. The former is writing
entries, while the latter is for querying them.

## Writing an entry from the CLI

    hmm hello world

This will write an entry to the default `.hmm` file location, which is in
your home directory.

## Writing an entry to a different file

Your `.hmm` file can be located wherever you want, and named whatever you
want.

    hmm --path ~/.notes hello world

## Writing long-form entries in your `EDITOR`

    hmm

Invoked with no arguments, or just a `--path` argument, `hmm` will open your
default `EDITOR` to compose an entry. Savings and quitting that editor will
then write the note to your `.hmm` file. If you don't have an `EDITOR`
configured, you can also pass one as a flag:

    hmm --editor vim

## Listing your entries

    hmmq

By default, this lists all of your entries in a default format in ascending
chronological order. This may not be desired, so there are a bunch of flags
to narrow down what is shown.

### Show the most recent 10 entries

    hmmq --descending -n 10

### Show entries on a specific day

    hmmq --start 2020-01-01 --end 2020-01-02

The `--start` flag is inclusive and the `--end` flag is exclusive, so the
above command will show all entries that were created on the 1st of January
2020.

Dates follow the RFC3339/ISO8601 format, allowing you to omit parts you don't
need. All dates are in your local timezone.

### Show entries on a given year

    hmmq --start 2019 --end 2020

This will show all of your notes from 2019.

### Show entries on a given year in descending order

    hmmq --start 2019 --end 2020 --descending

This will show all of your notes from 2019 but in reverse chronological order.

### Show all entries from a given date

    hmmq --start 2020-02-20

This will print all of your notes from the 20th of February 2020.

### Show a random entry

    hmmq --random

Prints out a random entry. The randomness comes from selecting a random byte
in your `.hmm` file, and as such longer entries are more likely to be picked.
This is a trade-off. Picking entries in a truly random fashion would require
reading the entire file, which is against the philosophy of `hmmq`.

[1]: https://rustup.rs/
[2]: https://rustup.rs/
[3]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git