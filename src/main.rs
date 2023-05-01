use lib::ServerResult;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ServerResult {
    color_eyre::install()?;
    lib::try_main().await?;
    Ok(())
}
