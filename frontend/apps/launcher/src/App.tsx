import './App.css'

function App() {
  return (
    <main className="app-shell">
      <header className="app-header">
        <strong>배움마루 런처</strong>
        <span className="status status-stopped">서버 중지</span>
      </header>
      <section className="app-content" aria-labelledby="launcher-title">
        <p className="eyebrow">HOST</p>
        <h1 id="launcher-title">호스트 관리</h1>
      </section>
    </main>
  )
}

export default App
