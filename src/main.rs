mod bluetooth;
pub use bluetooth::BluetoothController;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    //print all bluetooth devices
    let controller = BluetoothController::new().await?;
    controller.scan_bluetooth();

    Ok(())
}
