use std::{io::{BufReader, Write}, fs::File, thread};
use rodio::{Decoder, OutputStream, Sink};
use termcolor::{StandardStream, ColorSpec, WriteColor};

use super::log_watcher::{EventType, LogEvent};

fn play_file(path: String) {// TODO: Remove panics
    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let file = BufReader::new(File::open(path).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}

pub fn alert(event: &LogEvent, trigger: &str, character_or_system_name: &str, sound_file: Option<&str>) {
    let event_type = event.event_type.to_owned();
    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
    _ = stdout.set_color(&get_color_spec(&event_type));
    match event_type {
        EventType::RangeOfSystem(_, _) | EventType::RangeOfCharacter(_, _) => {
            _ = write!(&mut stdout, "{}", event.message);
        },
        EventType::FactionSpawn | EventType::DreadSpawn | EventType::OfficerSpawn => {
            _ = write!(&mut stdout, "[{}] {}", character_or_system_name, trigger);
        },
        _ => {
            // TODO: Everything else
        },
    }
    if let Some(filename) = sound_file {
        play_file(filename.to_owned());
    }
    _ = stdout.set_color(ColorSpec::new().set_fg(None).set_bg(None).set_bold(false));
    _ = writeln!(&mut stdout, "");
}

fn get_color_spec(event_type: &EventType) -> ColorSpec {
    match event_type {
        EventType::RangeOfSystem(_, _) | EventType::RangeOfCharacter(_, _) => {
            ColorSpec::new()
                .set_bg(None)
                .set_fg(Some(termcolor::Color::Red))
                .set_bold(true)
                .to_owned()
        },
        EventType::FactionSpawn => {
            ColorSpec::new()
                .set_bg(Some(termcolor::Color::White))
                //.set_fg(Some(termcolor::Color::Rgb(252, 119, 3)))
                .set_fg(Some(termcolor::Color::Green))
                .set_bold(true)
                .to_owned()
        },
        EventType::DreadSpawn => {
            ColorSpec::new()
                .set_bg(Some(termcolor::Color::Red))
                .set_fg(Some(termcolor::Color::White))
                .set_bold(true)
                .to_owned()
        },
        EventType::OfficerSpawn => {
            ColorSpec::new()
                .set_bg(Some(termcolor::Color::White))
                .set_fg(Some(termcolor::Color::Magenta))
                .set_bold(true)
                .to_owned()
        },
        _ => {
            ColorSpec::new().set_fg(None).set_bg(None).set_bold(false).to_owned()
        },
    }
}
