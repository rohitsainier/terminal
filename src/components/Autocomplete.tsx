interface Suggestion {
  text: string;
  type: "history" | "path" | "command" | "snippet";
  icon: string;
}

interface Props {
  suggestions: Suggestion[];
  visible: boolean;
  x: number;
  y: number;
  selectedIndex: number;
  onSelect: (suggestion: Suggestion) => void;
  onHover: (index: number) => void;
}

export default function Autocomplete(props: Props) {
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
          class={`autocomplete-item ${index === props.selectedIndex ? "selected" : ""}`}
          onClick={() => props.onSelect(suggestion)}
          onMouseEnter={() => props.onHover(index)}
        >
          <span class="autocomplete-icon">{suggestion.icon}</span>
          <span class="autocomplete-text">{suggestion.text}</span>
          <span class="autocomplete-type">{suggestion.type}</span>
        </div>
      ))}
    </div>
  );
}

export type { Suggestion };