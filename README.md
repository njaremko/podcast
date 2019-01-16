 # podcast
 ---
 `podcast` is a command line podcast player.
 
 [Linux x64 binary download](https://github.com/njaremko/podcast/releases/download/0.7.0/podcast-x86-64-linux)
 
 SHA256 Checksum: 4faf1e9fa941d4eb161eb330b86e6f02168bd9cb006413ae0dcb92ae2f693cc7
 
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

You can also use a portion of the name. 
Podcast will pick the first podcast alphabetically that contains the given word (Case-Insensitive).

Example Usage:
```sh
$ podcast subscribe "http://feeds.feedburner.com/mbmbam"
$ podcast ls
My Brother, My Brother And Me
$ podcast ls bro # List all the episodes of My Brother, My Brother, and Me
(447) MBMBaM 440: The Naming of 2019
(446) MBMBaM 439: Face 2 Face: Candlenights 2018
...
(2) My Brother, My Brother and Me: Episode 02
(1) My Brother, My Brother and Me: Episode 01
$ podcast play bro # Play the latest episode of mbmbam
$ podcast play "my brother" 446 # Play "MBMBaM 439: Face 2 Face: Candlenights 2018"
$ podcast download bro # Download all episodes of mbmbam
$ podcast download brother -e "The Naming" # Download the latest episode containing "The Naming"
Downloading: /home/njaremko/Podcasts/My Brother, My Brother And Me/MBMBaM 440: The Naming of 2019.mp3
$ podcast download bro 44 -e -a # Download all episodes containing "44"
File already exists: /home/njaremko/Podcasts/My Brother, My Brother And Me/MBMBaM 440: The Naming of 2019.mp3
Downloading: /home/njaremko/Podcasts/My Brother, My Brother And Me/MBMBaM 344: The Cream Beams to the Tower of Flavortown.mp3
Downloading: /home/njaremko/Podcasts/My Brother, My Brother And Me/MBMBaM 244: Slimefoot.mp3
Downloading: /home/njaremko/Podcasts/My Brother, My Brother And Me/MBMBaM 144: Kick it Forward.mp3
Downloading: /home/njaremko/Podcasts/My Brother, My Brother And Me/My Brother, My Brother and Me 44: Chunk Pump.mp3
```
