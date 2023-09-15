use std::{/*env, */time::Duration};

use burrito::burrito::{burrito_cfg::BurritoCfg, burrito_data::BurritoData, systems::SystemContext, log_watcher::{EventType, LogWatcher}};
use burrito::burrito::systems;
use burrito::burrito::alert;

fn main() {
    eprintln!("Burrito starting up");
    //let args: Vec<String> = env::args().collect();
    let cfg = BurritoCfg::load_from_file();
    let data = BurritoData::load_from_file();
    let ctx = SystemContext::new(&cfg);

    run_burrito(ctx, cfg, data);
}

fn run_burrito(ctx: SystemContext, cfg: BurritoCfg, data: BurritoData) {
    let sys_map = systems::load_saved_system_map();
    // TODO: add some way to configure this with files or arguments
    let mut log_watcher = LogWatcher::new(
        ctx.clone(),
        cfg.clone(),
        data.clone(),
        sys_map.clone(),
    );
    log_watcher.init();
    eprintln!("Burrito ready!");
    let mut range_alert_played = false;
    loop {
        log_watcher.get_events().into_iter().for_each(|event| {
            println!("{}", &event.trigger);
            match event.event_type {
                EventType::RangeOfSystem(event_distance, _) | EventType::RangeOfCharacter(event_distance, _) => {
                    for alert in &cfg.sound_config.audio_alerts {
                        if let EventType::RangeOfCharacter(alert_distance, _) = alert.trigger {
                            if event_distance <= alert_distance {
                                if !range_alert_played {
                                    alert::alert(&event, &event.trigger, &event.character_name, Some(&alert.sound_file));
                                    range_alert_played = true;
                                }
                                break;
                            }
                        }
                        if let EventType::RangeOfSystem(alert_distance, _) = alert.trigger {
                            if event_distance <= alert_distance {
                                if !range_alert_played {
                                    alert::alert(&event, &event.trigger, &event.character_name, Some(&alert.sound_file));
                                    range_alert_played = true;
                                }
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
        range_alert_played = false;
        std::thread::sleep(Duration::from_millis(cfg.log_update_interval_ms))
    }
}
