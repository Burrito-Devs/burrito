use std::{io::{BufReader, Write}, fs::File, thread};
use rodio::{Decoder, OutputStream, Sink};
use termcolor::{StandardStream, ColorSpec, WriteColor};

// TODO: standardize color printing

pub fn hostiles(dist: u32, sound_file: &str) {
    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
    _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Red)).set_bold(true));
    _ = write!(&mut stdout, "Alert! Hostiles {} jumps away!", dist);
    _ = stdout.set_color(ColorSpec::new().set_fg(None).set_bg(None).set_bold(false));
    _ = writeln!(&mut stdout, "");
    //play_file("/home/bernard/Downloads/metal_pipes.mp3".to_owned());
    play_file(sound_file.to_owned());
}

pub fn faction_spawn(character_name: &str, trigger: &str, sound_file: &str) {
    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
    _ = stdout.set_color(ColorSpec::new()
        .set_bg(Some(termcolor::Color::White))
        //.set_fg(Some(termcolor::Color::Rgb(252, 119, 3)))
        .set_fg(Some(termcolor::Color::Green))
        .set_bold(true));
    _ = write!(&mut stdout, "[{}] {}", character_name, trigger);
    _ = stdout.set_color(ColorSpec::new().set_fg(None).set_bg(None).set_bold(false));
    _ = writeln!(&mut stdout, "");
    play_file(sound_file.to_owned());
}

pub fn special_npc_spawn(character_name: &str, trigger: &str, sound_file: &str) {
    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
    _ = stdout.set_color(ColorSpec::new()
        .set_bg(Some(termcolor::Color::Red))
        .set_fg(Some(termcolor::Color::White))
        .set_bold(true));
    _ = write!(&mut stdout, "[{}] {}", character_name, trigger);
    _ = stdout.set_color(ColorSpec::new().set_fg(None).set_bg(None).set_bold(false));
    _ = writeln!(&mut stdout, "");
    play_file(sound_file.to_owned());
}

pub fn officer_spawn(character_name: &str, trigger: &str, sound_file: &str) {
    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
    _ = stdout.set_color(ColorSpec::new()
        .set_bg(Some(termcolor::Color::White))
        .set_fg(Some(termcolor::Color::Magenta))
        .set_bold(true));
    _ = write!(&mut stdout, "[{}] {}", character_name, trigger);
    _ = stdout.set_color(ColorSpec::new().set_fg(None).set_bg(None).set_bold(false));
    _ = writeln!(&mut stdout, "");
    play_file(sound_file.to_owned());
}

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
