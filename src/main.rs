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
        if device.name.unwrap() == "Jarvis" {
            println!("Found Jarvis!");
            controller.connect_device(device.address).await?;
            println!("Connected to Jarvis!");
            break;
        }
    }

    Ok(())
}
