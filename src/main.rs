use clap::{Arg, ArgAction, ArgMatches, Command};
use cronjob::CronJob;
use dotenvy::dotenv_override;
use headless_chrome::{Browser, LaunchOptions};
use std::{env, thread, time};

fn login(
    username: &str,
    password: &str,
    default_gateway: &str,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .ignore_certificate_errors(true)
            .build()
            .expect("Could not find Chrome binary"),
    )?;

    if debug {
        println!("[1/5] Opening page ...");
    }
    let tab = browser.new_tab()?;
    tab.navigate_to(&default_gateway)?;
    tab.wait_until_navigated()?;

    if debug {
        println!("[2/5] Logging in ...");
    }

    tab.wait_for_element("#loginform-username")?.click()?;
    tab.type_str(&username)?.press_key("Enter")?;

    tab.wait_for_element("#loginform-password")?.click()?;
    tab.type_str(&password)?.press_key("Enter")?;

    if debug {
        println!("[3/5] Confirming login ...");
    }
    tab.wait_until_navigated()?;

    if debug {
        println!("[4/5] Rebooting connection ...");
    }
    thread::sleep(time::Duration::from_secs(3));
    tab.wait_for_element(".ubnt-icon--refresh")?.click()?;

    if debug {
        println!("[5/5] Done!");
    }

    Ok(())
}

fn reboot_router() -> Result<(), Box<dyn std::error::Error>> {
    dotenv_override()?;
    let username = env::var("USERNAME").expect("USERNAME is not defined");
    let password = env::var("PASSWORD").expect("PASSWORD is not defined");
    let default_gateway = env::var("DEFAULT_GATEWAY").expect("DEFAULT_GATEWAY is not defined");

    let matches = get_args();
    let debug = *matches.get_one::<bool>("debug").unwrap_or(&false);

    login(&username, &password, &default_gateway, debug)?;

    Ok(())
}

fn on_cron(str: &str) {
    println!("Running cron job: {}", str);

    // TODO: Handle this error
    if let Err(_e) = reboot_router() {
        println!("Error!");
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

    if let Some(matches) = matches.subcommand_matches("cron") {
        println!("Running as a cron job");

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

        println!("Closing in 3 seconds ...");
        thread::sleep(time::Duration::from_secs(3));
    }

    Ok(())
}
