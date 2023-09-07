use std::{env, time::Duration};

use burrito::burrito::{burrito_cfg::BurritoCfg, burrito_data::BurritoData, systems::SystemContext, log_watcher::{EventType, LogWatcher}, log_reader::{LogReader, self}};
use burrito::burrito::systems;
use burrito::burrito::alert;

fn main() {
    eprintln!("Burrito starting up");
    let args: Vec<String> = env::args().collect();
    let cfg = BurritoCfg::load_from_file();
    let data = BurritoData::load_from_file();
    let mut current_system = None;
    if args.len() > 1 {
        current_system = Some(args[1].to_owned());
    }
    let ctx = SystemContext::new(current_system);
    if ctx.get_current_system().len() < 1 {
        eprintln!("No system specified. To set/change current system, use `burrito <system>`");
        std::process::exit(1)
    }
    eprintln!("Setting current system to {}", ctx.get_current_system());

    run_burrito(ctx, cfg, data);
}

fn run_burrito(ctx: SystemContext, cfg: BurritoCfg, data: BurritoData) {
    let sys_map = systems::load_saved_system_map();
    // TODO: add some way to configure this with files or arguments
    let mut chat_watcher = LogWatcher::new(
        ctx.clone(),
        cfg.clone(),
        data.clone(),
        create_chat_log_readers(&cfg),
        sys_map.clone(),
    );
    let mut game_watcher = LogWatcher::new(
        ctx.clone(),
        cfg.clone(),
        data.clone(),
        create_game_log_readers(&cfg),
        sys_map.clone(),
    );
    loop {
        for event in chat_watcher.get_events() {
            println!("{}", &event.trigger);
            match event.event_type {
                EventType::NeutInRange(dist) => {
                    if dist <= 15 {// TODO: configurable
                        alert::hostiles(dist);
                    }
                }
                _ => {}
            }
        }
        for event in game_watcher.get_events() {
            match event.event_type {
                EventType::FactionSpawn => {
                    alert::faction_spawn(&event.character_name, &event.trigger);
                }
                EventType::DreadSpawn => {
                    alert::special_npc_spawn(&event.character_name, &event.trigger);
                }
                EventType::OfficerSpawn => {
                    alert::officer_spawn(&event.character_name, &event.trigger);
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(cfg.log_update_interval_ms))
    }
}

fn create_chat_log_readers(cfg: &BurritoCfg) -> Vec<LogReader> {
    let mut delve_reader: LogReader = LogReader::new_intel_reader(cfg.clone(), log_reader::IntelChannel::Delve);
    _ = delve_reader.read_to_end();
    let mut querious_reader: LogReader = LogReader::new_intel_reader(cfg.clone(), log_reader::IntelChannel::Querious);
    _ = querious_reader.read_to_end();
    let mut fountain_reader: LogReader = LogReader::new_intel_reader(cfg.clone(), log_reader::IntelChannel::Fountain);
    _ = fountain_reader.read_to_end();
    let mut gj_reader: LogReader = LogReader::new_intel_reader(cfg.clone(), log_reader::IntelChannel::Gj);
    _ = gj_reader.read_to_end();
    let mut log_readers: Vec<LogReader> = vec![];
    log_readers.push(delve_reader);
    log_readers.push(querious_reader);
    log_readers.push(fountain_reader);
    log_readers.push(gj_reader);
    log_readers
}

fn create_game_log_readers(cfg: &BurritoCfg) -> Vec<LogReader> {
    const NUM_GAME_LOGS: u32 = 10;// TODO: configurable
    LogReader::new_game_log_readers(cfg.clone(), NUM_GAME_LOGS)
}
