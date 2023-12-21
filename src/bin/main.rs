use std::{env, time::Duration, process::exit};

use burrito::burrito::{burrito_cfg::BurritoCfg, burrito_data::BurritoData, systems::{SystemContext, SystemMap, get_system_id}, log_watcher::{EventType, LogWatcher}};
use burrito::burrito::systems;
use burrito::burrito::alert;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cfg = BurritoCfg::load_from_file();
    let sys_map = systems::load_saved_system_map();
    let mut ctx = SystemContext::new(&sys_map);
    if args.len() > 1 {
        if args[1] == "cfg" {
            cli_cfg(args.into_iter().skip(2).collect(), &mut cfg, &mut ctx, &sys_map);
            exit(0);
        }
        else if args[1] == "help" {
            print_help();
            exit(0);
        }
        else {
            println!("Invalid command: for usage instructions, use `burrito help`.");
            exit(1);
        }
    }
    let data = BurritoData::load_from_file();
    if ctx.get_current_system_ids().len() < 1 {
        eprintln!("No systems specified. To set/add to current systems, use `burrito cfg watch system <system name>`");
        std::process::exit(1)
    }
    eprintln!("Burrito starting up");
    eprintln!("Setting current systems to {:?}", ctx.get_current_systems());

    run_burrito(ctx, cfg, data, sys_map);
}

fn run_burrito(ctx: SystemContext, cfg: BurritoCfg, data: BurritoData, sys_map: SystemMap) {
    let mut log_watcher = LogWatcher::new(
        ctx.clone(),
        cfg.clone(),
        data.clone(),
        sys_map.clone(),
    );
    log_watcher.init();
    eprintln!("Burrito ready!");
    loop {
        log_watcher.get_events().into_iter().for_each(|event| {
            match event.event_type {
                EventType::ChatlogMessage => {
                    if !cfg.hide_chat_messages {
                        println!("{}", &event.trigger);
                    }
                },
                EventType::RangeOfSystem(event_distance) => {
                    // TODO: change how alerts are handled so that this doesn't need to be two conditions
                    if !cfg.hide_out_of_range_events {
                        println!("{}", &event.trigger);
                    }
                    for alert in &cfg.sound_config.audio_alerts {
                        if let EventType::RangeOfSystem(alert_distance) = alert.trigger {
                            if event_distance <= alert_distance {
                                // TODO: change how alerts are handled so that this doesn't need to be two conditions
                                if cfg.hide_out_of_range_events {
                                    println!("{}", &event.trigger);
                                }
                                alert::alert(&event, &event.trigger, &event.character_name, Some(&alert.sound_file));
                                break;
                            }
                        }
                    }
                },
                EventType::FactionSpawn => {
                    if let Some(audio_alert) = cfg.sound_config.audio_alerts.iter()
                        .find(|a| a.trigger == event.event_type) {
                        alert::alert(&event, &event.trigger, &event.character_name, Some(&audio_alert.sound_file))
                    }
                },
                EventType::DreadSpawn => {
                    if let Some(audio_alert) = cfg.sound_config.audio_alerts.iter()
                        .find(|a| a.trigger == event.event_type) {
                        alert::alert(&event, &event.trigger, &event.character_name, Some(&audio_alert.sound_file))
                    }
                },
                EventType::OfficerSpawn => {
                    if let Some(audio_alert) = cfg.sound_config.audio_alerts.iter()
                        .find(|a| a.trigger == event.event_type) {
                        alert::alert(&event, &event.trigger, &event.character_name, Some(&audio_alert.sound_file))
                    }
                },
                _ => {}// TODO: The rest of the events
            }
        });
        std::thread::sleep(Duration::from_millis(cfg.log_update_interval_ms))
    }
}

fn cli_cfg(args: Vec<String>, _cfg: &mut BurritoCfg, ctx: &mut SystemContext, sys_map: &SystemMap) {
    guard_arg_len(1, args.len(), "No configuration option specified");
    let cmd = args[0].as_str();
    match cmd {
        "watch" | "unwatch" => {
            guard_arg_len(2, args.len(), format!("{cmd} requires `system`, `character`, or `list`"));
            let watch_type = args[1].as_str();
            match watch_type {
                "system" | "character" => {
                    guard_arg_len(3, args.len(), format!("{cmd} {watch_type} requires a name"));
                    let name = join_args(2, &args);
                    match cmd {
                        "watch" => {
                            match watch_type {
                                "system" => {
                                    if let Some(_) = get_system_id(&name, sys_map) {
                                        ctx.watch_system(&name);
                                    }
                                    else {
                                        println!("Unknown system name: {name}");
                                        exit(1);
                                    }
                                },
                                "character" => ctx.watch_character(&name),
                                _ => panic!("Unreachable code"),
                            }
                            println!("Added {name} to {watch_type} watch list");
                        },
                        "unwatch" => {
                            match watch_type {
                                "system" => ctx.unwatch_system(&name),
                                "character" => ctx.unwatch_character(&name),
                                _ => panic!("Unreachable code"),
                            }
                            println!("Removed {name} from {watch_type} watch list");
                        },
                        _ => panic!("Unreachable code"),
                    }
                },
                "list" => {
                    match cmd {
                        "watch" => println!("Unwatched systems: {:?}", ctx.get_current_systems()),
                        "unwatch" => {
                            let unwatched_systems: Vec<String> = sys_map.get_systems()
                                .into_iter()
                                .map(|e| e.1.name.to_owned())
                                .filter(|name| !ctx.get_current_systems().contains(name))
                                .collect();
                            println!("Not watching the following systems: {:?}", unwatched_systems);
                        },
                        _ => panic!("Unreachable code"),
                    }
                },
                _ => {
                    println!("Unrecognized type: {watch_type}")
                },
            }
        },
        _ => {
            println!("Unrecognized command: {cmd}");
            exit(1);
        },
    }
}

fn guard_arg_len(minimum: usize, actual: usize, message: impl ToString) {
    let message = message.to_string();
    if actual < minimum {
        println!("{message}");
        exit(1);
    }
}

fn join_args<T>(skip: usize, args: T) -> String
where
    T: IntoIterator,
    T::Item: ToString,
{
    args.into_iter()
        .skip(skip)
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn print_help() {
    println!("
        Usage: burrito [command [args ...]]

        Examples:
        `burrito`\t\t\tRuns burrito
        `burrito help`\t\t\tPrints this output
        `burrito cfg watch system 1DQ1-A`\tAdds 1DQ1-A to system watch list
        `burrito cfg unwatch system 1DQ1-A`\tRemoves 1DQ1-A from system watch list
    ");
}
