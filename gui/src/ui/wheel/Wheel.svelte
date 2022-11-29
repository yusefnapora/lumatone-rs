<script lang="ts">
  import {Wedge} from "./Wedge.svelte";
  import Palette from "../../lib/Palette";

  export let radius: number = 300
  export let divisions: number = 12

  export let palette: Palette = new Palette()

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

<!-- TODO: set layout styles on wrapper div -->
<div>
  <svg style="width: inherit; height: inherit;">
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
  </svg>
</div>