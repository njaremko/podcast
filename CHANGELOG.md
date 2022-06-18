0.19.3
- Updating dependencies
- Adding the ability to configure file name templates, initially supporting `{title}` and `{number}`
- Add the ability to download episodes matching a regex using `podcast download <podcast name> -p ".*goldfish.*"`

0.19.0
- Migrate from `smol` to `tokio`, makes life easier

0.18.1
- Some minor internal refactoring to clean stuff up

0.18.0
- Update a bunch of dependencies to latest version

0.17.6
- Fix mpv CLI argument to not show image popup

0.17.5
- Check for new release before checking for new podcast episodes

0.17.4
- Make download progress bar layout dynamic as terminal size changes

0.17.3
- Improve layout of download progress bar for small terminal widths

0.17.2
- Fix unsubscribe throwing an error if you unsubscribed from the last subscription in the list

0.17.1
- Fix running just `podcast` causing a panic
- Fix auto-download when subscribing to a podcast

0.17.0
- Search is improved to handle spaces without quotes
podcast search my brother will correctly return My Brother My Brother And Me
- Podcast downloading has been significantly improved
    - We now show you download progress in a subjectively prettier manner
    - Downloads are optimally distributed across threads
    - We take advantage of keep-alive connection pooling if possible
- Fixed the --quiet option not being parsed correctly
- Fairly significant work towards cleaning up the code base, and making everything more readable. More to come.

0.16.0
- Refactor podcast searching logic into it's own library: [podcast_search](https://crates.io/crates/podcast_search)

0.15.0
- Proper podcast searching through iTunes API

0.12.0
- Remove nightly only features, works on stable now.

0.11.0
- Add podcast search support. Podcast index is a work in progress, but I'm working on it.

0.10.0
- Partial re-write of the application to be idiomatic
- Improves performance throughout the application (downloads are also faster)
- Changed from error-chain to failure for error handling

0.9.1
- Improve unsubscribe messages

0.9.0
- Removed `-d` from sub and subscribe subcommands. Behaviour of subscribing is defined by the .config.yaml file now.

0.8.2
- Add completion generation for all major shells

0.8.1
- Fix parser to actually see "sub" subcommand

0.8.0
- Add a few subcommands / subcommand shortcuts
- Internal cleanup

0.6.0
- Update to rust 2018 edition

0.5.11
- Code cleanup

0.5.10
- Fix update check functionality

0.5.9
- Update remaining dependencies

0.5.8
- Update regex crate to 1.0

0.5.7
- Updates filename escaping to generally only affect Windows (because Microsoft filesystems can't handle a bunch of characters)

0.5.6
- Escape filenames to prevent issues on some filesystems

0.5.5
- Attempt at better handling file handles to fix windows bug regarding renaming .subscriptions.tmp

0.5.4
- Improve error handling throughout the application (using error-chain)

0.5.0
- Fix downloading all episodes of podcast not working if folder didn't exist
- Confirm before downloading all episodes of a podcast

0.4.7
- Add some tests
- Improve handling of file extensions

0.4.6
- Add travis-ci support
- Add category to cargo.toml

0.4.5
- Improve subscribe default behaviour
    - Without an option, we'll just subscribe to them
    - with -d or --download we will download according to auto-download limit in $PODCAST/.config

0.4.4
- Add ability to play latest episode by omitting episode number
- Fix update check working correctly
- Fix download still being case-sensitive

0.4.3
- Display correct version in help screen

0.4.2
- Changed the save format to include the current version to allow for automatic check for updates functionality

0.4.1
- Whoops, never actually published this...

0.4.0
- Add ability to print zsh shell completion
- Add ability to unsubscribe from podcasts
- Add check for updates functionality
- Ignore case when checking podcast titles
- Update all dependencies to their latest respective versions
- rename list -> ls 
- rename update -> refresh
