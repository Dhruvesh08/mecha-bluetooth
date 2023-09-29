mod bluetooth;
pub use bluetooth::BluetoothController;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    //print all bluetooth devices
    let controller = BluetoothController::new().await?;
    println!("Scanning for bluetooth devices...");
    let bt_list = controller.scan_bluetooth(None).await?;
    print!("Found {} devices: ", bt_list.discovered_devices.len());
    for device in bt_list.discovered_devices {
        println!("{}: {:?}", device.name.unwrap(), device.address);
    }  
    Ok(())
}
