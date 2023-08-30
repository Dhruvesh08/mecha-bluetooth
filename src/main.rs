#[tokio::main]
async fn main() {
    // Create a BlueZ session and get the adapter
    let session = bluer::Session::new().await.expect("Failed to create session");
    let adapter_name = "hci0"; // Change this to the appropriate adapter name
    let adapter = Adapter::new(session, adapter_name).await.expect("Failed to get adapter");

    // Enable Bluetooth
    adapter.set_powered(true).await.expect("Failed to enable Bluetooth");

    println!("Bluetooth has been enabled.");
}
