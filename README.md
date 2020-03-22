[![Build status](https://github.com/samwho/hmm/workflows/Build/badge.svg)](https://github.com/samwho/hmm/actions)
[![Crates.io](https://img.shields.io/crates/v/hmmcli.svg)](https://crates.io/crates/hmmcli)

`hmm` is a small command-line note taking app written in Rust. Entries are
written in plain text and indexed by the time they were written.

`hmm` is inspired by [jrnl][1], except with a different use-case in mind.
Where `jrnl` excels at a journaling use case, where users to can entries
with arbitrary times and the file format is human-readable, `hmm` only
allows you to add an entry at the current time and has a machine-readable
format that's optimised for fast time-based querying.

* [Comparison to jrnl](#comparison-to-jrnl)
* [Installation](#installation)
    * [Arch Linux (AUR)](#arch-linux-aur)
    * [Using cargo](#using-cargo)
    * [From source](#from-source)
* [Usage](#usage)
* [hmm](#hmm)
    * [Writing an entry from the CLI](#writing-an-entry-from-the-cli)
    * [Writing an entry to a different .hmm file](#writing-an-entry-to-a-different-hmm-file)
    * [Writing long-form entries in your EDITOR](#writing-long-form-entries-in-your-editor)
* [hmmq](#hmmq)
    * [Listing your entries](#listing-your-entries)
        * [Show the most recent 10 entries](#show-the-most-recent-10-entries)
        * [Show the frst 10 entries](#show-the-frst-10-entries)
        * [Show entries on a specific day](#show-entries-on-a-specific-day)
        * [Show entries on a given year](#show-entries-on-a-given-year)
        * [Count entries in a given year](#count-entries-in-a-given-year)
        * [Show all entries from a given date](#show-all-entries-from-a-given-date)
        * [Show a random entry](#show-a-random-entry)
    * [Formatting entries](#formatting-entries)
* [hmmp](#hmmp)
* [Benchmarking](#benchmarking)

# Comparison to `jrnl`

Features `jrnl` has that `hmm` doesn't:

- Encryption.
- Ability to add entries at arbitrary points in time.
- In-built notion of tags.
- In-built notion of starring.
- Ability to edit entries.
- Ability to parse English dates/times, e.g. "yesterday" and "2 weeks ago."

Features `hmm` has that `jrnl` doesn't:

- Unambigous date-format (RFC3339).
- File-format optimised for searching by time.
- Ability to format entries however you want.
- No external dependencies.
- Lots of flexibility.

If you need to add entries at times in the past, or you need encryption, or
you need your file format to be purely plain text, or you need to edit entries
after they're written, `hmm` isn't for you. Other than that, I believe `hmm`
can be made to work exactly how you want it to.

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

`hmm` is split in to three binaries: `hmm`, `hmmq` and `hmmp`.

- `hmm` is for writing new entries via the CLI.
- `hmmq` is for querying entries by time and content.
- `hmmp` is for printing entries if you want to use tools other than
  `hmmq` to query them.

# `hmm`

## Writing an entry from the CLI

    hmm hello world

This will write an entry to the default `.hmm` file location, which is in
your home directory.

## Writing an entry to a different `.hmm` file

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

The editor variable can be arbitrarily complex, the only thing to keep in mind
is that `hmm` will call it with a temporary file as the last argument. It will
read the contents of that temporary file after your editor command exits
successfully. If your editor does not exit successfully, nothing is written to
your `.hmm` file.

# `hmmq`

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

    hmmq --format $'╭ {{ color "blue" (strftime "%Y-%m-%d %H:%M" datetime) }}\n{{ indent (markdown message) }}╰─────────────────"

The keen reader will notice the `$` before the format argument. This is a bash
quirk. Without it, the `\n` inside the format argument will print literally
instead of being interpreted as a newline.

# `hmmp`

If you want to use other tools to filter through your `.hmm` file, that's completely
file and even encouraged. The `hmmp` tool exists to let you pipe filtered `.hmm` file
contents and have it formatted how you want it.

The following two commands are equivalent:

    tail -n 10 ~/.hmm | hmmp
    hmmq --last 10

As are the following two:

    tail -n 10 ~/.hmm | hmmp --format "{{ message }}"
    hmmq --last 10 --format "{{ message }}"

# Benchmarking

There's a script in the repository root called `bench.sh` that shows the methodology
behind the following table if you're interested.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/hmmq --path /tmp/out --random` | 13.5 ± 0.8 | 11.9 | 15.4 | 1.00 |
| `target/release/hmmq --path /tmp/out --last 10` | 15.0 ± 0.8 | 12.8 | 17.1 | 1.11 ± 0.09 |
| `target/release/hmmq --path /tmp/out --first 10` | 13.6 ± 1.0 | 9.0 | 16.2 | 1.01 ± 0.09 |
| `target/release/hmmq --path /tmp/out --start 2019 --first 10` | 16.8 ± 0.8 | 15.3 | 19.2 | 1.24 ± 0.09 |
| `target/release/hmmq --path /tmp/out --end 2019 --last 10` | 18.8 ± 0.9 | 16.4 | 21.4 | 1.40 ± 0.10 |
| `target/release/hmmq --path /tmp/out --start 2019-01 --end 2019-02` | 325.6 ± 11.9 | 309.9 | 379.9 | 24.11 ± 1.65 |
| `target/release/hmmq --path /tmp/out --start 2019 --end 2020 --count` | 346.6 ± 13.6 | 336.7 | 427.6 | 25.67 ± 1.79 |
| `target/release/hmmq --path /tmp/out --start 2019-01 --end 2019-06 --contains lorum` | 232.3 ± 5.2 | 226.4 | 262.7 | 17.21 ± 1.07 |
| `target/release/hmmq --path /tmp/out --start 2019 --end 2020 --regex "(lorum\|ipsum)"` | 565.3 ± 13.3 | 548.1 | 622.1 | 41.87 ± 2.62 |

[1]: https://jrnl.sh/
[2]: https://rustup.rs/
[3]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
[4]: https://handlebarsjs.com/
