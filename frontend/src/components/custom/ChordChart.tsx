import * as d3 from "d3"
import { useEffect, useRef } from "react"

export function ChordChart() {
    const ref = useRef<SVGSVGElement | null>(null)

    useEffect(() => {
        const matrix = [
            [11975, 5871, 8916, 2868],
            [1951, 10048, 2060, 6171],
            [8010, 16145, 8090, 8045],
            [1013, 990, 940, 6907],
        ]

        const svg = d3.select(ref.current)
        const width = 500
        const height = 500
        const innerRadius = Math.min(width, height) * 0.5 - 40
        const outerRadius = innerRadius + 10

        const chord = d3
            .chord()
            .padAngle(0.05)
            .sortSubgroups(d3.descending)(matrix)

        const arc = d3.arc<d3.ChordGroup>()
            .innerRadius(innerRadius)
            .outerRadius(outerRadius)

        const ribbon = d3.ribbon()
            .radius(innerRadius)

        const color = d3.scaleOrdinal<string, string>(d3.schemeCategory10)

        svg
            .attr("viewBox", [-width / 2, -height / 2, width, height].toString())
            .attr("width", width)
            .attr("height", height)

        svg.append("g")
            .selectAll("path")
            .data(chord.groups)
            .join("path")
            .attr("fill", (d) => color(String(d.index)))
            .attr("stroke", (d) =>
                d3.rgb(color(String(d.index))).darker().toString()
            )
            .attr("d", arc)

        svg.append("g")
            .attr("fill-opacity", 0.67)
            .selectAll("path")
            .data(chord)
            .join("path")
            .attr("fill", (d) => color(String(d.target.index)))
            .attr("stroke", (d) =>
                d3.rgb(color(String(d.target.index))).darker().toString()
            )
            .attr("d", d => ribbon(d as d3.Ribbon))
    }, [])

    return <svg ref={ref} />
}