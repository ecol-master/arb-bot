import Menu from "./components/custom/Menu.tsx"
import './App.css'
import "./index.css"
import ArbitrageTable from "./components/custom/ArbitrageTable.tsx";
import Analytics from "./components/custom/Analytics.tsx";

function App() {

  return (
    <>
      <div className="min-h-screen dark:bg-[#0f0f0f] dark:text-white p-6 dark">
        <h1 className="mx-auto max-w-5xl text-left pb-10 pl-6 font-mono text-3xl">Arbitrage View</h1>
        <h2 className="font-mono max-w-5xl mx-auto text-lg pb-5 font-bold pl-6">Latest triangular loops</h2>
        <ArbitrageTable />

        <h2 className="font-mono max-w-5xl mx-auto text-lg pb-10 pt-20 font-bold pl-6">Arbitrage data analytics</h2>
        <Analytics />
      </div>
    </>
  )
}

export default App
