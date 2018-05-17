 # podcast
 ---
 `podcast` is a command line podcast player.
 
 NOTE: Playback requires either mpv or vlc to be installed
 
 It currently supports:
- [x] Subscribing to RSS feeds
- [x] Unsubscribing from RSS feeds
- [x] Streaming podcasts
- [x] Parallel downloading of multiple podcasts 
- [x] Playing podcasts
- [x] Auto-download new episodes
- [x] Automatically check for updates
- [ ] Shell Completions
    - [x] zsh
    - [ ] bash
    - [ ] sh
- [ ] Searching for podcasts...(WIP)

By default, podcasts are downloaded to $HOME/Podcasts, but this folder can be set with the $PODCAST environmental variable.

How many latest episodes to download when subscibing to new podcasts can be set in the $PODCAST/.config YAML file

Downloads can be done a variety of ways:

Individually: `podcast download $podcast_name 4`

Multiple: `podcast download $podcast_name 1,5,9-12,14`

All: `podcast download $podcast_name`
