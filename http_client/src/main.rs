
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // curl -v -H 'Accept-Language:de' 'http://wttr.in/erlangen?0QT'

    let resp = reqwest::Client::new()
        .get("http://wttr.in/erlangen?0QT")
        .header("Accept", "text/plain")
        .header("Accept-Language", "de-DE")
        .header("User-Agent", "curl/8.1.7")
        .send()
        .await?
        .text()
        .await?;

    println!("{resp}");
    Ok(())
}
