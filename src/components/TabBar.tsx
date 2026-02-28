interface Tab {
  id: string;
  title: string;
  cwd: string;
}

interface Props {
  tabs: Tab[];
  activeTab: string;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onCreate: () => void;
}

export default function TabBar(props: Props) {
  return (
    <div class="tab-bar">
      {props.tabs.map((tab) => (
        <div
          class={`tab ${tab.id === props.activeTab ? "active" : ""}`}
          onClick={() => props.onSelect(tab.id)}
        >
          <span class="tab-icon">❯</span>
          <span class="tab-title">{tab.title}</span>
          <span
            class="tab-close"
            onClick={(e) => {
              e.stopPropagation();
              props.onClose(tab.id);
            }}
          >
            ×
          </span>
        </div>
      ))}
      <div class="tab-new" onClick={() => props.onCreate()}>
        +
      </div>
    </div>
  );
}