use clap::{Arg, ArgAction, ArgMatches, Command};
use cronjob::CronJob;
use dotenvy::dotenv_override;
use headless_chrome::{Browser, LaunchOptions};
use log::{debug, error, info};
use std::{env, path::PathBuf, thread, time};

fn login(
    username: &str,
    password: &str,
    default_gateway: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .ignore_certificate_errors(true)
            .build()
            .expect("Could not find Chrome binary"),
    )?;

    debug!("[1/5] Opening page ...");

    let tab = browser.new_tab()?;
    tab.navigate_to(&default_gateway)?;
    tab.wait_until_navigated()?;

    debug!("[2/5] Logging in ...");

    tab.wait_for_element("#loginform-username")?.click()?;
    tab.type_str(&username)?.press_key("Enter")?;

    tab.wait_for_element("#loginform-password")?.click()?;
    tab.type_str(&password)?.press_key("Enter")?;

    debug!("[3/5] Confirming login ...");

    tab.wait_until_navigated()?;

    debug!("[4/5] Rebooting connection ...");

    thread::sleep(time::Duration::from_secs(3));
    tab.wait_for_element(".ubnt-icon--refresh")?.click()?;

    debug!("[5/5] Done!");

    Ok(())
}

fn reboot_router() -> Result<(), Box<dyn std::error::Error>> {
    dotenv_override()?;
    let username = env::var("USERNAME").expect("USERNAME is not defined");
    let password = env::var("PASSWORD").expect("PASSWORD is not defined");
    let default_gateway = env::var("DEFAULT_GATEWAY").expect("DEFAULT_GATEWAY is not defined");

    login(&username, &password, &default_gateway)?;

    Ok(())
}

fn on_cron(str: &str) {
    info!("Running cron job: {}", str);

    // TODO: Handle this error
    if let Err(e) = reboot_router() {
        error!("{}", e);
    }
}

fn get_args() -> ArgMatches {
    Command::new("Reboot Ubiquiti Nano Beam")
        .arg(
            Arg::new("debug")
                .short('d')
                .help("Turn debugging information on")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("log")
                .long("logfile")
                .short('l')
                .help("Path to the file containing the logs")
                .value_parser(clap::value_parser!(PathBuf))
                .action(ArgAction::Set),
        )
        .subcommand(
            Command::new("cron")
                .long_flag("cronjob")
                .short_flag('c')
                .about("Runs the application as a cron job")
                .arg(
                    Arg::new("sec")
                        .long("seconds")
                        .short('s')
                        .required(false)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("min")
                        .long("minutes")
                        .short('m')
                        .required(false)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("hour")
                        .long("hours")
                        .required(false)
                        .action(ArgAction::Set),
                ),
        )
        .get_matches()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = get_args();

    let log_level = match matches.get_one::<bool>("debug") {
        Some(true) => log::LevelFilter::Debug,
        _ => log::LevelFilter::Info,
    };

    let log_target = if let Some(log_file) = matches.get_one::<PathBuf>("log") {
        env_logger::Target::Pipe(Box::from(std::fs::File::create(log_file)?))
    } else {
        env_logger::Target::Stdout
    };

    env_logger::builder()
        .filter_level(log_level)
        .filter_module("headless_chrome", log::LevelFilter::Warn)
        .filter_module("tungstenite", log::LevelFilter::Warn)
        .target(log_target)
        .init();

    if let Some(matches) = matches.subcommand_matches("cron") {
        info!("Running as a cron job");

        let mut cron = CronJob::new("Reboot Ubiquiti Nano Beam Cron Job", on_cron);

        if let Some(seconds) = matches.get_one::<String>("sec") {
            cron.seconds(seconds);
        }

        if let Some(minutes) = matches.get_one::<String>("min") {
            cron.minutes(minutes);
        }

        if let Some(hours) = matches.get_one::<String>("hour") {
            cron.hours(hours);
        }

        cron.start_job();
    } else if !(matches.get_one::<bool>("help").unwrap_or(&false)) {
        reboot_router()?;

        info!("Closing in 3 seconds ...");
        thread::sleep(time::Duration::from_secs(3));
    }

    Ok(())
}
