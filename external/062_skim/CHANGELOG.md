# Change Log

## 0.8.2: 2020-06-26

Bug fixes:
- fix skim executable in bash completion
- fix [#291](https://github.com/lotabout/skim/issues/291) hide scroll in when
    content fit
- fix [#308](https://github.com/lotabout/skim/issues/308) hangs on
    initialization

## 0.8.1: 2020-02-23

Feature:

- [#63](https://github.com/lotabout/skim/issues/63) could save to and read
    from history for query and command query via `--history` and `--cmd-history`
- [#273](https://github.com/lotabout/skim/issues/273) inline-info now has
    spinner
- [#276](https://github.com/lotabout/skim/issues/276) new action:
    `if-non-matched` will execute if non of the items matches
- reduce memory footprint
- [#248](https://github.com/lotabout/skim/issues/248) implement `{n}`
    placeholder, used to refer to current items's index(zero based).

Bug fixes:

- [PR #279](https://github.com/lotabout/skim/pull/279) exit gracefully on
    SIGPIPE error. (e.g. Ctrl-C on pipes)
- [#276](https://github.com/lotabout/skim/issues/276) `execute` panic on zero
    results
- [#278](https://github.com/lotabout/skim/issues/278) `NUL` character not
    working in preview command
- handle `print0` correctly in filter mode
- Preview's fields now based on original text, not transformed.
- [#295](https://github.com/lotabout/skim/issues/295) skim not exits
    sometimes (occasionally happens on Ubuntu)

## 0.8.0: 2020-02-23

**Breaking Changes in API**

- `Skim::run_with` now accept a stream of `SkimItem` instead of a `BufRead`.

Feature:
- [#233](https://github.com/lotabout/skim/issues/233) support mouse
    scroll/click event
- [#254](https://github.com/lotabout/skim/issues/254) support `{+}` in preview
    and execute command
- [#226](https://github.com/lotabout/skim/issues/226) support exact match
    combination(e.g. `^abc$`)
- [#216](https://github.com/lotabout/skim/issues/216) support item specific
    preview hook method
- [#219](https://github.com/lotabout/skim/issues/219) support case insensitive
    match

Bug fixes:
- [#252](https://github.com/lotabout/skim/issues/252) Deal with `\b` correctly
- [#210](https://github.com/lotabout/skim/issues/210) exclude current item in
    multi-selection
- [#225](https://github.com/lotabout/skim/issues/225) disable score in filter
    output

## 0.7.0: 2020-01-15

Feature:
- New fuzzy matching algorithm, should be more precise and faster.

Bug fixes:
- [PR #227](https://github.com/lotabout/skim/pull/227)
    Fix `isatty` check on more OS.
- Fix various cases where ANSI code not behave correctly.

## 0.6.9: 2019-09-22

Bug fixes:
- [PR #171](https://github.com/lotabout/skim/pull/171)
    search with more than one multi-byte condition would crash skim.
- [#194](https://github.com/lotabout/skim/issues/194)
    color not working with ag
- [#196](https://github.com/lotabout/skim/issues/196)
    `+` in execute expression was eaten by skim
- bind `Home` key to `begining-of-line` by default.
- [#192](https://github.com/lotabout/skim/issues/192)
    Prompt was eaten in shell completion
- [#205](https://github.com/lotabout/skim/issues/205)
    tabstop of selection was initialized to `0`, now to `8`.
- [#207](https://github.com/lotabout/skim/issues/207)
    color config not working for header

## 0.6.8: 2019-06-23

Feature:
- New action: `if-query-empty`, `if-query-not-empty`. Execute actions on
  certain query conditions.
- New action: `append-and-select` allows you to append current query to the
    item pool and select it. It would help to turn skim into a tag manager
    where new tags could be added to the candidate list.

Bug fixes:
- Fix #188: crates.io breaks on 0.6.7
- Fix: `run_with` will break if called multiple times from the same process.
- Update nix to 0.14

## 0.6.7: 2019-05-31

Feature:
- Refer to query and command query with `{q}` and `{cq}` in preview command.
- Support fzf's theme strings, e.g. `bg+` for current line's background.
- Support customizing styles of query strings.

Bug fixes:
- skim would crash if multiple CJK items are matched in an `OR` query.
- SKIM_DEFAULT_COMMAND not correctly recognized in `sk-tmux`
- UI responses are slow on large input

## 0.6.6: 2019-04-03

fix #158: preview window not udpate correctly.

## 0.6.5: 2019-04-01

Bug Fixes:

- #155: screen is not fully cleared upon resize
- #156: preview dies on large chunk of input
- #157: cursor overflow on empty input
- #154: reduce CPU usage on idle
- wrong matches on empty input lines

## 0.6.4: 2019-03-26

Fix: #153 build fail with rust 2018 (1.31.0)

## 0.6.3: 2019-03-25

Feature:
- support action: `execute`
- support action chaining
- preview window actions: `toggle-preview-wrap`, `preview-[up|down|left|right]`, `preview-page-[up|down]`
- support `--filter` mode, it will print out the screen and matched item
- support more (alt) keys

Bug Fixes:
- wrong cursor position after item changed
- #142: NULL character was dropped with `--ansi`
- regression: `--margin` not working
- #148: screen won't clear in interactive mode
- number of matched item not showing correctly (during matching)
- lag in changing query on large collection of inputs

## 0.6.2: 2019-03-19

Feature:
- Support `--header-lines`
- Support `--layout`
- Update the latest fzf.vim

## 0.6.1: 2019-03-17

Fix:
- compile fail with rust 2018 (1.31.0)
- reduce the time on exit. It took time to free memories on large
    collections.

## 0.6.0: 2019-03-17

Performance improvement.

This is a large rewrite of skim, previously there are 4 major components of
skim:

- reader: for reading from command or piped input
- sender: will cache the lines from reader and re-send all lines to matcher on restart
- matcher: match against the lines and send the matched items to model
- model: handle the selection of items and draw on screen.

They are communicated using rust's `channel` which turned out to be too slow
in skim's use case. Now we use `SpinLock` for sharing data. The performance on
large collections are greatly improved.

Besides, use `tuikit` for buferred rendering.

## 0.5.5: 2019-02-23

Bug fixes:
- fix: regression on `--with-nth` feature
- fix: 100% CPU on not enough printing area

## 0.5.4: 2019-02-20

Emergency release that fix test failures which breaks
[APKBUILD](https://github.com/5paceToast/user-aports/blob/master/toast/skim/APKBUILD).
Check out [#128](https://github.com/lotabout/skim/issues/128).

## 0.5.3: 2019-02-20

Features:
- `--header` for adding header line
- `--inline-info` for displaying info besides query
- run preview commands asynchronizely
- implement action `delete-charEOF`
- support key: `ctrl+space`

More bug fixes, noticable ones are:
- Panic on reading non-utf8 characters
- 100% CPU when input is not ready

## 0.5.2: 2018-10-22

- fix: stop command immediately on accept or abort.
- minor optimization over ASCII inputs.
- #90: escape quotes in specified preview command

## 0.5.1: 2018-06-24

Use [cross](https://github.com/japaric/cross) to build targets.

## 0.5.0: 2018-06-12

Change the field syntax to be fzf compatible.
- Previously it was git style
    - fields starts with `0`
    - `1..3` results in `2, 3` (which is `0, 1, 2, 3` minus `0, 1`)
- Now it is `cut` style
    - fields starts with `1`
    - `1..3` results in `1, 2, 3`

## 0.4.0: 2018-06-03

Refactor skim into a library. With minor bug fixes:
- support multiple arguments, to be a drop-in replacement of fzf.
- support negative range field. (e.g. `-1` to specify the last field)
- respond to terminal resize event on Mac.

## 0.3.2: 2018-01-18
Some minor enhancements that might comes handy.
- Reserve all fzf options, so that skim can be a drop-in replacement of fzf.
- Fix: the number of columns a unicode character occupies
- Accept multiple values for most options. So that you can safely put them
    in `$SKIM_DEFAULT_OPTIONS` and override it in command line.

Thanks to @magnetophon for the bug report and feature requests.

## 0.3.1: 2017-12-04
Support more options, and reserve several others. The purpose is to reuse
`fzf.vim` as much as possible.
- `--print0`: use NUL(\0) as field separator for output.
- `--read0`: read input delimited by NUL(\0) characters
- `--tabstop`: allow customizing tabstop (default to 8).
- `--no-hscroll`: disable hscroll on match.
- reserve several other options, skim will do nothing on them instead of throwing errors.

## 0.3.0: 2017-09-21
This release starts from adding `--height` featuren, ends up a big change in
the code base.
- feature: `--bind` accept character keys. Only Ctrl/Alt/F keys were accepted.
- feature: support multiple `--bind` options. (replace getopts with clap.rs)
- feature: `--tac` to reverse the order of input lines.
- feature: `--preview` to show preview of current selected line.
- feature: `--height` to use only part instead of full of the screen.
- test: use tmux for integration test
- replace [ncurses-rs](https://github.com/jeaye/ncurses-rs) with [termion](https://github.com/ticki/termion), now skim is fully rust, no C bindings.
