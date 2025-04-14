import { Bar, BarChart, XAxis, YAxis } from "recharts"

import { ChartConfig, ChartContainer } from "@/components/ui/chart"

const data = [
    { date: "06.03", loops: 80 },
    { date: "07.03", loops: 200 },
    { date: "08.03", loops: 100 },
    { date: "09.03", loops: 110 },
    { date: "10.03", loops: 98 },
    { date: "11.03", loops: 111 },
    { date: "12.03", loops: 122 },
    { date: "13.03", loops: 190 },
    { date: "14.03", loops: 130 },
    { date: "15.03", loops: 150 },
    { date: "16.03", loops: 173 },
    { date: "17.03", loops: 164 },
    { date: "18.03", loops: 204 },
    { date: "19.03", loops: 150 },
    { date: "20.03", loops: 138 },
    { date: "21.03", loops: 170 },
    { date: "22.03", loops: 210 },
    { date: "23.03", loops: 160 },
    { date: "24.03", loops: 204 },
    { date: "25.03", loops: 130 },
    { date: "26.03", loops: 145 },
    { date: "27.03", loops: 120 },
    { date: "28.03", loops: 161 },
]

const chartConfig = {
    loops: {
        label: "Triangular loops",
        color: "oklch(70.2% 0.183 293.541)",
        // color: "oklch(60.6% 0.25 292.717)"
    },
} satisfies ChartConfig

export function Analytics() {
    return (
        <ChartContainer config={chartConfig} className="min-h-[200px] max-w-[700px] mx-auto">
            <BarChart accessibilityLayer data={data}>
                <XAxis
                    dataKey="date"
                    tickLine={false}
                    tickMargin={10}
                    axisLine={false}
                    tickFormatter={(value) => value.slice(0, 5)}
                    interval={3}
                />
                <YAxis
                    label={{ value: "Arbitrages", angle: -90, position: "insideLeft" }}
                />
                <Bar dataKey="loops" fill="var(--color-loops)" radius={4} />
            </BarChart>
        </ChartContainer>
    )
}

export default Analytics;