 # podcast
 ---
`podcast` is a command line podcast manager and player.
 
Binaries can be found here: https://github.com/njaremko/podcast/releases
 
NOTE: Playback requires either mpv or vlc to be installed
 
It currently supports:
- [x] Subscribing to RSS feeds
- [x] Searching for podcasts
- [x] Unsubscribing from RSS feeds
- [x] Streaming podcasts
- [x] Parallel downloading of multiple podcasts 
- [x] Playing podcasts
- [x] Auto-download new episodes
- [x] Automatically check for updates
- [x] Shell Completions
    - [x] zsh
    - [x] bash
    - [x] fish
    - [x] powershell
    - [x] elvish

By default, podcasts are downloaded to `$HOME/Podcasts`, but this folder can be set with the `$PODCAST` environmental variable.

How many latest episodes to download when first subscibing to new podcasts can be set in the `$PODCAST/.subscriptions.json` file

Downloads can be done a variety of ways:

Individually: `podcast download $podcast_name 4`

Multiple: `podcast download $podcast_name 1,5,9-12,14`

All: `podcast download $podcast_name`

You can also use a portion of the name. 
Podcast will pick the first podcast alphabetically that contains the given word (Case-Insensitive).

# Example Usage:
```sh
$ podcast search my brother my brother and me
Using PODCAST dir: "/Users/jaremn/Podcasts"
(0) My Brother, My Brother And Me [https://feeds.simplecast.com/wjQvYtdl]
(1) My Brother, My Brother And Me [https://anchor.fm/s/2b7f0c44/podcast/rss]
(2) My Brother's Funnier Than Me [https://anchor.fm/s/106a0d0/podcast/rss]
(3) My Brother's Wife and Me and Him [https://anchor.fm/s/e9402ec/podcast/rss]
(4) My Brother & Me [https://anchor.fm/s/d6a4e6c/podcast/rss]
(5) Me & My Brother [https://anchor.fm/s/37aa1f4/podcast/rss]
Would you like to subscribe to any of these? (y/n): y
Which one? (#): 0
Downloading RSS feed...
Subscribe auto-download limit set to: 1
Downloading episode(s)...
[00:03:06] MBMBaM 518: Pepperoni and Vicki.mp3 [294.32KB/s] [3.08MB/56.54MB]
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
Using PODCAST dir: "/Users/jaremn/Podcasts"
[00:07:36] MBMBaM 449: The Cable Pie.mp3 [114.59KB/s] [527.34KB/51.50MB]
[00:00:12] MBMBaM 448: Bird Words.mp3 [3.63MB/s] [16.19MB/58.05MB]
[00:08:03] MBMBaM 447: Valentine’s Escape Room.mp3 [105.69KB/s] [494.59KB/50.28MB]
[00:00:05] MBMBaM 446: Face 2 Face: The Cupture.mp3 [5.85MB/s] [26.19MB/55.63MB]
[00:09:46] MBMBaM 445: Pizzalicious Turbo-Moths.mp3 [93.65KB/s] [443.57KB/54.01MB]
[00:08:07] MBMBaM 444: The 100 Wishes of the Pandemonium Cube.mp3 [109.15KB/s] [495.49KB/52.40MB]
[00:09:08] MBMBaM 443: Face 2 Face: Apple Time!.mp3 [106.42KB/s] [495.15KB/57.38MB]
[00:10:56] MBMBaM 442: Justin’s Special Shower Sauce.mp3 [93.04KB/s] [451.47KB/59.98MB]
[00:08:44] MBMBaM 441: In a New York Whoopsie.mp3 [113.30KB/s] [521.91KB/58.44MB]
[00:00:13] MBMBaM 440: The Naming of 2019.mp3 [3.24MB/s] [14.45MB/55.49MB]
[00:00:29] MBMBaM 344: The Cream Beams to the Tower of Flavortown.mp3 [1.73MB/s] [7.72MB/56.55MB]
[00:12:13] MBMBaM 244: Slimefoot.mp3 [97.95KB/s] [454.36KB/70.58MB]
[00:10:25] MBMBaM 144: Kick it Forward.mp3 [105.41KB/s] [492.53KB/64.79MB]
[00:08:17] My Brother, My Brother and Me 44: Chunk Pump.mp3 [95.76KB/s] [441.53KB/46.85MB]
```

# Generating completions:
```sh
# Generating completion for current shell:
$ podcast completion
... outputs stuff that needs to be loaded by your shell on startup ...

# Fish Shell Example
$ podcast completion fish > podcast.fish
$ sudo mv podcast.fish /usr/share/fish/completions
```

# Building
Building requires nightly rust
```sh
git clone git@github.com:njaremko/podcast.git
cd podcast
cargo install --path=.
```
