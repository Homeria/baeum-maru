import './App.css'

function App() {
  return (
    <main className="app-shell">
      <header className="app-header">
        <strong>배움마루</strong>
        <span className="status">준비됨</span>
      </header>
      <section className="app-content" aria-labelledby="operator-title">
        <p className="eyebrow">OPERATOR</p>
        <h1 id="operator-title">업무 웹</h1>
      </section>
    </main>
  )
}

export default App
