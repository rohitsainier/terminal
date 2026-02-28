import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface AIResponse {
  command: string;
  explanation: string;
  dangerous: boolean;
  danger_reason: string | null;
}

export function useAI() {
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [result, setResult] = createSignal<AIResponse | null>(null);

  async function translate(query: string, cwd: string = "~") {
    setLoading(true);
    setError("");
    setResult(null);

    try {
      const response = (await invoke("ai_translate_command", {
        query,
        cwd,
      })) as AIResponse;
      setResult(response);
      return response;
    } catch (e: any) {
      setError(e.toString());
      return null;
    } finally {
      setLoading(false);
    }
  }

  async function explain(command: string) {
    setLoading(true);
    setError("");

    try {
      const explanation = (await invoke("ai_explain_command", {
        command,
      })) as string;
      return explanation;
    } catch (e: any) {
      setError(e.toString());
      return null;
    } finally {
      setLoading(false);
    }
  }

  async function suggestFix(command: string, errorOutput: string) {
    setLoading(true);
    setError("");
    setResult(null);

    try {
      const response = (await invoke("ai_suggest_fix", {
        command,
        errorOutput,
      })) as AIResponse;
      setResult(response);
      return response;
    } catch (e: any) {
      setError(e.toString());
      return null;
    } finally {
      setLoading(false);
    }
  }

  return {
    loading,
    error,
    result,
    translate,
    explain,
    suggestFix,
  };
}