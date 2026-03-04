import CoreWLAN
import Foundation

func decodeSecurity(_ raw: Int) -> String {
    if raw == 0 { return "Open" }
    let known: [Int: String] = [
        1: "WEP",
        8: "WPA",
        12: "WPA/WPA2",
        64: "WPA2",
        192: "WPA2",
        128: "WPA2-Enterprise",
        256: "WPA2-Enterprise",
        512: "WPA3",
        576: "WPA2/WPA3",
        768: "WPA3-Enterprise",
        4224: "WPA3-Enterprise",
    ]
    if let name = known[raw] { return name }
    var gen: [String] = []
    if raw & 0x3E != 0 { gen.append("WPA") }
    if raw & 0x1C0 != 0 { gen.append("WPA2") }
    if raw & 0x1E00 != 0 { gen.append("WPA3") }
    if gen.isEmpty { return "Secured" }
    return gen.joined(separator: "/")
}

if let iface = CWWiFiClient.shared().interface() {
    do {
        let networks = try iface.scanForNetworks(withSSID: nil)
        for n in networks.sorted(by: { $0.rssiValue > $1.rssiValue }) {
            let obj = n as NSObject
            let raw = (obj.value(forKey: "securityType") as? Int) ?? 0
            let sec = decodeSecurity(raw)
            print("\(n.ssid ?? "(hidden)")|\(n.bssid ?? "")|\(n.rssiValue)|\(n.wlanChannel?.channelNumber ?? 0)|\(sec)")
        }
    } catch {
        fputs("ERROR: \(error)\n", stderr)
        Darwin.exit(1)
    }
} else {
    fputs("ERROR: No WiFi interface found\n", stderr)
    Darwin.exit(1)
}
