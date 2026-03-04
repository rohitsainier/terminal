import { Show, For } from "solid-js";
import type { MonitorStore } from "./types";

interface BottomTickerProps {
  store: MonitorStore;
}

export default function BottomTicker(props: BottomTickerProps) {
  const { store } = props;

  return (
    <footer class="fcmd-ticker">
      <div class="fcmd-ticker-label">MARKET</div>
      <div class="fcmd-crypto-strip">
        <Show when={store.crypto().length > 0} fallback={
          <span class="fcmd-ticker-text" style={{ opacity: 0.3 }}>Loading prices...</span>
        }>
          <For each={store.crypto()}>
            {(coin) => (
              <div class="fcmd-crypto-item">
                <span class="fcmd-crypto-symbol">{coin.symbol}</span>
                <span class="fcmd-crypto-price">
                  ${coin.price >= 1 ? coin.price.toLocaleString(undefined, { maximumFractionDigits: 0 }) : coin.price.toFixed(4)}
                </span>
                <span class={`fcmd-crypto-change ${coin.change_24h >= 0 ? "up" : "down"}`}>
                  {coin.change_24h >= 0 ? "▲" : "▼"}{Math.abs(coin.change_24h).toFixed(1)}%
                </span>
              </div>
            )}
          </For>
        </Show>
      </div>
      <div class="fcmd-ticker-divider" />
      <div class="fcmd-ticker-track">
        <div
          class="fcmd-ticker-text"
          style={{ transform: `translateX(-${store.tickerOffset() % (store.tickerText().length * 7.5)}px)` }}
        >
          {store.tickerText()}{store.tickerText()}{store.tickerText()}
        </div>
      </div>
    </footer>
  );
}
