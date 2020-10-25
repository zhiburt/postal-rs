


use postal_rs::{Client, DetailsInterest, Message, SendResult};
use std::env;
use std::process;

fn help() -> &'static str {
    r#"
An console application which sends an email via `Postal`.

A binary expects to get:
    A subject as a first argument.
    A message as the second one.
    `From` email as the third one.
    A list of `To` emails as the next argumets (At least one To email should be provided).

There must be set `POSTAL_ADDRESS` and `POSTAL_TOKEN` to use this application.
"#
}

#[tokio::main]
async fn main() {
    let address = env::var("POSTAL_ADDRESS").unwrap_or_default();
    let token = env::var("POSTAL_TOKEN").unwrap_or_default();
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 || address.is_empty() || token.is_empty() {
        println!("{}", help());
        process::exit(-1);
    }

    let subject = &args[1];
    let message = &args[2];
    let from = &args[3];
    let to = &args[4..];

    let client = Client::new(address, token).expect("A client creating error");

    let message = Message::default()
        .to(to)
        .from(from)
        .subject(subject)
        .text(message);
    let results = client
        .send(message)
        .await
        .expect("An error occured while sending an email");

    for SendResult { to, id } in results {
        println!("Message to {}", to);

        let interest = DetailsInterest::new(id).with_details().with_status();
        let data = client
            .get_message_details(interest)
            .await
            .expect("An error occured while gettings details of the message");

        println!("Details");
        println!("{}", serde_json::to_string_pretty(&data).unwrap());

        let data = client
            .get_message_deliveries(id)
            .await
            .expect("An error occured while gettings deliveries of the message");

        println!("Deliveries");
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    }
}
