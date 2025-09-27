# Unreleased changes

## Features

- Support some actions with pointer (e.g mouse).
- Add flag to hide input bar.

## Changes

- Log to stderr instead of stdout.

## Fixes

- Handle key repeatitions properly.

# 0.2.2 - 2024-03-10

## Fixes

- Invalid scale handling

# 0.2.1 - 2024-03-10

## Features

- Launch multiple apps from one yofi launch based on fork (#24).
- Nix build (#98).
- Corner roundings config param.
- Background border.
- Font overflow hint for long text items.

## Changes

- Dropped support for font-kit backend.
- Use native fontconfig instead of rust-fontconfig.
- More errors are handled over panic.

## Fixes

- Don't reload font on each ListView redraw.
- Normalize indexed pngs.
- Icon themes suport multiple inheritance.

# 0.2.0 - 2022-10-12

## Features

- Support prompt message
- Input masking for password
- Better icon support
- Migrate from fzyr to sublime_fuzzy matcher

## Changes

- Use syslog instead of journald that could be disabled

## Fixes

- Don't panic on malformed icons
- Skip desktop files without appropriate file extension
- Reduce allocations for icon loading
- Skip folders listing for binapps mode
- Handle panic gracefully
- Wrong font being selected for list_items
- Account font kerning for font-kit backend

# 0.1.5 - 2022-01-30

## Features

- Font loading by path without fs scans (#79)
- Render desktop actions for apps mode (#78)
- Fontdue backend supoorted and used by default (#63, #67, #69)
- Support of blacklisting entries (#62)
- Support grayscale/indexed png icons (#61)
- Redirect logs by default to systemd (#58) and stdout (#75)
- More hotkeys for naviation (#35, #49)
- Specify colors in css-like hex (#47)
- Fallback to input at dialog overflow (#43)
- Support environments without layer-shell protocol (#42)

## Bug fixes

- Prioritize the local desktop files over global
- Handle missing glyphs (#40)

# 0.1.4 - 2021-01-10

## Features

- Support localization (#33)
- Search by keywords in apps mode (#20)
- Magic separators support: `!!` for args, `#` for envs and `~` for workdir (#19)
- Display full path for ambiguous binapps (0b47575)
- ctrl+backspace is alias for ctrl+w (b3fca99)

## Bug fixes

- Update HiDPI scale on each draw (#20)
- Deduplicate binapps entries with the same path (c6b73f2)
- With highligting enabled search may crash sometimes (b990057)

# 0.1.3 - 2020-12-26

## Features

- HiDPI scaling support (79cb8dd)
- Matched chars highlighting (9d36ab0)
- Intuitive scroll (7958fce)
- Better fuzzy search, thanks for fzyr lib (73b002f)
- ctrl+w hotkey removes last word (3524df6)
- Launch binaries (cf16596)
- Configure font size (1a34eb2)

# 0.1.2 - 2020-12-19

## Features

- Pixmap icons support #12
- Configurable layout #11

## Bug fixes

- Support absolute paths in Icon desktop entries (7474a12)
- Search for scalable folder for icons as well (125363a)
- Skip placeholders in Exec desktop entries (c548191)

# 0.1.1 - 2020-12-10

## Features

- Basic icon support #10
- Startup ordering based on usage statistic (d7d40d4)

# 0.1.0 - 2020-12-06

## Features

- Dialog aka dmenu mode (abbd722)
- CLI arguments for log/config parameters (f4befb1)

## Bug fixes

- Show output even on empty input buffer (6790c4a)
