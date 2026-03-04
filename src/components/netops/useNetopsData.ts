import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  NetopsTool, ToolResult, PingResult, PortScanResult,
  DnsLookupResult, WhoisResult, WifiNetwork, HttpHeaderResult,
  SslCertInfo, GeoIpResult, ArpEntry, SubnetCalcResult,
  ReverseDnsResult, TracerouteResult, HistoryEntry, NetopsStore,
} from "./types";

export function useNetopsData(): NetopsStore {
  const [activeTool, setActiveTool] = createSignal<NetopsTool>("ping");
  const [utc, setUtc] = createSignal("");
  const [target, setTarget] = createSignal("");
  const [extraParam, setExtraParam] = createSignal("");
  const [result, setResult] = createSignal<ToolResult | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [pingHistory, setPingHistory] = createSignal<PingResult[]>([]);
  const [history, setHistory] = createSignal<HistoryEntry[]>([]);

  function addHistory(tool: NetopsTool, tgt: string, success: boolean) {
    setHistory((prev) => [
      { tool, target: tgt, timestamp: Date.now(), success },
      ...prev,
    ].slice(0, 50));
  }

  async function runTool() {
    const tool = activeTool();
    const tgt = target().trim();
    if (loading()) return;

    // Tools that don't need a target
    const noTargetTools: NetopsTool[] = ["wifi", "arp"];
    if (!noTargetTools.includes(tool) && !tgt) {
      setError("Target is required");
      return;
    }

    setLoading(true);
    setError("");
    setResult(null);

    try {
      switch (tool) {
        case "ping": {
          const data = await invoke<PingResult>("netops_ping", { host: tgt });
          setResult({ kind: "ping", data });
          setPingHistory((prev) => [...prev, data].slice(-20));
          break;
        }
        case "portscan": {
          const ports = extraParam()
            ? extraParam().split(",").map((p) => parseInt(p.trim())).filter((p) => !isNaN(p))
            : undefined;
          const data = await invoke<PortScanResult>("netops_port_scan", { host: tgt, ports: ports && ports.length > 0 ? ports : null });
          setResult({ kind: "portscan", data });
          break;
        }
        case "dns": {
          const recordType = extraParam() || "A";
          const data = await invoke<DnsLookupResult>("netops_dns_lookup", { hostname: tgt, recordType });
          setResult({ kind: "dns", data });
          break;
        }
        case "whois": {
          const data = await invoke<WhoisResult>("netops_whois", { domain: tgt });
          setResult({ kind: "whois", data });
          break;
        }
        case "wifi": {
          const data = await invoke<WifiNetwork[]>("netops_wifi_scan");
          setResult({ kind: "wifi", data });
          break;
        }
        case "httpheaders": {
          const data = await invoke<HttpHeaderResult>("netops_http_headers", { url: tgt });
          setResult({ kind: "httpheaders", data });
          break;
        }
        case "ssl": {
          const data = await invoke<SslCertInfo>("netops_ssl_inspect", { domain: tgt });
          setResult({ kind: "ssl", data });
          break;
        }
        case "geoip": {
          const data = await invoke<GeoIpResult>("netops_ip_geolocate", { ip: tgt });
          setResult({ kind: "geoip", data });
          break;
        }
        case "arp": {
          const data = await invoke<ArpEntry[]>("netops_arp_table");
          setResult({ kind: "arp", data });
          break;
        }
        case "subnet": {
          const data = await invoke<SubnetCalcResult>("netops_subnet_calc", { cidr: tgt });
          setResult({ kind: "subnet", data });
          break;
        }
        case "reversedns": {
          const data = await invoke<ReverseDnsResult>("netops_reverse_dns", { ip: tgt });
          setResult({ kind: "reversedns", data });
          break;
        }
        case "traceroute": {
          const data = await invoke<TracerouteResult>("netops_traceroute", { target: tgt });
          setResult({ kind: "traceroute", data });
          break;
        }
      }
      addHistory(tool, tgt, true);
    } catch (err: any) {
      setError(String(err));
      addHistory(tool, tgt, false);
    }
    setLoading(false);
  }

  function statusText(): string {
    if (loading()) return "SCANNING...";
    if (error()) return `ERROR: ${error()}`;
    if (result()) return "COMPLETE";
    return "READY";
  }

  return {
    activeTool, setActiveTool, utc, setUtc,
    target, setTarget, extraParam, setExtraParam,
    result, setResult, loading, setLoading, error, setError,
    pingHistory, setPingHistory, history, setHistory,
    runTool, statusText,
  };
}
