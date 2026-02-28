import { createSignal } from "solid-js";

interface Suggestion {
  text: string;
  type: "history" | "path" | "command" | "ai";
  icon: string;
}

interface Props {
  suggestions: Suggestion[];
  visible: boolean;
  x: number;
  y: number;
  onSelect: (suggestion: Suggestion) => void;
}

export default function Autocomplete(props: Props) {
  const [selectedIndex, setSelectedIndex] = createSignal(0);

  if (!props.visible || props.suggestions.length === 0) return null;

  return (
    <div
      class="autocomplete"
      style={{
        left: `${props.x}px`,
        top: `${props.y}px`,
      }}
    >
      {props.suggestions.map((suggestion, index) => (
        <div
          class={`autocomplete-item ${
            index === selectedIndex() ? "selected" : ""
          }`}
          onClick={() => props.onSelect(suggestion)}
          onMouseEnter={() => setSelectedIndex(index)}
        >
          <span class="autocomplete-icon">{suggestion.icon}</span>
          <span class="autocomplete-text">{suggestion.text}</span>
          <span class="autocomplete-type">{suggestion.type}</span>
        </div>
      ))}
    </div>
  );
}