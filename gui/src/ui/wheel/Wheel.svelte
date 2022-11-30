<script lang="ts">
  import Wedge from "./Wedge.svelte";
  import Palette from "../../lib/Palette";

  export let radius: number = 280
  export let divisions: number = 12
  export let palette: Palette = new Palette()
  let ringRotation = 0 // TODO: rotate whee so that tonic is at 0 degrees

  $: size = radius * 2
  $: center = {x: radius, y: radius}
  $: holeRadius = radius * 0.8
  $: arcDegrees = 360.0 / divisions

  function wedgeProps(index: number) {
    const rotation = arcDegrees * index
    const color = palette.primary(index)
    const textColor = palette.complementary(index, -0.8)
    // TODO: note names
    const label = `${index}`
    return {
      radius,
      center,
      arcDegrees,
      rotation,
      color,
      textColor,
      label,
    }
  }
</script>

<div>
  <svg style={`width: ${size}; height: ${size};`}>
    <defs>
      <!--  Clipping mask to cut out the center of the circle, leaving just the rim -->
      <mask id="rim-clip">
        <circle cx={center.x} cy={center.y} r={radius} fill="white"/>
        <circle
            cx={center.x}
            cy={center.y}
            r={holeRadius}
            fill="black"
        ></circle>
      </mask>
    </defs>
    <g
        mask="url(#rim-clip)"
        transform={`rotate(${ringRotation}, ${center.x}, ${center.y})`}
    >
      {#each Array(divisions) as _, i}
        <Wedge {...wedgeProps(i)} />
      {/each}      <circle
          cx={center.x}
          cy={center.y}
          r={holeRadius}
          onClick={(e) => e.preventDefault()}
      />
    </g>
  </svg>
</div>