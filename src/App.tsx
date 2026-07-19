import { Sidebar } from "./components/Sidebar";
import { LibraryView } from "./views/LibraryView";
import "./App.css";

function App() {
  return (
    <div className="flex h-full bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <Sidebar active="library" />
      <main className="flex-1 min-w-0">
        <LibraryView />
      </main>
    </div>
  );
}

export default App;
