[![Build status](https://github.com/samwho/hmm/workflows/Build/badge.svg)](https://github.com/samwho/hmm/actions)
[![Crates.io](https://img.shields.io/crates/v/hmmcli.svg)](https://crates.io/crates/hmmcli)

`hmm` is a small command-line note taking app written in Rust. Entries are
written in plain text and indexed by the time they were written.

`hmm` is inspired by [jrnl][1], though with a slightly different use-case in
mind.

* [Comparison to jrnl](#comparison-to-jrnl)
* [Installation](#installation)
    * [Arch Linux (AUR)](#arch-linux-aur)
    * [Using cargo](#using-cargo)
    * [From source](#from-source)
* [Usage](#usage)
    * [Writing an entry from the CLI](#writing-an-entry-from-the-cli)
    * [Writing an entry to a different file](#writing-an-entry-to-a-different-file)
    * [Writing long-form entries in your EDITOR](#writing-long-form-entries-in-your-editor)
    * [Listing your entries](#listing-your-entries)
        * [Show the most recent 10 entries](#show-the-most-recent-10-entries)
        * [Show the frst 10 entries](#show-the-frst-10-entries)
        * [Show entries on a specific day](#show-entries-on-a-specific-day)
        * [Show entries on a given year](#show-entries-on-a-given-year)
        * [Show all entries from a given date](#show-all-entries-from-a-given-date)
        * [Show a random entry](#show-a-random-entry)
    * [Formatting entries](#formatting-entries)
    * [Benchmarking](#benchmarking)
        * [Random entries](#random-entries)
        * [Entries from a given random start date](#entries-from-a-given-random-start-date)
        * [A large number of entries on a given random start date](#a-large-number-of-entries-on-a-given-random-start-date)
        * [Printing the whole file](#printing-the-whole-file)

# Comparison to `jrnl`

Features `jrnl` has that `hmm` doesn't:

- Encryption of your entries.
- Ability to add entries at arbitrary points in the past.
- In-built notion of tags.
- In-built notion of starring.
- Ability to edit entries.
- Ability to parse English dates/times, e.g. "yesterday" and "2 weeks ago."

Features `hmm` has that `jrnl` doesn't:

- Unambigous date-format.
- File-format optimised for searching by time.
- Ability to format entries however you want.
- No external dependencies.

If any of the features `jrnl` has that `hmm` is missing are essential to your
workflow, `hmm` isn't for you. That said, I am open to feature requests but
very much plan to keep `hmm` focused on the use-case I designed it for: quick
note taking in the terminal with the ability to search later.

# Installation

No support for Homebrew yet, so Mac users will need to go down the `cargo`
route, but I plan to get it in to the Homebrew repos soon.

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
default `EDITOR` to compose an entry. Saving and quitting that editor will
then write the note to your `.hmm` file. If you don't have an `EDITOR`
configured, you can also pass one as a flag:

    hmm --editor vim

## Listing your entries

    hmmq

By default, this lists all of your entries in a default format in ascending
chronological order. This may not be desired, so there are a bunch of flags
to narrow down what is shown.

### Show the most recent 10 entries

    hmmq --last 10

### Show the frst 10 entries

    hmmq --first 10

### Show entries on a specific day

    hmmq --start 2020-01-01 --end 2020-01-02

The `--start` flag is inclusive and the `--end` flag is exclusive, so the
above command will show all entries that were created on the 1st of January
2020.

Dates follow the RFC3339/ISO8601 format, allowing you to omit parts you don't
need. All dates are in your local timezone.

### Show entries on a given year

    hmmq --start 2019 --end 2020

This will show all of your entries from 2019.

### Count entries in a given year

    hmmq --start 2019 --end 2020 --count

This will show you how many entries you made in 2019.

### Show all entries from a given date

    hmmq --start 2020-02-20

This will print all of your entries from the 20th of February 2020.

### Show a random entry

    hmmq --random

Prints out a random entry. The randomness comes from selecting a random byte
in your `.hmm` file, and as such longer entries are more likely to be picked.
This is a trade-off. Picking entries in a truly random fashion would require
reading the entire file, which is against the philosophy of `hmmq`.

## Formatting entries

`hmmq` makes use of the [Handlebars][4] templating format to determine how entries
are printed to the terminal. Here's an example of a really simple template:

    hmmq --format "{{ datetime }}: {{ message }}"

It's not much to look at, but it shows how the templates look and all of the
variables you have access to inside a template.

`hmmq` offers some helper functions to make your templates look nicer. Here's
the default output format specified explicitly:

    hmmq --format $'{{ color "blue" (strftime "%Y-%m-%d %H:%M:%S" datetime) }}\n{{ indent message }}'

The keen reader will notice the `$` before the format argument. This is a bash
quirk. Without it, the `\n` inside the format argument will print literally
instead of being interpreted as a newline.

## Benchmarking

There's a script in the repository root called `bench.sh` that shows the methodology
behind the following table if you're interested.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/hmmq --path /tmp/out --random` | 15.5 ± 1.5 | 12.9 | 22.0 | 1.03 ± 0.12 |
| `target/release/hmmq --path /tmp/out --last 10` | 16.5 ± 1.4 | 14.7 | 24.3 | 1.10 ± 0.12 |
| `target/release/hmmq --path /tmp/out --first 10` | 15.0 ± 1.0 | 12.9 | 17.7 | 1.00 |
| `target/release/hmmq --path /tmp/out --start 2019 --first 10` | 18.2 ± 1.0 | 15.8 | 21.6 | 1.22 ± 0.10 |
| `target/release/hmmq --path /tmp/out --end 2019 --last 10` | 19.7 ± 1.0 | 16.4 | 22.2 | 1.31 ± 0.11 |
| `target/release/hmmq --path /tmp/out --start 2019-01 --end 2019-02` | 318.5 ± 16.0 | 306.1 | 418.1 | 21.27 ± 1.73 |
| `target/release/hmmq --path /tmp/out --start 2019 --end 2020 --count` | 337.9 ± 11.6 | 328.7 | 394.5 | 22.56 ± 1.63 |
| `target/release/hmmq --path /tmp/out --start 2019-01 --end 2019-06 --contains lorum` | 228.4 ± 12.0 | 217.3 | 282.2 | 15.25 ± 1.26 |
| `target/release/hmmq --path /tmp/out --start 2019 --end 2020 --regex "(lorum\|ipsum)"` | 554.0 ± 25.9 | 529.0 | 684.6 | 36.99 ± 2.92 |

[1]: https://jrnl.sh/
[2]: https://rustup.rs/
[3]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
[4]: https://handlebarsjs.com/
