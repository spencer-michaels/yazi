> ## 🧪 Test fork
>
> This is a personal fork of [sxyazi/yazi](https://github.com/sxyazi/yazi) that adds an `exclude` option for hiding files by glob pattern — proposed upstream in [#3835](https://github.com/sxyazi/yazi/issues/3835).
>
> Files matching any pattern are always hidden, even when `show_hidden` is on — a separate axis from dotfile visibility, for things you never want to see (build artifacts, OS junk, etc.).
>
> ### Build
>
> ```bash
> git clone https://github.com/spencer-michaels/yazi
> cd yazi
> cargo build --release --bin yazi
> ./target/release/yazi
> ```
>
> Requires Rust 1.92+ (MSRV) and nightly rustfmt if you plan to format.
>
> ### Configure
>
> Add an `exclude` list under `[mgr]` in `~/.config/yazi/yazi.toml`:
>
> ```toml
> [mgr]
> exclude = [
>   ".DS_Store",          # exact filename
>   "Thumbs.db",          # exact filename
>   "*.pyc",              # filename glob (matches anywhere)
>   "*.o",
>   "__pycache__",        # bare directory name
>   "node_modules",
>   "build/output/*.tmp", # path pattern — auto-prefixed with **/, only matches under build/output
>   "**/cache.bin",       # explicit **/ — left as-is
> ]
> ```
>
> **Pattern rules:**
> - Patterns **without `/`** match against filenames only (e.g. `*.pyc` hides every `.pyc` in any directory).
> - Patterns **with `/`** match against the full path, auto-prefixed with `**/` if not rooted (so `build/output/*.tmp` is treated as `**/build/output/*.tmp`).
> - Patterns starting with `/` or `**/` are used as-is.
> - Default is `exclude = []` — no behavior change.
>
> Stable yazi silently ignores the unknown `exclude` field, so this config is safe to keep across both binaries.
>
> ### Keybindings
>
> | Key | Action | What it does |
> | --- | --- | --- |
> | <kbd>E</kbd> | `excluded toggle` | Toggle visibility of excluded files (per-tab) |
> | <kbd>.</kbd> | `hidden toggle` | Toggle dotfile visibility — unchanged, independent of `exclude` |
>
> The `excluded` action accepts the same state arguments as `hidden`:
>
> ```
> excluded show     # always show excluded files in this tab
> excluded hide     # always hide them
> excluded toggle   # flip current state
> ```
>
> To rebind, override in `~/.config/yazi/keymap.toml`:
>
> ```toml
> [[mgr.prepend_keymap]]
> on  = "<C-e>"
> run = "excluded toggle"
> desc = "Toggle excluded files"
> ```
>
> ### Behavior notes
>
> - `show_excluded` is **per-tab** — toggling in one tab doesn't affect others.
> - Toggling `show_hidden` (with <kbd>.</kbd>) does **not** reveal excluded files; the two axes are independent.
> - Excluded files are still on disk and can be operated on by name — they're just hidden from the listing.
>
> ---

<div align="center">
	<sup>Special thanks to:</sup><br>

| <a href="https://go.warp.dev/yazi" target="_blank"><img alt="Warp sponsorship" width=350 src="https://github.com/warpdotdev/brand-assets/blob/main/Github/Sponsor/Warp-Github-LG-02.png"><br><b>Warp, built for coding with multiple AI agents</b><br><sup>Available for macOS, Linux and Windows</sup></a> |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |

</div>

## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi (means "duck") is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

💡 A new article explaining its internal workings: [Why is Yazi Fast?](https://yazi-rs.github.io/blog/why-is-yazi-fast)

- 🚀 **Full Asynchronous Support**: All I/O operations are asynchronous, CPU tasks are spread across multiple threads, making the most of available resources.
- 💪 **Powerful Async Task Scheduling and Management**: Provides real-time progress updates, task cancellation, and internal task priority assignment.
- 🖼️ **Built-in Support for Multiple Image Protocols**: Also integrated with Überzug++ and Chafa, covering almost all terminals.
- 🌟 **Built-in Code Highlighting and Image Decoding**: Combined with the pre-loading mechanism, greatly accelerates image and normal file loading.
- 🔌 **Concurrent Plugin System**: UI plugins (rewriting most of the UI), functional plugins, custom previewer/preloader/spotter/fetcher; Just some pieces of Lua.
- ☁️ **Virtual Filesystem**: Remote file management, custom search engines.
- 📡 **Data Distribution Service**: Built on a client-server architecture (no additional server process required), integrated with a Lua-based publish-subscribe model, achieving cross-instance communication and state persistence.
- 📦 **Package Manager**: Install plugins and themes with one command, keeping them up-to-date, or pin them to a specific version.
- 🧰 Integration with ripgrep, fd, fzf, zoxide
- 💫 Vim-like input/pick/confirm/which/notify component, auto-completion for cd paths
- 🏷️ Multi-Tab Support, Cross-directory selection, Scrollable Preview (for videos, PDFs, archives, code, directories, etc.)
- 🔄 Bulk Renaming, Archive Extraction, Visual Mode, File Chooser, [Git Integration](https://github.com/yazi-rs/plugins/tree/main/git.yazi), [Mount Manager](https://github.com/yazi-rs/plugins/tree/main/mount.yazi)
- 🎨 Theme System, Mouse Support, Trash Bin, Custom Layouts, CSI u, OSC 52
- ... and more!

https://github.com/sxyazi/yazi/assets/17523360/92ff23fa-0cd5-4f04-b387-894c12265cc7

## Project status

Public beta, can be used as a daily driver.

Yazi is currently in heavy development, expect breaking changes.

## Documentation

- Usage: https://yazi-rs.github.io/docs/installation
- Features: https://yazi-rs.github.io/features

## Discussion

- Discord Server (English mainly): https://discord.gg/qfADduSdJu
- Telegram Group (Chinese mainly): https://t.me/yazi_rs

## Image Preview

| Platform                                                                     | Protocol                               | Support                                |
| ---------------------------------------------------------------------------- | -------------------------------------- | -------------------------------------- |
| [kitty](https://github.com/kovidgoyal/kitty) (>= 0.28.0)                     | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [iTerm2](https://iterm2.com)                                                 | [Inline images protocol][iip]          | ✅ Built-in                            |
| [WezTerm](https://github.com/wez/wezterm)                                    | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Konsole](https://invent.kde.org/utilities/konsole)                          | [Kitty old protocol][kgp-old]          | ✅ Built-in                            |
| [foot](https://codeberg.org/dnkl/foot)                                       | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Ghostty](https://github.com/ghostty-org/ghostty)                            | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [Windows Terminal](https://github.com/microsoft/terminal) (>= v1.22.10352.0) | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [st with Sixel patch](https://github.com/bakkeby/st-flexipatch)              | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Warp](https://www.warp.dev) (macOS/Linux only)                              | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Tabby](https://github.com/Eugeny/tabby)                                     | [Inline images protocol][iip]          | ✅ Built-in                            |
| [VSCode](https://github.com/microsoft/vscode)                                | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Rio](https://github.com/raphamorim/rio) (>= 0.3.9)                          | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [Black Box](https://gitlab.gnome.org/raggesilver/blackbox)                   | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Bobcat](https://github.com/ismail-yilmaz/Bobcat)                            | [Inline images protocol][iip]          | ✅ Built-in                            |
| X11 / Wayland                                                                | Window system protocol                 | ☑️ [Überzug++][ueberzug] required      |
| Fallback                                                                     | [ASCII art (Unicode block)][ascii-art] | ☑️ [Chafa][chafa] required (>= 1.16.0) |

See https://yazi-rs.github.io/docs/image-preview for details.

<!-- Protocols -->

[kgp]: https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders
[kgp-old]: https://github.com/sxyazi/yazi/blob/main/yazi-adapter/src/drivers/kgp_old.rs
[iip]: https://iterm2.com/documentation-images.html
[sixel]: https://www.vt100.net/docs/vt3xx-gp/chapter14.html
[ascii-art]: https://en.wikipedia.org/wiki/ASCII_art

<!-- Dependencies -->

[ueberzug]: https://github.com/jstkdng/ueberzugpp
[chafa]: https://hpjansson.org/chafa/

## Special Thanks

<img alt="RustRover logo" align="right" width="200" src="https://resources.jetbrains.com/storage/products/company/brand/logos/RustRover.svg">

Thanks to RustRover team for providing open-source licenses to support the maintenance of Yazi.

Active code contributors can contact @sxyazi to get a license (if any are still available).

## License

Yazi is MIT-licensed. For more information check the [LICENSE](LICENSE) file.
