use std::{env, time::Duration};

use burrito::burrito::{burrito_cfg::BurritoCfg, burrito_data::BurritoData, systems::{SystemContext, SystemMap}, log_watcher::{EventType, LogWatcher}};
use burrito::burrito::systems;
use burrito::burrito::alert;

fn main() {
    eprintln!("Burrito starting up");
    let args: Vec<String> = env::args().collect();
    let cfg = BurritoCfg::load_from_file();
    let data = BurritoData::load_from_file();
    let sys_map = systems::load_saved_system_map();
    let mut current_system = None;
    if args.len() > 1 {
        current_system = Some(args[1].to_owned());
    }
    let ctx = SystemContext::new(current_system, &sys_map);
    if ctx.get_current_system_ids().len() < 1 {
        eprintln!("No systems specified. To set/add to current systems, use `burrito <system>`");
        std::process::exit(1)
    }
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
                    println!("{}", &event.message);
                },
                EventType::RangeOfSystem(event_distance) => {
                    println!("{}", &event.trigger);
                    for alert in &cfg.sound_config.audio_alerts {
                        if let EventType::RangeOfSystem(alert_distance) = alert.trigger {
                            if event_distance <= alert_distance {
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
