import { Show, For, Switch, Match } from "solid-js";
import type { NetopsStore, NetopsTool } from "./types";

interface Props {
  store: NetopsStore;
}

const TOOL_PLACEHOLDERS: Record<NetopsTool, string> = {
  ping: "google.com",
  portscan: "192.168.1.1",
  dns: "example.com",
  whois: "github.com",
  wifi: "",
  httpheaders: "https://github.com",
  ssl: "github.com",
  geoip: "8.8.8.8",
  arp: "",
  subnet: "192.168.1.0/24",
  reversedns: "8.8.8.8",
  traceroute: "google.com",
};

const TOOL_LABELS: Record<NetopsTool, string> = {
  ping: "Host",
  portscan: "Host",
  dns: "Hostname",
  whois: "Domain",
  wifi: "",
  httpheaders: "URL",
  ssl: "Domain",
  geoip: "IP Address",
  arp: "",
  subnet: "CIDR",
  reversedns: "IP Address",
  traceroute: "Target",
};

const DNS_TYPES = ["A", "AAAA", "MX", "CNAME", "TXT", "NS", "SOA"];
const NO_TARGET_TOOLS: NetopsTool[] = ["wifi", "arp"];

export default function ResultPanel(props: Props) {
  const tool = () => props.store.activeTool();
  const needsTarget = () => !NO_TARGET_TOOLS.includes(tool());
  const needsExtra = () => tool() === "dns" || tool() === "portscan";

  function handleSubmit(e: Event) {
    e.preventDefault();
    props.store.runTool();
  }

  return (
    <div class="nops-center">
      {/* Input bar */}
      <form class="nops-input-bar" onSubmit={handleSubmit}>
        <Show when={needsTarget()}>
          <input
            class="nops-input"
            type="text"
            placeholder={TOOL_PLACEHOLDERS[tool()]}
            value={props.store.target()}
            onInput={(e) => props.store.setTarget(e.currentTarget.value)}
            autofocus
          />
        </Show>

        <Show when={tool() === "dns"}>
          <select
            class="nops-select"
            value={props.store.extraParam() || "A"}
            onChange={(e) => props.store.setExtraParam(e.currentTarget.value)}
          >
            <For each={DNS_TYPES}>
              {(t) => <option value={t}>{t}</option>}
            </For>
          </select>
        </Show>

        <Show when={tool() === "portscan"}>
          <input
            class="nops-input nops-input-small"
            type="text"
            placeholder="Ports (e.g. 22,80,443)"
            value={props.store.extraParam()}
            onInput={(e) => props.store.setExtraParam(e.currentTarget.value)}
          />
        </Show>

        <button
          class={`nops-run-btn ${props.store.loading() ? "loading" : ""}`}
          type="submit"
          disabled={props.store.loading()}
        >
          {props.store.loading() ? "SCANNING..." : "RUN"}
        </button>
      </form>

      {/* Error */}
      <Show when={props.store.error()}>
        <div class="nops-error">{props.store.error()}</div>
      </Show>

      {/* Results */}
      <div class="nops-results">
        <Show when={props.store.result()}>
          {(res) => (
            <Switch>
              <Match when={res().kind === "ping"}>
                {(() => {
                  const d = res() as { kind: "ping"; data: import("./types").PingResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">PING {d.data.host}</div>
                      <div class="nops-kv">
                        <span>STATUS</span>
                        <span class={d.data.status === "ok" ? "nops-status-open" : "nops-status-closed"}>{d.data.status.toUpperCase()}</span>
                      </div>
                      <div class="nops-kv">
                        <span>LATENCY</span>
                        <span>{d.data.latency_ms.toFixed(1)} ms</span>
                      </div>
                      <Show when={props.store.pingHistory().length > 1}>
                        <div class="nops-panel-hdr" style="margin-top: 12px">LATENCY HISTORY</div>
                        <div class="nops-sparkline">
                          <For each={props.store.pingHistory()}>
                            {(p) => {
                              const max = Math.max(...props.store.pingHistory().map((h) => h.latency_ms), 1);
                              const height = Math.max(2, (p.latency_ms / max) * 36);
                              return <div class="nops-spark-bar" style={`height: ${height}px`} title={`${p.latency_ms.toFixed(1)}ms`} />;
                            }}
                          </For>
                        </div>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "portscan"}>
                {(() => {
                  const d = res() as { kind: "portscan"; data: import("./types").PortScanResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">PORT SCAN — {d.data.host} ({d.data.scan_duration_ms}ms)</div>
                      <table class="nops-table">
                        <thead><tr><th>PORT</th><th>SERVICE</th><th>STATUS</th></tr></thead>
                        <tbody>
                          <For each={d.data.ports}>
                            {(p) => (
                              <tr>
                                <td>{p.port}</td>
                                <td>{p.service}</td>
                                <td class={`nops-status-${p.status}`}>{p.status.toUpperCase()}</td>
                              </tr>
                            )}
                          </For>
                        </tbody>
                      </table>
                      <div class="nops-result-summary">
                        {d.data.ports.filter((p) => p.status === "open").length} open / {d.data.ports.length} scanned
                      </div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "dns"}>
                {(() => {
                  const d = res() as { kind: "dns"; data: import("./types").DnsLookupResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">DNS — {d.data.hostname} ({d.data.query_time_ms}ms)</div>
                      <Show when={d.data.records.length > 0} fallback={<div class="nops-no-data">No records found</div>}>
                        <table class="nops-table">
                          <thead><tr><th>TYPE</th><th>NAME</th><th>VALUE</th><th>TTL</th></tr></thead>
                          <tbody>
                            <For each={d.data.records}>
                              {(r) => (
                                <tr>
                                  <td class="nops-status-open">{r.record_type}</td>
                                  <td>{r.name}</td>
                                  <td>{r.value}</td>
                                  <td>{r.ttl ?? "—"}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "whois"}>
                {(() => {
                  const d = res() as { kind: "whois"; data: import("./types").WhoisResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">WHOIS — {d.data.domain}</div>
                      <div class="nops-kv"><span>REGISTRAR</span><span>{d.data.registrar || "—"}</span></div>
                      <div class="nops-kv"><span>CREATED</span><span>{d.data.creation_date || "—"}</span></div>
                      <div class="nops-kv"><span>EXPIRES</span><span>{d.data.expiry_date || "—"}</span></div>
                      <Show when={d.data.name_servers.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">NAME SERVERS</div>
                        <For each={d.data.name_servers}>
                          {(ns) => <div class="nops-list-item">{ns}</div>}
                        </For>
                      </Show>
                      <Show when={d.data.status.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">STATUS</div>
                        <For each={d.data.status}>
                          {(s) => <div class="nops-list-item">{s}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "wifi"}>
                {(() => {
                  const d = res() as { kind: "wifi"; data: import("./types").WifiNetwork[] };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">WiFi NETWORKS ({d.data.length} found)</div>
                      <Show when={d.data.length > 0} fallback={<div class="nops-no-data">No networks found</div>}>
                        <table class="nops-table">
                          <thead><tr><th>SSID</th><th>SIGNAL</th><th>CH</th><th>SECURITY</th></tr></thead>
                          <tbody>
                            <For each={d.data}>
                              {(n) => (
                                <tr>
                                  <td>{n.ssid || "(hidden)"}</td>
                                  <td class={n.rssi > -50 ? "nops-status-open" : n.rssi > -70 ? "nops-status-filtered" : "nops-status-closed"}>
                                    {n.rssi} dBm {signalBars(n.rssi)}
                                  </td>
                                  <td>{n.channel}</td>
                                  <td>{n.security}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "httpheaders"}>
                {(() => {
                  const d = res() as { kind: "httpheaders"; data: import("./types").HttpHeaderResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">HTTP HEADERS — {d.data.status_code} ({d.data.response_time_ms}ms)</div>

                      <div class="nops-panel-hdr" style="margin-top: 8px">SECURITY HEADERS</div>
                      <For each={d.data.security_headers}>
                        {(h) => (
                          <div class="nops-kv">
                            <span>{h.name}</span>
                            <span class={`nops-badge nops-badge-${h.rating}`}>
                              {h.present ? "PRESENT" : "MISSING"}
                            </span>
                          </div>
                        )}
                      </For>

                      <div class="nops-panel-hdr" style="margin-top: 12px">ALL HEADERS</div>
                      <For each={d.data.headers}>
                        {([k, v]) => (
                          <div class="nops-kv">
                            <span>{k}</span>
                            <span class="nops-header-val">{v}</span>
                          </div>
                        )}
                      </For>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "ssl"}>
                {(() => {
                  const d = res() as { kind: "ssl"; data: import("./types").SslCertInfo };
                  const daysClass = () =>
                    d.data.days_remaining > 30 ? "nops-status-open" :
                    d.data.days_remaining > 7 ? "nops-status-filtered" : "nops-status-closed";
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">SSL/TLS — {d.data.domain}</div>
                      <div class="nops-kv"><span>SUBJECT</span><span>{d.data.subject || "—"}</span></div>
                      <div class="nops-kv"><span>ISSUER</span><span>{d.data.issuer || "—"}</span></div>
                      <div class="nops-kv"><span>VALID FROM</span><span>{d.data.valid_from || "—"}</span></div>
                      <div class="nops-kv"><span>VALID TO</span><span>{d.data.valid_to || "—"}</span></div>
                      <div class="nops-kv">
                        <span>DAYS LEFT</span>
                        <span class={daysClass()}>{d.data.days_remaining >= 0 ? d.data.days_remaining : "EXPIRED"}</span>
                      </div>
                      <div class="nops-kv"><span>SERIAL</span><span>{d.data.serial || "—"}</span></div>
                      <div class="nops-kv"><span>PROTOCOL</span><span>{d.data.protocol || "—"}</span></div>
                      <Show when={d.data.sans.length > 0}>
                        <div class="nops-panel-hdr" style="margin-top: 8px">SUBJECT ALT NAMES</div>
                        <For each={d.data.sans}>
                          {(san) => <div class="nops-list-item">{san}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "geoip"}>
                {(() => {
                  const d = res() as { kind: "geoip"; data: import("./types").GeoIpResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">GEO IP — {d.data.ip}</div>
                      <div class="nops-kv"><span>COUNTRY</span><span>{d.data.country} ({d.data.country_code})</span></div>
                      <div class="nops-kv"><span>REGION</span><span>{d.data.region}</span></div>
                      <div class="nops-kv"><span>CITY</span><span>{d.data.city}</span></div>
                      <div class="nops-kv"><span>COORDS</span><span>{d.data.lat.toFixed(4)}, {d.data.lon.toFixed(4)}</span></div>
                      <div class="nops-kv"><span>ISP</span><span>{d.data.isp}</span></div>
                      <div class="nops-kv"><span>ORG</span><span>{d.data.org}</span></div>
                      <div class="nops-kv"><span>TIMEZONE</span><span>{d.data.timezone}</span></div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "arp"}>
                {(() => {
                  const d = res() as { kind: "arp"; data: import("./types").ArpEntry[] };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">ARP TABLE ({d.data.length} devices)</div>
                      <Show when={d.data.length > 0} fallback={<div class="nops-no-data">No ARP entries</div>}>
                        <table class="nops-table">
                          <thead><tr><th>IP</th><th>MAC</th><th>HOST</th><th>IFACE</th></tr></thead>
                          <tbody>
                            <For each={d.data}>
                              {(e) => (
                                <tr>
                                  <td>{e.ip}</td>
                                  <td>{e.mac}</td>
                                  <td>{e.hostname}</td>
                                  <td>{e.interface_name}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "subnet"}>
                {(() => {
                  const d = res() as { kind: "subnet"; data: import("./types").SubnetCalcResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">SUBNET — {d.data.cidr}</div>
                      <div class="nops-kv"><span>NETWORK</span><span>{d.data.network}</span></div>
                      <div class="nops-kv"><span>BROADCAST</span><span>{d.data.broadcast}</span></div>
                      <div class="nops-kv"><span>FIRST HOST</span><span>{d.data.first_host}</span></div>
                      <div class="nops-kv"><span>LAST HOST</span><span>{d.data.last_host}</span></div>
                      <div class="nops-kv"><span>NETMASK</span><span>{d.data.netmask}</span></div>
                      <div class="nops-kv"><span>WILDCARD</span><span>{d.data.wildcard}</span></div>
                      <div class="nops-kv"><span>HOST COUNT</span><span class="nops-status-open">{d.data.host_count.toLocaleString()}</span></div>
                      <div class="nops-kv"><span>PREFIX</span><span>/{d.data.prefix_len}</span></div>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "reversedns"}>
                {(() => {
                  const d = res() as { kind: "reversedns"; data: import("./types").ReverseDnsResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">REVERSE DNS — {d.data.ip} ({d.data.query_time_ms}ms)</div>
                      <Show when={d.data.hostnames.length > 0} fallback={<div class="nops-no-data">No PTR records found</div>}>
                        <For each={d.data.hostnames}>
                          {(h) => <div class="nops-list-item nops-status-open">{h}</div>}
                        </For>
                      </Show>
                    </div>
                  );
                })()}
              </Match>

              <Match when={res().kind === "traceroute"}>
                {(() => {
                  const d = res() as { kind: "traceroute"; data: import("./types").TracerouteResult };
                  return (
                    <div class="nops-output">
                      <div class="nops-result-header">TRACEROUTE — {d.data.target}</div>
                      <table class="nops-table">
                        <thead><tr><th>#</th><th>HOST</th><th>IP</th><th>RTT</th></tr></thead>
                        <tbody>
                          <For each={d.data.hops}>
                            {(hop) => (
                              <tr>
                                <td>{hop.hop}</td>
                                <td>{hop.hostname}</td>
                                <td class={hop.ip === "*" ? "nops-status-closed" : ""}>{hop.ip}</td>
                                <td>
                                  {hop.rtt_ms.length > 0
                                    ? hop.rtt_ms.map((r) => `${r.toFixed(1)}ms`).join(" / ")
                                    : "* * *"}
                                </td>
                              </tr>
                            )}
                          </For>
                        </tbody>
                      </table>
                      <div class="nops-result-summary">
                        {d.data.hops.length} hops — {d.data.completed ? "COMPLETE" : "PARTIAL"}
                      </div>
                    </div>
                  );
                })()}
              </Match>
            </Switch>
          )}
        </Show>

        <Show when={!props.store.result() && !props.store.error() && !props.store.loading()}>
          <div class="nops-placeholder">
            <div class="nops-placeholder-icon">⚡</div>
            <div class="nops-placeholder-text">Select a tool and enter a target to begin</div>
            <div class="nops-placeholder-hint">Press ⌘+Enter or click RUN</div>
          </div>
        </Show>
      </div>
    </div>
  );
}

function signalBars(rssi: number): string {
  if (rssi > -50) return "▂▄▆█";
  if (rssi > -60) return "▂▄▆░";
  if (rssi > -70) return "▂▄░░";
  if (rssi > -80) return "▂░░░";
  return "░░░░";
}
