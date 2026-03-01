import { createSignal, createEffect, onMount, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  onClose: () => void;
  onThemeChange: (themeName: string) => void;
  currentConfig: any;
}

export default function Settings(props: Props) {
  const [activeTab, setActiveTab] = createSignal("appearance");
  const [config, setConfig] = createSignal<any>(props.currentConfig || {});
  const [saved, setSaved] = createSignal(false);

  // Appearance
  const [theme, setTheme] = createSignal(config()?.theme || "hacker-green");
  const [fontSize, setFontSize] = createSignal(config()?.font_size || 14);
  const [fontFamily, setFontFamily] = createSignal(
    config()?.font_family || "JetBrains Mono"
  );
  const [cursorStyle, setCursorStyle] = createSignal(
    config()?.cursor_style || "block"
  );
  const [cursorBlink, setCursorBlink] = createSignal(
    config()?.cursor_blink ?? true
  );

  // Effects
  const [glow, setGlow] = createSignal(config()?.effects?.glow ?? true);
  const [glowIntensity, setGlowIntensity] = createSignal(
    config()?.effects?.glow_intensity || 0.3
  );
  const [crtScanlines, setCrtScanlines] = createSignal(
    config()?.effects?.crt_scanlines ?? false
  );
  const [matrixRain, setMatrixRain] = createSignal(
    config()?.effects?.matrix_rain ?? false
  );
  const [matrixOpacity, setMatrixOpacity] = createSignal(
    config()?.effects?.matrix_rain_opacity || 0.05
  );
  const [particles, setParticles] = createSignal(
    config()?.effects?.particles_on_keystroke ?? false
  );

  // AI Provider
  const [aiType, setAiType] = createSignal(getAIType());
  const [ollamaModel, setOllamaModel] = createSignal(getOllamaModel());
  const [ollamaUrl, setOllamaUrl] = createSignal(getOllamaUrl());
  const [openaiKey, setOpenaiKey] = createSignal(getOpenAIKey());
  const [openaiModel, setOpenaiModel] = createSignal(getOpenAIModel());
  const [anthropicKey, setAnthropicKey] = createSignal(getAnthropicKey());
  const [anthropicModel, setAnthropicModel] = createSignal(
    getAnthropicModel()
  );

  // ── Dynamic Ollama models ──
  const [ollamaModels, setOllamaModels] = createSignal<string[]>([]);
  const [modelsLoading, setModelsLoading] = createSignal(false);
  const [modelsError, setModelsError] = createSignal("");

  // Auto-fetch when switching to Ollama settings
  createEffect(() => {
    if (activeTab() === "ai" && aiType() === "ollama") {
      fetchOllamaModels();
    }
  });

  async function fetchOllamaModels() {
    setModelsLoading(true);
    setModelsError("");
    try {
      const models = (await invoke("list_ollama_models", {
        baseUrl: ollamaUrl(),
      })) as string[];
      setOllamaModels(models);
    } catch (e: any) {
      setModelsError(e.toString());
      // Static fallback so dropdown isn't empty
      setOllamaModels([]);
    } finally {
      setModelsLoading(false);
    }
  }

  function getAIType(): string {
    const p = config()?.ai_provider;
    if (!p) return "ollama";
    if (p.Ollama) return "ollama";
    if (p.OpenAI) return "openai";
    if (p.Anthropic) return "anthropic";
    return "ollama";
  }
  function getOllamaModel(): string {
    return config()?.ai_provider?.Ollama?.model || "llama3.2";
  }
  function getOllamaUrl(): string {
    return (
      config()?.ai_provider?.Ollama?.base_url || "http://localhost:11434"
    );
  }
  function getOpenAIKey(): string {
    return config()?.ai_provider?.OpenAI?.api_key || "";
  }
  function getOpenAIModel(): string {
    return config()?.ai_provider?.OpenAI?.model || "gpt-4o-mini";
  }
  function getAnthropicKey(): string {
    return config()?.ai_provider?.Anthropic?.api_key || "";
  }
  function getAnthropicModel(): string {
    return (
      config()?.ai_provider?.Anthropic?.model ||
      "claude-3-5-sonnet-20241022"
    );
  }

  async function saveConfig() {
    let aiProvider: any = null;
    if (aiType() === "ollama") {
      aiProvider = {
        Ollama: { model: ollamaModel(), base_url: ollamaUrl() },
      };
    } else if (aiType() === "openai") {
      aiProvider = {
        OpenAI: { api_key: openaiKey(), model: openaiModel() },
      };
    } else if (aiType() === "anthropic") {
      aiProvider = {
        Anthropic: { api_key: anthropicKey(), model: anthropicModel() },
      };
    }

    const newConfig = {
      theme: theme(),
      font_family: fontFamily(),
      font_size: fontSize(),
      cursor_style: cursorStyle(),
      cursor_blink: cursorBlink(),
      opacity: config()?.opacity || 0.95,
      blur: config()?.blur ?? true,
      effects: {
        crt_scanlines: crtScanlines(),
        crt_curvature: config()?.effects?.crt_curvature ?? false,
        glow: glow(),
        glow_intensity: glowIntensity(),
        matrix_rain: matrixRain(),
        matrix_rain_opacity: matrixOpacity(),
        particles_on_keystroke: particles(),
        screen_flicker: config()?.effects?.screen_flicker ?? false,
      },
      ai_provider: aiProvider,
      default_shell: config()?.default_shell || null,
      default_cwd: config()?.default_cwd || null,
      snippets: config()?.snippets || [],
      keybindings: config()?.keybindings || {
        ai_bar: "CommandOrControl+K",
        command_palette: "CommandOrControl+P",
        new_tab: "CommandOrControl+T",
        close_tab: "CommandOrControl+W",
        split_vertical: "CommandOrControl+D",
        split_horizontal: "CommandOrControl+Shift+D",
        snippet_library: "CommandOrControl+Shift+L",
      },
    };

    try {
      await invoke("set_config", { config: newConfig });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
      props.onThemeChange(theme());
    } catch (e) {
      console.error("Failed to save config:", e);
      alert("Failed to save: " + e);
    }
  }

  const themes = [
    { id: "hacker-green", name: "Hacker Green", color: "#00ff41" },
    { id: "cyberpunk", name: "Cyberpunk", color: "#ff00ff" },
    { id: "matrix", name: "Matrix", color: "#00ff00" },
    { id: "ghost-protocol", name: "Ghost Protocol", color: "#39bae6" },
    { id: "tron", name: "Tron", color: "#6fc3df" },
    { id: "midnight", name: "Midnight", color: "#7c8ef5" },
  ];

  const fonts = [
    "JetBrains Mono",
    "Fira Code",
    "Cascadia Code",
    "SF Mono",
    "Menlo",
    "Monaco",
    "Source Code Pro",
    "IBM Plex Mono",
    "Hack",
    "Consolas",
  ];

  return (
    <div class="settings-overlay" onClick={() => props.onClose()}>
      <div class="settings-panel" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="settings-header">
          <span>⚙️ Settings</span>
          <div class="settings-header-right">
            <Show when={saved()}>
              <span class="settings-saved">✅ Saved!</span>
            </Show>
            <button class="settings-save-btn" onClick={saveConfig}>
              Save
            </button>
            <span class="settings-close" onClick={() => props.onClose()}>
              ×
            </span>
          </div>
        </div>

        {/* Tabs */}
        <div class="settings-tabs">
          <button
            class={`settings-tab ${activeTab() === "appearance" ? "active" : ""}`}
            onClick={() => setActiveTab("appearance")}
          >
            🎨 Appearance
          </button>
          <button
            class={`settings-tab ${activeTab() === "effects" ? "active" : ""}`}
            onClick={() => setActiveTab("effects")}
          >
            ✨ Effects
          </button>
          <button
            class={`settings-tab ${activeTab() === "ai" ? "active" : ""}`}
            onClick={() => setActiveTab("ai")}
          >
            🤖 AI Provider
          </button>
        </div>

        {/* Content */}
        <div class="settings-content">
          {/* ── Appearance Tab ── */}
          <Show when={activeTab() === "appearance"}>
            <div class="settings-section">
              <h4>Theme</h4>
              <div class="theme-grid">
                {themes.map((t) => (
                  <div
                    class={`theme-card ${theme() === t.id ? "active" : ""}`}
                    onClick={() => setTheme(t.id)}
                  >
                    <div
                      class="theme-preview"
                      style={{ "border-color": t.color }}
                    >
                      <div class="theme-dot" style={{ background: t.color }} />
                    </div>
                    <span class="theme-name">{t.name}</span>
                  </div>
                ))}
              </div>
            </div>

            <div class="settings-section">
              <h4>Font</h4>
              <div class="settings-row">
                <label>Family</label>
                <select
                  class="settings-select"
                  value={fontFamily()}
                  onChange={(e) => setFontFamily(e.currentTarget.value)}
                >
                  {fonts.map((f) => (
                    <option value={f}>{f}</option>
                  ))}
                </select>
              </div>
              <div class="settings-row">
                <label>Size</label>
                <div class="settings-range-row">
                  <input
                    type="range"
                    min="10"
                    max="24"
                    step="1"
                    value={fontSize()}
                    onInput={(e) =>
                      setFontSize(Number(e.currentTarget.value))
                    }
                  />
                  <span class="settings-range-value">{fontSize()}px</span>
                </div>
              </div>
            </div>

            <div class="settings-section">
              <h4>Cursor</h4>
              <div class="settings-row">
                <label>Style</label>
                <div class="settings-button-group">
                  {["block", "beam", "underline"].map((style) => (
                    <button
                      class={`settings-btn-option ${cursorStyle() === style ? "active" : ""}`}
                      onClick={() => setCursorStyle(style)}
                    >
                      {style}
                    </button>
                  ))}
                </div>
              </div>
              <div class="settings-row">
                <label>Blink</label>
                <button
                  class={`settings-toggle ${cursorBlink() ? "on" : "off"}`}
                  onClick={() => setCursorBlink(!cursorBlink())}
                >
                  {cursorBlink() ? "ON" : "OFF"}
                </button>
              </div>
            </div>
          </Show>

          {/* ── Effects Tab ── */}
          <Show when={activeTab() === "effects"}>
            <div class="settings-section">
              <h4>Text Glow</h4>
              <div class="settings-row">
                <label>Enable</label>
                <button
                  class={`settings-toggle ${glow() ? "on" : "off"}`}
                  onClick={() => setGlow(!glow())}
                >
                  {glow() ? "ON" : "OFF"}
                </button>
              </div>
              <Show when={glow()}>
                <div class="settings-row">
                  <label>Intensity</label>
                  <div class="settings-range-row">
                    <input
                      type="range"
                      min="0.1"
                      max="1.0"
                      step="0.1"
                      value={glowIntensity()}
                      onInput={(e) =>
                        setGlowIntensity(Number(e.currentTarget.value))
                      }
                    />
                    <span class="settings-range-value">
                      {glowIntensity()}
                    </span>
                  </div>
                </div>
              </Show>
            </div>

            <div class="settings-section">
              <h4>CRT Scanlines</h4>
              <div class="settings-row">
                <label>Enable</label>
                <button
                  class={`settings-toggle ${crtScanlines() ? "on" : "off"}`}
                  onClick={() => setCrtScanlines(!crtScanlines())}
                >
                  {crtScanlines() ? "ON" : "OFF"}
                </button>
              </div>
            </div>

            <div class="settings-section">
              <h4>Matrix Rain Background</h4>
              <div class="settings-row">
                <label>Enable</label>
                <button
                  class={`settings-toggle ${matrixRain() ? "on" : "off"}`}
                  onClick={() => setMatrixRain(!matrixRain())}
                >
                  {matrixRain() ? "ON" : "OFF"}
                </button>
              </div>
              <Show when={matrixRain()}>
                <div class="settings-row">
                  <label>Opacity</label>
                  <div class="settings-range-row">
                    <input
                      type="range"
                      min="0.01"
                      max="0.2"
                      step="0.01"
                      value={matrixOpacity()}
                      onInput={(e) =>
                        setMatrixOpacity(Number(e.currentTarget.value))
                      }
                    />
                    <span class="settings-range-value">
                      {matrixOpacity()}
                    </span>
                  </div>
                </div>
              </Show>
            </div>

            <div class="settings-section">
              <h4>Keystroke Particles</h4>
              <div class="settings-row">
                <label>Enable</label>
                <button
                  class={`settings-toggle ${particles() ? "on" : "off"}`}
                  onClick={() => setParticles(!particles())}
                >
                  {particles() ? "ON" : "OFF"}
                </button>
              </div>
            </div>
          </Show>

          {/* ── AI Tab ── */}
          <Show when={activeTab() === "ai"}>
            <div class="settings-section">
              <h4>AI Provider</h4>
              <div class="settings-row">
                <label>Provider</label>
                <div class="settings-button-group">
                  {["ollama", "openai", "anthropic"].map((type) => (
                    <button
                      class={`settings-btn-option ${aiType() === type ? "active" : ""}`}
                      onClick={() => setAiType(type)}
                    >
                      {type === "ollama"
                        ? "🦙 Ollama"
                        : type === "openai"
                          ? "🤖 OpenAI"
                          : "🧠 Claude"}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            {/* ── Ollama ── */}
            <Show when={aiType() === "ollama"}>
              <div class="settings-section">
                <h4>🦙 Ollama Settings</h4>
                <p class="settings-hint">
                  Free & local. Run <code>ollama serve</code> first.
                </p>

                <div class="settings-row">
                  <label>URL</label>
                  <input
                    class="settings-input"
                    type="text"
                    value={ollamaUrl()}
                    onInput={(e) => setOllamaUrl(e.currentTarget.value)}
                    placeholder="http://localhost:11434"
                  />
                </div>

                <div class="settings-row">
                  <label>Model</label>
                  <div style={{ flex: "1", display: "flex", gap: "6px" }}>
                    <Show
                      when={ollamaModels().length > 0}
                      fallback={
                        <input
                          class="settings-input"
                          type="text"
                          value={ollamaModel()}
                          onInput={(e) =>
                            setOllamaModel(e.currentTarget.value)
                          }
                          placeholder="e.g. llama3.2"
                        />
                      }
                    >
                      <select
                        class="settings-select"
                        value={ollamaModel()}
                        onChange={(e) =>
                          setOllamaModel(e.currentTarget.value)
                        }
                      >
                        <For each={ollamaModels()}>
                          {(m) => (
                            <option value={m}>{m}</option>
                          )}
                        </For>
                      </select>
                    </Show>
                    <button
                      class="settings-btn-option"
                      style={{
                        flex: "none",
                        width: "36px",
                        opacity: modelsLoading() ? "0.3" : "0.7",
                      }}
                      onClick={fetchOllamaModels}
                      disabled={modelsLoading()}
                      title="Refresh models"
                    >
                      🔄
                    </button>
                  </div>
                </div>

                <Show when={modelsLoading()}>
                  <p class="settings-hint">
                    ⏳ Fetching models from Ollama...
                  </p>
                </Show>

                <Show when={modelsError()}>
                  <p
                    class="settings-hint"
                    style={{ color: "#ff6b6b", opacity: "0.8" }}
                  >
                    ⚠️ {modelsError()}
                    <br />
                    Type model name manually above, or start Ollama and hit 🔄
                  </p>
                </Show>

                <Show when={!modelsLoading() && !modelsError() && ollamaModels().length === 0}>
                  <p class="settings-hint">
                    No models found. Install one:{" "}
                    <code>ollama pull llama3.2</code>
                  </p>
                </Show>
              </div>
            </Show>

            {/* ── OpenAI ── */}
            <Show when={aiType() === "openai"}>
              <div class="settings-section">
                <h4>🤖 OpenAI Settings</h4>
                <p class="settings-hint">
                  Get API key from <code>platform.openai.com</code>
                </p>
                <div class="settings-row">
                  <label>API Key</label>
                  <input
                    class="settings-input"
                    type="password"
                    value={openaiKey()}
                    onInput={(e) => setOpenaiKey(e.currentTarget.value)}
                    placeholder="sk-..."
                  />
                </div>
                <div class="settings-row">
                  <label>Model</label>
                  <select
                    class="settings-select"
                    value={openaiModel()}
                    onChange={(e) => setOpenaiModel(e.currentTarget.value)}
                  >
                    <option value="gpt-4o-mini">gpt-4o-mini</option>
                    <option value="gpt-4o">gpt-4o</option>
                    <option value="gpt-4-turbo">gpt-4-turbo</option>
                    <option value="gpt-3.5-turbo">gpt-3.5-turbo</option>
                    <option value="o3-mini">o3-mini</option>
                  </select>
                </div>
              </div>
            </Show>

            {/* ── Anthropic ── */}
            <Show when={aiType() === "anthropic"}>
              <div class="settings-section">
                <h4>🧠 Anthropic (Claude) Settings</h4>
                <p class="settings-hint">
                  Get API key from <code>console.anthropic.com</code>
                </p>
                <div class="settings-row">
                  <label>API Key</label>
                  <input
                    class="settings-input"
                    type="password"
                    value={anthropicKey()}
                    onInput={(e) =>
                      setAnthropicKey(e.currentTarget.value)
                    }
                    placeholder="sk-ant-..."
                  />
                </div>
                <div class="settings-row">
                  <label>Model</label>
                  <select
                    class="settings-select"
                    value={anthropicModel()}
                    onChange={(e) =>
                      setAnthropicModel(e.currentTarget.value)
                    }
                  >
                    <option value="claude-sonnet-4-20250514">
                      Claude Sonnet 4 (latest)
                    </option>
                    <option value="claude-3-5-sonnet-20241022">
                      Claude 3.5 Sonnet
                    </option>
                    <option value="claude-3-haiku-20240307">
                      Claude 3 Haiku (fast)
                    </option>
                    <option value="claude-3-opus-20240229">
                      Claude 3 Opus (smartest)
                    </option>
                  </select>
                </div>
              </div>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
}