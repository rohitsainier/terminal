import { onMount, onCleanup, createEffect, on, Show } from "solid-js";
import type { DashboardMode } from "./types";
import { useMonitorData } from "./useMonitorData";
import { generateThreats } from "./utils";
import {
  initGlobe, destroyGlobe, switchMode, setupGlobeEffects,
  pauseGlobeRotation, type GlobeTimers,
} from "./globeManager";
import TopBar from "./TopBar";
import LeftPanel from "./LeftPanel";
import RightPanel from "./RightPanel";
import GlobeOverlays from "./GlobeOverlays";
import BottomTicker from "./BottomTicker";

interface Props {
  onClose: () => void;
}

export default function MonitorDashboard(props: Props) {
  let globeContainerRef!: HTMLDivElement;
  let globeInstance: any = null;
  const timers: GlobeTimers = { satProp: undefined, autoRotateResume: undefined };

  const store = useMonitorData();

  function handleSwitchMode(newMode: DashboardMode) {
    switchMode(globeInstance, store, timers, newMode);
  }

  function handleFocusGlobe(lat: number, lng: number, alt?: number) {
    if (globeInstance) {
      globeInstance.pointOfView({ lat, lng, altitude: alt ?? 1.5 }, 600);
    }
  }

  function handlePauseRotation() {
    pauseGlobeRotation(globeInstance, timers);
  }

  onMount(async () => {
    // Clocks & timers
    store.setUtc(new Date().toISOString().slice(11, 19));
    const clockTimer = setInterval(() => store.setUtc(new Date().toISOString().slice(11, 19)), 1000);
    const tickerTimer = setInterval(() => store.setTickerOffset((o) => o + 1), 40);

    // Net throughput — reactive toggle
    let netTimer: ReturnType<typeof setInterval> | undefined;
    createEffect(on(store.netMonitorEnabled, (enabled) => {
      if (netTimer) { clearInterval(netTimer); netTimer = undefined; }
      if (enabled) {
        store.fetchNetThroughput();
        netTimer = setInterval(() => store.fetchNetThroughput(), 1000);
      } else {
        store.setNetThroughput(null);
      }
    }));

    // Threats
    store.setThreats(generateThreats(8));
    const threatTimer = setInterval(() => {
      store.setThreats((prev) => [...generateThreats(1), ...prev].slice(0, 14));
    }, 5000);

    // Data
    store.fetchCoreData();
    const dataTimer = setInterval(() => store.fetchCoreData(), 20000);
    const issTimer = setInterval(() => store.fetchISS(), 5000);
    const cryptoTimer = setInterval(() => store.fetchCrypto(), 120000);

    // Globe
    try {
      const { default: Globe } = await import("globe.gl");
      globeInstance = initGlobe(Globe, globeContainerRef, store, timers);
    } catch (err) {
      console.error("[FLUX] Globe init failed:", err);
    }

    // Reactive globe effects
    setupGlobeEffects(() => globeInstance, store);

    // Keyboard
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (store.showModeMenu()) { store.setShowModeMenu(false); return; }
        props.onClose();
      }
      const modeKeys: Record<string, DashboardMode> = {
        "1": "INTEL", "2": "CYBER", "3": "SAT",
        "4": "FLIGHTS", "5": "CAMS", "6": "WEATHER", "7": "QUAKE",
      };
      if (modeKeys[e.key]) {
        handleSwitchMode(modeKeys[e.key]);
        store.setShowModeMenu(false);
      }
      if (e.key === "m" || e.key === "M") store.setStreamMuted((m) => !m);
    };
    window.addEventListener("keydown", onKey);

    onCleanup(() => {
      clearInterval(clockTimer);
      clearInterval(tickerTimer);
      if (netTimer) clearInterval(netTimer);
      clearInterval(threatTimer);
      clearInterval(dataTimer);
      clearInterval(issTimer);
      clearInterval(cryptoTimer);
      if (timers.satProp) clearInterval(timers.satProp);
      clearTimeout(timers.autoRotateResume);
      window.removeEventListener("keydown", onKey);
      destroyGlobe(globeInstance, globeContainerRef);
      globeInstance = null;
    });
  });

  return (
    <div class="fcmd-overlay" onClick={() => props.onClose()}>
      <div class="fcmd-dashboard" onClick={(e) => e.stopPropagation()}>
        <div class="fcmd-scanlines" />
        <div class="fcmd-vignette" />

        <TopBar store={store} onClose={props.onClose} />

        <main class="fcmd-main">
          <LeftPanel
            store={store}
            onFocusGlobe={handleFocusGlobe}
            onPauseRotation={handlePauseRotation}
          />

          <div class="fcmd-globe-wrap" onClick={() => store.showModeMenu() && store.setShowModeMenu(false)}>
            <div ref={globeContainerRef!} class="fcmd-globe-container" />

            <Show when={!store.globeReady()}>
              <div class="fcmd-globe-loading">
                <div class="fcmd-spinner" />
                <span>INITIALIZING 3D GLOBE...</span>
              </div>
            </Show>

            <GlobeOverlays store={store} onModeSwitch={handleSwitchMode} />
          </div>

          <RightPanel store={store} />
        </main>

        <BottomTicker store={store} />
      </div>
    </div>
  );
}
