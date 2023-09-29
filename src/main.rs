mod bluetooth;
pub use bluetooth::BluetoothController;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    //print all bluetooth devices
    let controller = BluetoothController::new().await?;
    let bt_list = controller.scan_bluetooth().await?;
    for device in bt_list.discovered_devices {
        println!("{:?}: {:?}", device.name, device.address);
    }
    Ok(())
}
