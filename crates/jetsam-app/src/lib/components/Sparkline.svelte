<script lang="ts">
  let { values }: { values: number[] } = $props();
  const width = 120;
  const height = 28;
  const points = $derived.by(() => {
    const max = Math.max(1, ...values);
    if (values.length === 0) return '';
    return values
      .map((v, i) => {
        const x = values.length === 1 ? width : (i / (values.length - 1)) * width;
        const y = height - (v / max) * height;
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(' ');
  });
</script>

<svg
  class="spark"
  viewBox="0 0 {width} {height}"
  preserveAspectRatio="none"
  role="img"
  aria-label="usage sparkline"
  style="width: {width}px; height: {height}px"
>
  <polyline points={points} fill="none" stroke="var(--accent)" stroke-width="1.5" />
</svg>

<style>
  .spark {
    display: block;
  }
</style>
