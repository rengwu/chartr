<script lang="ts">
  // A one-line marquee that only moves when it has to: if the text fits, it sits
  // still and truncates like any label; if it overflows, it scrolls its content in
  // a slow, seamless loop and pauses under the pointer so it can be read. Motion is
  // dropped entirely when the operator has asked for less of it — the text then
  // truncates rather than scrolls, so nothing is ever hidden, only still.
  //
  // It measures itself: the container's width against one copy of the text. Two
  // copies ride the track with a gap between them, and the track slides by exactly
  // one copy-plus-gap, so the second copy lands where the first began and the loop
  // has no seam.

  let {
    text,
    class: klass = "",
  }: {
    text: string;
    // Styling from the parent — the marquee is monochrome and muted by default, but
    // the caller sets size and colour so it reads as a subtitle beside its label.
    class?: string;
  } = $props();

  // Constant scroll speed in px/s, so a long title doesn't whip past faster than a
  // short one — the duration is derived from the distance, not fixed.
  const SPEED = 32;
  // The gap between the two copies, in px, so the loop breathes rather than butting
  // the text against its own tail.
  const GAP = 32;

  let containerWidth = $state(0);
  let textWidth = $state(0);

  const reduceMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;

  // Scroll only when the text genuinely overflows its container and motion is
  // allowed. A one-pixel slack avoids a jittery scroll on a title that only just
  // fits.
  const scrolling = $derived(
    !reduceMotion && textWidth > 0 && textWidth > containerWidth + 1,
  );
  const shift = $derived(textWidth + GAP);
  const duration = $derived(shift / SPEED);
</script>

<div
  class={["marquee", klass]}
  bind:clientWidth={containerWidth}
  title={text}
  aria-label={text}
>
  {#if scrolling}
    <!-- The moving track: two copies a gap apart, sliding by one copy-plus-gap so
       the second copy seamlessly takes the first's place. -->
    <div
      class="marquee-track"
      style="--marquee-shift: {shift}px; --marquee-duration: {duration}s; gap: {GAP}px;"
    >
      <span class="marquee-copy">{text}</span>
      <span class="marquee-copy" aria-hidden="true">{text}</span>
    </div>
  {:else}
    <!-- Fits (or reduced motion): a plain, still, truncating label. A hidden
       measuring copy keeps textWidth up to date so a later resize can start it
       scrolling without a mount. -->
    <span class="marquee-static">{text}</span>
    <span class="marquee-measure" aria-hidden="true" bind:clientWidth={textWidth}
      >{text}</span
    >
  {/if}
</div>

<style>
  .marquee {
    position: relative;
    overflow: hidden;
    white-space: nowrap;
    min-width: 0;
  }

  .marquee-track {
    display: inline-flex;
    width: max-content;
    will-change: transform;
    animation: marquee-scroll var(--marquee-duration) linear infinite;
  }

  /* Reading beats motion: hold the scroll while the pointer is over it. */
  .marquee:hover .marquee-track {
    animation-play-state: paused;
  }

  .marquee-copy {
    flex: 0 0 auto;
  }

  .marquee-static {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* The measuring copy is laid out (so it has a width) but never seen: off the
     flow, zero-height, non-interactive. */
  .marquee-measure {
    position: absolute;
    visibility: hidden;
    height: 0;
    white-space: nowrap;
    pointer-events: none;
  }

  @keyframes marquee-scroll {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(calc(-1 * var(--marquee-shift)));
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .marquee-track {
      animation: none;
    }
  }
</style>
