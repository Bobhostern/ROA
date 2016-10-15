extern crate rand;
extern crate kei;
#[macro_use]
extern crate clap;

use clap::{Arg, App, SubCommand, AppSettings};
use rand::Rng;

// Our key generator, but is NOT distributed w/ the app
fn main() {
    let matches = App::with_defaults("keigen")
                        .setting(AppSettings::SubcommandRequired)
                        .setting(AppSettings::GlobalVersion)
                        .about("Create/checks keys for a kei-protected application")
                        .subcommand(SubCommand::with_name("generate")
                                    .about("Generates keys")
                                    .arg(Arg::from_usage("[debug] -d, --debug 'Adds debug output'"))
                                    .arg(Arg::from_usage("[random_seed] -r, --random 'Use a random seed for key generation'")
                                        .conflicts_with("custom_seed"))
                                    .arg(Arg::from_usage("<custom_seed> -c, --custom_seed=<SEED1> <SEED2> 'Sets seed values'"))
                                    .arg(Arg::from_usage("[userdata1] -u, --userdata1=[UD] 'Sets userdata 1 (byte-folded)'"))
                                    .arg(Arg::from_usage("[userdata2] -v, --userdata2=[UD] 'Sets userdata 2 (byte-folded)'"))
                                    .arg(Arg::from_usage("[userdata3] -w, --userdata3=[UD] 'Sets userdata 3 (number)'"))
                                    .arg(Arg::from_usage("[userdata4] -t, --userdata4=[UD] 'Sets userdata 4 (number)'")))
                        .subcommand(SubCommand::with_name("check")
                                    .about("Checks keys for validity")
                                    .arg(Arg::from_usage("<key>"))
                                    .arg(Arg::from_usage("[repeat] -r, --repeat 'Repeat the key (useful for batch recording)'")))
                        .subcommand(SubCommand::with_name("info")
                                    .about("Gets key information (w/o checking for validity)")
                                    .arg(Arg::from_usage("<key>")))
                .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        let seed: (u64, u64);
        let debug = matches.is_present("debug");
        if matches.is_present("random_seed") {
            seed = rand::thread_rng().gen();
        } else {
            // custom_seed* must be defined
            seed = (
                matches.values_of("custom_seed").unwrap().nth(0).unwrap().parse().expect("Seed values should be a number"),
                matches.values_of("custom_seed").unwrap().nth(1).unwrap().parse().expect("Seed values should be a number"));
        }
        let mut key = kei::Key::generate(seed);
        if let Some(maty) = matches.value_of("userdata1") {
            key.set_userdata(0, maty.bytes().fold(0u64, |a, i| a.overflowing_add(i as u64).0));
        }
        if let Some(maty) = matches.value_of("userdata2") {
            key.set_userdata(1, maty.bytes().fold(0u64, |a, i| a.overflowing_add(i as u64).0));
        }
        if let Some(maty) = matches.value_of("userdata3") {
            key.set_userdata(2, maty.parse().expect("userdata3 should be an integer"));
        }
        if let Some(maty) = matches.value_of("userdata4") {
            key.set_userdata(3, maty.parse().expect("userdata3 should be an integer"));
        }
        if debug {
            println!("Seed: {:?}\n{:#?}", seed, key);
        }
        println!("{}-{}", Into::<String>::into(key), key.checksum());
    } else if let Some(matches) = matches.subcommand_matches("check") {
        let key = matches.value_of("key").unwrap();
        if matches.is_present("repeat") {
            println!("{} -> {:?}", key, kei::Key::check_key_from_string(key))
        } else {
            println!("Validity: {:?}", kei::Key::check_key_from_string(key))
        }
    } else if let Some(matches) = matches.subcommand_matches("info") {
        let key = matches.value_of("key").unwrap();
        match  kei::Key::parse_key(key) {
            Some(key) => println!("{:#?}", key),
            None => println!("Invalid key!")
        }
    }
}
