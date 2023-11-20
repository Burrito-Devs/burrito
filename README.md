# Burrito

Burrito is an open source, cross-platform intel tool for Eve Online. If functions similarly to other programs such as T.A.C.O by reading Eve chat and game logs. These messages are parsed by Burrito and processed into events. Burrito can be configured to alert the user when various events occurr.

## Features

Out of the box, Burrito is configured to notify the user using text and sound when specific events are detected in Eve's log files. The following events trigger alerts by default:

* Character reported within 5 jumps of the current system
* Pirate faction NPC spawn
* NPC capital ship spawn
* NPC officer spawn

All new chatlog files for configured channels will be monitored after starting Burrito. New game log files will also be monitored. Burrito also watches old log files that have been modified recently. If clients are closed or crash, there is no need to restart Burrito. It will find the new log files when you start new clients. Likewise, if Burrito either crashes or is closed, there is no need to restart any clients.

### Event triggers

Chatlog readers are capable of detecting and differentiating events such as characters being reported in specified systems as well as things such as system status requests, system clear messages, and even when the Eve client has lost/regained connection to the Eve Online chat server. Game log readers produce events such as faction/officer/dread spawns etc. Burrito specifically parses combat notifications for the NPCs, so this avoids false positives from things like trading ships in station with faction modules.

When an event is produced, it may be ignored, it may be logged, or it may play an audio file. The response to events can be configured in `burrito.cfg`. Check the [here](./example_cfg.cfg) for some examples. When processing the `RangeOfSystem` events, only the one with the lowest distance will trigger an alert and the rest will be ignored. So if the configuration has an alert for 5 jumps and 10 jumps, an event of players within 3 jumps (`RangeOfSystem(3)`) will only cause the alert for 5 jumps to be played. This will function properly no matter how you order your alerts in `burrito.cfg`. They are sorted automatically when the configuration is parsed. Events are also always processed in the order they are created.

## Getting Started

### Installation

#### Windows

* Download the Windows zip file corresponding to your architecture. (if you don't know, probably 64 bit)
* Extract it to a temp directory.
* Run the `install` script.
* Open a Command Prompt window
* Run `burrito cfg watch system <system_name>` where `<system_name>` should be replaced the exact system name you want to monitor (case-sensitive for now).
* Run `burrito`

Burrito should now be running and both monitoring recent logs and waiting for logs from your Eve clients. To exit burrito, press Ctrl+C in the Command Prompt window.

#### Mac / Linux

* Download the linux zip file.
* Create a `.burrito` directory under your home directory
* Extract the contents of the `data` folder into the new `.burrito` folder. Do not copy the `data` folder itself, just its contents
* Extract the Burrito binary wherever you want to run it from. Either your home directory or any place that is already in your PATH is probably easiest
* Run `burrito cfg watch <system_name>` where `<system_name>` should be replaced the exact system name you want to monitor.
* Run `burrito`

### Running Burrito

To generate the default Burrito configuration files, simply run the Burrito program. This will create some files that Burrito needs in the `.burrito` folder under your home directory. `burrito.cfg` contains the main configuration file for Burrito. This file can be edited by the user to change the behavior of the program. Below is a non-exhaustive list of some of the most important values that users might be changing:

* `log_dir`: This is the base log directory for your Eve Online installation. This should be set to the folder that contains both your `Chatlogs` and `Gamelogs` directories, not either one of those.
* `sound_config`: This is the alert sound configuration. The `audio_alerts` sub-field contains a set of pairings of alert types and the sound files to play when they occurr. Values can be added, changed, or removed from here in order to customize the user experience.
* `text_channel_config`: This value tells Burrito which in-game chat channels to monitor for events. An exhaustive list of values can be found in the [example configuration](./example_cfg.cfg).

To specify system(s) to watch, run Burrito like this: `burrito cfg watch system <system name>`. This will add the specified system to the watch list in ctx.json. To remove a system from this list, use `burrito cfg unwatch system <system name>`.

After configuring Burrito, you can start it like this: `burrito`. If Burrito is configured correctly, it will begin watching the log files that it is configured to read. New chatlog messages  will show up in the output as they are received in-game. Game log messages are only displayed if they trigger an event that Burrito is configured to listen to. Game log messages will also be displayed with the name of the client that it came from. When multiboxing, this makes it easy to find out which client needs attention if a faction spawn occurrs, for example.

## Configuring Burrito

The [example configuration](./example_cfg.cfg) shows how to modify `burrito.cfg` to get the desired behavior out of Burrito. The file is formatted as JSON, so it is easy to view and edit by hand. But if invalid JSON is inserted into `burrito.cfg`, the configuration cannot be loaded.

### Adding alerts

Burrito supports a variety of user-added audio alerts. By default, it alerts the user if there is a character reported within 5 jumps of their specified system and if one of the monitored clients encounters a special NPC spawn. Burrito comes with sound files for all of these alerts. In addition to enabling users to add their own alerts, Burrito also allows for custom sound files to be played for default or user-created alerts. The alerts can be found in the `burrito.cfg` file under the  heading `sound_config` -> `audio_alerts`. An audio alert looks like this:

```JSON
{
    "trigger": {
        "RangeOfSystem": 5
    },
    "sound_file": "/home/the_bernie/.burrito/sounds/neut_in_range.mp3"
}
```

This alert will play the sound file `/home/the_bernie/.burrito/sounds/neut_in_range.mp3` if a character is reported within `5` jumps of the player's system.

`RangeOfSystem` is the event type that triggers this alert. There are a variety of event types that can be used to trigger alerts. [LogWatcher](./src/burrito/log_watcher.rs) has as list of all the event types that Burrito currently recognizes. The `trigger` field specifies what event type causes this alert to play. `RangeOfSystem` is a special event type becacuse it contains extra data about the event—the number of jumps away from the user's system that it occurred. This can be set to any positive integer less than 2<sup>32</sup>. Since this data makes the event unique, additional `RangeOfSystem` alerts can be added for different ranges. To add an alert for 3 jumps away, this alert can be copied and pasted. You only have to change the `5` to a `3` and specify the path to the sound in the `sound_file` field. Make sure all alerts are separated by a comma (`,`). The last alert in your list must also not have a trailing comma as that is not valid JSON.

### Modifying Intel Channels

By default, Burrito will only listen to the channels `delve.imperium` and `querious.imperium`. These are the two most active and relevant Imperium intel channels. Burrito has built-in support for all current and past Imperium intel channels. An exhaustive list of these can be found [here](src/burrito/log_reader.rs). Burrito also supports custom channels. If you are using different intel channels, you can specify them like this:

```JSON
{
    "Custom": {
        "channel": "My Intel Channel"
    }
}
```

Replace `My Intel Channel` with the name of your in-game channel. Burrito will monitor all channels in comma separated list under `text_channel_config` -> `text_channels` in `burrito.cfg`. Check the [example configuration](./example_cfg.cfg) for a sample list including custom channels.

## Future Plans

Burrito is currently in a pre-release state. Presently it is considered to have the minimum functionality needed to be useful as an intel tool. The full release will include additional features such as a full GUI and multi-system support. Here is a **non-exhaustive** list of future work:

* Multi-system support: multi-boxers may wish to receive notifications for characters in range of multiple systems. So if using characters in 3 different systems, alerts will trigger based on the configured ranges from each of those systems.
* CLI configuration: Manually editing config files is annoying for small changes. A CLI for changing the configuration files would simplify the process of changing the configuration and make it less error-prone
* GUI: A full GUI is planned for Burrito. The CLI will always be available, but many users may prefer a GUI. Configuration in the GUI will also be supported.
* Loads of test coverage
* Test tool to preview the behavior of your Burrito configuration by injecting data into a log file
* Basically every `TODO` item throughout the source code
* Fixing basically every `panic`
* A lot of heavy refactoring and cleaning/optimization of existing code: Burrito was originally written in 2 hours because I wanted to AFK rat and T.A.C.O did not work for me on Linux. There were a lot of short-cuts taken in order to get it working faster and I have slowly been going through and fixing them. This is also the first full project I've started from scratch in Rust, so I am also getting used to using Rust to do things the "Rust" way.
* `.clone() .clone() .clone() .clone() .clone() .clone() .clone() .clone() .clone() .clone()`
* Configurable sound output device
