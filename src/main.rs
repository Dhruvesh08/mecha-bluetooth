mod bluetooth;
pub use bluetooth::BluetoothController;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    //print all bluetooth devices
    let controller = BluetoothController::new().await?;
    let _ = controller.scan_bluetooth().await?;
    Ok(())
}
