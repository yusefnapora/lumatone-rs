<script lang="ts">
    import type { Point, HexColor } from "../../lib/drawing";
    import {polarToCartesian, describeArc, lineTo} from "../../lib/drawing";

    export let radius: number = 300
    export let center: Point = { x: 0, y: 0}
    export let rotation: number = 0
    export let arcDegrees: number = 30

    export let color: HexColor
    export let label: string
    export let textColor: HexColor

    export let fill: string | undefined = undefined
    export let stroke: string | undefined = undefined

    $: halfArc = arcDegrees / 2.0
    $: fontSizePct = (radius / 300) * 100
    $: fontSize = `${fontSizePct}%`
    $: point = polarToCartesian(center, radius, halfArc)
    $: labelPoint = polarToCartesian(center, radius * 0.9, 0)
    $: wedgePath = [
        describeArc(center, radius, -halfArc, halfArc),
        lineTo(center),
        lineTo(point),
    ].join(' ')

</script>

<g transform={`rotate(${rotation}, ${center.x}, ${center.y})`}
   fill={fill || color}
   stroke={stroke || color}
   onClick={() => console.log('clicked', label)}
>
    <path d={wedgePath} stroke-width="{0}" stroke="none"></path>
    <text
        font-size="{fontSize}"
        text-anchor="middle"
        x={labelPoint.x}
        y={labelPoint.y}
        stroke={textColor}
        fill={textColor}
    >
        {label}
    </text>
</g>

