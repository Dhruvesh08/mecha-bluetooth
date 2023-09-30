use bluer::{
    Adapter, AdapterEvent, Address, AddressType, Device, DeviceEvent, DiscoveryFilter,
    DiscoveryTransport, Result, Session, Uuid,
};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env, time::Duration};
use tokio::time::timeout;

pub struct BluetoothController {
    session: Session,
}

pub struct BluetoothScanResult {
    // Define fields to hold the scan results or any other relevant information.
    pub discovered_devices: Vec<DeviceInfo>,
    // Add more fields as needed.
}

#[derive(Debug)]
pub struct DeviceInfo {
    // Define fields to hold information about a Bluetooth device.
    pub address: Address,
    pub name: Option<String>,
    pub is_connected: bool,
    pub device_id: HashSet<Uuid>,
    pub is_pairing: bool,
    pub is_trusted: bool,
    pub address_type: AddressType,
    // Add more fields as needed.
}

impl BluetoothController {
    pub async fn new() -> Result<Self> {
        let session = Session::new().await?;
        Ok(Self { session })
    }

    pub async fn start(&self) -> Result<()> {
        let adapter = self.session.default_adapter().await?;
        adapter.set_powered(true).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let adapter = self.session.default_adapter().await?;
        adapter.set_powered(false).await?;
        Ok(())
    }

    pub async fn scan_bluetooth(
        &self,
        scan_duration_secs: Option<u64>,
    ) -> Result<BluetoothScanResult> {
        let scan_duration_secs = scan_duration_secs.unwrap_or(5);
        let with_changes = env::args().any(|arg| arg == "--changes");
        let all_properties = env::args().any(|arg| arg == "--all-properties");
        let le_only = env::args().any(|arg| arg == "--le");
        let br_edr_only = env::args().any(|arg| arg == "--bredr");
        let filter_addr: HashSet<_> = env::args()
            .filter_map(|arg| arg.parse::<Address>().ok())
            .collect();

        let adapter = self.session.default_adapter().await?;
        if adapter.is_discovering().await? {
            println!("Waiting for the existing discovery process to complete...");
            // Wait for the existing discovery process to complete
            while adapter.is_discovering().await? {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            println!("Existing discovery process has completed.");
        }

        let mut discovered_devices = Vec::new();
        let filter = DiscoveryFilter {
            transport: if le_only {
                DiscoveryTransport::Le
            } else if br_edr_only {
                DiscoveryTransport::BrEdr
            } else {
                DiscoveryTransport::Auto
            },
            ..Default::default()
        };

        adapter.set_discovery_filter(filter).await?;
        println!(
            "Using discovery filter:\n{:#?}\n\n",
            adapter.discovery_filter().await
        );

        let device_events = adapter.discover_devices().await?;
        pin_mut!(device_events);

        let mut all_change_events = SelectAll::new();

        // Track the start time of the scan.
        let start_time = std::time::Instant::now();
        let scan_duration = std::time::Duration::from_secs(scan_duration_secs);

        // Use tokio::time::timeout to implement the timeout mechanism.
        let scan_result = timeout(scan_duration, async {
            loop {
                tokio::select! {
                    Some(device_event) = device_events.next() => {
                        match device_event {
                            AdapterEvent::DeviceAdded(addr) => {
                                if !filter_addr.is_empty() && !filter_addr.contains(&addr) {
                                    continue;
                                }

                                if with_changes {
                                    let device = adapter.device(addr)?;
                                    let change_events = device.events().await?.map(move |evt| (addr, evt));
                                    all_change_events.push(change_events);
                                }

                                let device_info = Self::query_device(&adapter, addr).await?;
                                println!("    {:?}", &device_info);
                                discovered_devices.push(device_info);
                            }
                            AdapterEvent::DeviceRemoved(addr) => {
                                println!("Device removed: {addr}");
                            }
                            _ => (),
                        }
                        println!();
                    }
                    Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                        println!("Device changed: {addr}");
                        println!("    {property:?}");
                    }
                    else => {
                        // Check if the elapsed time has exceeded the specified scan duration.
                        if std::time::Instant::now() - start_time >= scan_duration {
                            break;
                        }
                    }
                }
            }

            // Reset the discovery filter to stop scanning.
            let no_filter = DiscoveryFilter {
                transport: DiscoveryTransport::default(),
                ..Default::default()
            };
            adapter.set_discovery_filter(no_filter).await?;
            println!(
                "Discover Bluetooth device Number {}",
                discovered_devices.len()
            );

            Ok(BluetoothScanResult { discovered_devices })
        });

        // Handle the result of the timeout.
        match scan_result.await {
            Ok(result) => result,
            Err(_) => {
                // Handle the timeout case here, e.g., return an empty list or an error.
                Ok(BluetoothScanResult {
                    discovered_devices: Vec::new(),
                })
            }
        }
    }

    async fn query_device(adapter: &Adapter, addr: Address) -> bluer::Result<DeviceInfo> {
        let device = adapter.device(addr)?;
        Ok(DeviceInfo {
            address: addr.clone(),
            name: Self::query_device_name(&adapter, addr).await.ok(),
            is_connected: device.is_connected().await?,
            device_id: device.uuids().await?.unwrap_or_default(),
            is_pairing: device.is_paired().await?,
            is_trusted: device.is_trusted().await?,
            address_type: device.address_type().await?,
        })
    }

    async fn query_all_device_properties(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
        let device = adapter.device(addr)?;
        let props = device.all_properties().await?;
        for prop in props {
            println!("    {:?}", &prop);
        }
        Ok(())
    }

    async fn query_device_name(adapter: &Adapter, addr: Address) -> bluer::Result<String> {
        let device = adapter.device(addr)?;
        device.name().await.map(|name| name.unwrap_or_default())
    }

    pub async fn remove_device(&self, address: Address) -> Result<()> {
        let adapter = self.session.default_adapter().await?;
        adapter.remove_device(address).await?;
        Ok(())
    }

    pub async fn connect_device(
        &self,
        address: Address,
        // address_type: AddressType,
    ) -> Result<Device> {
        let adapter = self.session.default_adapter().await?;
        let device = adapter.device(address)?;
        device.connect().await?;
        Ok(device)
    }
}
