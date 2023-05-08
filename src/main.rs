use lib::obs;
use lib::ServerResult;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ServerResult {
    let sub = obs::get_sub();
    obs::init_with(sub);

    color_eyre::install()?;
    lib::try_main().await?;
    Ok(())
}
