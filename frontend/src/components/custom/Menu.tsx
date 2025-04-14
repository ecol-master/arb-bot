import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import ArbitrageTable from "./ArbitrageTable.tsx";
import Analytics from "./Analytics.tsx";


function Menu() {
    return (<>
        <Tabs defaultValue="account" className="mx-auto max-w-5xl">
            <TabsList className="mx-auto">
                <TabsTrigger value="arbitrages-list" className="w-[150px] bg-zinc-900 font-bold text-lg pt-5 pb-5 mr-2">List</TabsTrigger>
                <TabsTrigger value="analytics" className="w-[150px] bg-zinc-900 font-bold text-lg pt-5 pb-5">Analytics</TabsTrigger>
            </TabsList>
            <TabsContent value="arbitrages-list">
                <h2 className="font-mono text-lg pb-5 font-bold pl-6">Latest triangular loops</h2>
                <ArbitrageTable />
            </TabsContent>
            <TabsContent value="analytics">
                <h2 className="font-mono text-lg pb-10 font-bold pl-6">Arbitrage data analytics</h2>
                <Analytics />
            </TabsContent>
        </Tabs>
    </>)
}

export default Menu;