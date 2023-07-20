use dotenvy::dotenv_override;
use headless_chrome::{Browser, LaunchOptions};
use std::{env, thread, time};

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

    println!("[1/5] Opening page ...");
    let tab = browser.new_tab()?;
    tab.navigate_to(&default_gateway)?;
    tab.wait_until_navigated()?;

    println!("[2/5] Logging in ...");

    tab.wait_for_element("#loginform-username")?.click()?;
    tab.type_str(&username)?.press_key("Enter")?;

    tab.wait_for_element("#loginform-password")?.click()?;
    tab.type_str(&password)?.press_key("Enter")?;

    println!("[3/5] Confirming login ...");
    tab.wait_until_navigated()?;

    println!("[4/5] Rebooting connection ...");
    thread::sleep(time::Duration::from_secs(3));
    tab.wait_for_element(".ubnt-icon--refresh")?.click()?;

    println!("[5/5] Done!");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv_override()?;
    let username = env::var("USERNAME").expect("USERNAME is not defined");
    let password = env::var("PASSWORD").expect("PASSWORD is not defined");
    let default_gateway = env::var("DEFAULT_GATEWAY").expect("DEFAULT_GATEWAY is not defined");

    login(&username, &password, &default_gateway)?;

    println!("Closing in 3 seconds ...");

    thread::sleep(time::Duration::from_secs(3));

    Ok(())
}
