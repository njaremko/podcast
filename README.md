 # podcast
 ---
 `podcast` is a command line podcast player.
 
 NOTE: Playback requires either mpv or vlc to be installed
 
 It currently supports:
- [x] Subscribing to RSS feeds
- [x] Unsubscribing from RSS feeds
- [x] Streaming podcasts
- [x] Downloading podcasts 
- [x] Playing podcasts
- [x] Auto-download new episodes
- [x] Automatically check for updates
- [ ] Auto-delete old episodes
- [ ] Shell Completions
    - [x] zsh
    - [ ] bash
    - [ ] sh
- [ ] Searching for podcasts...(WIP)

By default, podcasts are downloaded to $HOME/Podcasts, but this folder can be set with the $PODCASTS environmental variable.

How many latest episodes to download when subscibing to new podcasts can be set in the $PODCASTS/.config YAML file

Downloads can be done a variety of ways:

Individually: `podcast download $podcast_name 4`

Multiple: `podcast download $podcast_name 1,5,9-12,14`

All: `podcast download $podcast_name`
