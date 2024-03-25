import { greet } from "gb_wasm_bindings";

function App() {
  return (
    <div>
      <button
        onClick={() => {
          greet("WASM");
        }}
      >
        Greet
      </button>
    </div>
  );
}

export default App;
